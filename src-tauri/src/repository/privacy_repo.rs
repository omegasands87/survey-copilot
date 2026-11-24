use chrono::Utc;
use rusqlite::{params, Connection};

use crate::domain::privacy::{DiagnosticsLogEntry, PrivacySettings};

pub struct PrivacyRepository;

impl PrivacyRepository {
    /// Mengambil konfigurasi privasi dari tabel app_settings
    pub fn get_privacy_settings(conn: &Connection) -> Result<PrivacySettings, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT value_json FROM app_settings WHERE key = 'privacy_settings' LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            if let Ok(settings) = serde_json::from_str::<PrivacySettings>(&json_str) {
                return Ok(settings);
            }
        }

        Ok(PrivacySettings::default())
    }

    /// Menyimpan atau memperbarui konfigurasi privasi ke tabel app_settings
    pub fn save_privacy_settings(
        conn: &Connection,
        settings: &PrivacySettings,
    ) -> Result<(), rusqlite::Error> {
        let value_json = serde_json::to_string(settings)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO app_settings (key, value_json, updated_at)
             VALUES ('privacy_settings', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET
               value_json = excluded.value_json,
               updated_at = excluded.updated_at",
            params![value_json, now],
        )?;

        Ok(())
    }

    /// Mencatat entri log diagnostik ke tabel app_diagnostic_logs
    pub fn insert_diagnostic_log(
        conn: &Connection,
        entry: &DiagnosticsLogEntry,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO app_diagnostic_logs (id, timestamp, level, category, message, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.id,
                entry.timestamp,
                entry.level,
                entry.category,
                entry.message,
                entry.metadata_json,
            ],
        )?;

        // Pertahankan maksimal 1000 entri log terbaru untuk menjaga ukuran basis data tetap ringan
        let _ = conn.execute(
            "DELETE FROM app_diagnostic_logs
             WHERE id NOT IN (
                 SELECT id FROM app_diagnostic_logs ORDER BY timestamp DESC LIMIT 1000
             )",
            [],
        );

        Ok(())
    }

    /// Mengambil daftar riwayat log diagnostik
    pub fn list_diagnostic_logs(
        conn: &Connection,
        limit: usize,
        level_filter: Option<&str>,
        category_filter: Option<&str>,
    ) -> Result<Vec<DiagnosticsLogEntry>, rusqlite::Error> {
        let mut query = String::from(
            "SELECT id, timestamp, level, category, message, metadata_json
             FROM app_diagnostic_logs WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(lvl) = level_filter {
            if !lvl.is_empty() && lvl != "ALL" {
                query.push_str(" AND level = ?");
                params_vec.push(Box::new(lvl.to_uppercase()));
            }
        }

        if let Some(cat) = category_filter {
            if !cat.is_empty() && cat != "ALL" {
                query.push_str(" AND category = ?");
                params_vec.push(Box::new(cat.to_uppercase()));
            }
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

        let rows = stmt.query_map(&param_refs[..], |row| {
            Ok(DiagnosticsLogEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                level: row.get(2)?,
                category: row.get(3)?,
                message: row.get(4)?,
                metadata_json: row.get(5)?,
            })
        })?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }

        Ok(entries)
    }

    /// Menghitung jumlah log yang tersimpan
    pub fn count_diagnostic_logs(conn: &Connection) -> Result<i64, rusqlite::Error> {
        conn.query_row("SELECT COUNT(*) FROM app_diagnostic_logs", [], |row| {
            row.get(0)
        })
    }

    /// Membersihkan seluruh log diagnostik
    pub fn clear_diagnostic_logs(conn: &Connection) -> Result<usize, rusqlite::Error> {
        conn.execute("DELETE FROM app_diagnostic_logs", [])
    }

    /// Mengambil metrik teknis basis data SQLite
    pub fn get_sqlite_stats(conn: &Connection) -> Result<(String, i64, i64), rusqlite::Error> {
        let version: String = conn.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |r| r.get(0))?;
        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
        let db_size = page_count * page_size;

        // Coba estimasi ukuran WAL
        let wal_pages: i64 = conn
            .query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |r| r.get(1))
            .unwrap_or(0);
        let wal_size = wal_pages * page_size;

        Ok((version, db_size, wal_size))
    }
}
