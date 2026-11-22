use crate::domain::provider::{NormalizedError, NormalizedRequest, NormalizedResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Supported provider selection strategies for the pool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoolStrategy {
    Adaptive,
    RoundRobin,
    Weighted,
    Priority,
}

impl PoolStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Adaptive => "adaptive",
            Self::RoundRobin => "round_robin",
            Self::Weighted => "weighted",
            Self::Priority => "priority",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "adaptive" => Some(Self::Adaptive),
            "round_robin" => Some(Self::RoundRobin),
            "weighted" => Some(Self::Weighted),
            "priority" => Some(Self::Priority),
            _ => None,
        }
    }
}

/// Global settings for the provider pool stored in `provider_pool_settings`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolSettings {
    pub id: String,
    pub strategy: PoolStrategy,
    pub max_attempts: u32,
    pub auto_retry: bool,
    pub retry_base_delay_ms: u64,
    pub max_retry_delay_ms: u64,
    pub updated_at: String,
}

impl Default for PoolSettings {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            strategy: PoolStrategy::Adaptive,
            max_attempts: 3,
            auto_retry: true,
            retry_base_delay_ms: 1000,
            max_retry_delay_ms: 15000,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Input payload to update pool settings
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePoolSettingsInput {
    pub strategy: Option<String>,
    pub max_attempts: Option<u32>,
    pub auto_retry: Option<bool>,
    pub retry_base_delay_ms: Option<u64>,
    pub max_retry_delay_ms: Option<u64>,
}

/// Provider health states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderHealthState {
    Ready,
    CoolingDown,
    QuotaExhausted,
    NeedsAttention,
    Disabled,
}

impl ProviderHealthState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::CoolingDown => "cooling_down",
            Self::QuotaExhausted => "quota_exhausted",
            Self::NeedsAttention => "needs_attention",
            Self::Disabled => "disabled",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "ready" => Some(Self::Ready),
            "cooling_down" | "cooldown" => Some(Self::CoolingDown),
            "quota_exhausted" | "exhausted" => Some(Self::QuotaExhausted),
            "needs_attention" | "error" => Some(Self::NeedsAttention),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

/// Comprehensive health record per provider
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderHealthInfo {
    pub provider_config_id: String,
    pub display_name: String,
    pub state: ProviderHealthState,
    pub cooldown_until: Option<String>,
    pub last_success_at: Option<String>,
    pub last_failure_at: Option<String>,
    pub last_error_category: Option<String>,
    pub consecutive_failures: u32,
    pub recent_latency_ms: Option<u64>,
    pub current_concurrency: u32,
    pub max_concurrency: u32,
    pub priority: i32,
    pub weight: i32,
    pub enabled: bool,
    pub updated_at: String,
}

/// Individual attempt log in a pool execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAttemptLog {
    pub attempt_number: u32,
    pub provider_config_id: String,
    pub model_id: String,
    pub latency_ms: u64,
    pub success: bool,
    pub error_category: Option<String>,
    pub error_message: Option<String>,
    pub timestamp: String,
}

/// Result returned from pooled request execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolExecutionResult {
    pub success: bool,
    pub response: Option<NormalizedResponse>,
    pub final_error: Option<NormalizedError>,
    pub attempts: Vec<ProviderAttemptLog>,
    pub preserved_request: NormalizedRequest,
}

/// Thread-safe cancellation token for pool requests (AT-036)
#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Time source abstraction for deterministic testing
pub trait TimeSource: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub struct FakeTimeSource {
    current: std::sync::Mutex<DateTime<Utc>>,
}

impl FakeTimeSource {
    pub fn new(initial: DateTime<Utc>) -> Self {
        Self {
            current: std::sync::Mutex::new(initial),
        }
    }

    pub fn advance_seconds(&self, secs: i64) {
        let mut t = self.current.lock().unwrap();
        *t += chrono::Duration::seconds(secs);
    }

    pub fn advance_millis(&self, ms: i64) {
        let mut t = self.current.lock().unwrap();
        *t += chrono::Duration::milliseconds(ms);
    }

    pub fn set_now(&self, new_now: DateTime<Utc>) {
        let mut t = self.current.lock().unwrap();
        *t = new_now;
    }
}

impl TimeSource for FakeTimeSource {
    fn now(&self) -> DateTime<Utc> {
        *self.current.lock().unwrap()
    }
}
