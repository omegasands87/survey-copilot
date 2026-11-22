use crate::db::Database;
use crate::domain::credential::{
    AppCredential, CredentialStatus, CredentialSummary, DeleteCredentialResult,
    SaveCredentialInput, TestCredentialResult, UpdateCredentialInput,
};
use crate::domain::error::AppError;
use crate::infrastructure::credentials::CredentialStore;
use crate::repository::CredentialRepository;
use chrono::Utc;
use uuid::Uuid;

pub struct CredentialService;

impl CredentialService {
    /// Validasi tipe provider yang didukung aplikasi
    fn validate_provider_type(provider_type: &str) -> Result<(), AppError> {
        match provider_type {
            "gemini" | "openai_compatible" | "anthropic_compatible" | "custom_http" => Ok(()),
            _ => Err(AppError::validation(
                "providerType",
                format!("Tipe provider '{provider_type}' tidak valid. Pilih salah satu dari: gemini, openai_compatible, anthropic_compatible, custom_http."),
            )),
        }
    }

    /// Menyimpan kredensial baru atau memperbarui kredensial yang ada.
    ///
    /// Urutan Operasi Aman:
    /// 1. Validasi tipe provider, label, dan panjang kunci rahasia.
    /// 2. Simpan secret ke Windows Credential Manager menggunakan referensi buram (`survey_copilot/cred/<id>`).
    /// 3. Simpan metadata buram (tanpa secret) ke SQLite dalam transaksi.
    /// 4. Jika simpan SQLite gagal, rollback secret dari Windows Credential Manager agar tidak tertinggal orphan secret.
    /// 5. Bersihkan nilai secret dari memori.
    pub fn save_credential(
        database: &Database,
        store: &dyn CredentialStore,
        input: SaveCredentialInput,
    ) -> Result<CredentialSummary, AppError> {
        let provider_type = input.provider_type.trim();
        Self::validate_provider_type(provider_type)?;

        let label = input.label.trim();
        if label.is_empty() || label.len() > 100 {
            return Err(AppError::validation(
                "label",
                "Label kredensial wajib diisi (antara 1 hingga 100 karakter).",
            ));
        }

        let secret = input.secret.trim();
        if secret.len() < 8 {
            return Err(AppError::validation(
                "secret",
                "Kunci API rahasia wajib diisi dan minimal 8 karakter.",
            ));
        }

        let id = match input.id {
            Some(ref custom_id) if !custom_id.trim().is_empty() => custom_id.trim().to_string(),
            _ => Uuid::new_v4().to_string(),
        };

        let credential_ref = format!("survey_copilot/cred/{id}");
        let now = Utc::now().to_rfc3339();

        // 1. Simpan secret ke Windows Credential Manager terlebih dahulu
        store.set_secret(&credential_ref, secret)?;

        // 2. Simpan metadata buram ke basis data lokal SQLite
        let conn_res = database.connection.lock();
        let conn = match conn_res {
            Ok(c) => c,
            Err(_) => {
                // Rollback secret jika lock database gagal
                let _ = store.delete_secret(&credential_ref);
                return Err(AppError::database(
                    "Basis data sedang terkunci atau tidak tersedia.",
                ));
            }
        };

        let existing = CredentialRepository::find_by_id(&conn, &id).map_err(|e| {
            let _ = store.delete_secret(&credential_ref);
            AppError::from(e)
        })?;

        let credential = AppCredential {
            id: id.clone(),
            provider_type: provider_type.to_string(),
            label: label.to_string(),
            credential_ref: credential_ref.clone(),
            status: CredentialStatus::Configured,
            created_at: existing
                .as_ref()
                .map(|e| e.created_at.clone())
                .unwrap_or_else(|| now.clone()),
            updated_at: now.clone(),
        };

        let db_write_result = if existing.is_some() {
            CredentialRepository::update(
                &conn,
                &id,
                Some(label),
                Some(CredentialStatus::Configured.as_str()),
                &now,
            )
            .map(|_| ())
        } else {
            CredentialRepository::create(&conn, &credential)
        };

        if let Err(err) = db_write_result {
            // Rollback secret dari OS jika penulisan metadata gagal
            let _ = store.delete_secret(&credential_ref);
            return Err(AppError::from(err));
        }

        let usage_count =
            CredentialRepository::count_references_in_provider_configs(&conn, &credential_ref)
                .unwrap_or(0);

        Ok(CredentialSummary {
            id: credential.id,
            provider_type: credential.provider_type,
            label: credential.label,
            credential_ref: credential.credential_ref,
            status: credential.status,
            usage_count,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
        })
    }

