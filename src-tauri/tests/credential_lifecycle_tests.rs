use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::credential::{
    CredentialStatus, SaveCredentialInput, UpdateCredentialInput,
};
use survey_copilot_lib::infrastructure::credentials::{CredentialStore, InMemoryCredentialStore};
use survey_copilot_lib::repository::CredentialRepository;
use survey_copilot_lib::services::{CredentialService, ExportService};

fn setup_test_db() -> Database {
    let mut conn = Connection::open_in_memory().expect("in-memory db open");
    let runner = survey_copilot_lib::migrations::MigrationRunner::with_app_migrations();
    runner.run(&mut conn).expect("run migrations");
    Database {
        connection: Arc::new(Mutex::new(conn)),
    }
}

#[test]
fn test_at_030_save_credential_stores_secret_in_manager_and_opaque_ref_in_db() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();

    let secret_val = "AIzaSyTestApiKeySecret1234567890";
    let input = SaveCredentialInput {
        id: None,
        provider_type: "gemini".to_string(),
        label: "Google Gemini Utama".to_string(),
        secret: secret_val.to_string(),
    };

    let summary = CredentialService::save_credential(&db, &store, input)
        .expect("save credential should succeed");

    assert!(!summary.id.is_empty());
    assert_eq!(summary.provider_type, "gemini");
    assert_eq!(summary.label, "Google Gemini Utama");
    assert_eq!(summary.status, CredentialStatus::Configured);
    assert_eq!(
        summary.credential_ref,
        format!("survey_copilot/cred/{}", summary.id)
    );
    assert_eq!(summary.usage_count, 0);

    // 1. Verifikasi secret tersimpan di CredentialStore (OS storage)
    let stored_secret = store
        .get_secret(&summary.credential_ref)
        .expect("secret should exist in store");
    assert_eq!(stored_secret, secret_val);

    // 2. Verifikasi SQLite HANYA memuat referensi buram dan metadata, BUKAN kunci API asli
    let conn = db.connection.lock().unwrap();
    let cred_in_db = CredentialRepository::find_by_id(&conn, &summary.id)
        .expect("find by id")
        .expect("credential should exist in db");

    assert_eq!(cred_in_db.id, summary.id);
    assert_eq!(cred_in_db.credential_ref, summary.credential_ref);

    // Pastikan seluruh kolom di baris SQLite tidak ada yang memuat secret
    let row_dump: String = conn
        .query_row(
            "SELECT id || '|' || provider_type || '|' || label || '|' || credential_ref || '|' || status FROM app_credentials WHERE id = ?1",
            rusqlite::params![summary.id],
            |r| r.get(0),
        )
        .unwrap();

    assert!(
        !row_dump.contains(secret_val),
        "Basis data SQLite TIDAK BOLEH memuat nilai kunci API asli!"
    );
}

#[test]
fn test_at_031_delete_credential_removes_from_store_and_db() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();

    let input = SaveCredentialInput {
        id: None,
        provider_type: "gemini".to_string(),
        label: "Key to Delete".to_string(),
        secret: "AIzaSySecretToDelete987654321".to_string(),
    };

    let summary = CredentialService::save_credential(&db, &store, input).unwrap();
    assert!(store.exists(&summary.credential_ref).unwrap());

    // Hapus kredensial
    let del_result = CredentialService::delete_credential(&db, &store, &summary.id, false)
        .expect("delete should succeed");

    assert_eq!(del_result.id, summary.id);
    assert_eq!(del_result.credential_ref, summary.credential_ref);
    assert!(del_result.deleted_from_store);
    assert!(del_result.deleted_from_database);

    // Verifikasi secret telah lenyap dari store
    assert!(!store.exists(&summary.credential_ref).unwrap());

    // Verifikasi metadata telah lenyap dari basis data
    let conn = db.connection.lock().unwrap();
    let found = CredentialRepository::find_by_id(&conn, &summary.id).unwrap();
    assert!(found.is_none());
}

#[test]
fn test_read_secret_internal_returns_original_value_and_validates_format() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();

    let secret_val = "MySuperSecretInternalKey99999";
    let input = SaveCredentialInput {
        id: None,
        provider_type: "openai_compatible".to_string(),
        label: "OpenAI Proxy Key".to_string(),
        secret: secret_val.to_string(),
    };

    let summary = CredentialService::save_credential(&db, &store, input).unwrap();

    // Pemanggilan internal backend dengan referensi valid
    let read_secret = CredentialService::read_secret_internal(&store, &summary.credential_ref)
        .expect("internal read should succeed");
    assert_eq!(read_secret, secret_val);

    // Format referensi ilegal harus ditolak
    let err = CredentialService::read_secret_internal(&store, "malicious/unscoped/key");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().code, "VALIDATION_ERROR");
}

