use chrono::Utc;
use rusqlite::Connection;
use serde_json::json;
use std::sync::{Arc, Mutex};
use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::pool::{
    CancellationToken, FakeTimeSource, ProviderHealthState, UpdatePoolSettingsInput,
};
use survey_copilot_lib::domain::provider::{
    CreateProviderConfigInput, NormalizedErrorCategory, NormalizedRequest,
};
use survey_copilot_lib::infrastructure::credentials::in_memory::InMemoryCredentialStore;
use survey_copilot_lib::infrastructure::http::{HttpResponse, MockHttpTransport};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::services::{CredentialService, ProviderPoolService, ProviderService};

fn setup_test_db() -> Database {
    let mut conn = Connection::open_in_memory().expect("in-memory db open");
    let runner = MigrationRunner::with_app_migrations();
    runner.run(&mut conn).expect("run migrations");
    Database {
        connection: Mutex::new(conn),
    }
}

fn create_sample_request() -> NormalizedRequest {
    NormalizedRequest {
        request_id: "req_test_pool_001".to_string(),
        operation: "survey_question_analysis".to_string(),
        model_id: "gemini-2.5-flash".to_string(),
        system_instruction: Some("Anda adalah asisten survei terpercaya.".to_string()),
        user_payload: "Pertanyaan: Berapa usia Anda saat ini?".to_string(),
        response_schema: Some(json!({
            "type": "object",
            "properties": {
                "answer": { "type": "string" },
                "confidence": { "type": "number" }
            },
            "required": ["answer"]
        })),
        temperature: Some(0.1),
        max_tokens: Some(512),
        timeout_ms: Some(15000),
    }
}

fn create_mock_gemini_success_body(answer: &str) -> String {
    json!({
        "candidates": [
            {
                "content": {
                    "parts": [
                        { "text": format!("{{\"answer\":\"{answer}\",\"confidence\":0.95}}") }
                    ],
                    "role": "model"
                },
                "finishReason": "STOP",
                "index": 0
            }
        ],
        "usageMetadata": {
            "promptTokenCount": 80,
            "candidatesTokenCount": 20,
            "totalTokenCount": 100
        }
    })
    .to_string()
}

