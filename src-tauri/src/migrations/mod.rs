use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fmt;
use std::time::Instant;

/// Represents a single versioned migration script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

impl Migration {
    pub const fn new(version: i64, name: &'static str, sql: &'static str) -> Self {
        Self { version, name, sql }
    }
}

/// A record stored in the `_migrations` table tracking applied migrations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationRecord {
    pub version: i64,
    pub name: String,
    pub checksum: String,
    pub applied_at: String,
    pub execution_duration_ms: i64,
}

/// Summary report returned after a migration run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub applied_count: usize,
    pub skipped_count: usize,
    pub applied_versions: Vec<i64>,
}

/// Errors that can occur during migration validation and execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    Database(String),
    ChecksumMismatch {
        version: i64,
        name: String,
        recorded: String,
        computed: String,
    },
    DuplicateVersion(i64),
    VersionGap {
        expected: i64,
        actual: i64,
    },
    UnknownFutureVersion {
        recorded_version: i64,
        max_known_version: i64,
    },
    ExecutionFailed {
        version: i64,
        name: String,
        reason: String,
    },
    BaselineAdoptionFailed(String),
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "Kesalahan basis data saat migrasi: {msg}"),
            Self::ChecksumMismatch {
                version,
                name,
                recorded,
                computed,
            } => write!(
                f,
                "Integritas migrasi v{version} ({name}) tidak cocok: terekam '{recorded}', dihitung '{computed}'"
            ),
            Self::DuplicateVersion(v) => {
                write!(f, "Versi migrasi terduplikasi pada registri: v{v}")
            }
            Self::VersionGap { expected, actual } => write!(
                f,
                "Registri migrasi tidak berurutan: diharapkan v{expected}, ditemukan v{actual}"
            ),
            Self::UnknownFutureVersion {
                recorded_version,
                max_known_version,
            } => write!(
                f,
                "Versi skema basis data v{recorded_version} lebih baru dari versi aplikasi yang didukung (maks v{max_known_version})"
            ),
            Self::ExecutionFailed {
                version,
                name,
                reason,
            } => write!(
                f,
                "Gagal mengeksekusi migrasi v{version} ({name}): {reason}"
            ),
            Self::BaselineAdoptionFailed(msg) => {
                write!(f, "Gagal mengadopsi baseline basis data: {msg}")
            }
        }
    }
}

impl std::error::Error for MigrationError {}

impl From<rusqlite::Error> for MigrationError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<MigrationError> for crate::domain::error::AppError {
    fn from(err: MigrationError) -> Self {
        crate::domain::error::AppError::migration(err.to_string())
    }
}

