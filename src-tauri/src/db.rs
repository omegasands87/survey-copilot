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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OptionalExtension;

    #[test]
    fn initial_migration_creates_persona_and_provider_tables() {
        let database = Database::initialize(Path::new(":memory:"))
            .expect("initial database migration should succeed");
        let connection = database
            .connection
            .lock()
            .expect("database lock should succeed");

        for table in [
            "personas",
            "profile_facts",
            "survey_sessions",
            "survey_opportunities",
            "survey_fit_results",
            "survey_attempts",
            "provider_configs",
        ] {
            let found: Option<String> = connection
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .optional()
                .expect("schema query should succeed");
            assert_eq!(found.as_deref(), Some(table));
        }

        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign key pragma should be readable");
        assert_eq!(foreign_keys, 1);
    }
}
