use serde::{Deserialize, Serialize};

/// Supported provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    Gemini,
    OpenaiCompatible,
    AnthropicCompatible,
    CustomHttp,
}

impl ProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenaiCompatible => "openai_compatible",
            Self::AnthropicCompatible => "anthropic_compatible",
            Self::CustomHttp => "custom_http",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "gemini" => Some(Self::Gemini),
            "openai_compatible" | "openai" => Some(Self::OpenaiCompatible),
            "anthropic_compatible" | "anthropic" => Some(Self::AnthropicCompatible),
            "custom_http" | "custom" => Some(Self::CustomHttp),
            _ => None,
        }
    }
}

/// Supported adapter protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterType {
    Gemini,
    OpenaiCompatible,
    AnthropicCompatible,
    CustomHttp,
}

impl AdapterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenaiCompatible => "openai_compatible",
            Self::AnthropicCompatible => "anthropic_compatible",
            Self::CustomHttp => "custom_http",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "gemini" => Some(Self::Gemini),
            "openai_compatible" | "openai" => Some(Self::OpenaiCompatible),
            "anthropic_compatible" | "anthropic" => Some(Self::AnthropicCompatible),
            "custom_http" | "custom" => Some(Self::CustomHttp),
            _ => None,
        }
    }
}

/// Stored database entity for `provider_configs`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub id: String,
    pub display_name: String,
    pub provider_type: String,
    pub base_url: Option<String>,
    pub adapter_type: String,
    pub model_id: String,
    pub credential_ref: String,
    pub enabled: bool,
    pub priority: i32,
    pub weight: i32,
    pub timeout_ms: u32,
    pub max_concurrency: u32,
    pub rate_limit_hint_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Safe IPC summary sent to UI (strictly zero raw secrets)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfigSummary {
    pub id: String,
    pub display_name: String,
    pub provider_type: String,
    pub base_url: Option<String>,
    pub adapter_type: String,
    pub model_id: String,
    pub credential_ref: String,
    pub credential_label: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub weight: i32,
    pub timeout_ms: u32,
    pub max_concurrency: u32,
    pub rate_limit_hint_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input payload to create a new provider configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProviderConfigInput {
    pub id: Option<String>,
    pub display_name: String,
    pub provider_type: String,
    pub base_url: Option<String>,
    pub adapter_type: String,
    pub model_id: String,
    pub credential_ref: String,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub timeout_ms: Option<u32>,
    pub max_concurrency: Option<u32>,
    pub rate_limit_hint_json: Option<String>,
}

/// Input payload to update an existing provider configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderConfigInput {
    pub id: String,
    pub display_name: Option<String>,
    pub base_url: Option<Option<String>>,
    pub model_id: Option<String>,
    pub credential_ref: Option<String>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub timeout_ms: Option<u32>,
    pub max_concurrency: Option<u32>,
    pub rate_limit_hint_json: Option<Option<String>>,
}

/// Normalized provider-neutral request contract
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedRequest {
    pub request_id: String,
    pub operation: String,
    pub model_id: String,
    pub system_instruction: Option<String>,
    pub user_payload: String,
    pub response_schema: Option<serde_json::Value>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub timeout_ms: Option<u64>,
}

/// Finish reason normalized across providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedFinishReason {
    Stop,
    MaxTokens,
    Safety,
    Other(String),
}

/// Normalized provider-neutral response contract
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedResponse {
    pub request_id: String,
    pub provider_request_id: Option<String>,
    pub provider_id: String,
    pub model_id: String,
    pub raw_text: Option<String>,
    pub structured_json: Option<serde_json::Value>,
    pub finish_reason: NormalizedFinishReason,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub latency_ms: u64,
    pub received_at: String,
}

/// Standardized error categories normalized across providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedErrorCategory {
    RateLimited,
    QuotaExhausted,
    AuthenticationFailed,
    PermissionDenied,
    ModelUnavailable,
    InvalidRequest,
    ContentRefused,
    Timeout,
    NetworkError,
    ProviderServerError,
    ResponseParseError,
    Cancelled,
    Unknown,
}

impl NormalizedErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RateLimited => "rate_limited",
            Self::QuotaExhausted => "quota_exhausted",
            Self::AuthenticationFailed => "authentication_failed",
            Self::PermissionDenied => "permission_denied",
            Self::ModelUnavailable => "model_unavailable",
            Self::InvalidRequest => "invalid_request",
            Self::ContentRefused => "content_refused",
            Self::Timeout => "timeout",
            Self::NetworkError => "network_error",
            Self::ProviderServerError => "provider_server_error",
            Self::ResponseParseError => "response_parse_error",
            Self::Cancelled => "cancelled",
            Self::Unknown => "unknown",
        }
    }
}

/// Normalized provider error contract
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedError {
    pub category: NormalizedErrorCategory,
    pub message: String,
    pub status_code: Option<u16>,
    pub retryable: bool,
    pub retry_after_ms: Option<u64>,
    pub provider_request_id: Option<String>,
}

impl NormalizedError {
    pub fn new(category: NormalizedErrorCategory, message: impl Into<String>) -> Self {
        let retryable = matches!(
            category,
            NormalizedErrorCategory::RateLimited
                | NormalizedErrorCategory::Timeout
                | NormalizedErrorCategory::NetworkError
                | NormalizedErrorCategory::ProviderServerError
        );
        Self {
            category,
            message: message.into(),
            status_code: None,
            retryable,
            retry_after_ms: None,
            provider_request_id: None,
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status_code = Some(status);
        self
    }

    pub fn with_retry_after(mut self, ms: u64) -> Self {
        self.retry_after_ms = Some(ms);
        self.retryable = true;
        self
    }

    pub fn with_provider_request_id(mut self, id: impl Into<String>) -> Self {
        self.provider_request_id = Some(id.into());
        self
    }
}

/// Model descriptor returned to UI for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelOption {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub supported_operations: Vec<String>,
}

/// Result returned from connection test
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub provider_config_id: Option<String>,
    pub success: bool,
    pub model_id: String,
    pub latency_ms: u64,
    pub message: String,
    pub tested_at: String,
    pub available_models: Option<Vec<ModelOption>>,
}
