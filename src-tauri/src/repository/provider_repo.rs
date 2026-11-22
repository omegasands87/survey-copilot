use crate::domain::provider::ProviderConfig;
use rusqlite::{params, Connection, OptionalExtension, Row};

pub struct ProviderRepository;

impl ProviderRepository {
    fn map_row(row: &Row) -> Result<ProviderConfig, rusqlite::Error> {
        let enabled_int: i32 = row.get(7)?;
        Ok(ProviderConfig {
            id: row.get(0)?,
            display_name: row.get(1)?,
            provider_type: row.get(2)?,
            base_url: row.get(3)?,
            adapter_type: row.get(4)?,
            model_id: row.get(5)?,
            credential_ref: row.get(6)?,
            enabled: enabled_int != 0,
            priority: row.get(8)?,
            weight: row.get(9)?,
            timeout_ms: row.get(10)?,
            max_concurrency: row.get(11)?,
            rate_limit_hint_json: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }

    /// Menyimpan konfigurasi provider baru ke tabel provider_configs
    pub fn create(conn: &Connection, config: &ProviderConfig) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO provider_configs (
                id, display_name, provider_type, base_url, adapter_type,
                model_id, credential_ref, enabled, priority, weight,
                timeout_ms, max_concurrency, rate_limit_hint_json,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                config.id,
                config.display_name,
                config.provider_type,
                config.base_url,
                config.adapter_type,
                config.model_id,
                config.credential_ref,
                if config.enabled { 1 } else { 0 },
                config.priority,
                config.weight,
                config.timeout_ms,
                config.max_concurrency,
                config.rate_limit_hint_json,
                config.created_at,
                config.updated_at,
            ],
        )?;

        // Inisialisasi baris status health provider
        conn.execute(
            "INSERT OR IGNORE INTO provider_health (
                provider_config_id, state, consecutive_failures, updated_at
            ) VALUES (?1, 'ready', 0, ?2)",
            params![config.id, config.updated_at],
        )?;

        Ok(())
    }

    /// Mencari konfigurasi provider berdasarkan ID
    pub fn find_by_id(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<ProviderConfig>, rusqlite::Error> {
        conn.query_row(
            "SELECT id, display_name, provider_type, base_url, adapter_type,
                    model_id, credential_ref, enabled, priority, weight,
                    timeout_ms, max_concurrency, rate_limit_hint_json,
                    created_at, updated_at
             FROM provider_configs
             WHERE id = ?1",
            params![id],
            Self::map_row,
        )
        .optional()
    }

    /// Mengambil seluruh konfigurasi provider diurutkan berdasarkan prioritas dan tanggal pembuatan
    pub fn list_all(conn: &Connection) -> Result<Vec<ProviderConfig>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, display_name, provider_type, base_url, adapter_type,
                    model_id, credential_ref, enabled, priority, weight,
                    timeout_ms, max_concurrency, rate_limit_hint_json,
                    created_at, updated_at
             FROM provider_configs
             ORDER BY priority ASC, created_at ASC",
        )?;

        let rows = stmt.query_map([], Self::map_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Mengambil konfigurasi provider yang aktif (enabled = 1)
    pub fn list_enabled(conn: &Connection) -> Result<Vec<ProviderConfig>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, display_name, provider_type, base_url, adapter_type,
                    model_id, credential_ref, enabled, priority, weight,
                    timeout_ms, max_concurrency, rate_limit_hint_json,
                    created_at, updated_at
             FROM provider_configs
             WHERE enabled = 1
             ORDER BY priority ASC, created_at ASC",
        )?;

        let rows = stmt.query_map([], Self::map_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Memperbarui konfigurasi provider
    pub fn update(conn: &Connection, config: &ProviderConfig) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE provider_configs SET
                display_name = ?2,
                base_url = ?3,
                model_id = ?4,
                credential_ref = ?5,
                enabled = ?6,
                priority = ?7,
                weight = ?8,
                timeout_ms = ?9,
                max_concurrency = ?10,
                rate_limit_hint_json = ?11,
                updated_at = ?12
             WHERE id = ?1",
            params![
                config.id,
                config.display_name,
                config.base_url,
                config.model_id,
                config.credential_ref,
                if config.enabled { 1 } else { 0 },
                config.priority,
                config.weight,
                config.timeout_ms,
                config.max_concurrency,
                config.rate_limit_hint_json,
                config.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Menghapus konfigurasi provider berdasarkan ID
    pub fn delete(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
        let count = conn.execute("DELETE FROM provider_configs WHERE id = ?1", params![id])?;
        Ok(count > 0)
    }

    /// Menghitung berapa banyak konfigurasi provider yang menggunakan credential_ref tertentu
    pub fn count_by_credential_ref(
        conn: &Connection,
        credential_ref: &str,
    ) -> Result<usize, rusqlite::Error> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM provider_configs WHERE credential_ref = ?1",
            params![credential_ref],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Memperbarui status health provider setelah pengujian atau operasi AI
    pub fn update_health(
        conn: &Connection,
        config_id: &str,
        state: &str,
        last_success_at: Option<&str>,
        last_failure_at: Option<&str>,
        last_error_category: Option<&str>,
        latency_ms: Option<u64>,
        now: &str,
    ) -> Result<(), rusqlite::Error> {
        if last_success_at.is_some() {
            conn.execute(
                "UPDATE provider_health SET
                    state = ?2,
                    last_success_at = ?3,
                    recent_latency_ms = ?4,
                    consecutive_failures = 0,
                    updated_at = ?5
                 WHERE provider_config_id = ?1",
                params![
                    config_id,
                    state,
                    last_success_at,
                    latency_ms.map(|l| l as i64),
                    now
                ],
            )?;
        } else if last_failure_at.is_some() {
            conn.execute(
                "UPDATE provider_health SET
                    state = ?2,
                    last_failure_at = ?3,
                    last_error_category = ?4,
                    consecutive_failures = consecutive_failures + 1,
                    updated_at = ?5
                 WHERE provider_config_id = ?1",
                params![config_id, state, last_failure_at, last_error_category, now],
            )?;
        } else {
            conn.execute(
                "UPDATE provider_health SET
                    state = ?2,
                    updated_at = ?3
                 WHERE provider_config_id = ?1",
                params![config_id, state, now],
            )?;
        }
        Ok(())
    }
}
