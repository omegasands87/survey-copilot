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

pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAiCompatibleAdapter {
    transport: Arc<dyn HttpTransport>,
}

impl OpenAiCompatibleAdapter {
    pub fn new(transport: Arc<dyn HttpTransport>) -> Self {
        Self { transport }
    }

    pub fn recommended_models() -> Vec<ModelOption> {
        vec![
            ModelOption {
                id: "gpt-4o-mini".to_string(),
                display_name: "GPT-4o Mini (Cepat & Ringan)".to_string(),
                description: Some("Model OpenAI yang cepat dan hemat biaya untuk tugas klasifikasi dan analisis survei.".to_string()),
                supported_operations: vec![
                    "question_analysis".to_string(),
                    "question_explanation".to_string(),
                    "visual_interpretation".to_string(),
                    "survey_criteria_extraction".to_string(),
                ],
            },
            ModelOption {
                id: "gpt-4o".to_string(),
                display_name: "GPT-4o".to_string(),
                description: Some("Model cerdas serbaguna untuk pemahaman instruksi survei berbelit-belit.".to_string()),
                supported_operations: vec![
                    "question_analysis".to_string(),
                    "question_explanation".to_string(),
                    "visual_interpretation".to_string(),
                    "survey_criteria_extraction".to_string(),
                ],
            },
            ModelOption {
                id: "llama3".to_string(),
                display_name: "Llama 3 / Ollama (Lokal)".to_string(),
                description: Some("Model open-weight yang berjalan di komputer lokal via Ollama atau vLLM.".to_string()),
                supported_operations: vec![
                    "question_analysis".to_string(),
                    "question_explanation".to_string(),
                    "survey_criteria_extraction".to_string(),
                ],
            },
        ]
    }

    fn resolve_endpoint(&self, custom_url: Option<&str>) -> String {
        let base = match custom_url {
            Some(u) if !u.trim().is_empty() => u.trim(),
            _ => DEFAULT_OPENAI_BASE_URL,
        };

        let trimmed_base = base.trim_end_matches('/');
        if trimmed_base.ends_with("/chat/completions") {
            trimmed_base.to_string()
        } else if trimmed_base.ends_with("/v1") {
            format!("{trimmed_base}/chat/completions")
        } else {
            format!("{trimmed_base}/v1/chat/completions")
        }
    }

    fn normalize_error_response(status: u16, body: &str) -> NormalizedError {
        let parsed: Option<serde_json::Value> = serde_json::from_str(body).ok();
        let api_msg = parsed
            .as_ref()
            .and_then(|v| v["error"]["message"].as_str())
            .unwrap_or(body)
            .to_string();

        let category = match status {
            429 => {
                if api_msg.to_lowercase().contains("quota") {
                    NormalizedErrorCategory::QuotaExhausted
                } else {
                    NormalizedErrorCategory::RateLimited
                }
            }
            401 => NormalizedErrorCategory::AuthenticationFailed,
            403 => NormalizedErrorCategory::PermissionDenied,
            404 => NormalizedErrorCategory::ModelUnavailable,
            400 => NormalizedErrorCategory::InvalidRequest,
            500..=599 => NormalizedErrorCategory::ProviderServerError,
            _ => NormalizedErrorCategory::Unknown,
        };

        NormalizedError::new(
            category,
            format!("Respon kesalahan OpenAI-compatible ({status}): {api_msg}"),
        )
        .with_status(status)
    }
}

impl AiProviderAdapter for OpenAiCompatibleAdapter {
    fn adapter_type(&self) -> &'static str {
        "openai_compatible"
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<(), AppError> {
        if config.model_id.trim().is_empty() {
            return Err(AppError::validation("modelId", "Model ID wajib diisi."));
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
        let endpoint = self.resolve_endpoint(base_url);

        let test_body = json!({
            "model": model_id,
            "messages": [
                { "role": "user", "content": "ping" }
            ],
            "max_tokens": 3,
            "temperature": 0.0
        });

        let auth_header = format!("Bearer {}", secret_key.trim());
        let start = Instant::now();

        let resp = self.transport.post_json(
            &endpoint,
            &[
                ("authorization", &auth_header),
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
                message: format!("Koneksi ke endpoint OpenAI-compatible ({model_id}) berhasil diverifikasi ({latency_ms} ms)."),
                tested_at: Utc::now().to_rfc3339(),
                available_models: Some(Self::recommended_models()),
            })
        } else {
            Err(Self::normalize_error_response(resp.status, &resp.body))
        }
    }

    fn generate_structured(
        &self,
        base_url: Option<&str>,
        secret_key: &str,
        request: &NormalizedRequest,
    ) -> Result<NormalizedResponse, NormalizedError> {
        let endpoint = self.resolve_endpoint(base_url);

        let mut messages = Vec::new();
        if let Some(ref sys) = request.system_instruction {
            messages.push(json!({
                "role": "system",
                "content": sys
            }));
        }

        messages.push(json!({
            "role": "user",
            "content": request.user_payload
        }));

        let mut body = json!({
            "model": request.model_id,
            "messages": messages,
            "temperature": request.temperature.unwrap_or(0.2),
            "max_tokens": request.max_tokens.unwrap_or(2048)
        });

        // Request JSON object output format if structured
        if request.response_schema.is_some() {
            body["response_format"] = json!({ "type": "json_object" });
        }

        let auth_header = format!("Bearer {}", secret_key.trim());
        let timeout_ms = request.timeout_ms.unwrap_or(30000);
        let start = Instant::now();

        let resp = self.transport.post_json(
            &endpoint,
            &[
                ("authorization", &auth_header),
                ("content-type", "application/json"),
            ],
            &body,
            timeout_ms,
        )?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !(200..300).contains(&resp.status) {
            return Err(Self::normalize_error_response(resp.status, &resp.body));
        }

        let parsed: serde_json::Value = serde_json::from_str(&resp.body).map_err(|e| {
            NormalizedError::new(
                NormalizedErrorCategory::ResponseParseError,
                format!("Gagal mem-parsing format respon JSON OpenAI-compatible: {e}"),
            )
        })?;

        let choice = parsed["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| {
                NormalizedError::new(
                    NormalizedErrorCategory::ResponseParseError,
                    "Respon OpenAI-compatible tidak memuat pilihan jawaban (choices).",
                )
            })?;

        let finish_reason_raw = choice["finish_reason"].as_str().unwrap_or("stop");
        let finish_reason = match finish_reason_raw {
            "stop" => NormalizedFinishReason::Stop,
            "length" => NormalizedFinishReason::MaxTokens,
            "content_filter" => NormalizedFinishReason::Safety,
            other => NormalizedFinishReason::Other(other.to_string()),
        };

        if finish_reason == NormalizedFinishReason::Safety {
            return Err(NormalizedError::new(
                NormalizedErrorCategory::ContentRefused,
                "Konten ditolak oleh filter keamanan provider OpenAI-compatible.",
            ));
        }

        let raw_text = choice["message"]["content"].as_str().map(|s| s.to_string());
        let structured_json = if let Some(ref text) = raw_text {
            serde_json::from_str::<serde_json::Value>(text).ok()
        } else {
            None
        };

        let input_tokens = parsed["usage"]["prompt_tokens"].as_u64().map(|n| n as u32);
        let output_tokens = parsed["usage"]["completion_tokens"]
            .as_u64()
            .map(|n| n as u32);
        let provider_req_id = parsed["id"].as_str().map(|s| s.to_string());

        Ok(NormalizedResponse {
            request_id: request.request_id.clone(),
            provider_request_id: provider_req_id,
            provider_id: "openai_compatible".to_string(),
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