/// Computes a deterministic SHA-256 checksum of SQL content.
///
/// Automatically normalizes CRLF line breaks to LF to ensure identical
/// hashes regardless of Git checkout line-ending conventions on Windows.
pub fn compute_checksum(sql: &str) -> String {
    let normalized = sql.replace("\r\n", "\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Registry and sequential runner for database migrations.
pub struct MigrationRunner {
    migrations: Vec<Migration>,
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRunner {
    /// Creates an empty migration runner.
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Creates a runner populated with all application-defined migrations.
    pub fn with_app_migrations() -> Self {
        let mut runner = Self::new();
        for migration in get_registered_migrations() {
            runner
                .register(migration)
                .expect("built-in migrations must have valid sequential versions");
        }
        runner
    }

    /// Registers a new migration into the runner.
    /// Validates version sequence and uniqueness.
    pub fn register(&mut self, migration: Migration) -> Result<(), MigrationError> {
        if let Some(last) = self.migrations.last() {
            if migration.version == last.version {
                return Err(MigrationError::DuplicateVersion(migration.version));
            }
            if migration.version != last.version + 1 {
                return Err(MigrationError::VersionGap {
                    expected: last.version + 1,
                    actual: migration.version,
                });
            }
        } else if migration.version != 1 {
            return Err(MigrationError::VersionGap {
                expected: 1,
                actual: migration.version,
            });
        }

        self.migrations.push(migration);
        Ok(())
    }

    /// Returns the list of registered migrations.
    pub fn migrations(&self) -> &[Migration] {
        &self.migrations
    }

    /// Maximum registered version, or 0 if empty.
    pub fn max_version(&self) -> i64 {
        self.migrations.last().map(|m| m.version).unwrap_or(0)
    }

    /// Runs all pending migrations against the given SQLite connection.
    ///
    /// - Creates `_migrations` tracking table if not present.
    /// - Adopts existing databases safely without dropping tables or losing data.
    /// - Validates checksums of previously applied migrations.
    /// - Runs unapplied migrations sequentially inside isolated transactions.
    pub fn run(&self, conn: &mut Connection) -> Result<MigrationReport, MigrationError> {
        let max_known = self.max_version();

        // 1. Ensure tracking infrastructure exists & adopt baseline if existing DB
        self.ensure_tracking_table_and_baseline(conn)?;

        // 2. Fetch all applied migrations from _migrations
        let applied_records = self.fetch_applied_records(conn)?;

        // 3. Verify integrity of all previously applied migrations
        let mut applied_version_set = std::collections::HashSet::new();
        for record in &applied_records {
            applied_version_set.insert(record.version);

            if record.version > max_known {
                return Err(MigrationError::UnknownFutureVersion {
                    recorded_version: record.version,
                    max_known_version: max_known,
                });
            }

            // Find matching registered migration and verify checksum
            if let Some(registered) = self.migrations.iter().find(|m| m.version == record.version) {
                let computed = compute_checksum(registered.sql);
                if computed != record.checksum {
                    return Err(MigrationError::ChecksumMismatch {
                        version: record.version,
                        name: record.name.clone(),
                        recorded: record.checksum.clone(),
                        computed,
                    });
                }
            }
        }

        // 4. Identify pending migrations in strictly sequential order
        let mut report = MigrationReport {
            applied_count: 0,
            skipped_count: applied_records.len(),
            applied_versions: Vec::new(),
        };

        for migration in &self.migrations {
            if applied_version_set.contains(&migration.version) {
                continue;
            }

            // Execute the pending migration inside an immediate transaction
            let duration_ms = self.execute_single_migration(conn, migration)?;

            report.applied_count += 1;
            report.applied_versions.push(migration.version);

            log_migration_success(migration.version, migration.name, duration_ms);
        }

        Ok(report)
    }

    /// Ensures the `_migrations` table exists.
    ///
    /// If the database was created by a previous version (which has `personas` or
    /// `schema_migrations` but no `_migrations` table), adopts migration 1 as a
    /// baseline so initial tables and existing user data are never wiped or re-executed.
    fn ensure_tracking_table_and_baseline(
        &self,
        conn: &mut Connection,
    ) -> Result<(), MigrationError> {
        let has_migrations_table: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = '_migrations'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| MigrationError::Database(e.to_string()))?;

        if !has_migrations_table {
            // Check if this is an existing database from v1.0 initial schema
            let has_personas_table: bool = conn
                .query_row(
                    "SELECT count(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'personas'",
                    [],
                    |row| row.get(0),
                )
                .map_err(|e| MigrationError::Database(e.to_string()))?;

            // Create tracking table
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS _migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    checksum TEXT NOT NULL,
                    applied_at TEXT NOT NULL,
                    execution_duration_ms INTEGER NOT NULL
                );",
            )
            .map_err(|e| MigrationError::Database(e.to_string()))?;

            if has_personas_table {
                // Existing database: adopt baseline for v1
                let applied_at: String = conn
                    .query_row(
                        "SELECT applied_at FROM schema_migrations WHERE version = 1",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

                // Find registered v1 migration if available to record proper checksum
                let (name, checksum) =
                    if let Some(v1) = self.migrations.iter().find(|m| m.version == 1) {
                        (v1.name, compute_checksum(v1.sql))
                    } else {
                        ("0001_initial", String::from("baseline_adopted"))
                    };

                conn.execute(
                    "INSERT INTO _migrations (version, name, checksum, applied_at, execution_duration_ms)
                     VALUES (1, ?1, ?2, ?3, 0)",
                    rusqlite::params![name, checksum, applied_at],
                )
                .map_err(|e| {
                    MigrationError::BaselineAdoptionFailed(format!(
                        "Gagal menyisipkan entri baseline v1: {e}"
                    ))
                })?;
            }
        }

        Ok(())
    }

    /// Fetches all applied migration records from `_migrations`.
    fn fetch_applied_records(
        &self,
        conn: &Connection,
    ) -> Result<Vec<MigrationRecord>, MigrationError> {
        let mut stmt = conn
            .prepare(
                "SELECT version, name, checksum, applied_at, execution_duration_ms
                 FROM _migrations
                 ORDER BY version ASC",
            )
            .map_err(|e| MigrationError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(MigrationRecord {
                    version: row.get(0)?,
                    name: row.get(1)?,
                    checksum: row.get(2)?,
                    applied_at: row.get(3)?,
                    execution_duration_ms: row.get(4)?,
                })
            })
            .map_err(|e| MigrationError::Database(e.to_string()))?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(|e| MigrationError::Database(e.to_string()))?);
        }

        Ok(records)
    }

    /// Executes a single migration inside an isolated transaction.
    ///
    /// If any statement fails, the transaction is automatically rolled back,
    /// leaving no partial changes or incomplete records in `_migrations`.
    fn execute_single_migration(
        &self,
        conn: &mut Connection,
        migration: &Migration,
    ) -> Result<i64, MigrationError> {
        let start = Instant::now();
        let checksum = compute_checksum(migration.sql);
        let applied_at = chrono::Utc::now().to_rfc3339();

        let tx = conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|e| MigrationError::Database(e.to_string()))?;

        // Execute migration SQL batch
        tx.execute_batch(migration.sql)
            .map_err(|e| MigrationError::ExecutionFailed {
                version: migration.version,
                name: migration.name.to_string(),
                reason: e.to_string(),
            })?;

        let duration_ms = start.elapsed().as_millis() as i64;

        // Record successful migration
        tx.execute(
            "INSERT INTO _migrations (version, name, checksum, applied_at, execution_duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                migration.version,
                migration.name,
                checksum,
                applied_at,
                duration_ms
            ],
        )
        .map_err(|e| MigrationError::Database(e.to_string()))?;

        // Commit transaction
        tx.commit()
            .map_err(|e| MigrationError::Database(e.to_string()))?;

        Ok(duration_ms)
    }
}

