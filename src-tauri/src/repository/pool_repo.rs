use crate::domain::pool::{PoolSettings, PoolStrategy, ProviderHealthInfo, ProviderHealthState};
use rusqlite::{params, Connection, OptionalExtension};

pub struct PoolRepository;

impl PoolRepository {
    /// Mengambil konfigurasi global provider pool dari basis data
    pub fn get_settings(conn: &Connection) -> Result<PoolSettings, rusqlite::Error> {
        let result = conn.query_row(
            "SELECT id, strategy, max_attempts, auto_retry, retry_base_delay_ms, max_retry_delay_ms, updated_at
             FROM provider_pool_settings
             WHERE id = 'default'",
            [],
            |row| {
                let strat_str: String = row.get(1)?;
                let strategy = PoolStrategy::from_str_loose(&strat_str).unwrap_or(PoolStrategy::Adaptive);
                let auto_retry_int: i32 = row.get(3)?;
                let base_delay: i64 = row.get(4)?;
                let max_delay: i64 = row.get(5)?;
                Ok(PoolSettings {
                    id: row.get(0)?,
                    strategy,
                    max_attempts: row.get(2)?,
                    auto_retry: auto_retry_int != 0,
                    retry_base_delay_ms: base_delay as u64,
                    max_retry_delay_ms: max_delay as u64,
                    updated_at: row.get(6)?,
                })
            },
        ).optional()?;

        match result {
            Some(settings) => Ok(settings),
            None => {
                let default_settings = PoolSettings::default();
                Self::create_default_settings(conn, &default_settings)?;
                Ok(default_settings)
            }
        }
    }

