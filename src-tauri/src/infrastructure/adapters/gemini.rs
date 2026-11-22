use crate::domain::endpoint_validator::EndpointValidator;
use crate::domain::error::AppError;
use crate::domain::provider::{
    ConnectionTestResult, ModelOption, NormalizedError, NormalizedErrorCategory,
    NormalizedFinishReason, NormalizedRequest, NormalizedResponse, ProviderConfig,
};
use crate::infrastructure::adapters::AiProviderAdapter;
use crate::infrastructure::http::HttpTransport;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;

pub const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com";

pub struct GeminiAdapter {
    transport: Arc<dyn HttpTransport>,
}

impl GeminiAdapter {
    pub fn new(transport: Arc<dyn HttpTransport>) -> Self {
        Self { transport }
    }

    /// Default recommended models for Gemini
    pub fn recommended_models() -> Vec<ModelOption> {
        vec![
            ModelOption {
                id: "gemini-2.5-flash".to_string(),
                display_name: "Gemini 2.5 Flash (Direkomendasikan)".to_string(),
                description: Some("Model generasi terbaru Google yang sangat cepat, efisien, dan akurat untuk analisis survei.".to_string()),
                supported_operations: vec![
                    "question_analysis".to_string(),
                    "question_explanation".to_string(),
                    "visual_interpretation".to_string(),
                    "survey_criteria_extraction".to_string(),
                ],
            },
            ModelOption {
                id: "gemini-2.5-pro".to_string(),
                display_name: "Gemini 2.5 Pro".to_string(),
                description: Some("Model dengan penalaran mendalam untuk pertanyaan survei kompleks dan multi-kondisi.".to_string()),
                supported_operations: vec![
                    "question_analysis".to_string(),
                    "question_explanation".to_string(),
                    "visual_interpretation".to_string(),
                    "survey_criteria_extraction".to_string(),
                ],
            },
            ModelOption {
                id: "gemini-1.5-flash".to_string(),
                display_name: "Gemini 1.5 Flash".to_string(),
                description: Some("Model cepat generasi 1.5 dengan stabilitas tinggi.".to_string()),
                supported_operations: vec![
                    "question_analysis".to_string(),
                    "question_explanation".to_string(),
                    "visual_interpretation".to_string(),
                    "survey_criteria_extraction".to_string(),
                ],
            },
            ModelOption {
                id: "gemini-1.5-pro".to_string(),
                display_name: "Gemini 1.5 Pro".to_string(),
                description: Some("Model generasi 1.5 untuk analisis dengan jendela konteks luas.".to_string()),
                supported_operations: vec![
                    "question_analysis".to_string(),
                    "question_explanation".to_string(),
                    "visual_interpretation".to_string(),
                    "survey_criteria_extraction".to_string(),
                ],
            },
        ]
    }

    fn resolve_base_url<'a>(&self, custom_url: Option<&'a str>) -> &'a str {
        match custom_url {
            Some(u) if !u.trim().is_empty() => u.trim(),
            _ => DEFAULT_GEMINI_BASE_URL,
        }
    }

    fn normalize_error_response(
        status: u16,
        body: &str,
        headers: &std::collections::HashMap<String, String>,
    ) -> NormalizedError {
        // Try parsing Google error envelope
        let parsed_json: Option<serde_json::Value> = serde_json::from_str(body).ok();

        let (api_message, api_status) = if let Some(ref val) = parsed_json {
            let msg = val["error"]["message"].as_str().unwrap_or("").to_string();
            let stat = val["error"]["status"].as_str().unwrap_or("").to_string();
            (msg, stat)
        } else {
            (body.to_string(), String::new())
        };

        let retry_after_ms = headers
            .get("retry-after")
            .and_then(|h| h.parse::<u64>().ok().map(|s| s * 1000));

        let (category, message) = match status {
            429 => {
                if api_status == "RESOURCE_EXHAUSTED" || api_message.to_lowercase().contains("quota") {
                    (
                        NormalizedErrorCategory::QuotaExhausted,
                        format!("Batas kuota Gemini telah habis: {api_message}"),
                    )
                } else {
                    (
                        NormalizedErrorCategory::RateLimited,
                        format!("Permintaan dibatasi lajunya (rate limit) oleh Gemini: {api_message}"),
                    )
                }
            }
            401 => (
                NormalizedErrorCategory::AuthenticationFailed,
                "Otentikasi Gemini gagal. Kunci API tidak valid atau telah kadaluarsa.".to_string(),
            ),
            403 => (
                NormalizedErrorCategory::PermissionDenied,
                format!("Akses ditolak oleh Google API: {api_message}"),
            ),
            404 => (
                NormalizedErrorCategory::ModelUnavailable,
                format!("Model Gemini tidak ditemukan atau tidak tersedia untuk akun ini: {api_message}"),
            ),
            400 => (
                NormalizedErrorCategory::InvalidRequest,
                format!("Format permintaan tidak valid untuk Gemini API: {api_message}"),
            ),
            500..=599 => (
                NormalizedErrorCategory::ProviderServerError,
                format!("Server Google Gemini sedang mengalami gangguan ({status}): {api_message}"),
            ),
            _ => (
                NormalizedErrorCategory::Unknown,
                format!("Kesalahan tidak dikenal dari Gemini API ({status}): {api_message}"),
            ),
        };

        let mut err = NormalizedError::new(category, message).with_status(status);
        if let Some(ms) = retry_after_ms {
            err = err.with_retry_after(ms);
        }
        err
    }
}