#[test]
fn test_at_033_pool_failover_on_429_rate_limited_and_respect_retry_after() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();
    let transport = Arc::new(MockHttpTransport::new());

    let initial_time = Utc::now();
    let fake_clock = Arc::new(FakeTimeSource::new(initial_time));
    let pool_service = ProviderPoolService::with_time_source(fake_clock.clone());

    // 1. Simpan dua kredensial
    let cred1 = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key 1".to_string(),
            secret: "AIzaSyKeyOneSecret123456789".to_string(),
        },
    )
    .unwrap();

    let cred2 = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key 2".to_string(),
            secret: "AIzaSyKeyTwoSecret987654321".to_string(),
        },
    )
    .unwrap();

    // 2. Buat dua konfigurasi provider: P1 (prioritas 10) dan P2 (prioritas 20)
    let _p1 = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("provider_key_1".to_string()),
            display_name: "Provider Utama".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred1.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(10),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(1),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    let _p2 = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("provider_key_2".to_string()),
            display_name: "Provider Cadangan".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred2.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(20),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(1),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    // Atur strategi pool menjadi Priority
    let _ = pool_service
        .update_settings(
            &db,
            UpdatePoolSettingsInput {
                strategy: Some("priority".to_string()),
                max_attempts: Some(3),
                auto_retry: Some(true),
                retry_base_delay_ms: Some(1000),
                max_retry_delay_ms: Some(15000),
            },
        )
        .unwrap();

    // 3. Mock respons: P1 mengembalikan 429 Rate Limited dengan Retry-After: 5 detik
    let rate_limit_body = json!({
        "error": {
            "code": 429,
            "message": "Resource has been exhausted (e.g. check quota).",
            "status": "RESOURCE_EXHAUSTED",
            "details": [
                {
                    "@type": "type.googleapis.com/google.rpc.ErrorInfo",
                    "reason": "RATE_LIMIT_EXCEEDED"
                }
            ]
        }
    });

    let resp_429 =
        HttpResponse::new(429, rate_limit_body.to_string()).with_header("retry-after", "5");
    transport.enqueue_response(resp_429);

    // P2 mengembalikan sukses 200
    let resp_200 = HttpResponse::new(200, create_mock_gemini_success_body("28 tahun"));
    transport.enqueue_response(resp_200);

    // 4. Eksekusi request melalui pool
    let request = create_sample_request();
    let result =
        pool_service.execute_pooled_request(&db, &store, transport.clone(), request.clone(), None);

    // Verifikasi failover berhasil ke P2
    assert!(
        result.success,
        "Eksekusi pool harus berhasil via failover ke P2"
    );
    assert!(result.response.is_some());
    assert_eq!(
        result.attempts.len(),
        2,
        "Harus ada 2 percobaan (P1 gagal 429, P2 sukses)"
    );
    assert_eq!(result.attempts[0].provider_config_id, "provider_key_1");
    assert!(!result.attempts[0].success);
    assert_eq!(
        result.attempts[0].error_category.as_deref(),
        Some("rate_limited")
    );

    assert_eq!(result.attempts[1].provider_config_id, "provider_key_2");
    assert!(result.attempts[1].success);

    // Verifikasi status health P1 berada dalam cooling_down
    let health_list = pool_service.list_health(&db).unwrap();
    let h1 = health_list
        .iter()
        .find(|h| h.provider_config_id == "provider_key_1")
        .unwrap();
    assert_eq!(h1.state, ProviderHealthState::CoolingDown);
    assert!(h1.cooldown_until.is_some());

    // 5. Coba request kedua selagi P1 masih dalam cooldown -> P1 harus dilewati, langsung ke P2
    transport.enqueue_response(HttpResponse::new(
        200,
        create_mock_gemini_success_body("28 tahun"),
    ));

    let result2 =
        pool_service.execute_pooled_request(&db, &store, transport.clone(), request.clone(), None);
    assert!(result2.success);
    assert_eq!(
        result2.attempts.len(),
        1,
        "Hanya P2 yang dicoba karena P1 masih cooldown"
    );
    assert_eq!(result2.attempts[0].provider_config_id, "provider_key_2");

    // 6. Majukan waktu fake clock melebihi cooldown (6 detik)
    fake_clock.advance_seconds(6);

    // Reset cooldown kadaluarsa dan periksa status kembali Ready
    let health_list_after = pool_service.list_health(&db).unwrap();
    let h1_after = health_list_after
        .iter()
        .find(|h| h.provider_config_id == "provider_key_1")
        .unwrap();
    assert_eq!(h1_after.state, ProviderHealthState::Ready);
}

#[test]
fn test_at_034_account_project_quota_exhausted_stops_pool_retries_and_preserves_request() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();
    let transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    // Buat 2 provider
    let cred1 = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key 1".to_string(),
            secret: "AIzaSyKeyOneSecret123456789".to_string(),
        },
    )
    .unwrap();

    let cred2 = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key 2".to_string(),
            secret: "AIzaSyKeyTwoSecret987654321".to_string(),
        },
    )
    .unwrap();

    let _ = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("p_quota_1".to_string()),
            display_name: "Provider Quota 1".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred1.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(10),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(1),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    let _ = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("p_quota_2".to_string()),
            display_name: "Provider Quota 2".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred2.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(20),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(1),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    // Mock respons Quota Exhausted tingkat akun/project Google Cloud (HTTP 429 atau 403 dengan quota exceeded)
    let quota_exhausted_body = json!({
        "error": {
            "code": 429,
            "message": "Quota exceeded for quota metric 'GenerateContent requests' and limit 'GenerateContent requests per project per day'. Please check your plan and billing details.",
            "status": "RESOURCE_EXHAUSTED",
            "details": [
                {
                    "@type": "type.googleapis.com/google.rpc.ErrorInfo",
                    "reason": "QUOTA_EXCEEDED"
                }
            ]
        }
    });

    transport.enqueue_response(HttpResponse::new(429, quota_exhausted_body.to_string()));

    let request = create_sample_request();
    let result =
        pool_service.execute_pooled_request(&db, &store, transport.clone(), request.clone(), None);

    // AT-034: Pool HARUS BERHENTI, tidak boleh melakukan rotasi kunci ke provider lain
    assert!(
        !result.success,
        "Eksekusi harus gagal karena kuota project habis"
    );
    assert_eq!(
        result.attempts.len(),
        1,
        "DILARANG merotasi ke provider lain saat kuota project/akun habis (AT-034)"
    );
    assert_eq!(result.attempts[0].provider_config_id, "p_quota_1");
    assert_eq!(
        result.attempts[0].error_category.as_deref(),
        Some("quota_exhausted")
    );

    // Verifikasi bahwa input request pengguna tetap terjaga (preserved)
    assert_eq!(result.preserved_request.request_id, request.request_id);
    assert_eq!(result.preserved_request.user_payload, request.user_payload);
}

