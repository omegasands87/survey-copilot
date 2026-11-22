use crate::migrations::{MigrationReport, MigrationRunner};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

pub struct Database {
    pub connection: Mutex<Connection>,
}

impl Database {
    /// Initializes database connection, sets pragmas, and runs all pending migrations.
    pub fn initialize(path: &Path) -> Result<Self, String> {
        let runner = MigrationRunner::with_app_migrations();
        Self::initialize_with_runner(path, &runner)
    }

    /// Initializes database connection using a custom migration runner (useful for tests and upgrades).
    pub fn initialize_with_runner(path: &Path, runner: &MigrationRunner) -> Result<Self, String> {
        let mut connection = Connection::open(path)
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

        let _report: MigrationReport = runner
            .run(&mut connection)
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
            "_migrations",
            "personas",
            "profile_facts",
            "fact_sources",
            "persona_preferences",
            "survey_sessions",
            "screen_captures",
            "survey_questions",
            "survey_opportunities",
            "survey_opportunity_criteria",
            "survey_fit_results",
            "survey_attempts",
            "survey_outcome_events",
            "provider_configs",
            "provider_pool_settings",
            "provider_health",
            "answer_recommendations",
            "recommendation_sources",
            "answer_history",
            "ai_usage_events",
            "app_settings",
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

        let migration_count: i64 = connection
            .query_row("SELECT count(*) FROM _migrations", [], |row| row.get(0))
            .expect("query _migrations should succeed");
        assert_eq!(migration_count, 1);
    }
}