impl AiProviderAdapter for GeminiAdapter {
    fn adapter_type(&self) -> &'static str {
        "gemini"
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<(), AppError> {
        if config.model_id.trim().is_empty() {
            return Err(AppError::validation(
                "modelId",
                "Model ID Gemini wajib diisi.",
            ));
        }

        if let Some(ref base_url) = config.base_url {
            EndpointValidator::validate_url(base_url)?;
        }

        Ok(())
    }

    fn test_connection(
        &self,
        base_url: Option<&str>,
        secret_key: &str,
        model_id: &str,
        timeout_ms: u64,
    ) -> Result<ConnectionTestResult, NormalizedError> {
        let root_url = self.resolve_base_url(base_url);
        let endpoint = format!("{root_url}/v1beta/models/{model_id}:generateContent");

        let test_body = json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": "ping" }]
            }],
            "generationConfig": {
                "maxOutputTokens": 3,
                "temperature": 0.0
            }
        });

        let start = Instant::now();
        let resp = self.transport.post_json(
            &endpoint,
            &[
                ("x-goog-api-key", secret_key),
                ("content-type", "application/json"),
            ],
            &test_body,
            timeout_ms,
        )?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if (200..300).contains(&resp.status) {
            Ok(ConnectionTestResult {
                provider_config_id: None,
                success: true,
                model_id: model_id.to_string(),
                latency_ms,
                message: format!("Koneksi ke Google Gemini ({model_id}) berhasil diverifikasi ({latency_ms} ms)."),
                tested_at: Utc::now().to_rfc3339(),
                available_models: Some(Self::recommended_models()),
            })
        } else {
            let err = Self::normalize_error_response(resp.status, &resp.body, &resp.headers);
            Err(err)
        }
    }

    fn generate_structured(
        &self,
        base_url: Option<&str>,
        secret_key: &str,
        request: &NormalizedRequest,
    ) -> Result<NormalizedResponse, NormalizedError> {
        let root_url = self.resolve_base_url(base_url);
        let endpoint = format!(
            "{root_url}/v1beta/models/{}:generateContent",
            request.model_id
        );

        let mut gen_config = json!({
            "temperature": request.temperature.unwrap_or(0.2),
            "maxOutputTokens": request.max_tokens.unwrap_or(2048),
        });

        // AT-032: If structured schema requested, configure Gemini structured JSON response
        if let Some(ref schema) = request.response_schema {
            gen_config["responseMimeType"] = json!("application/json");
            gen_config["responseSchema"] = schema.clone();
        }

        let mut body = json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": request.user_payload }]
            }],
            "generationConfig": gen_config
        });

        if let Some(ref sys_instruction) = request.system_instruction {
            body["systemInstruction"] = json!({
                "parts": [{ "text": sys_instruction }]
            });
        }

        let timeout_ms = request.timeout_ms.unwrap_or(30000);
        let start = Instant::now();

        let resp = self.transport.post_json(
            &endpoint,
            &[
                ("x-goog-api-key", secret_key),
                ("content-type", "application/json"),
            ],
            &body,
            timeout_ms,
        )?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !(200..300).contains(&resp.status) {
            return Err(Self::normalize_error_response(
                resp.status,
                &resp.body,
                &resp.headers,
            ));
        }

        let parsed: serde_json::Value = serde_json::from_str(&resp.body).map_err(|e| {
            NormalizedError::new(
                NormalizedErrorCategory::ResponseParseError,
                format!("Gagal mem-parsing format respon JSON dari Gemini: {e}"),
            )
        })?;

        // Extract candidate text
        let candidate = parsed["candidates"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| {
                NormalizedError::new(
                    NormalizedErrorCategory::ResponseParseError,
                    "Respon Gemini tidak memuat kandidat jawaban yang valid.",
                )
            })?;

        let finish_reason_raw = candidate["finishReason"].as_str().unwrap_or("STOP");
        let finish_reason = match finish_reason_raw {
            "STOP" => NormalizedFinishReason::Stop,
            "MAX_TOKENS" => NormalizedFinishReason::MaxTokens,
            "SAFETY" => NormalizedFinishReason::Safety,
            other => NormalizedFinishReason::Other(other.to_string()),
        };

        if finish_reason == NormalizedFinishReason::Safety {
            return Err(NormalizedError::new(
                NormalizedErrorCategory::ContentRefused,
                "Konten survei atau jawaban ditolak oleh filter keamanan Gemini.",
            ));
        }

        let raw_text = candidate["content"]["parts"]
            .as_array()
            .and_then(|parts| parts.first())
            .and_then(|p| p["text"].as_str())
            .map(|s| s.to_string());

        // Parse structured JSON if schema was expected or text is valid JSON
        let structured_json = if let Some(ref text) = raw_text {
            serde_json::from_str::<serde_json::Value>(text).ok()
        } else {
            None
        };

        let input_tokens = parsed["usageMetadata"]["promptTokenCount"]
            .as_u64()
            .map(|n| n as u32);
        let output_tokens = parsed["usageMetadata"]["candidatesTokenCount"]
            .as_u64()
            .map(|n| n as u32);

        Ok(NormalizedResponse {
            request_id: request.request_id.clone(),
            provider_request_id: None,
            provider_id: "gemini".to_string(),
            model_id: request.model_id.clone(),
            raw_text,
            structured_json,
            finish_reason,
            input_tokens,
            output_tokens,
            latency_ms,
            received_at: Utc::now().to_rfc3339(),
        })
    }
}
