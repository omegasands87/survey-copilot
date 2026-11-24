use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::capture::{
    CaptureMode, CaptureRect, CaptureRegionInput, CaptureWindowInput,
};
use survey_copilot_lib::infrastructure::capture::InMemoryScreenCaptureEngine;
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::repository::PersonaRepository;
use survey_copilot_lib::services::{CaptureService, DeletionService, SurveyService};

fn setup_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("in-memory db must open");
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    let runner = MigrationRunner::with_app_migrations();
    runner.run(&mut conn).expect("migrations must succeed");
    conn
}

fn create_test_persona(conn: &Connection, name: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    PersonaRepository::create(conn, &id, name, Some("Profil responden uji"), true, &now)
        .expect("create persona must succeed");
    id
}

fn get_unique_temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "survey_copilot_test_captures_{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_at_050_explicit_capture_creates_preview_and_temp_file() {
    let conn = setup_test_db();
    let db = Arc::new(Database {
        connection: Mutex::new(conn),
    });
    let temp_dir = get_unique_temp_dir();
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let service = CaptureService::new(engine, temp_dir.clone(), db).unwrap();

    // Explicit user capture of full screen
    let input = CaptureRegionInput {
        session_id: None,
        rect: None,
    };
    let preview = service.capture_region(input).expect("capture must succeed");

    assert_eq!(preview.source_mode, CaptureMode::Region);
    assert!(!preview.capture_id.is_empty());
    assert!(preview.width > 0);
    assert!(preview.height > 0);
    assert!(preview.file_size_bytes > 0);

    // Verify preview_data_url is a valid base64 PNG data URL
    let data_url = preview
        .preview_data_url
        .expect("data url must be generated");
    assert!(data_url.starts_with("data:image/png;base64,"));

    // Verify physical file was saved strictly inside app-managed temp directory
    let file_path = PathBuf::from(&preview.file_path);
    assert!(
        file_path.exists(),
        "temporary image file must exist on disk"
    );
    assert_eq!(file_path.parent().unwrap(), temp_dir.as_path());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_at_051_idle_app_performs_no_captures() {
    let conn = setup_test_db();
    let db = Arc::new(Database {
        connection: Mutex::new(conn),
    });
    let temp_dir = get_unique_temp_dir();
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let _service = CaptureService::new(engine, temp_dir.clone(), db).unwrap();

    // Without explicit capture command invocation, directory must have 0 capture files
    let entries = fs::read_dir(&temp_dir).unwrap().count();
    assert_eq!(
        entries, 0,
        "no screen captures may occur in idle state (AT-051)"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_at_054_discard_capture_removes_temporary_image_file() {
    let conn = setup_test_db();
    let db = Arc::new(Database {
        connection: Mutex::new(conn),
    });
    let temp_dir = get_unique_temp_dir();
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let service = CaptureService::new(engine, temp_dir.clone(), db).unwrap();

    let preview = service
        .capture_region(CaptureRegionInput {
            session_id: None,
            rect: Some(CaptureRect {
                x: 10,
                y: 20,
                width: 400,
                height: 300,
            }),
        })
        .unwrap();

    let file_path = PathBuf::from(&preview.file_path);
    assert!(file_path.exists());

    // Discard capture
    let discard_result = service.discard_capture(&preview.capture_id).unwrap();
    assert!(discard_result.deleted);
    assert!(
        !file_path.exists(),
        "temporary image file must be removed from disk upon discard (AT-054)"
    );

    // Discarding again returns deleted = false without crashing
    let second_discard = service.discard_capture(&preview.capture_id).unwrap();
    assert!(!second_discard.deleted);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_window_capture_and_window_enumeration() {
    let conn = setup_test_db();
    let db = Arc::new(Database {
        connection: Mutex::new(conn),
    });
    let temp_dir = get_unique_temp_dir();
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let service = CaptureService::new(engine, temp_dir.clone(), db).unwrap();

    // List windows
    let windows = service.list_windows().unwrap();
    assert!(!windows.is_empty());
    let chrome_win = windows
        .iter()
        .find(|w| w.id == 1001)
        .expect("mock Chrome window must exist");
    assert_eq!(chrome_win.title, "Google Chrome - YouGov Survey 2026");

    // Capture window
    let preview = service
        .capture_window(CaptureWindowInput {
            session_id: None,
            window_id: 1001,
        })
        .unwrap();

    assert_eq!(preview.source_mode, CaptureMode::Window);
    assert!(preview.target_label.unwrap().contains("Google Chrome"));
    assert!(PathBuf::from(&preview.file_path).exists());

    // Capture invalid window
    let err = service
        .capture_window(CaptureWindowInput {
            session_id: None,
            window_id: 999999,
        })
        .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_capture_error_handling_invalid_bounds() {
    let conn = setup_test_db();
    let db = Arc::new(Database {
        connection: Mutex::new(conn),
    });
    let temp_dir = get_unique_temp_dir();
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let service = CaptureService::new(engine, temp_dir.clone(), db).unwrap();

    // 0 width or height must fail validation
    let err = service
        .capture_region(CaptureRegionInput {
            session_id: None,
            rect: Some(CaptureRect {
                x: 0,
                y: 0,
                width: 0,
                height: 500,
            }),
        })
        .unwrap_err();

    assert_eq!(err.code, "VALIDATION_ERROR");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_security_path_traversal_prevention() {
    let conn = setup_test_db();
    let db = Arc::new(Database {
        connection: Mutex::new(conn),
    });
    let temp_dir = get_unique_temp_dir();
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let service = CaptureService::new(engine, temp_dir.clone(), db).unwrap();

    // Attempt path traversal via invalid capture_id
    let malicious_ids = vec![
        "../outside",
        "..\\windows\\system32",
        "../../etc/passwd",
        "invalid-non-uuid",
    ];

    for bad_id in malicious_ids {
        let err = service.discard_capture(bad_id).unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_temporary_file_retention_and_cleanup() {
    let conn = setup_test_db();
    let db = Arc::new(Database {
        connection: Mutex::new(conn),
    });
    let temp_dir = get_unique_temp_dir();
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let service = CaptureService::new(engine, temp_dir.clone(), db).unwrap();

    // Capture 3 files
    let _p1 = service
        .capture_region(CaptureRegionInput {
            session_id: None,
            rect: None,
        })
        .unwrap();
    let _p2 = service
        .capture_region(CaptureRegionInput {
            session_id: None,
            rect: None,
        })
        .unwrap();
    let _p3 = service
        .capture_region(CaptureRegionInput {
            session_id: None,
            rect: None,
        })
        .unwrap();

    let count_before = fs::read_dir(&temp_dir).unwrap().count();
    assert_eq!(count_before, 3);

    // Clean up with 0 seconds max age (everything expired)
    let cleanup = service.cleanup_expired_captures(0).unwrap();
    assert_eq!(cleanup.deleted_count, 3);
    assert!(cleanup.reclaimed_bytes > 0);

    let count_after = fs::read_dir(&temp_dir).unwrap().count();
    assert_eq!(count_after, 0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_capture_linked_to_survey_session() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Budi Responden");

    // Create session
    let session = SurveyService::create_session(
        &conn,
        survey_copilot_lib::domain::survey::CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Perilaku Konsumen".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let temp_dir = get_unique_temp_dir();
    let db = Arc::new(Database {
        connection: Mutex::new(conn),
    });
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let service = CaptureService::new(engine, temp_dir.clone(), db.clone()).unwrap();

    let preview = service
        .capture_region(CaptureRegionInput {
            session_id: Some(session.id.clone()),
            rect: None,
        })
        .unwrap();

    // Verify row was inserted into screen_captures
    {
        let conn = db.connection.lock().unwrap();
        let (db_session_id, db_mode, db_path, db_deleted): (String, String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT session_id, capture_mode, temporary_image_path, image_deleted_at FROM screen_captures WHERE id = ?",
                rusqlite::params![preview.capture_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert_eq!(db_session_id, session.id);
        assert_eq!(db_mode, "region");
        assert!(db_path.is_some());
        assert!(db_deleted.is_none());
    }

    // Now discard
    service.discard_capture(&preview.capture_id).unwrap();

    // Verify DB updated image_deleted_at
    {
        let conn = db.connection.lock().unwrap();
        let (db_path, db_deleted): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT temporary_image_path, image_deleted_at FROM screen_captures WHERE id = ?",
                rusqlite::params![preview.capture_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert!(
            db_path.is_none(),
            "temporary_image_path must be cleared in DB upon deletion"
        );
        assert!(
            db_deleted.is_some(),
            "image_deleted_at must be populated in DB"
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_deletion_service_wipes_temp_captures() {
    let conn = setup_test_db();
    let _persona_id = create_test_persona(&conn, "Siti Konsumen");
    let app_data_dir = get_unique_temp_dir();
    let temp_captures = app_data_dir.join("temp_captures");
    fs::create_dir_all(&temp_captures).unwrap();

    let db = Database {
        connection: Mutex::new(conn),
    };
    let engine = Arc::new(InMemoryScreenCaptureEngine::new());
    let service = CaptureService::new(
        engine,
        temp_captures.clone(),
        Arc::new(Database {
            connection: Mutex::new(setup_test_db()),
        }),
    )
    .unwrap();

    let _p = service
        .capture_region(CaptureRegionInput {
            session_id: None,
            rect: None,
        })
        .unwrap();

    assert_eq!(fs::read_dir(&temp_captures).unwrap().count(), 1);

    // Full wipe
    let report =
        DeletionService::wipe_all_data(&db, Some(&app_data_dir), "HAPUS SEMUA DATA").unwrap();
    assert!(report.deleted_temp_files_count >= 1);
    assert_eq!(fs::read_dir(&temp_captures).unwrap().count(), 0);

    let _ = fs::remove_dir_all(&app_data_dir);
}