fn log_migration_success(version: i64, name: &str, duration_ms: i64) {
    #[cfg(debug_assertions)]
    println!("[MIGRATION] Sukses menerapkan v{version} ({name}) dalam {duration_ms}ms");
    let _ = (version, name, duration_ms);
}

/// Returns the registry of all built-in migrations for the application.
pub fn get_registered_migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            1,
            "0001_initial",
            include_str!("../../migrations/0001_initial.sql"),
        ),
        Migration::new(
            2,
            "0002_typed_facts_and_history",
            include_str!("../../migrations/0002_typed_facts_and_history.sql"),
        ),
        Migration::new(
            3,
            "0003_app_credentials",
            include_str!("../../migrations/0003_app_credentials.sql"),
        ),
        Migration::new(
            4,
            "0004_provider_config_support",
            include_str!("../../migrations/0004_provider_config_support.sql"),
        ),
        Migration::new(
            5,
            "0005_survey_sessions_and_questions",
            include_str!("../../migrations/0005_survey_sessions_and_questions.sql"),
        ),
        Migration::new(
            6,
            "0006_answer_recommendations_fields",
            include_str!("../../migrations/0006_answer_recommendations_fields.sql"),
        ),
        Migration::new(
            7,
            "0007_review_actions_and_history",
            include_str!("../../migrations/0007_review_actions_and_history.sql"),
        ),
        Migration::new(
            8,
            "0008_ocr_extractions",
            include_str!("../../migrations/0008_ocr_extractions.sql"),
        ),
        Migration::new(
            9,
            "0009_survey_opportunities_and_criteria",
            include_str!("../../migrations/0009_survey_opportunities_and_criteria.sql"),
        ),
        Migration::new(
            10,
            "0010_survey_attempts_indexes",
            include_str!("../../migrations/0010_survey_attempts_indexes.sql"),
        ),
        Migration::new(
            11,
            "0011_survey_memory_and_recognition",
            include_str!("../../migrations/0011_survey_memory_and_recognition.sql"),
        ),
        Migration::new(
            12,
            "0012_next_question_intelligence_and_branch_prediction",
            include_str!("../../migrations/0012_next_question_intelligence_and_branch_prediction.sql"),
        ),
        Migration::new(
            13,
            "0013_privacy_settings_and_diagnostics",
            include_str!("../../migrations/0013_privacy_settings_and_diagnostics.sql"),
        ),
    ]
}

