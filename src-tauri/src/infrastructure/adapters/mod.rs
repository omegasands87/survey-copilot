pub mod gemini;
pub mod openai_compatible;

use crate::domain::error::AppError;
use crate::domain::provider::{
    ConnectionTestResult, NormalizedError, NormalizedRequest, NormalizedResponse, ProviderConfig,
};
use crate::infrastructure::http::HttpTransport;
pub use gemini::GeminiAdapter;
pub use openai_compatible::OpenAiCompatibleAdapter;
use std::sync::Arc;

/// Provider-neutral AI adapter trait
pub trait AiProviderAdapter: Send + Sync {
    /// Identifier of the adapter protocol
    fn adapter_type(&self) -> &'static str;

    /// Validates adapter-specific configuration parameters
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), AppError>;

    /// Tests connection and model availability
    fn test_connection(
        &self,
        base_url: Option<&str>,
        secret_key: &str,
        model_id: &str,
        timeout_ms: u64,
    ) -> Result<ConnectionTestResult, NormalizedError>;

    /// Executes structured AI generation
    fn generate_structured(
        &self,
        base_url: Option<&str>,
        secret_key: &str,
        request: &NormalizedRequest,
    ) -> Result<NormalizedResponse, NormalizedError>;
}

/// Factory creating the appropriate adapter based on adapter_type string
pub fn get_adapter(
    adapter_type: &str,
    transport: Arc<dyn HttpTransport>,
) -> Result<Box<dyn AiProviderAdapter>, AppError> {
    match adapter_type.trim().to_lowercase().as_str() {
        "gemini" => Ok(Box::new(GeminiAdapter::new(transport))),
        "openai_compatible" | "openai" => Ok(Box::new(OpenAiCompatibleAdapter::new(transport))),
        _ => Err(AppError::validation(
            "adapterType",
            format!("Protokol adapter '{adapter_type}' belum didukung. Protokol yang didukung saat ini: gemini, openai_compatible."),
        )),
    }
}
