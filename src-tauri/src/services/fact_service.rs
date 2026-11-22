use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::fact::{
    normalize_typed_value, AddFactSourceInput, ConflictWarning, CreateFactInput, DataType,
    FactSource, FactSummary, ProfileFactDetail, Sensitivity, UpdateFactInput, VerificationStatus,
};
use crate::repository::{FactRepository, PersonaRepository};
use uuid::Uuid;

pub struct FactService;

impl FactService {
    /// Mengambil daftar ringkasan fakta milik persona tertentu.
    pub fn list(
        database: &Database,
        persona_id: &str,
        include_archived: bool,
    ) -> Result<Vec<FactSummary>, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let now = chrono_timestamp();
        FactRepository::list_by_persona(&connection, persona_id, include_archived, &now)
            .map_err(AppError::from)
    }

    /// Mengambil detail lengkap fakta beserta seluruh sumber bukti dan riwayat perubahannya.
    pub fn get_detail(
        database: &Database,
        persona_id: &str,
        fact_id: &str,
    ) -> Result<ProfileFactDetail, AppError> {
        let persona_id = persona_id.trim();
        let fact_id = fact_id.trim();
        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan ID fakta wajib diisi.",
            ));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let now = chrono_timestamp();
        FactRepository::get_detail(&connection, persona_id, fact_id, &now)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Fakta profil tidak ditemukan."))
    }

    /// Membuat fakta baru beserta sumber bukti dan entri riwayat awal secara atomik.
    pub fn create(database: &Database, input: CreateFactInput) -> Result<FactSummary, AppError> {
        let validated = input.validate()?;

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|error| AppError::database(format!("Gagal memulai transaksi: {error}")))?;

        let persona_exists =
            PersonaRepository::is_active_and_exists(&transaction, &validated.persona_id)
                .map_err(AppError::from)?;
        if !persona_exists {
            return Err(AppError::not_found(
                "Profil tidak ditemukan atau sedang diarsipkan.",
            ));
        }

        let fact_id = Uuid::new_v4().to_string();
        let source_id = Uuid::new_v4().to_string();
        let history_id = Uuid::new_v4().to_string();
        let now = chrono_timestamp();

        FactRepository::insert_fact_and_source(
            &transaction,
            &validated.persona_id,
            &fact_id,
            &source_id,
            &history_id,
            &validated,
            &now,
        )
        .map_err(AppError::from)?;

        transaction.commit().map_err(|error| {
            AppError::database(format!("Gagal menyimpan informasi fakta: {error}"))
        })?;

        let detail = FactRepository::get_detail(&connection, &validated.persona_id, &fact_id, &now)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Fakta yang baru disimpan tidak ditemukan."))?;

        Ok(FactSummary {
            id: detail.id,
            persona_id: detail.persona_id,
            category: detail.category,
            fact_key: detail.fact_key,
            attribute: detail.attribute,
            data_type: detail.data_type,
            value: detail.value,
            normalized_value: detail.normalized_value,
            unit: detail.unit,
            verification_status: detail.verification_status,
            sensitivity: detail.sensitivity,
            source_description: detail.sources.first().map(|s| s.title.clone()),
            valid_from: detail.valid_from,
            valid_until: detail.valid_until,
            is_valid: detail.is_valid,
            has_conflict: false,
            sources_count: detail.sources.len() as i64,
            created_at: detail.created_at,
            updated_at: detail.updated_at,
            archived_at: detail.archived_at,
        })
    }

    /// Memperbarui nilai atau metadata fakta dan mencatat riwayat perubahan sebelumnya.
    pub fn update(
        database: &Database,
        input: UpdateFactInput,
    ) -> Result<ProfileFactDetail, AppError> {
        let persona_id = input.persona_id.trim();
        let fact_id = input.fact_id.trim();
        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan ID fakta wajib diisi.",
            ));
        }

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        let now = chrono_timestamp();
        let current = FactRepository::get_detail(&transaction, persona_id, fact_id, &now)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Fakta profil tidak ditemukan."))?;

        let dt = match input.data_type.as_deref() {
            Some(s) => DataType::parse(s)?,
            None => DataType::parse(&current.data_type)?,
        };

        let (new_value_json_str, new_normalized_val) = match input.value {
            Some(ref raw) => {
                let (val, norm) = normalize_typed_value(dt, raw)?;
                let json_str = serde_json::to_string(&val).unwrap_or(norm.clone());
                (Some(json_str), Some(norm))
            }
            None => (None, None),
        };

        let history_id = Uuid::new_v4().to_string();
        let updated = FactRepository::update_fact_value_with_history(
            &transaction,
            persona_id,
            fact_id,
            &history_id,
            input.category.as_deref().map(str::trim),
            input.attribute.as_deref().map(str::trim),
            Some(dt.as_str()),
            new_value_json_str.as_deref(),
            new_normalized_val.as_deref(),
            input.unit.as_deref().map(str::trim),
            input.valid_from.as_deref().map(str::trim),
            input.valid_until.as_deref().map(str::trim),
            input.notes.as_deref().map(str::trim),
            input.change_reason.as_deref().map(str::trim),
            &now,
        )
        .map_err(AppError::from)?;

        if updated == 0 {
            return Err(AppError::not_found("Fakta profil tidak ditemukan."));
        }

        // Jika ada status verifikasi yang disertakan pada update
        if let Some(ref st) = input.verification_status {
            let parsed_status = VerificationStatus::parse(st)?;
            let status_hist_id = Uuid::new_v4().to_string();
            FactRepository::update_status_with_history(
                &transaction,
                persona_id,
                fact_id,
                &status_hist_id,
                parsed_status.as_str(),
                &now,
            )
            .map_err(AppError::from)?;
        }

        // Jika ada sensitivitas yang disertakan pada update
        if let Some(ref sens) = input.sensitivity {
            let parsed_sens = Sensitivity::parse(sens)?;
            FactRepository::update_sensitivity(
                &transaction,
                persona_id,
                fact_id,
                parsed_sens.as_str(),
                &now,
            )
            .map_err(AppError::from)?;
        }

        transaction
            .commit()
            .map_err(|e| AppError::database(format!("Gagal menyimpan pembaruan fakta: {e}")))?;

        FactRepository::get_detail(&connection, persona_id, fact_id, &now)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Fakta profil tidak ditemukan setelah diperbarui."))
    }

    /// Memperbarui status verifikasi fakta (confirmed, unverified, disputed, outdated).
    /// Tindakan ini mencatat perubahan pada fact_history.
    pub fn set_status(
        database: &Database,
        persona_id: &str,
        fact_id: &str,
        status: &str,
    ) -> Result<ProfileFactDetail, AppError> {
        let persona_id = persona_id.trim();
        let fact_id = fact_id.trim();

        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan ID fakta wajib diisi.",
            ));
        }

        let verification_status = VerificationStatus::parse(status)?;

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        let now = chrono_timestamp();
        let history_id = Uuid::new_v4().to_string();

        let changed = FactRepository::update_status_with_history(
            &transaction,
            persona_id,
            fact_id,
            &history_id,
            verification_status.as_str(),
            &now,
        )
        .map_err(AppError::from)?;

        if changed == 0 {
            return Err(AppError::not_found("Fakta profil tidak ditemukan."));
        }

        transaction
            .commit()
            .map_err(|e| AppError::database(format!("Gagal memperbarui status fakta: {e}")))?;

        FactRepository::get_detail(&connection, persona_id, fact_id, &now)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Fakta tidak ditemukan setelah status diperbarui."))
    }

    /// Memperbarui klasifikasi sensitivitas fakta (ordinary, sensitive, highly_sensitive).
    pub fn set_sensitivity(
        database: &Database,
        persona_id: &str,
        fact_id: &str,
        sensitivity: &str,
    ) -> Result<(), AppError> {
        let persona_id = persona_id.trim();
        let fact_id = fact_id.trim();
        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan fakta wajib diisi.",
            ));
        }

        let sens = Sensitivity::parse(sensitivity)?;
        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let now = chrono_timestamp();
        let changed = FactRepository::update_sensitivity(
            &connection,
            persona_id,
            fact_id,
            sens.as_str(),
            &now,
        )
        .map_err(AppError::from)?;

        if changed == 0 {
            return Err(AppError::not_found("Fakta profil tidak ditemukan."));
        }

        Ok(())
    }

    /// Menambahkan sumber bukti baru ke fakta yang sudah ada.
    pub fn add_source(
        database: &Database,
        input: AddFactSourceInput,
    ) -> Result<FactSource, AppError> {
        let persona_id = input.persona_id.trim();
        let fact_id = input.fact_id.trim();
        let title = input.title.trim();
        let desc = input.description.trim();

        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan fakta wajib diisi.",
            ));
        }
        if title.is_empty() {
            return Err(AppError::validation(
                "title",
                "Judul sumber bukti wajib diisi.",
            ));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let source_id = Uuid::new_v4().to_string();
        let now = chrono_timestamp();

        FactRepository::add_source(
            &connection,
            &source_id,
            persona_id,
            fact_id,
            &input.source_type,
            title,
            desc,
            input.reference_uri.as_deref(),
            input.observed_at.as_deref(),
            input.valid_from.as_deref(),
            input.valid_until.as_deref(),
            input.notes.as_deref(),
            &now,
        )
        .map_err(AppError::from)?;

        Ok(FactSource {
            id: source_id,
            persona_id: persona_id.to_string(),
            fact_id: fact_id.to_string(),
            source_type: input.source_type,
            title: title.to_string(),
            description: desc.to_string(),
            reference_uri: input.reference_uri,
            observed_at: input.observed_at,
            valid_from: input.valid_from,
            valid_until: input.valid_until,
            notes: input.notes,
            created_at: now,
        })
    }

    /// Mengarsipkan fakta secara aman.
    pub fn archive(database: &Database, persona_id: &str, fact_id: &str) -> Result<(), AppError> {
        let persona_id = persona_id.trim();
        let fact_id = fact_id.trim();
        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan fakta wajib diisi.",
            ));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let now = chrono_timestamp();
        let changed = FactRepository::archive(&connection, persona_id, fact_id, &now)
            .map_err(AppError::from)?;

        if changed == 0 {
            return Err(AppError::not_found(
                "Fakta profil tidak ditemukan atau sudah diarsipkan.",
            ));
        }

        Ok(())
    }

    /// Memulihkan fakta dari arsip.
    pub fn restore(database: &Database, persona_id: &str, fact_id: &str) -> Result<(), AppError> {
        let persona_id = persona_id.trim();
        let fact_id = fact_id.trim();
        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan fakta wajib diisi.",
            ));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let now = chrono_timestamp();
        let changed = FactRepository::restore(&connection, persona_id, fact_id, &now)
            .map_err(AppError::from)?;

        if changed == 0 {
            return Err(AppError::not_found(
                "Fakta profil tidak ditemukan atau tidak sedang diarsipkan.",
            ));
        }

        Ok(())
    }

    /// Menghapus fakta secara permanen.
    pub fn delete(database: &Database, persona_id: &str, fact_id: &str) -> Result<(), AppError> {
        let persona_id = persona_id.trim();
        let fact_id = fact_id.trim();

        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan ID fakta wajib diisi.",
            ));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let changed =
            FactRepository::delete(&connection, persona_id, fact_id).map_err(|error| {
                AppError::conflict(format!(
                    "Fakta ini masih dipakai oleh rekomendasi riwayat dan tidak dapat dihapus: {error}"
                ))
            })?;

        if changed == 0 {
            return Err(AppError::not_found("Fakta profil tidak ditemukan."));
        }

        Ok(())
    }

    /// Mendeteksi konflik antar-fakta pada profil persona aktif.
    pub fn detect_conflicts(
        database: &Database,
        persona_id: &str,
    ) -> Result<Vec<ConflictWarning>, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        FactRepository::find_conflicts(&connection, persona_id).map_err(AppError::from)
    }

    /// Mengambil daftar fakta yang layak digunakan sebagai bukti untuk AI recommendation.
    ///
    /// Aturan:
    /// 1. Berstatus 'confirmed'.
    /// 2. Masih dalam masa berlaku (valid_until tidak lewat).
    /// 3. Tidak sedang dalam kondisi konflik nilai.
    /// 4. Sensitivitas dilindungi (fakta sensitive/highly_sensitive ditolak jika tanpa izin eksplisit).
    pub fn get_eligible_facts_for_ai(
        database: &Database,
        persona_id: &str,
        allow_sensitive: bool,
    ) -> Result<Vec<FactSummary>, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let now = chrono_timestamp();
        FactRepository::list_eligible_for_ai(&connection, persona_id, allow_sensitive, &now)
            .map_err(AppError::from)
    }
}