/// Convenience entry point to run all registered migrations on a connection.
pub fn run_migrations(conn: &mut Connection) -> Result<MigrationReport, MigrationError> {
    let runner = MigrationRunner::with_app_migrations();
    runner.run(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OptionalExtension;

    #[test]
    fn test_compute_checksum_is_deterministic_and_crlf_agnostic() {
        let unix_sql = "CREATE TABLE test (\n  id INTEGER PRIMARY KEY\n);\n";
        let win_sql = "CREATE TABLE test (\r\n  id INTEGER PRIMARY KEY\r\n);\r\n";

        let hash1 = compute_checksum(unix_sql);
        let hash2 = compute_checksum(win_sql);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_runner_validates_sequential_versions() {
        let mut runner = MigrationRunner::new();
        assert!(runner.register(Migration::new(2, "two", "")).is_err()); // Must start at 1

        assert!(runner.register(Migration::new(1, "one", "")).is_ok());
        assert!(runner.register(Migration::new(1, "dup", "")).is_err()); // Duplicate
        assert!(runner.register(Migration::new(3, "three", "")).is_err()); // Gap
        assert!(runner.register(Migration::new(2, "two", "")).is_ok()); // Sequential
    }

    #[test]
    fn test_fresh_database_initialization() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        let runner = MigrationRunner::with_app_migrations();
        let total_migrations = runner.migrations().len();

        let report = runner
            .run(&mut conn)
            .expect("initial migration should succeed");
        assert_eq!(report.applied_count, total_migrations);
        assert_eq!(report.skipped_count, 0);
        assert_eq!(
            report.applied_versions,
            (1..=total_migrations as i64).collect::<Vec<_>>()
        );

        // Check _migrations table
        let count: i64 = conn
            .query_row("SELECT count(*) FROM _migrations", [], |row| row.get(0))
            .expect("query _migrations should succeed");
        assert_eq!(count, total_migrations as i64);

        // Check core tables exist
        let personas_table: Option<String> = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='personas'",
                [],
                |row| row.get(0),
            )
            .optional()
            .expect("query master should succeed");
        assert_eq!(personas_table.as_deref(), Some("personas"));
    }

    #[test]
    fn test_idempotent_repeated_startup() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        let runner = MigrationRunner::with_app_migrations();
        let total_migrations = runner.migrations().len();

        // Run 1
        let report1 = runner.run(&mut conn).expect("first run should succeed");
        assert_eq!(report1.applied_count, total_migrations);

        // Run 2: idempotent, nothing applied
        let report2 = runner.run(&mut conn).expect("second run should succeed");
        assert_eq!(report2.applied_count, 0);
        assert_eq!(report2.skipped_count, total_migrations);
        assert!(report2.applied_versions.is_empty());
    }

    #[test]
    fn test_sequential_upgrade_migration() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");

        // Initial v1
        let mut runner_v1 = MigrationRunner::new();
        runner_v1
            .register(Migration::new(
                1,
                "0001_initial",
                "CREATE TABLE test_v1 (id INTEGER PRIMARY KEY, val TEXT);",
            ))
            .unwrap();

        let report1 = runner_v1.run(&mut conn).unwrap();
        assert_eq!(report1.applied_count, 1);

        // Upgrade to v2
        let mut runner_v2 = MigrationRunner::new();
        runner_v2
            .register(Migration::new(
                1,
                "0001_initial",
                "CREATE TABLE test_v1 (id INTEGER PRIMARY KEY, val TEXT);",
            ))
            .unwrap();
        runner_v2
            .register(Migration::new(
                2,
                "0002_add_v2",
                "CREATE TABLE test_v2 (id INTEGER PRIMARY KEY, label TEXT);",
            ))
            .unwrap();

        let report2 = runner_v2.run(&mut conn).unwrap();
        assert_eq!(report2.applied_count, 1);
        assert_eq!(report2.skipped_count, 1);
        assert_eq!(report2.applied_versions, vec![2]);

        // Verify both tables exist
        let t1: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='test_v1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let t2: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='test_v2'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(t1 && t2);
    }

    #[test]
    fn test_checksum_mismatch_detected() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");

        let mut runner = MigrationRunner::new();
        runner
            .register(Migration::new(
                1,
                "0001_initial",
                "CREATE TABLE test (id INT);",
            ))
            .unwrap();
        runner.run(&mut conn).unwrap();

        // Runner with modified SQL for v1
        let mut tampered_runner = MigrationRunner::new();
        tampered_runner
            .register(Migration::new(
                1,
                "0001_initial",
                "CREATE TABLE test (id INT, modified INT);",
            ))
            .unwrap();

        let result = tampered_runner.run(&mut conn);
        match result {
            Err(MigrationError::ChecksumMismatch { version, name, .. }) => {
                assert_eq!(version, 1);
                assert_eq!(name, "0001_initial");
            }
            other => panic!("Expected ChecksumMismatch error, got: {other:?}"),
        }
    }

    #[test]
    fn test_migration_failure_rolls_back_transaction() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");

        let mut runner = MigrationRunner::new();
        runner
            .register(Migration::new(
                1,
                "0001_good",
                "CREATE TABLE base_table (id INT PRIMARY KEY);",
            ))
            .unwrap();
        runner.run(&mut conn).unwrap();

        // Migration 2 creates a table then fails with bad syntax
        let mut runner_with_bad = MigrationRunner::new();
        runner_with_bad
            .register(Migration::new(
                1,
                "0001_good",
                "CREATE TABLE base_table (id INT PRIMARY KEY);",
            ))
            .unwrap();
        runner_with_bad
            .register(Migration::new(
                2,
                "0002_bad",
                "CREATE TABLE partial_table (id INT); INVALID SQL SYNTAX;",
            ))
            .unwrap();

        let result = runner_with_bad.run(&mut conn);
        assert!(result.is_err());

        // partial_table must NOT exist due to rollback
        let partial_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='partial_table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!partial_exists, "partial table should be rolled back");

        // Version 2 must NOT be in _migrations
        let count_v2: i64 = conn
            .query_row(
                "SELECT count(*) FROM _migrations WHERE version = 2",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count_v2, 0);
    }

    #[test]
    fn test_existing_database_baseline_adoption() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");

        // Simulate an existing database created before SC-003:
        // has full initial schema and `schema_migrations`, but NO `_migrations` table
        conn.execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .unwrap();
        conn.execute_batch(
            "UPDATE schema_migrations SET applied_at = '2026-11-20T12:00:00Z' WHERE version = 1;
            INSERT INTO personas (id, display_name, is_active, status, created_at, updated_at)
            VALUES ('p-existing', 'Budi Santoso', 1, 'active', '2026-11-20T12:00:00Z', '2026-11-20T12:00:00Z');",
        )
        .unwrap();

        let runner = MigrationRunner::with_app_migrations();
        let expected_applied = (runner.migrations().len() - 1) as usize;
        let expected_total = runner.migrations().len() as i64;
        let report = runner
            .run(&mut conn)
            .expect("baseline adoption should succeed");

        assert_eq!(report.applied_count, expected_applied);
        assert_eq!(report.skipped_count, 1);

        // Verify _migrations table was created
        let count: i64 = conn
            .query_row("SELECT count(*) FROM _migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, expected_total);

        let (ver, name, applied_at): (i64, String, String) = conn
            .query_row(
                "SELECT version, name, applied_at FROM _migrations WHERE version = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(ver, 1);
        assert_eq!(name, "0001_initial");
        assert_eq!(applied_at, "2026-11-20T12:00:00Z");

        // Verify user data was preserved
        let person_name: String = conn
            .query_row(
                "SELECT display_name FROM personas WHERE id = 'p-existing'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(person_name, "Budi Santoso");
    }

    #[test]
    fn test_unknown_future_schema_version_rejected() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");

        // Manually create _migrations with future version 99
        conn.execute_batch(
            "CREATE TABLE _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                checksum TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                execution_duration_ms INTEGER NOT NULL
            );
            INSERT INTO _migrations (version, name, checksum, applied_at, execution_duration_ms)
            VALUES (99, 'future_version', 'abcdef', '2026-11-22T00:00:00Z', 10);",
        )
        .unwrap();

        let runner = MigrationRunner::with_app_migrations();
        let max_ver = runner.max_version();
        let result = runner.run(&mut conn);
        match result {
            Err(MigrationError::UnknownFutureVersion {
                recorded_version,
                max_known_version,
            }) => {
                assert_eq!(recorded_version, 99);
                assert_eq!(max_known_version, max_ver);
            }
            other => panic!("Expected UnknownFutureVersion error, got: {other:?}"),
        }
    }
}