#[test]
fn test_update_credential_preserves_old_on_failure() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();

    let original_secret = "OriginalSecretKey12345";
    let input = SaveCredentialInput {
        id: None,
        provider_type: "gemini".to_string(),
        label: "Original Label".to_string(),
        secret: original_secret.to_string(),
    };

    let summary = CredentialService::save_credential(&db, &store, input).unwrap();

    // 1. Pembaruan gagal karena secret baru terlalu pendek (< 8 karakter)
    let invalid_update = UpdateCredentialInput {
        id: summary.id.clone(),
        label: Some("New Label".to_string()),
        secret: Some("short".to_string()),
    };
    let update_err = CredentialService::update_credential(&db, &store, invalid_update);
    assert!(update_err.is_err());

    // Verifikasi secret lama dan label lama tetap utuh tanpa rusak
    let current_secret = store.get_secret(&summary.credential_ref).unwrap();
    assert_eq!(current_secret, original_secret);

    let status = CredentialService::get_credential_status(&db, &store, &summary.id).unwrap();
    assert_eq!(status.label, "Original Label");

    // 2. Pembaruan valid
    let valid_update = UpdateCredentialInput {
        id: summary.id.clone(),
        label: Some("Updated Label".to_string()),
        secret: Some("NewValidSecretKey9876543".to_string()),
    };
    let updated_summary = CredentialService::update_credential(&db, &store, valid_update).unwrap();
    assert_eq!(updated_summary.label, "Updated Label");

    let updated_secret = store.get_secret(&summary.credential_ref).unwrap();
    assert_eq!(updated_secret, "NewValidSecretKey9876543");
}

#[test]
fn test_delete_shared_credential_requires_force_when_in_use() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();

    let input = SaveCredentialInput {
        id: None,
        provider_type: "gemini".to_string(),
        label: "Shared Gemini Key".to_string(),
        secret: "SharedSecretKey987654321".to_string(),
    };

    let summary = CredentialService::save_credential(&db, &store, input).unwrap();

    // Simulasikan konfigurasi provider yang mereferensikan credential ini
    {
        let conn = db.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO provider_configs
                (id, display_name, provider_type, base_url, adapter_type, model_id, credential_ref, enabled, priority, weight, timeout_ms, max_concurrency, created_at, updated_at)
             VALUES
                ('prov-1', 'Gemini Flash Config', 'gemini', NULL, 'gemini', 'gemini-1.5-flash', ?1, 1, 100, 1, 30000, 1, '2026-11-23T00:00:00Z', '2026-11-23T00:00:00Z')",
            rusqlite::params![summary.credential_ref],
        )
        .unwrap();
    }

    // Periksa status: usage_count harus bernilai 1
    let status = CredentialService::get_credential_status(&db, &store, &summary.id).unwrap();
    assert_eq!(status.usage_count, 1);

    // Mencoba menghapus tanpa konfirmasi (force=false) harus DITOLAK dengan CONFLICT_ERROR
    let del_err = CredentialService::delete_credential(&db, &store, &summary.id, false);
    assert!(del_err.is_err());
    let err = del_err.unwrap_err();
    assert_eq!(err.code, "CONFLICT_ERROR");
    assert!(err
        .message
        .contains("masih digunakan oleh 1 konfigurasi provider"));

    // Kredensial dan secret tetap ada
    assert!(store.exists(&summary.credential_ref).unwrap());

    // Sekarang coba hapus dengan force=true
    let force_del = CredentialService::delete_credential(&db, &store, &summary.id, true)
        .expect("forced delete should succeed");
    assert!(force_del.deleted_from_store);
    assert!(force_del.deleted_from_database);
    assert!(!store.exists(&summary.credential_ref).unwrap());
}

#[test]
fn test_test_credential_differentiates_status() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();

    let input = SaveCredentialInput {
        id: None,
        provider_type: "gemini".to_string(),
        label: "Probe Key".to_string(),
        secret: "ProbeSecretKey123456789".to_string(),
    };

    let summary = CredentialService::save_credential(&db, &store, input).unwrap();

    // 1. Uji saat secret tersedia di store
    let test_ok = CredentialService::test_credential(&db, &store, &summary.id).unwrap();
    assert!(test_ok.is_valid);
    assert_eq!(test_ok.status, CredentialStatus::Configured);

    // 2. Hapus secret dari store secara manual (simulasikan secret hilang di OS)
    store.delete_secret(&summary.credential_ref).unwrap();

    // 3. Uji kembali: status harus Unconfigured
    let test_missing = CredentialService::test_credential(&db, &store, &summary.id).unwrap();
    assert!(!test_missing.is_valid);
    assert_eq!(test_missing.status, CredentialStatus::Unconfigured);
}

#[test]
fn test_ipc_summary_and_export_contain_zero_secrets() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();

    let distinct_secret = "AIzaSySuperSecretNeverExportThisToken_XYZ_999";
    let input = SaveCredentialInput {
        id: None,
        provider_type: "gemini".to_string(),
        label: "Top Secret Key".to_string(),
        secret: distinct_secret.to_string(),
    };

    let summary = CredentialService::save_credential(&db, &store, input).unwrap();

    // Serialisasi JSON dari CredentialSummary
    let json_summary = serde_json::to_string(&summary).unwrap();
    assert!(
        !json_summary.contains(distinct_secret),
        "JSON IPC Summary tidak boleh memuat secret!"
    );

    // Ekspor seluruh data via ExportService (SC-012)
    let export_json = ExportService::export_data_as_json_string(&db, None).unwrap();
    assert!(
        !export_json.contains(distinct_secret),
        "Ekspor SC-012 tidak boleh memuat API key!"
    );
    assert!(
        !export_json.contains(&summary.credential_ref),
        "Ekspor SC-012 tidak boleh memuat credential reference!"
    );
}
