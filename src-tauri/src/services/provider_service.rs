use crate::db::Database;
use crate::domain::endpoint_validator::EndpointValidator;
use crate::domain::error::AppError;
use crate::domain::provider::{
    AdapterType, ConnectionTestResult, CreateProviderConfigInput, ModelOption, NormalizedError,
    NormalizedRequest, NormalizedResponse, ProviderConfig, ProviderConfigSummary, ProviderType,
    UpdateProviderConfigInput,
};
use crate::infrastructure::adapters::{get_adapter, GeminiAdapter, OpenAiCompatibleAdapter};
use crate::infrastructure::credentials::CredentialStore;
use crate::infrastructure::http::HttpTransport;
use crate::repository::{CredentialRepository, ProviderRepository};
use crate::services::CredentialService;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ProviderService;

impl ProviderService {
    /// Validasi tipe provider dan adapter
    fn validate_types(provider_type: &str, adapter_type: &str) -> Result<(), AppError> {
        if ProviderType::from_str_loose(provider_type).is_none() {
            return Err(AppError::validation(
                "providerType",
                format!("Tipe provider '{provider_type}' tidak valid. Pilih dari: gemini, openai_compatible, anthropic_compatible, custom_http."),
            ));
        }

        if AdapterType::from_str_loose(adapter_type).is_none() {
            return Err(AppError::validation(
                "adapterType",
                format!("Tipe adapter '{adapter_type}' tidak valid. Pilih dari: gemini, openai_compatible, anthropic_compatible, custom_http."),
            ));
        }

        Ok(())
    }

