use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::export::{RetentionPolicyInfo, WipeReport};
use crate::infrastructure::credentials::CredentialStore;
use crate::repository::{AllRecordCounts, CredentialRepository, DeletionRepository};
use std::path::Path;

pub struct DeletionService;

impl DeletionService {
    /// Mendapatkan jumlah seluruh record yang akan dihapus sebelum konfirmasi.
    pub fn get_wipe_summary(database: &Database) -> Result<AllRecordCounts, AppError> {
        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        DeletionRepository::count_all_records(&connection).map_err(AppError::from)
    }

    fn restore_deleted_secrets(
        store: &dyn CredentialStore,
        secrets: &[(String, String)],
    ) -> Vec<String> {
        let mut failures = Vec::new();
        for (credential_ref, secret) in secrets {
            if let Err(error) = store.set_secret(credential_ref, secret) {
                failures.push(format!("{credential_ref}: {}", error.message));
            }
        }
        failures
    }

    /// Menghapus seluruh data lokal, termasuk secret OS yang dikelola aplikasi.
    /// Jika cleanup credential OS gagal, penghapusan database tidak dilanjutkan.
    pub fn wipe_all_data(
        database: &Database,
        store: &dyn CredentialStore,
        app_data_dir: Option<&Path>,
        confirmation_text: &str,
    ) -> Result<WipeReport, AppError> {
        if confirmation_text.trim() != "HAPUS SEMUA DATA" {
            return Err(AppError::validation(
                "confirmationText",
                "Teks konfirmasi tidak sesuai. Harap ketik tepat: HAPUS SEMUA DATA",
            ));
        }

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        // Read all required metadata before changing any external or database state.
        let temp_file_paths =
            DeletionRepository::get_all_capture_image_paths(&connection).map_err(AppError::from)?;
        let credentials = CredentialRepository::list_all(&connection).map_err(AppError::from)?;

        // Preflight every managed secret. A credential that is already absent is acceptable;
        // an access error aborts before any deletion begins.
        let mut secrets_to_remove: Vec<(String, String)> = Vec::new();
        for credential in &credentials {
            match store.get_secret(&credential.credential_ref) {
                Ok(secret) => secrets_to_remove.push((credential.credential_ref.clone(), secret)),
                Err(error) if error.code == "CREDENTIAL_NOT_FOUND" => {}
                Err(error) => {
                    return Err(AppError::credential_error(format!(
                        "Full wipe dibatalkan sebelum database diubah karena credential '{}' tidak dapat dibaca dari Windows Credential Manager: {}",
                        credential.label, error.message
                    )));
                }
            }
        }

        // Remove OS secrets first. If a later delete fails, restore already-removed secrets.
        let mut deleted_secrets: Vec<(String, String)> = Vec::new();
        for (credential_ref, secret) in &secrets_to_remove {
            if let Err(delete_error) = store.delete_secret(credential_ref) {
                let restore_failures = Self::restore_deleted_secrets(store, &deleted_secrets);
                let restore_message = if restore_failures.is_empty() {
                    "Secret yang telah dihapus berhasil dipulihkan.".to_string()
                } else {
                    format!(
                        "Pemulihan sebagian secret gagal untuk: {}.",
                        restore_failures.join(", ")
                    )
                };
                return Err(AppError::credential_error(format!(
                    "Full wipe dihentikan karena penghapusan credential OS gagal. Database belum diubah. {} Detail: {}",
                    restore_message, delete_error.message
                )));
            }
            deleted_secrets.push((credential_ref.clone(), secret.clone()));
        }

        // Use a transaction for every relational deletion. Restore OS secrets if SQL fails.
        let transaction = connection.transaction().map_err(|error| {
            let restore_failures = Self::restore_deleted_secrets(store, &deleted_secrets);
            AppError::database(format!(
                "Gagal memulai transaksi full wipe: {error}. Pemulihan credential: {}",
                if restore_failures.is_empty() {
                    "berhasil".to_string()
                } else {
                    format!("perlu pemeriksaan: {}", restore_failures.join(", "))
                }
            ))
        })?;

        let table_counts = match DeletionRepository::wipe_all_data(&transaction) {
            Ok(counts) => counts,
            Err(error) => {
                drop(transaction);
                let restore_failures = Self::restore_deleted_secrets(store, &deleted_secrets);
                return Err(AppError::database(format!(
                    "Full wipe database dibatalkan: {error}. Pemulihan credential: {}",
                    if restore_failures.is_empty() {
                        "berhasil".to_string()
                    } else {
                        format!("perlu pemeriksaan: {}", restore_failures.join(", "))
                    }
                )));
            }
        };

        if let Err(error) = transaction.commit() {
            let restore_failures = Self::restore_deleted_secrets(store, &deleted_secrets);
            return Err(AppError::database(format!(
                "Commit full wipe gagal: {error}. Pemulihan credential: {}",
                if restore_failures.is_empty() {
                    "berhasil".to_string()
                } else {
                    format!("perlu pemeriksaan: {}", restore_failures.join(", "))
                }
            )));
        }

        // Database and OS credential cleanup are complete. Clean capture files and report
        // any failures rather than silently claiming that all files were removed.
        let mut deleted_files_count = 0i64;
        let mut failed_files_count = 0i64;
        let mut cleanup_warnings = Vec::new();

        for path_str in temp_file_paths {
            let path = Path::new(&path_str);
            if path.exists() {
                match std::fs::remove_file(path) {
                    Ok(()) => deleted_files_count += 1,
                    Err(_) => {
                        failed_files_count += 1;
                        cleanup_warnings.push(
                            "Satu atau lebih file capture dari metadata tidak dapat dihapus."
                                .to_string(),
                        );
                    }
                }
            }
        }

        if let Some(data_dir) = app_data_dir {
            for subdirectory in &["captures", "temp_captures"] {
                let dir = data_dir.join(subdirectory);
                if !dir.exists() || !dir.is_dir() {
                    continue;
                }

                match std::fs::read_dir(&dir) {
                    Ok(entries) => {
                        for entry in entries {
                            match entry {
                                Ok(entry) if entry.path().is_file() => {
                                    if std::fs::remove_file(entry.path()).is_ok() {
                                        deleted_files_count += 1;
                                    } else {
                                        failed_files_count += 1;
                                    }
                                }
                                Ok(_) => {}
                                Err(_) => failed_files_count += 1,
                            }
                        }
                    }
                    Err(_) => failed_files_count += 1,
                }
            }
        }

        if failed_files_count > 0 {
            cleanup_warnings.push(format!(
                "{failed_files_count} file sementara tidak dapat dipastikan terhapus; periksa direktori data aplikasi."
            ));
        }

        let wal_status = DeletionRepository::wal_checkpoint_truncate(&connection)
            .unwrap_or_else(|error| format!("WAL checkpoint gagal: {error}"));
        let vacuum_status = DeletionRepository::vacuum(&connection)
            .unwrap_or_else(|error| format!("VACUUM gagal: {error}"));

        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let unaffected_external_items = vec![
            "Berkas ekspor JSON yang telah disimpan di luar direktori aplikasi oleh pengguna tidak terhapus otomatis.".to_string(),
            "Cadangan sistem eksternal atau backup drive di luar kendali aplikasi tidak terpengaruh.".to_string(),
            "Sisa alokasi blok fisik media penyimpanan tidak dijamin terhapus dengan cryptographic secure erase.".to_string(),
        ];

        Ok(WipeReport {
            deleted_personas_count: table_counts.personas,
            deleted_facts_count: table_counts.facts,
            deleted_sources_count: table_counts.sources,
            deleted_history_count: table_counts.history,
            deleted_sessions_count: table_counts.sessions,
            deleted_questions_count: table_counts.questions,
            deleted_recommendations_count: table_counts.recommendations,
            deleted_answers_count: table_counts.answers,
            deleted_preferences_count: table_counts.preferences,
            deleted_fit_results_count: table_counts.fit_results,
            deleted_attempts_count: table_counts.attempts,
            deleted_provider_configs_count: table_counts.provider_configs,
            deleted_credentials_count: table_counts.credentials,
            deleted_ocr_extractions_count: table_counts.ocr_extractions,
            deleted_recognitions_count: table_counts.recognitions,
            deleted_branch_transitions_count: table_counts.branch_transitions,
            deleted_question_predictions_count: table_counts.question_predictions,
            deleted_diagnostic_logs_count: table_counts.diagnostic_logs,
            deleted_temp_files_count: deleted_files_count,
            failed_temp_files_count: failed_files_count,
            cleanup_warnings,
            wal_checkpoint_status: wal_status,
            vacuum_status,
            wiped_at: now,
            unaffected_external_items,
        })
    }