fn chrono_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::persona::CreatePersonaInput;
    use crate::services::PersonaService;
    use std::path::Path;

    fn setup_test_db() -> Database {
        Database::initialize(Path::new(":memory:")).expect("memory database should initialize")
    }

    #[test]
    fn test_create_typed_facts_various_types() {
        let db = setup_test_db();
        let persona = PersonaService::create(&db, CreatePersonaInput::new("Budi")).expect("p");

        // 1. String fact
        let f_str = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Demografi".into(),
                fact_key: Some("city".into()),
                attribute: "Kota Domisili".into(),
                data_type: Some("string".into()),
                value: "Surabaya".into(),
                unit: None,
                source_type: Some("manual_entry".into()),
                source_title: Some("KTP".into()),
                source_description: "Dokumen KTP asli".into(),
                reference_uri: None,
                observed_at: Some("2024-01-01".into()),
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("confirmed".into()),
                sensitivity: Some("ordinary".into()),
            },
        )
        .expect("create string fact");
        assert_eq!(f_str.data_type, "string");
        assert_eq!(f_str.value, serde_json::json!("Surabaya"));
        assert_eq!(f_str.normalized_value, "surabaya");

        // 2. Number fact
        let f_num = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Finansial".into(),
                fact_key: Some("monthly_income".into()),
                attribute: "Penghasilan Bulanan".into(),
                data_type: Some("number".into()),
                value: "15.000.000".into(),
                unit: Some("IDR".into()),
                source_type: Some("uploaded_document".into()),
                source_title: Some("Slip Gaji".into()),
                source_description: "Slip gaji bulan terakhir".into(),
                reference_uri: None,
                observed_at: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("confirmed".into()),
                sensitivity: Some("sensitive".into()),
            },
        )
        .expect("create number fact");
        assert_eq!(f_num.data_type, "number");
        assert_eq!(f_num.value, serde_json::json!(15000000));
        assert_eq!(f_num.unit, Some("IDR".into()));

        // 3. Boolean fact
        let f_bool = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Transportasi".into(),
                fact_key: Some("owns_car".into()),
                attribute: "Kepemilikan Mobil".into(),
                data_type: Some("boolean".into()),
                value: "ya".into(),
                unit: None,
                source_type: Some("user_declaration".into()),
                source_title: Some("Pernyataan".into()),
                source_description: "Pengguna menyatakan memiliki mobil".into(),
                reference_uri: None,
                observed_at: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("confirmed".into()),
                sensitivity: Some("ordinary".into()),
            },
        )
        .expect("create boolean fact");
        assert_eq!(f_bool.data_type, "boolean");
        assert_eq!(f_bool.value, serde_json::json!(true));
        assert_eq!(f_bool.normalized_value, "true");
    }

    #[test]
    fn test_verification_lifecycle_and_history_tracking() {
        let db = setup_test_db();
        let persona = PersonaService::create(&db, CreatePersonaInput::new("Dewi")).expect("p");

        // Newly created fact defaults to unverified
        let fact = FactService::create(
            &db,
            CreateFactInput::new(
                &persona.id,
                "Pekerjaan",
                "Profesi",
                "Desainer Grafis",
                "Input",
            ),
        )
        .expect("create");
        assert_eq!(fact.verification_status, "unverified");

        let detail1 = FactService::get_detail(&db, &persona.id, &fact.id).expect("detail1");
        assert_eq!(detail1.history.len(), 1);
        assert_eq!(detail1.history[0].new_status, "unverified");

        // User explicitly confirms fact
        let confirmed =
            FactService::set_status(&db, &persona.id, &fact.id, "confirmed").expect("confirm");
        assert_eq!(confirmed.verification_status, "confirmed");

        let detail2 = FactService::get_detail(&db, &persona.id, &fact.id).expect("detail2");
        assert_eq!(detail2.history.len(), 2);
        assert_eq!(
            detail2.history[0].previous_status,
            Some("unverified".into())
        );
        assert_eq!(detail2.history[0].new_status, "confirmed");

        // Update value with reason: retains previous value in history
        FactService::update(
            &db,
            UpdateFactInput {
                persona_id: persona.id.clone(),
                fact_id: fact.id.clone(),
                category: None,
                attribute: None,
                data_type: None,
                value: Some("Product Designer".into()),
                unit: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                change_reason: Some("Promosi jabatan terbaru".into()),
                verification_status: None,
                sensitivity: None,
            },
        )
        .expect("update value");

        let detail3 = FactService::get_detail(&db, &persona.id, &fact.id).expect("detail3");
        assert_eq!(detail3.value, serde_json::json!("Product Designer"));
        assert_eq!(detail3.history.len(), 3);
        assert_eq!(
            detail3.history[0].change_reason,
            Some("Promosi jabatan terbaru".into())
        );
    }

    #[test]
    fn test_conflict_detection_between_contradictory_facts() {
        let db = setup_test_db();
        let persona = PersonaService::create(&db, CreatePersonaInput::new("Eko")).expect("p");

        // Fact 1: Status Pekerjaan = Karyawan Tetap
        let _f1 = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Pekerjaan".into(),
                fact_key: Some("status_pekerjaan".into()),
                attribute: "Status Pekerjaan".into(),
                data_type: Some("string".into()),
                value: "Karyawan Tetap".into(),
                unit: None,
                source_type: None,
                source_title: None,
                source_description: "Surat Kontrak".into(),
                reference_uri: None,
                observed_at: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("confirmed".into()),
                sensitivity: Some("ordinary".into()),
            },
        )
        .expect("f1");

        // Fact 2: Status Pekerjaan = Wirausaha (Conflicting active fact on same key!)
        let _f2 = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Pekerjaan".into(),
                fact_key: Some("status_pekerjaan".into()),
                attribute: "Status Pekerjaan".into(),
                data_type: Some("string".into()),
                value: "Wirausaha".into(),
                unit: None,
                source_type: None,
                source_title: None,
                source_description: "Deklarasi".into(),
                reference_uri: None,
                observed_at: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("confirmed".into()),
                sensitivity: Some("ordinary".into()),
            },
        )
        .expect("f2");

        let conflicts = FactService::detect_conflicts(&db, &persona.id).expect("conflicts");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].fact_key, "status_pekerjaan");

        // When listing facts, has_conflict flag must be true
        let list = FactService::list(&db, &persona.id, false).expect("list");
        assert!(list.iter().all(|f| f.has_conflict));
    }

    #[test]
    fn test_ai_eligibility_policy_filters_unqualified_facts() {
        let db = setup_test_db();
        let persona = PersonaService::create(&db, CreatePersonaInput::new("Sari")).expect("p");

        // 1. Confirmed ordinary fact -> ELIGIBLE
        let _f_ok = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Demografi".into(),
                fact_key: Some("umur".into()),
                attribute: "Usia".into(),
                data_type: Some("number".into()),
                value: "25".into(),
                unit: Some("tahun".into()),
                source_type: None,
                source_title: None,
                source_description: "KTP".into(),
                reference_uri: None,
                observed_at: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("confirmed".into()),
                sensitivity: Some("ordinary".into()),
            },
        )
        .expect("f_ok");

        // 2. Unverified fact -> INELIGIBLE FOR AI
        let _f_unverified = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Hobi".into(),
                fact_key: Some("hobi".into()),
                attribute: "Hobi Utama".into(),
                data_type: Some("string".into()),
                value: "Fotografi".into(),
                unit: None,
                source_type: None,
                source_title: None,
                source_description: "Input".into(),
                reference_uri: None,
                observed_at: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("unverified".into()),
                sensitivity: Some("ordinary".into()),
            },
        )
        .expect("f_unverified");

        // 3. Disputed fact -> INELIGIBLE FOR AI
        let f_disputed = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Demografi".into(),
                fact_key: Some("golongan_darah".into()),
                attribute: "Golongan Darah".into(),
                data_type: Some("string".into()),
                value: "O+".into(),
                unit: None,
                source_type: None,
                source_title: None,
                source_description: "Input".into(),
                reference_uri: None,
                observed_at: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("confirmed".into()),
                sensitivity: Some("ordinary".into()),
            },
        )
        .expect("f_disputed");
        FactService::set_status(&db, &persona.id, &f_disputed.id, "disputed")
            .expect("set disputed");

        // 4. Sensitive fact -> INELIGIBLE WITHOUT CONSENT
        let _f_sensitive = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Finansial".into(),
                fact_key: Some("gaji".into()),
                attribute: "Penghasilan".into(),
                data_type: Some("number".into()),
                value: "10000000".into(),
                unit: Some("IDR".into()),
                source_type: None,
                source_title: None,
                source_description: "Slip Gaji".into(),
                reference_uri: None,
                observed_at: None,
                valid_from: None,
                valid_until: None,
                notes: None,
                verification_status: Some("confirmed".into()),
                sensitivity: Some("sensitive".into()),
            },
        )
        .expect("f_sensitive");

        // Case A: allow_sensitive = false
        let eligible_no_consent =
            FactService::get_eligible_facts_for_ai(&db, &persona.id, false).expect("eligible");
        assert_eq!(eligible_no_consent.len(), 1);
        assert_eq!(eligible_no_consent[0].attribute, "Usia");

        // Case B: allow_sensitive = true (User gave explicit consent for this request)
        let eligible_with_consent =
            FactService::get_eligible_facts_for_ai(&db, &persona.id, true).expect("eligible");
        assert_eq!(eligible_with_consent.len(), 2);
        let attrs: Vec<_> = eligible_with_consent
            .iter()
            .map(|f| f.attribute.as_str())
            .collect();
        assert!(attrs.contains(&"Usia"));
        assert!(attrs.contains(&"Penghasilan"));
    }
}