#[test]
fn test_at_035_invalid_request_and_content_refusal_do_not_rotate_through_keys() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();
    let transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    let cred1 = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key 1".to_string(),
            secret: "AIzaSyKeyOneSecret123456789".to_string(),
        },
    )
    .unwrap();

    let cred2 = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key 2".to_string(),
            secret: "AIzaSyKeyTwoSecret987654321".to_string(),
        },
    )
    .unwrap();

    let _ = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("p_invalid_1".to_string()),
            display_name: "Provider 1".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred1.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(10),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(1),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    let _ = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("p_invalid_2".to_string()),
            display_name: "Provider 2".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred2.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(20),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(1),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    // Mock HTTP 400 Bad Request / Schema Invalid
    let bad_request_body = json!({
        "error": {
            "code": 400,
            "message": "Invalid response schema format or unsupported parameter.",
            "status": "INVALID_ARGUMENT"
        }
    });

    transport.enqueue_response(HttpResponse::new(400, bad_request_body.to_string()));

    let request = create_sample_request();
    let result =
        pool_service.execute_pooled_request(&db, &store, transport.clone(), request.clone(), None);

    // AT-035: Kesalahan format/permintaan tidak boleh di-retry ke provider lain
    assert!(!result.success);
    assert_eq!(
        result.attempts.len(),
        1,
        "Dilarang merotasi seluruh key untuk invalid request (AT-035)"
    );
    assert_eq!(
        result.attempts[0].error_category.as_deref(),
        Some("invalid_request")
    );
    assert_eq!(result.preserved_request.user_payload, request.user_payload);
}

#[test]
fn test_at_036_user_cancellation_stops_queued_retries() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();
    let transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    let cred = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key".to_string(),
            secret: "AIzaSyKeyOneSecret123456789".to_string(),
        },
    )
    .unwrap();

    let _ = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("p_cancel_1".to_string()),
            display_name: "Provider 1".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(10),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(1),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    // Buat cancellation token dan langsung batalkan
    let token = CancellationToken::new();
    token.cancel();

    let request = create_sample_request();
    let result = pool_service.execute_pooled_request(
        &db,
        &store,
        transport.clone(),
        request.clone(),
        Some(token),
    );

    // AT-036: Tidak ada request yang dikirim ke provider
    assert!(!result.success);
    assert_eq!(
        result.attempts.len(),
        0,
        "Tidak ada request yang boleh terkirim setelah pembatalan"
    );
    assert_eq!(
        result.final_error.unwrap().category,
        NormalizedErrorCategory::Cancelled
    );
    assert_eq!(result.preserved_request.user_payload, request.user_payload);
}

