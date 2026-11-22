use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::pool::{
    CancellationToken, PoolExecutionResult, PoolSettings, PoolStrategy, ProviderAttemptLog,
    ProviderHealthInfo, ProviderHealthState, SystemTimeSource, TimeSource, UpdatePoolSettingsInput,
};
use crate::domain::provider::{
    NormalizedError, NormalizedErrorCategory, NormalizedRequest, ProviderConfig,
};
use crate::infrastructure::adapters::{get_adapter, AiProviderAdapter};
use crate::infrastructure::credentials::CredentialStore;
use crate::infrastructure::http::HttpTransport;
use crate::repository::{PoolRepository, ProviderRepository};
use crate::services::CredentialService;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// RAII Concurrency Guard to decrement active request counter on drop
struct ConcurrencyGuard {
    provider_id: String,
    tracker: Arc<Mutex<HashMap<String, u32>>>,
}

impl ConcurrencyGuard {
    fn new(provider_id: String, tracker: Arc<Mutex<HashMap<String, u32>>>) -> Self {
        {
            let mut lock = tracker.lock().unwrap();
            let count = lock.entry(provider_id.clone()).or_insert(0);
            *count += 1;
        }
        Self {
            provider_id,
            tracker,
        }
    }
}

impl Drop for ConcurrencyGuard {
    fn drop(&mut self) {
        let mut lock = self.tracker.lock().unwrap();
        if let Some(count) = lock.get_mut(&self.provider_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}

/// Provider Pool Service managing resilient dispatch, health, and failover
#[derive(Clone)]
pub struct ProviderPoolService {
    concurrency_tracker: Arc<Mutex<HashMap<String, u32>>>,
    round_robin_counter: Arc<AtomicUsize>,
    time_source: Arc<dyn TimeSource>,
}

impl ProviderPoolService {
    pub fn new() -> Self {
        Self {
            concurrency_tracker: Arc::new(Mutex::new(HashMap::new())),
            round_robin_counter: Arc::new(AtomicUsize::new(0)),
            time_source: Arc::new(SystemTimeSource),
        }
    }

    /// Inisialisasi dengan TimeSource khusus (berguna untuk deterministic automated testing)
    pub fn with_time_source(time_source: Arc<dyn TimeSource>) -> Self {
        Self {
            concurrency_tracker: Arc::new(Mutex::new(HashMap::new())),
            round_robin_counter: Arc::new(AtomicUsize::new(0)),
            time_source,
        }
    }

    /// Mengambil pengaturan global provider pool
    pub fn get_settings(&self, database: &Database) -> Result<PoolSettings, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        PoolRepository::get_settings(&conn).map_err(AppError::from)
    }

    /// Memperbarui pengaturan global provider pool
    pub fn update_settings(
        &self,
        database: &Database,
        input: UpdatePoolSettingsInput,
    ) -> Result<PoolSettings, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let mut current = PoolRepository::get_settings(&conn).map_err(AppError::from)?;

        if let Some(strat_str) = input.strategy {
            let strategy = PoolStrategy::from_str_loose(&strat_str).ok_or_else(|| {
                AppError::validation(
                    "strategy",
                    format!(
                        "Strategi '{strat_str}' tidak valid. Pilih dari: adaptive, round_robin, weighted, priority."
                    ),
                )
            })?;
            current.strategy = strategy;
        }

        if let Some(attempts) = input.max_attempts {
            if !(1..=10).contains(&attempts) {
                return Err(AppError::validation(
                    "maxAttempts",
                    "Batas percobaan (maxAttempts) harus bernilai antara 1 hingga 10.",
                ));
            }
            current.max_attempts = attempts;
        }

        if let Some(auto_retry) = input.auto_retry {
            current.auto_retry = auto_retry;
        }

        if let Some(base_delay) = input.retry_base_delay_ms {
            current.retry_base_delay_ms = base_delay.clamp(100, 60000);
        }

        if let Some(max_delay) = input.max_retry_delay_ms {
            current.max_retry_delay_ms = max_delay.clamp(1000, 120000);
        }

        current.updated_at = self.time_source.now().to_rfc3339();

        PoolRepository::update_settings(&conn, &current).map_err(AppError::from)?;

        Ok(current)
    }

    /// Mengambil seluruh status kesehatan provider beserta konkurensi aktif
    pub fn list_health(&self, database: &Database) -> Result<Vec<ProviderHealthInfo>, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        let now_str = self.time_source.now().to_rfc3339();
        let _ = PoolRepository::reset_cooldown_if_expired(&conn, &now_str);

