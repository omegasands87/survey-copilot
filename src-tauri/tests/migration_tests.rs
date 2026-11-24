use rusqlite::Connection;
use survey_copilot_lib::migrations::{
    compute_checksum, Migration, MigrationError, MigrationRunner,
};

#[test]
fn at_002_fresh_database_initialization() {
    let mut conn = Connection::open_in_memory().expect("in-memory db open");
    let app_runner = MigrationRunner::with_app_migrations();
    let total_migrations = app_runner.migrations().len();
    let report = app_runner.run(&mut conn).expect("run app migrations");

    assert_eq!(report.applied_count, total_migrations);
    assert_eq!(report.skipped_count, 0);
    assert_eq!(
        report.applied_versions,
        (1..=total_migrations as i64).collect::<Vec<_>>()
    );

    let count: i64 = conn
        .query_row("SELECT count(*) FROM _migrations", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, total_migrations as i64);
}

#[test]
fn at_003_idempotent_repeated_startup() {
    let mut conn = Connection::open_in_memory().expect("in-memory db open");
    let app_runner = MigrationRunner::with_app_migrations();
    let total_migrations = app_runner.migrations().len();

    let report1 = app_runner.run(&mut conn).expect("first run");
    assert_eq!(report1.applied_count, total_migrations);

    let report2 = app_runner.run(&mut conn).expect("second run");
    assert_eq!(report2.applied_count, 0);
    assert_eq!(report2.skipped_count, total_migrations);
    assert!(report2.applied_versions.is_empty());
}

#[test]
fn at_004_transaction_rollback_on_failure() {
    let mut conn = Connection::open_in_memory().unwrap();
    let mut runner = MigrationRunner::new();
    runner
        .register(Migration::new(1, "good", "CREATE TABLE base (id INT);"))
        .unwrap();
    runner.run(&mut conn).unwrap();

    let mut runner_bad = MigrationRunner::new();
    runner_bad
        .register(Migration::new(1, "good", "CREATE TABLE base (id INT);"))
        .unwrap();
    runner_bad
        .register(Migration::new(
            2,
            "bad",
            "CREATE TABLE bad (id INT); INVALID SQL SYNTAX;",
        ))
        .unwrap();

    let err = runner_bad.run(&mut conn);
    assert!(err.is_err());

    let bad_exists: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='bad'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!bad_exists);

    let v2_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM _migrations WHERE version = 2",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(v2_count, 0);
}

#[test]
fn test_checksum_crlf_invariance() {
    let unix_sql = "CREATE TABLE test (\n  id INTEGER PRIMARY KEY\n);\n";
    let win_sql = "CREATE TABLE test (\r\n  id INTEGER PRIMARY KEY\r\n);\r\n";
    assert_eq!(compute_checksum(unix_sql), compute_checksum(win_sql));
}

#[test]
fn test_sequential_version_validation() {
    let mut runner = MigrationRunner::new();
    assert!(runner.register(Migration::new(2, "two", "")).is_err());
    assert!(runner.register(Migration::new(1, "one", "")).is_ok());
    assert!(runner.register(Migration::new(1, "dup", "")).is_err());
    assert!(runner.register(Migration::new(3, "three", "")).is_err());
    assert!(runner.register(Migration::new(2, "two", "")).is_ok());
}

#[test]
fn test_checksum_mismatch_detected() {
    let mut conn = Connection::open_in_memory().unwrap();
    let mut runner = MigrationRunner::new();
    runner
        .register(Migration::new(1, "v1", "CREATE TABLE t (id INT);"))
        .unwrap();
    runner.run(&mut conn).unwrap();

    let mut runner_tampered = MigrationRunner::new();
    runner_tampered
        .register(Migration::new(
            1,
            "v1",
            "CREATE TABLE t (id INT, tampered INT);",
        ))
        .unwrap();

    match runner_tampered.run(&mut conn) {
        Err(MigrationError::ChecksumMismatch { version, .. }) => {
            assert_eq!(version, 1);
        }
        other => panic!("Expected ChecksumMismatch, got: {other:?}"),
    }
}

#[test]
fn test_existing_database_baseline_adoption() {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    conn.execute_batch(
        "INSERT INTO personas (id, display_name, is_active, status, created_at, updated_at)
        VALUES ('p-exist', 'Existing Person', 1, 'active', '2026-11-20T12:00:00Z', '2026-11-20T12:00:00Z');",
    )
    .unwrap();

    let runner = MigrationRunner::with_app_migrations();
    let total_migrations = runner.migrations().len();
    let report = runner.run(&mut conn).expect("baseline adoption");
    assert_eq!(report.applied_count, total_migrations - 1);
    assert_eq!(report.skipped_count, 1);

    let p_name: String = conn
        .query_row(
            "SELECT display_name FROM personas WHERE id='p-exist'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(p_name, "Existing Person");
}

#[test]
fn test_unknown_future_version_error() {
    let mut conn = Connection::open_in_memory().unwrap();
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
    match runner.run(&mut conn) {
        Err(MigrationError::UnknownFutureVersion {
            recorded_version,
            max_known_version,
        }) => {
            assert_eq!(recorded_version, 99);
            assert_eq!(max_known_version, runner.max_version());
        }
        other => panic!("Expected UnknownFutureVersion error, got: {other:?}"),
    }
}