    /// Menyimpan baris default provider_pool_settings jika belum ada
    fn create_default_settings(
        conn: &Connection,
        settings: &PoolSettings,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT OR IGNORE INTO provider_pool_settings (
                id, strategy, max_attempts, auto_retry, retry_base_delay_ms, max_retry_delay_ms, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                settings.id,
                settings.strategy.as_str(),
                settings.max_attempts,
                if settings.auto_retry { 1 } else { 0 },
                settings.retry_base_delay_ms as i64,
                settings.max_retry_delay_ms as i64,
                settings.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Memperbarui pengaturan pool
    pub fn update_settings(
        conn: &Connection,
        settings: &PoolSettings,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE provider_pool_settings SET
                strategy = ?2,
                max_attempts = ?3,
                auto_retry = ?4,
                retry_base_delay_ms = ?5,
                max_retry_delay_ms = ?6,
                updated_at = ?7
             WHERE id = ?1",
            params![
                settings.id,
                settings.strategy.as_str(),
                settings.max_attempts,
                if settings.auto_retry { 1 } else { 0 },
                settings.retry_base_delay_ms as i64,
                settings.max_retry_delay_ms as i64,
                settings.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Mengambil seluruh status health provider digabung dengan metadata konfigurasi
    pub fn list_all_health(conn: &Connection) -> Result<Vec<ProviderHealthInfo>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT
                c.id,
                c.display_name,
                COALESCE(h.state, 'ready'),
                h.cooldown_until,
                h.last_success_at,
                h.last_failure_at,
                h.last_error_category,
                COALESCE(h.consecutive_failures, 0),
                h.recent_latency_ms,
                c.max_concurrency,
                c.priority,
                c.weight,
                c.enabled,
                COALESCE(h.updated_at, c.updated_at)
             FROM provider_configs c
             LEFT JOIN provider_health h ON c.id = h.provider_config_id
             ORDER BY c.priority ASC, c.created_at ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let state_str: String = row.get(2)?;
            let enabled_int: i32 = row.get(12)?;
            let latency: Option<i64> = row.get(8)?;
            let consecutive_failures: i64 = row.get(7)?;

            let state = if enabled_int == 0 {
                ProviderHealthState::Disabled
            } else {
                ProviderHealthState::from_str_loose(&state_str)
                    .unwrap_or(ProviderHealthState::Ready)
            };

            Ok(ProviderHealthInfo {
                provider_config_id: row.get(0)?,
                display_name: row.get(1)?,
                state,
                cooldown_until: row.get(3)?,
                last_success_at: row.get(4)?,
                last_failure_at: row.get(5)?,
                last_error_category: row.get(6)?,
                consecutive_failures: consecutive_failures as u32,
                recent_latency_ms: latency.map(|l| l as u64),
                current_concurrency: 0, // In-memory runtime concurrency akan diisi oleh service
                max_concurrency: row.get(9)?,
                priority: row.get(10)?,
                weight: row.get(11)?,
                enabled: enabled_int != 0,
                updated_at: row.get(13)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Mengambil status health untuk konfigurasi spesifik
    pub fn get_health(
        conn: &Connection,
        config_id: &str,
    ) -> Result<Option<ProviderHealthInfo>, rusqlite::Error> {
        let result = conn
            .query_row(
                "SELECT
                c.id,
                c.display_name,
                COALESCE(h.state, 'ready'),
                h.cooldown_until,
                h.last_success_at,
                h.last_failure_at,
                h.last_error_category,
                COALESCE(h.consecutive_failures, 0),
                h.recent_latency_ms,
                c.max_concurrency,
                c.priority,
                c.weight,
                c.enabled,
                COALESCE(h.updated_at, c.updated_at)
             FROM provider_configs c
             LEFT JOIN provider_health h ON c.id = h.provider_config_id
             WHERE c.id = ?1",
                params![config_id],
                |row| {
                    let state_str: String = row.get(2)?;
                    let enabled_int: i32 = row.get(12)?;
                    let latency: Option<i64> = row.get(8)?;
                    let consecutive_failures: i64 = row.get(7)?;

                    let state = if enabled_int == 0 {
                        ProviderHealthState::Disabled
                    } else {
                        ProviderHealthState::from_str_loose(&state_str)
                            .unwrap_or(ProviderHealthState::Ready)
                    };

                    Ok(ProviderHealthInfo {
                        provider_config_id: row.get(0)?,
                        display_name: row.get(1)?,
                        state,
                        cooldown_until: row.get(3)?,
                        last_success_at: row.get(4)?,
                        last_failure_at: row.get(5)?,
                        last_error_category: row.get(6)?,
                        consecutive_failures: consecutive_failures as u32,
                        recent_latency_ms: latency.map(|l| l as u64),
                        current_concurrency: 0,
                        max_concurrency: row.get(9)?,
                        priority: row.get(10)?,
                        weight: row.get(11)?,
                        enabled: enabled_int != 0,
                        updated_at: row.get(13)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Mereset provider yang masa cooldown-nya sudah kadaluarsa kembali ke 'ready'
    pub fn reset_cooldown_if_expired(
        conn: &Connection,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        let count = conn.execute(
            "UPDATE provider_health
             SET state = 'ready', cooldown_until = NULL, updated_at = ?1
             WHERE state = 'cooling_down'
               AND cooldown_until IS NOT NULL
               AND cooldown_until <= ?1",
            params![now],
        )?;
        Ok(count)
    }

    /// Memperbarui record status health provider
    #[allow(clippy::too_many_arguments)]
    pub fn update_health_record(
        conn: &Connection,
        config_id: &str,
        state: &str,
        cooldown_until: Option<&str>,
        last_success_at: Option<&str>,
        last_failure_at: Option<&str>,
        last_error_category: Option<&str>,
        consecutive_failures: Option<u32>,
        recent_latency_ms: Option<u64>,
        now: &str,
    ) -> Result<(), rusqlite::Error> {
        // Upsert ke provider_health
        conn.execute(
            "INSERT INTO provider_health (
                provider_config_id, state, cooldown_until, last_success_at,
                last_failure_at, last_error_category, consecutive_failures,
                recent_latency_ms, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, COALESCE(?7, 0), ?8, ?9)
            ON CONFLICT(provider_config_id) DO UPDATE SET
                state = excluded.state,
                cooldown_until = excluded.cooldown_until,
                last_success_at = COALESCE(excluded.last_success_at, provider_health.last_success_at),
                last_failure_at = COALESCE(excluded.last_failure_at, provider_health.last_failure_at),
                last_error_category = COALESCE(excluded.last_error_category, provider_health.last_error_category),
                consecutive_failures = COALESCE(excluded.consecutive_failures, provider_health.consecutive_failures),
                recent_latency_ms = COALESCE(excluded.recent_latency_ms, provider_health.recent_latency_ms),
                updated_at = excluded.updated_at",
            params![
                config_id,
                state,
                cooldown_until,
                last_success_at,
                last_failure_at,
                last_error_category,
                consecutive_failures.map(|f| f as i64),
                recent_latency_ms.map(|l| l as i64),
                now,
            ],
        )?;
        Ok(())
    }
}