#[test]
fn test_selection_strategies_round_robin_weighted_priority_adaptive() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();
    let transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    let cred1 = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key 1".to_string(),
            secret: "AIzaSyKeyOneSecret123456789".to_string(),
        },
    )
    .unwrap();

    let cred2 = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key 2".to_string(),
            secret: "AIzaSyKeyTwoSecret987654321".to_string(),
        },
    )
    .unwrap();

    let _ = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("strat_p1".to_string()),
            display_name: "P1".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred1.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(10),
            weight: Some(3),
            timeout_ms: Some(30000),
            max_concurrency: Some(5),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    let _ = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("strat_p2".to_string()),
            display_name: "P2".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred2.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(20),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(5),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    // 1. Uji Round-Robin: bergantian antara P1 dan P2
    let _ = pool_service
        .update_settings(
            &db,
            UpdatePoolSettingsInput {
                strategy: Some("round_robin".to_string()),
                max_attempts: Some(1),
                auto_retry: Some(false),
                retry_base_delay_ms: None,
                max_retry_delay_ms: None,
            },
        )
        .unwrap();

    transport.enqueue_response(HttpResponse::new(200, create_mock_gemini_success_body("A")));
    transport.enqueue_response(HttpResponse::new(200, create_mock_gemini_success_body("B")));

    let r1 = pool_service.execute_pooled_request(
        &db,
        &store,
        transport.clone(),
        create_sample_request(),
        None,
    );
    let r2 = pool_service.execute_pooled_request(
        &db,
        &store,
        transport.clone(),
        create_sample_request(),
        None,
    );

    assert!(r1.success && r2.success);
    // Bergantian memilih provider yang berbeda
    assert_ne!(
        r1.attempts[0].provider_config_id, r2.attempts[0].provider_config_id,
        "Round-robin harus merotasi antar provider"
    );

    // 2. Uji Priority: P1 (prioritas 10) selalu dipilih sebelum P2 (prioritas 20)
    let _ = pool_service
        .update_settings(
            &db,
            UpdatePoolSettingsInput {
                strategy: Some("priority".to_string()),
                max_attempts: Some(1),
                auto_retry: Some(false),
                retry_base_delay_ms: None,
                max_retry_delay_ms: None,
            },
        )
        .unwrap();

    transport.enqueue_response(HttpResponse::new(200, create_mock_gemini_success_body("C")));
    let r_prio = pool_service.execute_pooled_request(
        &db,
        &store,
        transport.clone(),
        create_sample_request(),
        None,
    );
    assert_eq!(r_prio.attempts[0].provider_config_id, "strat_p1");

    // 3. Uji Adaptive: P1 dipilih karena kebugaran lebih tinggi (prioritas lebih tinggi dan latensi rendah)
    let _ = pool_service
        .update_settings(
            &db,
            UpdatePoolSettingsInput {
                strategy: Some("adaptive".to_string()),
                max_attempts: Some(1),
                auto_retry: Some(false),
                retry_base_delay_ms: None,
                max_retry_delay_ms: None,
            },
        )
        .unwrap();

    transport.enqueue_response(HttpResponse::new(200, create_mock_gemini_success_body("D")));
    let r_adapt = pool_service.execute_pooled_request(
        &db,
        &store,
        transport.clone(),
        create_sample_request(),
        None,
    );
    assert_eq!(r_adapt.attempts[0].provider_config_id, "strat_p1");
}

#[test]
fn test_zero_credential_leakage_in_pool_results_and_history() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();
    let transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    let secret_raw = "SUPER_SECRET_API_KEY_NEVER_LEAK_999";
    let cred = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Gemini Key Leak Check".to_string(),
            secret: secret_raw.to_string(),
        },
    )
    .unwrap();

    let _ = ProviderService::create_provider_config(
        &db,
        CreateProviderConfigInput {
            id: Some("p_leak_check".to_string()),
            display_name: "Provider Leak Check".to_string(),
            provider_type: "gemini".to_string(),
            base_url: None,
            adapter_type: "gemini".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            credential_ref: cred.credential_ref.clone(),
            enabled: Some(true),
            priority: Some(10),
            weight: Some(1),
            timeout_ms: Some(30000),
            max_concurrency: Some(1),
            rate_limit_hint_json: None,
        },
    )
    .unwrap();

    transport.enqueue_response(HttpResponse::new(
        200,
        create_mock_gemini_success_body("Aman"),
    ));

    let result = pool_service.execute_pooled_request(
        &db,
        &store,
        transport.clone(),
        create_sample_request(),
        None,
    );

    let serialized = serde_json::to_string(&result).unwrap();
    assert!(
        !serialized.contains(secret_raw),
        "Hasil eksekusi pool TIDAK BOLEH memuat secret API key mentah!"
    );

    let health_list = pool_service.list_health(&db).unwrap();
    let health_serialized = serde_json::to_string(&health_list).unwrap();
    assert!(
        !health_serialized.contains(secret_raw),
        "Ringkasan status health TIDAK BOLEH memuat secret API key mentah!"
    );
}