    /// Mengambil seluruh daftar kredensial yang tersimpan beserta status dan jumlah referensinya.
    /// Hasil TIDAK PERNAH memuat rahasia atau kunci API asli.
    pub fn list_credentials(database: &Database) -> Result<Vec<CredentialSummary>, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let credentials = CredentialRepository::list_all(&conn).map_err(AppError::from)?;
        let mut summaries = Vec::with_capacity(credentials.len());

        for c in credentials {
            let usage_count = CredentialRepository::count_references_in_provider_configs(
                &conn,
                &c.credential_ref,
            )
            .unwrap_or(0);

            summaries.push(CredentialSummary {
                id: c.id,
                provider_type: c.provider_type,
                label: c.label,
                credential_ref: c.credential_ref,
                status: c.status,
                usage_count,
                created_at: c.created_at,
                updated_at: c.updated_at,
            });
        }

        Ok(summaries)
    }

    /// Mengambil status satu kredensial berdasarkan ID.
    pub fn get_credential_status(
        database: &Database,
        store: &dyn CredentialStore,
        id: &str,
    ) -> Result<CredentialSummary, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let cred = CredentialRepository::find_by_id(&conn, id)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::not_found(format!("Kredensial dengan ID '{id}' tidak ditemukan."))
            })?;

        // Verifikasi apakah secret benar-benar ada di OS Credential Store
        let exists = store.exists(&cred.credential_ref).unwrap_or(false);
        let actual_status = if exists {
            cred.status
        } else {
            CredentialStatus::Unconfigured
        };

        let usage_count =
            CredentialRepository::count_references_in_provider_configs(&conn, &cred.credential_ref)
                .unwrap_or(0);

        Ok(CredentialSummary {
            id: cred.id,
            provider_type: cred.provider_type,
            label: cred.label,
            credential_ref: cred.credential_ref,
            status: actual_status,
            usage_count,
            created_at: cred.created_at,
            updated_at: cred.updated_at,
        })
    }

    /// Memperbarui label atau secret kredensial yang ada.
    pub fn update_credential(
        database: &Database,
        store: &dyn CredentialStore,
        input: UpdateCredentialInput,
    ) -> Result<CredentialSummary, AppError> {
        let id = input.id.trim();
        if id.is_empty() {
            return Err(AppError::validation("id", "ID kredensial wajib diisi."));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let existing = CredentialRepository::find_by_id(&conn, id)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::not_found(format!("Kredensial dengan ID '{id}' tidak ditemukan."))
            })?;

        // Jika secret baru disediakan, simpan ke store terlebih dahulu
        if let Some(ref new_secret) = input.secret {
            let trimmed_secret = new_secret.trim();
            if trimmed_secret.len() < 8 {
                return Err(AppError::validation(
                    "secret",
                    "Kunci API rahasia baru minimal 8 karakter.",
                ));
            }
            store.set_secret(&existing.credential_ref, trimmed_secret)?;
        }

        let now = Utc::now().to_rfc3339();
        let new_label = match input.label {
            Some(ref l) => {
                let trimmed = l.trim();
                if trimmed.is_empty() || trimmed.len() > 100 {
                    return Err(AppError::validation(
                        "label",
                        "Label kredensial harus antara 1 hingga 100 karakter.",
                    ));
                }
                Some(trimmed)
            }
            None => None,
        };

        CredentialRepository::update(
            &conn,
            id,
            new_label,
            Some(CredentialStatus::Configured.as_str()),
            &now,
        )
        .map_err(AppError::from)?;

        let updated = CredentialRepository::find_by_id(&conn, id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::internal("Gagal memuat kredensial setelah pembaruan."))?;

        let usage_count = CredentialRepository::count_references_in_provider_configs(
            &conn,
            &updated.credential_ref,
        )
        .unwrap_or(0);

        Ok(CredentialSummary {
            id: updated.id,
            provider_type: updated.provider_type,
            label: updated.label,
            credential_ref: updated.credential_ref,
            status: updated.status,
            usage_count,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        })
    }

    /// Menghapus kredensial dari Windows Credential Manager dan SQLite.
    ///
    /// Perlindungan Kredensial Bersama:
    /// Jika kredensial masih digunakan oleh konfigurasi provider (`usage_count > 0`),
    /// operasi akan ditolak kecuali parameter `force` diatur ke `true`.
    pub fn delete_credential(
        database: &Database,
        store: &dyn CredentialStore,
        id: &str,
        force: bool,
    ) -> Result<DeleteCredentialResult, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError::validation("id", "ID kredensial wajib diisi."));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let existing = CredentialRepository::find_by_id(&conn, id)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::not_found(format!("Kredensial dengan ID '{id}' tidak ditemukan."))
            })?;

        // Periksa apakah kredensial sedang digunakan oleh konfigurasi provider
        let usage_count = CredentialRepository::count_references_in_provider_configs(
            &conn,
            &existing.credential_ref,
        )
        .unwrap_or(0);

        if usage_count > 0 && !force {
            return Err(AppError::conflict(format!(
                "Kredensial '{}' masih digunakan oleh {} konfigurasi provider. Konfirmasi penghapusan (force=true) diperlukan.",
                existing.label, usage_count
            )));
        }

        // Hapus secret dari Windows Credential Manager
        let store_del_res = store.delete_secret(&existing.credential_ref);
        let deleted_from_store = store_del_res.is_ok();

        // Hapus rekaman metadata dari SQLite
        let deleted_from_database =
            CredentialRepository::delete_by_id(&conn, id).map_err(AppError::from)?;

        let message = if deleted_from_store && deleted_from_database {
            format!("Kredensial '{}' berhasil dihapus secara bersih dari Windows Credential Manager dan basis data lokal.", existing.label)
        } else if deleted_from_database {
            format!("Metadata kredensial '{}' dihapus dari basis data lokal (secret di Windows Credential Manager sebelumnya telah terhapus atau tidak ditemukan).", existing.label)
        } else {
            format!(
                "Peringatan: Gagal menghapus kredensial '{}' secara tuntas.",
                existing.label
            )
        };

        Ok(DeleteCredentialResult {
            id: id.to_string(),
            credential_ref: existing.credential_ref,
            deleted_from_store,
            deleted_from_database,
            message,
        })
    }

    /// Menguji keberadaan dan keterbacaan kredensial di Windows Credential Manager.
    pub fn test_credential(
        database: &Database,
        store: &dyn CredentialStore,
        id: &str,
    ) -> Result<TestCredentialResult, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError::validation("id", "ID kredensial wajib diisi."));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let existing = CredentialRepository::find_by_id(&conn, id)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::not_found(format!("Kredensial dengan ID '{id}' tidak ditemukan."))
            })?;

        let now = Utc::now().to_rfc3339();

        match store.exists(&existing.credential_ref) {
            Ok(true) => {
                let _ = CredentialRepository::update(
                    &conn,
                    id,
                    None,
                    Some(CredentialStatus::Configured.as_str()),
                    &now,
                );
                Ok(TestCredentialResult {
                    id: id.to_string(),
                    credential_ref: existing.credential_ref,
                    is_valid: true,
                    status: CredentialStatus::Configured,
                    message: "Kredensial berhasil diverifikasi dan dapat diakses dengan aman di Windows Credential Manager.".to_string(),
                    tested_at: now,
                })
            }
            Ok(false) => {
                let _ = CredentialRepository::update(
                    &conn,
                    id,
                    None,
                    Some(CredentialStatus::Unconfigured.as_str()),
                    &now,
                );
                Ok(TestCredentialResult {
                    id: id.to_string(),
                    credential_ref: existing.credential_ref,
                    is_valid: false,
                    status: CredentialStatus::Unconfigured,
                    message: "Kredensial tidak ditemukan di Windows Credential Manager. Silakan perbarui kunci API.".to_string(),
                    tested_at: now,
                })
            }
            Err(err) => {
                let _ = CredentialRepository::update(
                    &conn,
                    id,
                    None,
                    Some(CredentialStatus::Inaccessible.as_str()),
                    &now,
                );
                Ok(TestCredentialResult {
                    id: id.to_string(),
                    credential_ref: existing.credential_ref,
                    is_valid: false,
                    status: CredentialStatus::Inaccessible,
                    message: format!(
                        "Akses kredensial di Windows Credential Manager gagal: {}",
                        err.message
                    ),
                    tested_at: now,
                })
            }
        }
    }

    /// Membaca secret rahasia HANYA untuk kebutuhan internal backend (otentikasi provider di SC-021).
    ///
    /// FUNGSI INI TIDAK PERNAH DIJADIKAN TAURI IPC COMMAND!
    pub fn read_secret_internal(
        store: &dyn CredentialStore,
        credential_ref: &str,
    ) -> Result<String, AppError> {
        let trimmed_ref = credential_ref.trim();
        if !trimmed_ref.starts_with("survey_copilot/") {
            return Err(AppError::validation(
                "credentialRef",
                "Format referensi kredensial tidak valid.",
            ));
        }
        store.get_secret(trimmed_ref)
    }
}