    /// Membuat konfigurasi provider baru di basis data lokal
    pub fn create_provider_config(
        database: &Database,
        input: CreateProviderConfigInput,
    ) -> Result<ProviderConfigSummary, AppError> {
        let display_name = input.display_name.trim();
        if display_name.is_empty() || display_name.len() > 100 {
            return Err(AppError::validation(
                "displayName",
                "Nama konfigurasi provider wajib diisi (antara 1 hingga 100 karakter).",
            ));
        }

        let provider_type = input.provider_type.trim();
        let adapter_type = input.adapter_type.trim();
        Self::validate_types(provider_type, adapter_type)?;

        let model_id = input.model_id.trim();
        if model_id.is_empty() || model_id.len() > 100 {
            return Err(AppError::validation(
                "modelId",
                "Model ID wajib diisi (antara 1 hingga 100 karakter).",
            ));
        }

        let credential_ref = input.credential_ref.trim();
        if !credential_ref.starts_with("survey_copilot/") {
            return Err(AppError::validation(
                "credentialRef",
                "Format referensi kredensial tidak valid.",
            ));
        }

        let base_url = match input.base_url {
            Some(ref url) if !url.trim().is_empty() => {
                let validated = EndpointValidator::validate_url(url)?;
                Some(validated)
            }
            _ => None,
        };

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        // Verifikasi bahwa credential_ref terdaftar di tabel app_credentials
        let cred = CredentialRepository::find_by_ref(&conn, credential_ref)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::validation(
                    "credentialRef",
                    "Kredensial yang dipilih tidak ditemukan dalam daftar kredensial tersimpan.",
                )
            })?;

        let id = match input.id {
            Some(ref custom_id) if !custom_id.trim().is_empty() => custom_id.trim().to_string(),
            _ => Uuid::new_v4().to_string(),
        };

        let now = Utc::now().to_rfc3339();
        let priority = input.priority.unwrap_or(100).clamp(1, 1000);
        let weight = input.weight.unwrap_or(1).max(1);
        let timeout_ms = input.timeout_ms.unwrap_or(30000).clamp(1000, 120000);
        let max_concurrency = input.max_concurrency.unwrap_or(1).clamp(1, 50);

        let config = ProviderConfig {
            id: id.clone(),
            display_name: display_name.to_string(),
            provider_type: provider_type.to_string(),
            base_url,
            adapter_type: adapter_type.to_string(),
            model_id: model_id.to_string(),
            credential_ref: credential_ref.to_string(),
            enabled: input.enabled.unwrap_or(true),
            priority,
            weight,
            timeout_ms,
            max_concurrency,
            rate_limit_hint_json: input.rate_limit_hint_json,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        ProviderRepository::create(&conn, &config).map_err(AppError::from)?;

        Ok(ProviderConfigSummary {
            id: config.id,
            display_name: config.display_name,
            provider_type: config.provider_type,
            base_url: config.base_url,
            adapter_type: config.adapter_type,
            model_id: config.model_id,
            credential_ref: config.credential_ref,
            credential_label: Some(cred.label),
            enabled: config.enabled,
            priority: config.priority,
            weight: config.weight,
            timeout_ms: config.timeout_ms,
            max_concurrency: config.max_concurrency,
            rate_limit_hint_json: config.rate_limit_hint_json,
            created_at: config.created_at,
            updated_at: config.updated_at,
        })
    }

    /// Mengambil konfigurasi provider berdasarkan ID
    pub fn get_provider_config(
        database: &Database,
        id: &str,
    ) -> Result<ProviderConfigSummary, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let config = ProviderRepository::find_by_id(&conn, id)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::not_found(format!("Konfigurasi provider '{id}' tidak ditemukan."))
            })?;

        let cred = CredentialRepository::find_by_ref(&conn, &config.credential_ref)
            .map_err(AppError::from)?;

        Ok(ProviderConfigSummary {
            id: config.id,
            display_name: config.display_name,
            provider_type: config.provider_type,
            base_url: config.base_url,
            adapter_type: config.adapter_type,
            model_id: config.model_id,
            credential_ref: config.credential_ref,
            credential_label: cred.map(|c| c.label),
            enabled: config.enabled,
            priority: config.priority,
            weight: config.weight,
            timeout_ms: config.timeout_ms,
            max_concurrency: config.max_concurrency,
            rate_limit_hint_json: config.rate_limit_hint_json,
            created_at: config.created_at,
            updated_at: config.updated_at,
        })
    }

    /// Mengambil seluruh konfigurasi provider yang tersimpan
    pub fn list_provider_configs(
        database: &Database,
    ) -> Result<Vec<ProviderConfigSummary>, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let configs = ProviderRepository::list_all(&conn).map_err(AppError::from)?;
        let mut summaries = Vec::with_capacity(configs.len());

        for c in configs {
            let cred = CredentialRepository::find_by_ref(&conn, &c.credential_ref)
                .map_err(AppError::from)?;

            summaries.push(ProviderConfigSummary {
                id: c.id,
                display_name: c.display_name,
                provider_type: c.provider_type,
                base_url: c.base_url,
                adapter_type: c.adapter_type,
                model_id: c.model_id,
                credential_ref: c.credential_ref,
                credential_label: cred.map(|cr| cr.label),
                enabled: c.enabled,
                priority: c.priority,
                weight: c.weight,
                timeout_ms: c.timeout_ms,
                max_concurrency: c.max_concurrency,
                rate_limit_hint_json: c.rate_limit_hint_json,
                created_at: c.created_at,
                updated_at: c.updated_at,
            });
        }

        Ok(summaries)
    }

    /// Memperbarui konfigurasi provider yang ada
    pub fn update_provider_config(
        database: &Database,
        input: UpdateProviderConfigInput,
    ) -> Result<ProviderConfigSummary, AppError> {
        let id = input.id.trim();
        if id.is_empty() {
            return Err(AppError::validation("id", "ID provider wajib diisi."));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let mut existing = ProviderRepository::find_by_id(&conn, id)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::not_found(format!("Konfigurasi provider '{id}' tidak ditemukan."))
            })?;

        if let Some(ref name) = input.display_name {
            let trimmed = name.trim();
            if trimmed.is_empty() || trimmed.len() > 100 {
                return Err(AppError::validation(
                    "displayName",
                    "Nama provider harus antara 1 hingga 100 karakter.",
                ));
            }
            existing.display_name = trimmed.to_string();
        }

        if let Some(ref base_url_opt) = input.base_url {
            match base_url_opt {
                Some(ref url) if !url.trim().is_empty() => {
                    let validated = EndpointValidator::validate_url(url)?;
                    existing.base_url = Some(validated);
                }
                _ => existing.base_url = None,
            }
        }

        if let Some(ref model) = input.model_id {
            let trimmed = model.trim();
            if trimmed.is_empty() || trimmed.len() > 100 {
                return Err(AppError::validation(
                    "modelId",
                    "Model ID harus antara 1 hingga 100 karakter.",
                ));
            }
            existing.model_id = trimmed.to_string();
        }

        if let Some(ref cred_ref) = input.credential_ref {
            let trimmed = cred_ref.trim();
            if !trimmed.starts_with("survey_copilot/") {
                return Err(AppError::validation(
                    "credentialRef",
                    "Format referensi kredensial tidak valid.",
                ));
            }
            // Verifikasi eksistensi kredensial
            CredentialRepository::find_by_ref(&conn, trimmed)
                .map_err(AppError::from)?
                .ok_or_else(|| {
                    AppError::validation(
                        "credentialRef",
                        "Kredensial yang dipilih tidak ditemukan dalam daftar kredensial tersimpan.",
                    )
                })?;
            existing.credential_ref = trimmed.to_string();
        }

        if let Some(enabled) = input.enabled {
            existing.enabled = enabled;
        }

        if let Some(priority) = input.priority {
            existing.priority = priority.clamp(1, 1000);
        }

        if let Some(weight) = input.weight {
            existing.weight = weight.max(1);
        }

        if let Some(timeout) = input.timeout_ms {
            existing.timeout_ms = timeout.clamp(1000, 120000);
        }

        if let Some(concurrency) = input.max_concurrency {
            existing.max_concurrency = concurrency.clamp(1, 50);
        }

        if let Some(ref hint) = input.rate_limit_hint_json {
            existing.rate_limit_hint_json = hint.clone();
        }

        let now = Utc::now().to_rfc3339();
        existing.updated_at = now;

        ProviderRepository::update(&conn, &existing).map_err(AppError::from)?;

        let cred = CredentialRepository::find_by_ref(&conn, &existing.credential_ref)
            .map_err(AppError::from)?;

        Ok(ProviderConfigSummary {
            id: existing.id,
            display_name: existing.display_name,
            provider_type: existing.provider_type,
            base_url: existing.base_url,
            adapter_type: existing.adapter_type,
            model_id: existing.model_id,
            credential_ref: existing.credential_ref,
            credential_label: cred.map(|c| c.label),
            enabled: existing.enabled,
            priority: existing.priority,
            weight: existing.weight,
            timeout_ms: existing.timeout_ms,
            max_concurrency: existing.max_concurrency,
            rate_limit_hint_json: existing.rate_limit_hint_json,
            created_at: existing.created_at,
            updated_at: existing.updated_at,
        })
    }

    /// Menghapus konfigurasi provider dari SQLite
    pub fn delete_provider_config(database: &Database, id: &str) -> Result<bool, AppError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError::validation("id", "ID provider wajib diisi."));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        ProviderRepository::delete(&conn, id).map_err(AppError::from)
    }

    /// Menguji koneksi provider menggunakan kunci API dari Windows Credential Manager
    pub fn test_provider_connection(
        database: &Database,
        credential_store: &dyn CredentialStore,
        transport: Arc<dyn HttpTransport>,
        provider_config_id: &str,
    ) -> Result<ConnectionTestResult, AppError> {
        let id = provider_config_id.trim();
        if id.is_empty() {
            return Err(AppError::validation("id", "ID provider wajib diisi."));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let config = ProviderRepository::find_by_id(&conn, id)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::not_found(format!("Konfigurasi provider '{id}' tidak ditemukan."))
            })?;

        // 1. Ambil secret secara aman dari Windows Credential Manager via referensi buram
        let secret =
            CredentialService::read_secret_internal(credential_store, &config.credential_ref)?;

        // 2. Muat adapter yang sesuai
        let adapter = get_adapter(&config.adapter_type, transport)?;

        let now = Utc::now().to_rfc3339();

        // 3. Jalankan tes koneksi
        match adapter.test_connection(
            config.base_url.as_deref(),
            &secret,
            &config.model_id,
            config.timeout_ms as u64,
        ) {
            Ok(mut res) => {
                res.provider_config_id = Some(config.id.clone());

                let _ = ProviderRepository::update_health(
                    &conn,
                    &config.id,
                    "ready",
                    Some(&now),
                    None,
                    None,
                    Some(res.latency_ms),
                    &now,
                );

                Ok(res)
            }
            Err(err) => {
                let _ = ProviderRepository::update_health(
                    &conn,
                    &config.id,
                    "error",
                    None,
                    Some(&now),
                    Some(err.category.as_str()),
                    None,
                    &now,
                );

                Err(AppError::internal(format!(
                    "Pengujian koneksi gagal ({}): {}",
                    err.category.as_str(),
                    err.message
                )))
            }
        }
    }

    /// Menjalankan operasi structured generation yang provider-neutral (untuk pengujian AT-032 dan Answer Engine)
    pub fn execute_structured_request(
        database: &Database,
        credential_store: &dyn CredentialStore,
        transport: Arc<dyn HttpTransport>,
        provider_config_id: &str,
        request: &NormalizedRequest,
    ) -> Result<NormalizedResponse, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let config = ProviderRepository::find_by_id(&conn, provider_config_id)
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Konfigurasi provider '{provider_config_id}' tidak ditemukan."
                ))
            })?;

        // Baca secret secara rahasia di backend Rust
        let secret =
            CredentialService::read_secret_internal(credential_store, &config.credential_ref)?;

        // Panggil adapter
        let adapter = get_adapter(&config.adapter_type, transport)?;

        let now = Utc::now().to_rfc3339();

        match adapter.generate_structured(config.base_url.as_deref(), &secret, request) {
            Ok(resp) => {
                let _ = ProviderRepository::update_health(
                    &conn,
                    &config.id,
                    "ready",
                    Some(&now),
                    None,
                    None,
                    Some(resp.latency_ms),
                    &now,
                );
                Ok(resp)
            }
            Err(err) => {
                let _ = ProviderRepository::update_health(
                    &conn,
                    &config.id,
                    "error",
                    None,
                    Some(&now),
                    Some(err.category.as_str()),
                    None,
                    &now,
                );

                Err(AppError::internal(format!(
                    "Operasi AI provider gagal ({}): {}",
                    err.category.as_str(),
                    err.message
                )))
            }
        }
    }

    /// Mengambil model rekomendasi berdasarkan tipe provider
    pub fn get_recommended_models(provider_type: &str) -> Vec<ModelOption> {
        match provider_type.trim().to_lowercase().as_str() {
            "gemini" => GeminiAdapter::recommended_models(),
            "openai_compatible" | "openai" => OpenAiCompatibleAdapter::recommended_models(),
            _ => vec![],
        }
    }
}