    /// Menampilkan informasi kebijakan retensi data dan transparansi privasi.
    pub fn get_retention_policy_info() -> RetentionPolicyInfo {
        RetentionPolicyInfo {
            active_facts_policy: "Disimpan secara lokal di SQLite; hanya fakta relevan, terverifikasi, dan bukan fakta sensitif yang dapat dikirimkan ke model AI atas persetujuan pengguna.".into(),
            archived_facts_policy: "Disimpan secara lokal di SQLite namun tidak aktif; tidak pernah dikirim ke model AI atau digunakan sebagai bukti jawaban rekomendasi.".into(),
            screen_capture_policy: "Tangkapan layar diproses di memori/file sementara dan otomatis dihapus segera setelah OCR selesai, kecuali pengguna secara eksplisit menyimpannya.".into(),
            ocr_data_policy: "Hasil teks OCR disimpan sebagai pertanyaan survei lokal yang dapat diedit oleh pengguna. Gambar mentah tidak diunggah ke penyedia eksternal tanpa izin eksplisit.".into(),
            survey_history_policy: "Riwayat sesi dan pertanyaan survei disimpan secara terisolasi per persona untuk audit dan pengenalan survei (Survey Memory).".into(),
            export_file_policy: "Berkas ekspor JSON dibuat atas permintaan eksplisit pengguna; tidak menyertakan kunci API atau kredensial rahasia.".into(),
            logs_and_diagnostics_policy: "Log diagnostik lokal menerapkan redaksi otomatis untuk nilai sensitif dan kunci rahasia.".into(),
            deletion_scope_disclaimer: "Full wipe menghapus seluruh record SQLite dan secret yang dikelola aplikasi di Windows Credential Manager, membersihkan file capture yang terdaftar, serta menjalankan WAL checkpoint dan VACUUM. Jika file tidak dapat dihapus, aplikasi melaporkan jumlah dan peringatannya. Backup di luar direktori aplikasi tidak terhapus.".into(),
        }
    }
}