        let mut health_list = PoolRepository::list_all_health(&conn).map_err(AppError::from)?;

        let tracker = self.concurrency_tracker.lock().unwrap();
        for h in &mut health_list {
            h.current_concurrency = tracker.get(&h.provider_config_id).copied().unwrap_or(0);
        }

        Ok(health_list)
    }

    /// Menghitung durasi cooldown eksponensial dengan batas atas
    fn calculate_cooldown_ms(
        consecutive_failures: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
        retry_after_ms: Option<u64>,
    ) -> u64 {
        if let Some(ra) = retry_after_ms {
            return ra.max(1000);
        }

        let shift = consecutive_failures.min(6);
        let multiplier = 1u64.checked_shl(shift).unwrap_or(64);
        let calculated = base_delay_ms.saturating_mul(multiplier);
        calculated.clamp(base_delay_ms, max_delay_ms)
    }

    /// Memilih kandidat provider terbaik berdasarkan strategi yang dipilih
    fn select_candidate(
        &self,
        strategy: PoolStrategy,
        candidates: &[(&ProviderConfig, &ProviderHealthInfo)],
    ) -> Option<usize> {
        if candidates.is_empty() {
            return None;
        }

        match strategy {
            PoolStrategy::Priority => {
                // Urutkan berdasarkan priority terkecil (nilai angka lebih kecil = prioritas lebih tinggi),
                // lalu latency terbaik, lalu ID untuk konsistensi
                let mut indexed: Vec<(usize, &ProviderConfig, &ProviderHealthInfo)> = candidates
                    .iter()
                    .enumerate()
                    .map(|(idx, (c, h))| (idx, *c, *h))
                    .collect();

                indexed.sort_by(|a, b| {
                    a.1.priority
                        .cmp(&b.1.priority)
                        .then_with(|| {
                            let lat_a = a.2.recent_latency_ms.unwrap_or(u64::MAX);
                            let lat_b = b.2.recent_latency_ms.unwrap_or(u64::MAX);
                            lat_a.cmp(&lat_b)
                        })
                        .then_with(|| a.1.id.cmp(&b.1.id))
                });

                indexed.first().map(|(idx, _, _)| *idx)
            }
            PoolStrategy::RoundRobin => {
                // Rotasi sekuensial deterministik
                let counter = self.round_robin_counter.fetch_add(1, Ordering::SeqCst);
                Some(counter % candidates.len())
            }
            PoolStrategy::Weighted => {
                // Distribusi bobot terbobot
                let total_weight: u64 =
                    candidates.iter().map(|(c, _)| c.weight.max(1) as u64).sum();
                if total_weight == 0 {
                    return Some(0);
                }

                let counter = self.round_robin_counter.fetch_add(1, Ordering::SeqCst) as u64;
                let mut target = counter % total_weight;

                for (idx, (c, _)) in candidates.iter().enumerate() {
                    let w = c.weight.max(1) as u64;
                    if target < w {
                        return Some(idx);
                    }
                    target -= w;
                }

                Some(0)
            }
            PoolStrategy::Adaptive => {
                // Kebugaran adaptif: ketersediaan, konkurensi, latensi, dan riwayat kegagalan
                let mut best_idx = 0;
                let mut best_score = f64::NEG_INFINITY;

                for (idx, (config, health)) in candidates.iter().enumerate() {
                    // 1. Base prioritas: priority lebih kecil mendapat skor lebih tinggi
                    let priority_score = (1000.0 - (config.priority.clamp(1, 999) as f64)).max(1.0);

                    // 2. Faktor beban konkurensi: pinalti jika sedang memproses konkurensi tinggi
                    let load_ratio = (health.current_concurrency as f64)
                        / (config.max_concurrency.max(1) as f64);
                    let concurrency_factor = (1.0 - load_ratio).max(0.1);

                    // 3. Faktor latensi: preferensikan yang lebih cepat
                    let latency = health.recent_latency_ms.unwrap_or(500).max(50) as f64;
                    let latency_factor = 1000.0 / latency;

                    // 4. Pinalti kegagalan berturut-turut
                    let failure_factor = 0.5f64.powi(health.consecutive_failures.min(5) as i32);

                    let composite_fitness =
                        priority_score * concurrency_factor * latency_factor * failure_factor;

                    if composite_fitness > best_score {
                        best_score = composite_fitness;
                        best_idx = idx;
                    }
                }

                Some(best_idx)
            }
        }
    }

    /// Menjalankan permintaan AI terstruktur melalui pool dengan perlindungan kuota, cooldown,
    /// batas konkurensi, pembatalan, dan preservasi input pengguna mutlak.
    pub fn execute_pooled_request(
        &self,
        database: &Database,
        credential_store: &dyn CredentialStore,
        transport: Arc<dyn HttpTransport>,
        request: NormalizedRequest,
        cancellation_token: Option<CancellationToken>,
    ) -> PoolExecutionResult {
        let settings = match self.get_settings(database) {
            Ok(s) => s,
            Err(e) => {
                return PoolExecutionResult {
                    success: false,
                    response: None,
                    final_error: Some(NormalizedError::new(
                        NormalizedErrorCategory::Unknown,
                        format!("Gagal memuat pengaturan provider pool: {}", e.message),
                    )),
                    attempts: vec![],
                    preserved_request: request,
                };
            }
        };

        let max_attempts = if settings.auto_retry {
            settings.max_attempts.clamp(1, 10)
        } else {
            1
        };

        let mut attempts_log: Vec<ProviderAttemptLog> = Vec::new();
        let mut failed_provider_ids: HashSet<String> = HashSet::new();
        let mut last_error: Option<NormalizedError> = None;

        for attempt_no in 1..=max_attempts {
            // 1. Verifikasi Pembatalan (AT-036)
            if let Some(ref token) = cancellation_token {
                if token.is_cancelled() {
                    let cancel_err = NormalizedError::new(
                        NormalizedErrorCategory::Cancelled,
                        "Operasi AI dibatalkan oleh pengguna.",
                    );
                    return PoolExecutionResult {
                        success: false,
                        response: None,
                        final_error: Some(cancel_err),
                        attempts: attempts_log,
                        preserved_request: request,
                    };
                }
            }

            // 2. Baca daftar konfigurasi dan status health terbaru dari SQLite
            let (configs, health_list) = {
                let conn = match database.connection.lock() {
                    Ok(c) => c,
                    Err(_) => {
                        let err = NormalizedError::new(
                            NormalizedErrorCategory::Unknown,
                            "Basis data sedang tidak tersedia.",
                        );
                        return PoolExecutionResult {
                            success: false,
                            response: None,
                            final_error: Some(err),
                            attempts: attempts_log,
                            preserved_request: request,
                        };
                    }
                };

                let now_str = self.time_source.now().to_rfc3339();
                let _ = PoolRepository::reset_cooldown_if_expired(&conn, &now_str);

                let cfgs = match ProviderRepository::list_enabled(&conn) {
                    Ok(c) => c,
                    Err(e) => {
                        let err = NormalizedError::new(
                            NormalizedErrorCategory::Unknown,
                            format!("Gagal membaca daftar provider aktif: {e}"),
                        );
                        return PoolExecutionResult {
                            success: false,
                            response: None,
                            final_error: Some(err),
                            attempts: attempts_log,
                            preserved_request: request,
                        };
                    }
                };

                let hlth = match PoolRepository::list_all_health(&conn) {
                    Ok(h) => h,
                    Err(e) => {
                        let err = NormalizedError::new(
                            NormalizedErrorCategory::Unknown,
                            format!("Gagal membaca status health provider: {e}"),
                        );
                        return PoolExecutionResult {
                            success: false,
                            response: None,
                            final_error: Some(err),
                            attempts: attempts_log,
                            preserved_request: request,
                        };
                    }
                };

                (cfgs, hlth)
            };

            let health_map: HashMap<String, ProviderHealthInfo> = health_list
                .into_iter()
                .map(|h| (h.provider_config_id.clone(), h))
                .collect();

            let now = self.time_source.now();
            let current_tracker = self.concurrency_tracker.lock().unwrap().clone();

            // 3. Saring provider yang eligible
            let mut eligible: Vec<(&ProviderConfig, ProviderHealthInfo)> = Vec::new();

            for cfg in &configs {
                // Abaikan provider yang sudah gagal pada turn/request ini
                if failed_provider_ids.contains(&cfg.id) {
                    continue;
                }

                let mut health =
                    health_map
                        .get(&cfg.id)
                        .cloned()
                        .unwrap_or_else(|| ProviderHealthInfo {
                            provider_config_id: cfg.id.clone(),
                            display_name: cfg.display_name.clone(),
                            state: ProviderHealthState::Ready,
                            cooldown_until: None,
                            last_success_at: None,
                            last_failure_at: None,
                            last_error_category: None,
                            consecutive_failures: 0,
                            recent_latency_ms: None,
                            current_concurrency: 0,
                            max_concurrency: cfg.max_concurrency,
                            priority: cfg.priority,
                            weight: cfg.weight,
                            enabled: cfg.enabled,
                            updated_at: now.to_rfc3339(),
                        });

                // Isi konkurensi in-memory terkini
                health.current_concurrency = current_tracker.get(&cfg.id).copied().unwrap_or(0);

                // Cek status eligibility
                if !cfg.enabled || health.state == ProviderHealthState::Disabled {
                    continue;
                }

                if health.state == ProviderHealthState::NeedsAttention {
                    continue;
                }

                if health.state == ProviderHealthState::QuotaExhausted {
                    continue;
                }

                if health.state == ProviderHealthState::CoolingDown {
                    if let Some(ref cd_str) = health.cooldown_until {
                        if let Ok(cd_time) = DateTime::parse_from_rfc3339(cd_str) {
                            if cd_time.with_timezone(&Utc) > now {
                                // Masih dalam masa cooldown
                                continue;
                            }
                        }
                    }
                }

                // Cek batas konkurensi (Bounded Concurrency)
                if health.current_concurrency >= cfg.max_concurrency {
                    continue;
                }

                eligible.push((cfg, health));
            }

            if eligible.is_empty() {
                let exhausted_msg = if configs.is_empty() {
                    "Tidak ada provider AI aktif yang dikonfigurasi. Silakan tambahkan provider di Pengaturan."
                } else if !failed_provider_ids.is_empty() {
                    "Seluruh provider yang memenuhi syarat telah dicoba dan mengalami kegagalan."
                } else {
                    "Seluruh provider AI sedang sibuk (mencapai batas konkurensi) atau berada dalam masa cooldown."
                };

                let pool_exhausted_err =
                    NormalizedError::new(NormalizedErrorCategory::RateLimited, exhausted_msg);

                return PoolExecutionResult {
                    success: false,
                    response: None,
                    final_error: Some(last_error.unwrap_or(pool_exhausted_err)),
                    attempts: attempts_log,
                    preserved_request: request,
                };
            }

            // 4. Pilih kandidat terbaik berdasarkan strategi pool
            let candidate_refs: Vec<(&ProviderConfig, &ProviderHealthInfo)> =
                eligible.iter().map(|(c, h)| (*c, h)).collect();

            let chosen_idx = match self.select_candidate(settings.strategy, &candidate_refs) {
                Some(idx) => idx,
                None => {
                    let err = NormalizedError::new(
                        NormalizedErrorCategory::Unknown,
                        "Gagal memilih kandidat provider dari pool.",
                    );
                    return PoolExecutionResult {
                        success: false,
                        response: None,
                        final_error: Some(err),
                        attempts: attempts_log,
                        preserved_request: request,
                    };
                }
            };

            let (chosen_config, chosen_health) = &eligible[chosen_idx];

            // 5. Akuisisi guard konkurensi RAII
            let _concurrency_guard =
                ConcurrencyGuard::new(chosen_config.id.clone(), self.concurrency_tracker.clone());

            // 6. Ambil secret dari Windows Credential Manager
            let secret = match CredentialService::read_secret_internal(
                credential_store,
                &chosen_config.credential_ref,
            ) {
                Ok(s) => s,
                Err(e) => {
                    // Kredensial tidak ditemukan atau akses ditolak di OS
                    let now_str = self.time_source.now().to_rfc3339();
                    if let Ok(conn) = database.connection.lock() {
                        let _ = PoolRepository::update_health_record(
                            &conn,
                            &chosen_config.id,
                            ProviderHealthState::NeedsAttention.as_str(),
                            None,
                            None,
                            Some(&now_str),
                            Some(NormalizedErrorCategory::AuthenticationFailed.as_str()),
                            Some(chosen_health.consecutive_failures + 1),
                            None,
                            &now_str,
                        );
                    }

                    attempts_log.push(ProviderAttemptLog {
                        attempt_number: attempt_no,
                        provider_config_id: chosen_config.id.clone(),
                        model_id: chosen_config.model_id.clone(),
                        latency_ms: 0,
                        success: false,
                        error_category: Some(
                            NormalizedErrorCategory::AuthenticationFailed
                                .as_str()
                                .to_string(),
                        ),
                        error_message: Some(format!(
                            "Kredensial tidak dapat dibaca dari Windows Credential Manager: {}",
                            e.message
                        )),
                        timestamp: now_str,
                    });

                    failed_provider_ids.insert(chosen_config.id.clone());
                    last_error = Some(NormalizedError::new(
                        NormalizedErrorCategory::AuthenticationFailed,
                        format!(
                            "Kredensial provider '{}' tidak valid atau tidak ditemukan.",
                            chosen_config.display_name
                        ),
                    ));

                    continue;
                }
            };

            // 7. Muat adapter yang sesuai
            let adapter: Box<dyn AiProviderAdapter> =
                match get_adapter(&chosen_config.adapter_type, transport.clone()) {
                    Ok(a) => a,
                    Err(e) => {
                        let now_str = self.time_source.now().to_rfc3339();
                        attempts_log.push(ProviderAttemptLog {
                            attempt_number: attempt_no,
                            provider_config_id: chosen_config.id.clone(),
                            model_id: chosen_config.model_id.clone(),
                            latency_ms: 0,
                            success: false,
                            error_category: Some(
                                NormalizedErrorCategory::InvalidRequest.as_str().to_string(),
                            ),
                            error_message: Some(e.message.clone()),
                            timestamp: now_str,
                        });

                        failed_provider_ids.insert(chosen_config.id.clone());
                        last_error = Some(NormalizedError::new(
                            NormalizedErrorCategory::InvalidRequest,
                            e.message,
                        ));
                        continue;
                    }
                };

            // 8. Eksekusi request terstruktur
            let start_time = Instant::now();
            let mut attempt_request = request.clone();
            // Sesuaikan timeout attempt jika spesifik
            let attempt_timeout = request
                .timeout_ms
                .unwrap_or(chosen_config.timeout_ms as u64)
                .min(chosen_config.timeout_ms as u64);
            attempt_request.timeout_ms = Some(attempt_timeout);

            let exec_result = adapter.generate_structured(
                chosen_config.base_url.as_deref(),
                &secret,
                &attempt_request,
            );

            let latency_ms = start_time.elapsed().as_millis() as u64;
            let now_str = self.time_source.now().to_rfc3339();

            match exec_result {
                Ok(response) => {
                    // Update status health ke 'ready' dan catat latency
                    if let Ok(conn) = database.connection.lock() {
                        let _ = PoolRepository::update_health_record(
                            &conn,
                            &chosen_config.id,
                            ProviderHealthState::Ready.as_str(),
                            None,
                            Some(&now_str),
                            None,
                            None,
                            Some(0),
                            Some(latency_ms),
                            &now_str,
                        );
                    }

                    attempts_log.push(ProviderAttemptLog {
                        attempt_number: attempt_no,
                        provider_config_id: chosen_config.id.clone(),
                        model_id: chosen_config.model_id.clone(),
                        latency_ms,
                        success: true,
                        error_category: None,
                        error_message: None,
                        timestamp: now_str,
                    });

                    return PoolExecutionResult {
                        success: true,
                        response: Some(response),
                        final_error: None,
                        attempts: attempts_log,
                        preserved_request: request,
                    };
                }
                Err(err) => {
                    attempts_log.push(ProviderAttemptLog {
                        attempt_number: attempt_no,
                        provider_config_id: chosen_config.id.clone(),
                        model_id: chosen_config.model_id.clone(),
                        latency_ms,
                        success: false,
                        error_category: Some(err.category.as_str().to_string()),
                        error_message: Some(err.message.clone()),
                        timestamp: now_str.clone(),
                    });

                    failed_provider_ids.insert(chosen_config.id.clone());
                    last_error = Some(err.clone());

                    // Analisis Kebijakan Kuota, Keamanan, dan Retry
                    match err.category {
                        // KASUS A: Pembatasan Kuota Tingkat Project / Akun (AT-034 - WAJIB HENTIKAN RETRY)
                        NormalizedErrorCategory::QuotaExhausted => {
                            if let Ok(conn) = database.connection.lock() {
                                let _ = PoolRepository::update_health_record(
                                    &conn,
                                    &chosen_config.id,
                                    ProviderHealthState::QuotaExhausted.as_str(),
                                    None,
                                    None,
                                    Some(&now_str),
                                    Some(err.category.as_str()),
                                    Some(chosen_health.consecutive_failures + 1),
                                    None,
                                    &now_str,
                                );
                            }

                            // BERHENTI RETRY SEGERA: Dilarang merotasi key untuk menghindari batas kuota akun
                            return PoolExecutionResult {
                                success: false,
                                response: None,
                                final_error: Some(err),
                                attempts: attempts_log,
                                preserved_request: request,
                            };
                        }

                        // KASUS B: Permintaan Tidak Valid, Penolakan Konten, atau Kesalahan Autentikasi (AT-035)
                        NormalizedErrorCategory::InvalidRequest
                        | NormalizedErrorCategory::ContentRefused
                        | NormalizedErrorCategory::AuthenticationFailed
                        | NormalizedErrorCategory::PermissionDenied => {
                            let new_state = if err.category
                                == NormalizedErrorCategory::AuthenticationFailed
                                || err.category == NormalizedErrorCategory::PermissionDenied
                            {
                                ProviderHealthState::NeedsAttention
                            } else {
                                ProviderHealthState::Ready
                            };

                            if let Ok(conn) = database.connection.lock() {
                                let _ = PoolRepository::update_health_record(
                                    &conn,
                                    &chosen_config.id,
                                    new_state.as_str(),
                                    None,
                                    None,
                                    Some(&now_str),
                                    Some(err.category.as_str()),
                                    Some(chosen_health.consecutive_failures + 1),
                                    None,
                                    &now_str,
                                );
                            }

                            // DILARANG merotasi membabi-buta untuk permintaan tidak valid / penolakan konten
                            if err.category == NormalizedErrorCategory::InvalidRequest
                                || err.category == NormalizedErrorCategory::ContentRefused
                            {
                                return PoolExecutionResult {
                                    success: false,
                                    response: None,
                                    final_error: Some(err),
                                    attempts: attempts_log,
                                    preserved_request: request,
                                };
                            }

                            // Untuk AuthFailed, jika auto_retry aktif dan ada key lain, lanjutkan failover
                        }

                        // KASUS C: Rate Limit Per-Credential (AT-033 - Hormati Retry-After & Cooldown, Boleh Failover)
                        NormalizedErrorCategory::RateLimited => {
                            let cooldown_ms = Self::calculate_cooldown_ms(
                                chosen_health.consecutive_failures,
                                settings.retry_base_delay_ms,
                                settings.max_retry_delay_ms,
                                err.retry_after_ms,
                            );

                            let cooldown_until_time = self.time_source.now()
                                + chrono::Duration::milliseconds(cooldown_ms as i64);
                            let cooldown_until_str = cooldown_until_time.to_rfc3339();

                            if let Ok(conn) = database.connection.lock() {
                                let _ = PoolRepository::update_health_record(
                                    &conn,
                                    &chosen_config.id,
                                    ProviderHealthState::CoolingDown.as_str(),
                                    Some(&cooldown_until_str),
                                    None,
                                    Some(&now_str),
                                    Some(err.category.as_str()),
                                    Some(chosen_health.consecutive_failures + 1),
                                    None,
                                    &now_str,
                                );
                            }
                        }

                        // KASUS D: Kesalahan Transient Jaringan / Server / Timeout
                        NormalizedErrorCategory::Timeout
                        | NormalizedErrorCategory::NetworkError
                        | NormalizedErrorCategory::ProviderServerError
                        | NormalizedErrorCategory::ResponseParseError
                        | NormalizedErrorCategory::ModelUnavailable
                        | NormalizedErrorCategory::Cancelled
                        | NormalizedErrorCategory::Unknown => {
                            if err.category == NormalizedErrorCategory::Cancelled {
                                return PoolExecutionResult {
                                    success: false,
                                    response: None,
                                    final_error: Some(err),
                                    attempts: attempts_log,
                                    preserved_request: request,
                                };
                            }

                            if let Ok(conn) = database.connection.lock() {
                                let _ = PoolRepository::update_health_record(
                                    &conn,
                                    &chosen_config.id,
                                    ProviderHealthState::Ready.as_str(),
                                    None,
                                    None,
                                    Some(&now_str),
                                    Some(err.category.as_str()),
                                    Some(chosen_health.consecutive_failures + 1),
                                    None,
                                    &now_str,
                                );
                            }
                        }
                    }
                }
            }
        }

        // Jika seluruh percobaan habis
        let final_err = last_error.unwrap_or_else(|| {
            NormalizedError::new(
                NormalizedErrorCategory::Unknown,
                "Seluruh upaya pengiriman ke provider AI telah dilakukan namun tidak berhasil.",
            )
        });

        PoolExecutionResult {
            success: false,
            response: None,
            final_error: Some(final_err),
            attempts: attempts_log,
            preserved_request: request,
        }
    }
}

impl Default for ProviderPoolService {
    fn default() -> Self {
        Self::new()
    }
}
