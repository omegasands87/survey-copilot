use chrono::Utc;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::db::Database;
use crate::domain::classifier::QuestionCategory;
use crate::domain::error::AppError;
use crate::domain::evidence::SensitivityConsent;
use crate::domain::fact::Sensitivity;
use crate::domain::privacy::{
    CleanupTempFilesInput, DiagnosticsLogEntry, InspectOutboundDataInput, LogRedactor,
    OutboundDataDisclosure, OutboundFactDisclosure, PrivacySettings, RecordLogInput,
    RetentionCleanupResult, RetentionStatus, SystemDiagnosticsInfo, UpdatePrivacySettingsInput,
};
use crate::repository::{
    DeletionRepository, FactRepository, PersonaRepository, PrivacyRepository, ProviderRepository,
};
use crate::services::question_classifier::QuestionClassifierService;

pub struct PrivacyService;

impl PrivacyService {
    /// Mengambil konfigurasi privasi aplikasi
    pub fn get_privacy_settings(database: &Database) -> Result<PrivacySettings, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        PrivacyRepository::get_privacy_settings(&conn).map_err(AppError::from)
    }

    /// Memperbarui konfigurasi privasi aplikasi
    pub fn update_privacy_settings(
        database: &Database,
        input: UpdatePrivacySettingsInput,
    ) -> Result<PrivacySettings, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let mut current = PrivacyRepository::get_privacy_settings(&conn)?;

        if let Some(auto_redact) = input.auto_redact_logs {
            current.auto_redact_logs = auto_redact;
        }
        if let Some(req_sens) = input.require_sensitive_consent_prompt {
            current.require_sensitive_consent_prompt = req_sens;
        }
        if let Some(req_vis) = input.require_vision_consent_prompt {
            current.require_vision_consent_prompt = req_vis;
        }
        if let Some(policy) = input.screenshot_retention_policy {
            let valid = matches!(
                policy.as_str(),
                "immediate_delete" | "retain_1_hour" | "retain_24_hours" | "retain_manual_only"
            );
            if !valid {
                return Err(AppError::validation(
                    "screenshotRetentionPolicy",
                    "Kebijakan retensi screenshot harus salah satu dari: immediate_delete, retain_1_hour, retain_24_hours, retain_manual_only",
                ));
            }
            current.screenshot_retention_policy = policy;
        }
        if let Some(auto_clean) = input.auto_cleanup_temp_on_exit {
            current.auto_cleanup_temp_on_exit = auto_clean;
        }
        if let Some(lvl) = input.log_level {
            let valid = matches!(
                lvl.to_uppercase().as_str(),
                "INFO" | "WARN" | "ERROR" | "DEBUG"
            );
            if !valid {
                return Err(AppError::validation(
                    "logLevel",
                    "Log level harus salah satu dari: INFO, WARN, ERROR, DEBUG",
                ));
            }
            current.log_level = lvl.to_uppercase();
        }

        current.updated_at = Utc::now().to_rfc3339();

        PrivacyRepository::save_privacy_settings(&conn, &current)?;

        // Catat log diagnostik
        let _ = PrivacyRepository::insert_diagnostic_log(
            &conn,
            &DiagnosticsLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().to_rfc3339(),
                level: "INFO".to_string(),
                category: "SECURITY".to_string(),
                message: "Pengaturan privasi dan retensi aplikasi berhasil diperbarui".to_string(),
                metadata_json: Some(
                    serde_json::to_string(&serde_json::json!({
                        "autoRedactLogs": current.auto_redact_logs,
                        "retentionPolicy": current.screenshot_retention_policy,
                    }))
                    .unwrap_or_default(),
                ),
            },
        );

        Ok(current)
    }

    /// Melakukan inspeksi data keluar sebelum analisis AI (What Data Leaves The Device)
    pub fn inspect_outbound_data(
        database: &Database,
        input: InspectOutboundDataInput,
    ) -> Result<OutboundDataDisclosure, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        // 1. Verifikasi persona aktif
        let persona = PersonaRepository::find_by_id(&conn, &input.persona_id)
            .map_err(|e| AppError::database(format!("Gagal mencari persona: {e}")))?
            .ok_or_else(|| AppError::not_found("Persona tidak ditemukan"))?;

        if persona.status == "archived" {
            return Err(AppError::validation(
                "personaId",
                "Persona ini diarsipkan dan tidak dapat digunakan.",
            ));
        }

        // 2. Tentukan provider dan endpoint tujuan
        let provider_configs = ProviderRepository::list_enabled(&conn)?;
        let (p_type, p_name, endpoint, model) = if let Some(first) = provider_configs.first() {
            (
                first.provider_type.as_str().to_string(),
                first.display_name.clone(),
                first
                    .base_url
                    .as_deref()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or("https://generativelanguage.googleapis.com")
                    .to_string(),
                first.model_id.clone(),
            )
        } else {
            (
                "gemini".to_string(),
                "Google Gemini (Default)".to_string(),
                "https://generativelanguage.googleapis.com".to_string(),
                "gemini-2.5-flash".to_string(),
            )
        };

        // 3. Evaluasi klasifikasi dan fakta yang relevan
        let classification = QuestionClassifierService::classify_raw(
            &input.question_text,
            input.instructions.as_deref(),
            input.options.as_deref(),
        );

        let now_iso = Utc::now().to_rfc3339();
        let all_facts = FactRepository::list_facts_with_sources_by_persona(
            &conn,
            &input.persona_id,
            false,
            &now_iso,
        )
        .map_err(|e| AppError::database(format!("Gagal mengambil fakta persona: {e}")))?;

        let allow_sens = input.allow_sensitive.unwrap_or(false);
        let allow_highly_sens = input.allow_highly_sensitive.unwrap_or(false);
        let allow_vision = input.allow_external_vision.unwrap_or(false);
        let has_image = input.has_image_attachment.unwrap_or(false);

        let mut included_facts = Vec::new();
        let mut excluded_facts = Vec::new();
        let mut has_sensitive_facts = false;

        for fact in all_facts {
            let sens = Sensitivity::parse(&fact.sensitivity).unwrap_or(Sensitivity::Ordinary);
            let is_sens = matches!(sens, Sensitivity::Sensitive | Sensitivity::HighlySensitive);
            if is_sens {
                has_sensitive_facts = true;
            }

            // Cek relevansi kasar terhadap kategori pertanyaan
            let is_category_relevant = match classification.primary_category {
                QuestionCategory::Demographic => {
                    fact.category == "demographics"
                        || fact.category == "identity"
                        || fact.category == "household"
                }
                QuestionCategory::EmploymentEducation => {
                    fact.category == "employment"
                        || fact.category == "industry"
                        || fact.category == "organization"
                        || fact.category == "education"
                }
                QuestionCategory::FinancialEconomic => {
                    fact.category == "financial"
                        || fact.category == "income"
                        || fact.category == "banking"
                }
                QuestionCategory::AssetOwnership => {
                    fact.category == "technology"
                        || fact.category == "devices"
                        || fact.category == "assets"
                        || fact.category == "vehicles"
                }
                _ => true, // Kategori umum/behavioral dapat mereferensikan fakta relevan
            };

            let preview = if fact.normalized_value.len() > 60 {
                format!("{}...", &fact.normalized_value[..60])
            } else {
                fact.normalized_value.clone()
            };

            if is_category_relevant {
                match sens {
                    Sensitivity::Ordinary => {
                        included_facts.push(OutboundFactDisclosure {
                            fact_id: fact.id.clone(),
                            category: fact.category.clone(),
                            attribute: fact.attribute.clone(),
                            value_preview: preview,
                            sensitivity: "ordinary".to_string(),
                            status: "included".to_string(),
                            is_sensitive: false,
                            exclusion_reason: None,
                        });
                    }
                    Sensitivity::Sensitive => {
                        if allow_sens {
                            included_facts.push(OutboundFactDisclosure {
                                fact_id: fact.id.clone(),
                                category: fact.category.clone(),
                                attribute: fact.attribute.clone(),
                                value_preview: preview,
                                sensitivity: "sensitive".to_string(),
                                status: "included".to_string(),
                                is_sensitive: true,
                                exclusion_reason: None,
                            });
                        } else {
                            excluded_facts.push(OutboundFactDisclosure {
                                fact_id: fact.id.clone(),
                                category: fact.category.clone(),
                                attribute: fact.attribute.clone(),
                                value_preview: "[REDAKSI_SENSITIF]".to_string(),
                                sensitivity: "sensitive".to_string(),
                                status: "excluded_no_consent".to_string(),
                                is_sensitive: true,
                                exclusion_reason: Some(
                                    "Izin fakta sensitif tidak diaktifkan untuk permintaan ini"
                                        .to_string(),
                                ),
                            });
                        }
                    }
                    Sensitivity::HighlySensitive => {
                        if allow_highly_sens {
                            included_facts.push(OutboundFactDisclosure {
                                fact_id: fact.id.clone(),
                                category: fact.category.clone(),
                                attribute: fact.attribute.clone(),
                                value_preview: preview,
                                sensitivity: "highly_sensitive".to_string(),
                                status: "included".to_string(),
                                is_sensitive: true,
                                exclusion_reason: None,
                            });
                        } else {
                            excluded_facts.push(OutboundFactDisclosure {
                                fact_id: fact.id.clone(),
                                category: fact.category.clone(),
                                attribute: fact.attribute.clone(),
                                value_preview: "[REDAKSI_SANGAT_SENSITIF]".to_string(),
                                sensitivity: "highly_sensitive".to_string(),
                                status: "excluded_no_consent".to_string(),
                                is_sensitive: true,
                                exclusion_reason: Some(
                                    "Izin fakta sangat sensitif tidak diaktifkan untuk permintaan ini".to_string(),
                                ),
                            });
                        }
                    }
                }
            } else {
                excluded_facts.push(OutboundFactDisclosure {
                    fact_id: fact.id.clone(),
                    category: fact.category.clone(),
                    attribute: fact.attribute.clone(),
                    value_preview: preview,
                    sensitivity: sens.as_str().to_string(),
                    status: "excluded_not_relevant".to_string(),
                    is_sensitive: is_sens,
                    exclusion_reason: Some(
                        "Fakta tidak relevan dengan kategori pertanyaan survei ini".to_string(),
                    ),
                });
            }
        }

        let guarantees = vec![
            "Basis data profil lengkap persona tidak pernah dikirimkan ke model AI eksternal.".to_string(),
            "Fakta dan riwayat dari profil persona lain terisolasi penuh dan tidak akan pernah bocor.".to_string(),
            "Kunci API (API Key) dan kredensial rahasia disimpan di Windows Credential Manager dan tidak pernah disertakan dalam payload model.".to_string(),
            "Tangkapan layar penuh atau seluruh desktop tidak pernah dikirimkan ke model AI.".to_string(),
            "Riwayat clipboard dan aktivitas aplikasi di luar area survei tidak pernah diakses.".to_string(),
        ];

        Ok(OutboundDataDisclosure {
            provider_type: p_type,
            provider_name: p_name,
            destination_endpoint: endpoint,
            model_id: model,
            question_text: input.question_text,
            instructions: input.instructions,
            options: input.options,
            included_facts,
            excluded_facts,
            includes_sensitive_data: has_sensitive_facts,
            sensitive_consent_granted: allow_sens || allow_highly_sens,
            includes_external_vision: has_image,
            vision_consent_granted: allow_vision,
            not_transmitted_guarantees: guarantees,
            generated_at: Utc::now().to_rfc3339(),
        })
    }

    /// Penegakan perizinan eksplisit per-permintaan (Enforce Explicit Consent Policy)
    pub fn enforce_per_request_consent(
        consent: Option<&SensitivityConsent>,
        has_sensitive_data: bool,
        has_external_vision: bool,
    ) -> Result<(), AppError> {
        if has_external_vision {
            let vision_allowed = consent.map(|c| c.allow_external_vision).unwrap_or(false);
            if !vision_allowed {
                return Err(AppError::validation(
                    "consent",
                    "Analisis visual eksternal ditolak: izin eksplisit per-permintaan (allowExternalVision) tidak diberikan.",
                ));
            }
        }

        if has_sensitive_data {
            let sensitive_allowed = consent
                .map(|c| c.allow_sensitive || c.allow_highly_sensitive)
                .unwrap_or(false);
            if !sensitive_allowed {
                return Err(AppError::validation(
                    "consent",
                    "Pengiriman fakta profil sensitif ditolak: izin eksplisit per-permintaan tidak diberikan.",
                ));
            }
        }

        Ok(())
    }

    /// Mencatat log diagnostik secara aman (Otomatis Diredaksi)
    pub fn record_log(
        database: &Database,
        input: RecordLogInput,
    ) -> Result<DiagnosticsLogEntry, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        // 1. Terapkan sensor otomatis mutlak pada pesan dan metadata
        let redacted_message = LogRedactor::redact_text(&input.message);
        let redacted_metadata = input.metadata_json.map(|meta_str| {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                serde_json::to_string(&LogRedactor::redact_json(&json_val)).unwrap_or_default()
            } else {
                LogRedactor::redact_text(&meta_str)
            }
        });

        let entry = DiagnosticsLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            level: input.level.to_uppercase(),
            category: input.category.to_uppercase(),
            message: redacted_message,
            metadata_json: redacted_metadata,
        };

        PrivacyRepository::insert_diagnostic_log(&conn, &entry)?;

        Ok(entry)
    }

    /// Mengambil daftar riwayat log diagnostik yang teredaksi
    pub fn list_logs(
        database: &Database,
        limit: Option<usize>,
        level: Option<String>,
        category: Option<String>,
    ) -> Result<Vec<DiagnosticsLogEntry>, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        PrivacyRepository::list_diagnostic_logs(
            &conn,
            limit.unwrap_or(100),
            level.as_deref(),
            category.as_deref(),
        )
        .map_err(AppError::from)
    }

    /// Mengosongkan seluruh riwayat log diagnostik
    pub fn clear_logs(database: &Database) -> Result<usize, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        PrivacyRepository::clear_diagnostic_logs(&conn).map_err(AppError::from)
    }

    /// Mengambil status retensi file tangkapan layar dan direktori sementara
    pub fn get_retention_status(
        database: &Database,
        app_data_dir: Option<&Path>,
    ) -> Result<RetentionStatus, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let settings = PrivacyRepository::get_privacy_settings(&conn)?;

        let temp_dir = app_data_dir
            .map(|p| p.join("temp_captures"))
            .unwrap_or_else(|| Path::new("temp_captures").to_path_buf());

        let mut file_count = 0i64;
        let mut total_bytes = 0i64;
        let mut oldest_ts: Option<String> = None;

        if temp_dir.exists() && temp_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        file_count += 1;
                        if let Ok(meta) = path.metadata() {
                            total_bytes += meta.len() as i64;
                            if let Ok(created) = meta.created().or_else(|_| meta.modified()) {
                                let dt: chrono::DateTime<Utc> = created.into();
                                let iso = dt.to_rfc3339();
                                if oldest_ts.as_ref().map(|o| o > &iso).unwrap_or(true) {
                                    oldest_ts = Some(iso);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(RetentionStatus {
            temp_directory_path: temp_dir.to_string_lossy().to_string(),
            temp_file_count: file_count,
            temp_total_bytes: total_bytes,
            oldest_file_timestamp: oldest_ts,
            active_retention_policy: settings.screenshot_retention_policy,
            auto_cleanup_on_exit: settings.auto_cleanup_temp_on_exit,
        })
    }

    /// Membersihkan file tangkapan layar sementara berdasarkan batasan usia
    pub fn cleanup_temp_files(
        database: &Database,
        app_data_dir: Option<&Path>,
        input: CleanupTempFilesInput,
    ) -> Result<RetentionCleanupResult, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let temp_dir = app_data_dir
            .map(|p| p.join("temp_captures"))
            .unwrap_or_else(|| Path::new("temp_captures").to_path_buf());

        let max_age_seconds = input.max_age_seconds.unwrap_or(0);
        let mut deleted_count = 0i64;
        let mut freed_bytes = 0i64;

        if temp_dir.exists() && temp_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&temp_dir) {
                let now = std::time::SystemTime::now();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let should_delete = if max_age_seconds <= 0 {
                            true
                        } else if let Ok(meta) = path.metadata() {
                            let file_time =
                                meta.created().or_else(|_| meta.modified()).unwrap_or(now);
                            if let Ok(elapsed) = now.duration_since(file_time) {
                                elapsed.as_secs() >= max_age_seconds as u64
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if should_delete {
                            let file_size = path.metadata().map(|m| m.len() as i64).unwrap_or(0);
                            if fs::remove_file(&path).is_ok() {
                                deleted_count += 1;
                                freed_bytes += file_size;
                            }
                        }
                    }
                }
            }
        }

        let now_str = Utc::now().to_rfc3339();
        let message = format!(
            "Pembersihan berhasil: {deleted_count} berkas sementara dihapus ({freed_bytes} byte dibebaskan)."
        );

        // Catat ke log diagnostik
        let _ = PrivacyRepository::insert_diagnostic_log(
            &conn,
            &DiagnosticsLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: now_str.clone(),
                level: "INFO".to_string(),
                category: "DATA_RETENTION".to_string(),
                message: message.clone(),
                metadata_json: Some(
                    serde_json::to_string(&serde_json::json!({
                        "deletedCount": deleted_count,
                        "freedBytes": freed_bytes,
                    }))
                    .unwrap_or_default(),
                ),
            },
        );

        Ok(RetentionCleanupResult {
            deleted_files_count: deleted_count,
            freed_bytes,
            cleaned_at: now_str,
            message,
        })
    }

    /// Mengambil metrik diagnostik sistem secara komprehensif
    pub fn get_system_diagnostics(
        database: &Database,
        app_data_dir: Option<&Path>,
    ) -> Result<SystemDiagnosticsInfo, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let (sqlite_ver, db_size, wal_size) = PrivacyRepository::get_sqlite_stats(&conn)?;
        let counts = DeletionRepository::count_all_records(&conn)?;

        let temp_dir = app_data_dir
            .map(|p| p.join("temp_captures"))
            .unwrap_or_else(|| Path::new("temp_captures").to_path_buf());

        let mut temp_count = 0i64;
        let mut temp_size = 0i64;
        if temp_dir.exists() && temp_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        temp_count += 1;
                        if let Ok(m) = path.metadata() {
                            temp_size += m.len() as i64;
                        }
                    }
                }
            }
        }

        let schema_ver: i64 = conn
            .query_row("SELECT MAX(version) FROM _migrations", [], |r| r.get(0))
            .unwrap_or(13);

        Ok(SystemDiagnosticsInfo {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_platform: std::env::consts::OS.to_string(),
            db_schema_version: schema_ver,
            sqlite_version: sqlite_ver,
            wal_size_bytes: wal_size,
            total_db_size_bytes: db_size,
            credential_store_type: "WindowsCredentialManager".to_string(),
            temp_captures_count: temp_count,
            temp_captures_size_bytes: temp_size,
            active_personas_count: counts.personas,
            total_facts_count: counts.facts,
            total_sessions_count: counts.sessions,
            provider_configs_count: counts.provider_configs,
            generated_at: Utc::now().to_rfc3339(),
        })
    }
}
