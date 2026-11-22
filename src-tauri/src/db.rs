use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

pub struct Database {
    pub(crate) connection: Mutex<Connection>,
}

impl Database {
    pub fn initialize(path: &Path) -> Result<Self, String> {
        let connection = Connection::open(path)
            .map_err(|error| format!("Gagal membuka database lokal: {error}"))?;

        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|error| format!("Gagal mengatur database: {error}"))?;

        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| format!("Gagal mengaktifkan foreign key: {error}"))?;

        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|error| format!("Gagal mengatur mode database: {error}"))?;

        connection
            .execute_batch(include_str!("../migrations/0001_initial.sql"))
            .map_err(|error| format!("Gagal menjalankan migrasi database: {error}"))?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }
}
