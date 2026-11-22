use rusqlite::Connection;
use serde_json::json;
use std::sync::Arc;
use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::endpoint_validator::EndpointValidator;
use survey_copilot_lib::domain::error::AppError;
use survey_copilot_lib::domain::provider::{
    CreateProviderConfigInput, NormalizedErrorCategory, NormalizedFinishReason, NormalizedRequest,
    UpdateProviderConfigInput,
};
use survey_copilot_lib::infrastructure::adapters::{get_adapter, GeminiAdapter};
use survey_copilot_lib::infrastructure::credentials::in_memory::InMemoryCredentialStore;
use survey_copilot_lib::infrastructure::http::{HttpResponse, MockHttpTransport};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::services::{CredentialService, ProviderService};

fn setup_test_db() -> Database {
    let mut conn = Connection::open_in_memory().expect("in-memory db must open");
    let mut runner = MigrationRunner::with_app_migrations();
    runner.run(&mut conn).expect("migrations must succeed");
    Database {
        connection: Arc::new(std::sync::Mutex::new(conn)),
    }
}

#[test]
fn test_at_032_gemini_structured_request_normalized_and_returned() {
    let mock_transport = Arc::new(MockHttpTransport::new());

    // Mock Gemini API response with structured JSON
    let gemini_json_response = json!({
        "candidates": [
            {
                "content": {
                    "parts": [
                        {
                            "text": "{\"status\":\"supported\",\"question_intent\":\"demographic\",\"answer_format\":\"single_choice\",\"interpretation\":\"Responden berprofesi sebagai Software Engineer.\",\"draft_answer\":\"Software Engineer\",\"supporting_fact_ids\":[\"fact_job_01\"],\"rationale\":\"Berdasarkan fakta profil pekerjaan responden.\",\"missing_information\":[],\"conflicts\":[],\"assumptions\":[]}"
                        }
                    ],
                    "role": "model"
                },
                "finishReason": "STOP",
                "index": 0
            }
        ],
        "usageMetadata": {
            "promptTokenCount": 115,
            "candidatesTokenCount": 58,
            "totalTokenCount": 173
        }
    });

    mock_transport.enqueue_response(HttpResponse::new(200, gemini_json_response.to_string()));

    let adapter = GeminiAdapter::new(mock_transport.clone());

    let request = NormalizedRequest {
        request_id: "req_test_001".to_string(),
        operation: "question_analysis".to_string(),
        model_id: "gemini-2.5-flash".to_string(),
        system_instruction: Some("Anda adalah asisten analisis survei berbasis bukti.".to_string()),
        user_payload: "Pertanyaan: Apa pekerjaan utama Anda?".to_string(),
        response_schema: Some(json!({
            "type": "object",
            "properties": {
                "status": { "type": "string" },
                "draft_answer": { "type": "string" },
                "supporting_fact_ids": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["status", "draft_answer", "supporting_fact_ids"]
        })),
        temperature: Some(0.1),
        max_tokens: Some(1024),
        timeout_ms: Some(15000),
    };

    let result = adapter.generate_structured(None, "dummy_gemini_api_key_12345", &request);
    assert!(result.is_ok(), "generate_structured must succeed");

    let response = result.unwrap();
    assert_eq!(response.request_id, "req_test_001");
    assert_eq!(response.provider_id, "gemini");
    assert_eq!(response.model_id, "gemini-2.5-flash");
    assert_eq!(response.finish_reason, NormalizedFinishReason::Stop);
    assert_eq!(response.input_tokens, Some(115));
    assert_eq!(response.output_tokens, Some(58));

    // Structured JSON must be correctly extracted and parsed
    assert!(response.structured_json.is_some());
    let structured = response.structured_json.unwrap();
    assert_eq!(structured["status"], "supported");
    assert_eq!(structured["draft_answer"], "Software Engineer");
    assert_eq!(structured["supporting_fact_ids"][0], "fact_job_01");

    // Strictly verify no secret was leaked in the response
    let serialized_response = serde_json::to_string(&response).unwrap();
    assert!(!serialized_response.contains("dummy_gemini_api_key"));

    // Verify request payload was sent with x-goog-api-key header
    let recorded = mock_transport.get_recorded_requests();
    assert_eq!(recorded.len(), 1);
    let (url, headers, body) = &recorded[0];
    assert!(url.contains("/v1beta/models/gemini-2.5-flash:generateContent"));
    assert!(headers
        .iter()
        .any(|(k, v)| k == "x-goog-api-key" && v == "dummy_gemini_api_key_12345"));
    assert!(body.is_some());
}

#[test]
fn test_at_037_http_endpoint_outside_localhost_rejected_default_https_only() {
    // 1. External HTTP must be rejected
    let ext_http = EndpointValidator::validate_url("http://api.external-ai.com/v1");
    assert!(ext_http.is_err());
    let err = ext_http.unwrap_err();
    assert!(err.message.contains("Endpoint HTTP hanya diperbolehkan"));

    // 2. Localhost HTTP variants must pass
    assert!(EndpointValidator::validate_url("http://localhost:11434").is_ok());
    assert!(EndpointValidator::validate_url("http://127.0.0.1:8000/v1").is_ok());
    assert!(EndpointValidator::validate_url("http://[::1]:8080").is_ok());

    // 3. HTTPS external must pass
    assert!(EndpointValidator::validate_url("https://generativelanguage.googleapis.com").is_ok());
    assert!(EndpointValidator::validate_url("https://api.openai.com/v1").is_ok());

    // 4. SSRF and illegal schemes must be rejected
    assert!(
        EndpointValidator::validate_url("file:///C:/Windows/System32/drivers/etc/hosts").is_err()
    );
    assert!(EndpointValidator::validate_url("https://169.254.169.254/latest/meta-data").is_err());
    assert!(EndpointValidator::validate_url("https://metadata.google.internal").is_err());
}

#[test]
fn test_at_038_redirect_to_another_host_never_forwards_credentials() {
    let original = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash";

    // Same origin redirect: allowed
    assert!(EndpointValidator::is_safe_redirect(
        original,
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
    ));

    // Cross origin redirect: strictly rejected
    assert!(!EndpointValidator::is_safe_redirect(
        original,
        "https://attacker-stealer.org/v1beta/models"
    ));
    assert!(!EndpointValidator::is_safe_redirect(
        original,
        "http://generativelanguage.googleapis.com/downgrade"
    ));
}

#[test]
fn test_gemini_error_normalization() {
    let mock_transport = Arc::new(MockHttpTransport::new());

    // 1. 429 Quota Exhausted
    mock_transport.enqueue_response(HttpResponse::new(
        429,
        json!({
            "error": {
                "code": 429,
                "message": "Resource has been exhausted (check your quota).",
                "status": "RESOURCE_EXHAUSTED"
            }
        })
        .to_string(),
    ));

    let adapter = GeminiAdapter::new(mock_transport.clone());
    let req = NormalizedRequest {
        request_id: "test".to_string(),
        operation: "test".to_string(),
        model_id: "gemini-2.5-flash".to_string(),
        system_instruction: None,
        user_payload: "test".to_string(),
        response_schema: None,
        temperature: None,
        max_tokens: None,
        timeout_ms: None,
    };

    let err1 = adapter.generate_structured(None, "key", &req).unwrap_err();
    assert_eq!(err1.category, NormalizedErrorCategory::QuotaExhausted);

    // 2. 401 Authentication Failure
    mock_transport.enqueue_response(HttpResponse::new(
        401,
        json!({
            "error": {
                "code": 401,
                "message": "API key not valid. Please pass a valid API key.",
                "status": "UNAUTHENTICATED"
            }
        })
        .to_string(),
    ));

    let err2 = adapter
        .generate_structured(None, "bad_key", &req)
        .unwrap_err();
    assert_eq!(err2.category, NormalizedErrorCategory::AuthenticationFailed);
    assert!(!err2.retryable);

    // 3. 404 Model Unavailable
    mock_transport.enqueue_response(HttpResponse::new(
        404,
        json!({
            "error": {
                "code": 404,
                "message": "models/unknown-model is not found.",
                "status": "NOT_FOUND"
            }
        })
        .to_string(),
    ));

    let err3 = adapter.generate_structured(None, "key", &req).unwrap_err();
    assert_eq!(err3.category, NormalizedErrorCategory::ModelUnavailable);
}

#[test]
fn test_provider_config_crud_and_credential_integration() {
    let db = setup_test_db();
    let store = InMemoryCredentialStore::new();

    // 1. Buat kredensial di Windows Credential Manager
    let cred_summary = CredentialService::save_credential(
        &db,
        &store,
        survey_copilot_lib::domain::credential::SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Kunci Gemini Produksi".to_string(),
            secret: "AIzaSySecretGeminiKey12345".to_string(),
        },
    )
    .expect("save credential must succeed");

    assert_eq!(cred_summary.usage_count, 0);

    // 2. Buat Provider Config menggunakan referensi buram dari kredensial
    let create_input = CreateProviderConfigInput {
        id: None,
        display_name: "Gemini 2.5 Flash Utama".to_string(),
        provider_type: "gemini".to_string(),
        base_url: None,
        adapter_type: "gemini".to_string(),
        model_id: "gemini-2.5-flash".to_string(),
        credential_ref: cred_summary.credential_ref.clone(),
        enabled: Some(true),
        priority: Some(10),
        weight: Some(1),
        timeout_ms: Some(25000),
        max_concurrency: Some(2),
        rate_limit_hint_json: None,
    };

    let config_summary = ProviderService::create_provider_config(&db, create_input)
        .expect("create config must succeed");

    assert_eq!(config_summary.display_name, "Gemini 2.5 Flash Utama");
    assert_eq!(config_summary.model_id, "gemini-2.5-flash");
    assert_eq!(
        config_summary.credential_label,
        Some("Kunci Gemini Produksi".to_string())
    );
    assert_eq!(config_summary.priority, 10);

    // Verifikasi usage_count kredensial bertambah menjadi 1
    let cred_status = CredentialService::get_credential_status(&db, &store, &cred_summary.id)
        .expect("get cred status must succeed");
    assert_eq!(cred_status.usage_count, 1);

    // Proteksi kredensial bersama: penghapusan kredensial harus dicegah jika force = false
    let delete_cred_res =
        CredentialService::delete_credential(&db, &store, &cred_summary.id, false);
    assert!(
        delete_cred_res.is_err(),
        "deleting shared credential without force must fail"
    );

    // 3. Test Connection
    let mock_transport = Arc::new(MockHttpTransport::new());
    mock_transport.enqueue_response(HttpResponse::new(
        200,
        json!({
            "candidates": [{
                "content": { "parts": [{ "text": "pong" }] },
                "finishReason": "STOP"
            }]
        })
        .to_string(),
    ));

    let test_res = ProviderService::test_provider_connection(
        &db,
        &store,
        mock_transport.clone(),
        &config_summary.id,
    )
    .expect("test connection must succeed");

    assert!(test_res.success);
    assert_eq!(test_res.model_id, "gemini-2.5-flash");
    assert!(test_res.available_models.is_some());

    // 4. Update Provider Config
    let updated = ProviderService::update_provider_config(
        &db,
        UpdateProviderConfigInput {
            id: config_summary.id.clone(),
            display_name: Some("Gemini 2.5 Flash (Terupdate)".to_string()),
            base_url: None,
            model_id: Some("gemini-2.5-pro".to_string()),
            credential_ref: None,
            enabled: Some(false),
            priority: Some(5),
            weight: None,
            timeout_ms: None,
            max_concurrency: None,
            rate_limit_hint_json: None,
        },
    )
    .expect("update config must succeed");

    assert_eq!(updated.display_name, "Gemini 2.5 Flash (Terupdate)");
    assert_eq!(updated.model_id, "gemini-2.5-pro");
    assert!(!updated.enabled);
    assert_eq!(updated.priority, 5);

    // 5. Delete Provider Config
    let deleted = ProviderService::delete_provider_config(&db, &config_summary.id)
        .expect("delete config must succeed");
    assert!(deleted);

    // Verifikasi usage_count kredensial kembali menjadi 0
    let cred_status_after = CredentialService::get_credential_status(&db, &store, &cred_summary.id)
        .expect("get cred status must succeed");
    assert_eq!(cred_status_after.usage_count, 0);

    // Sekarang kredensial dapat dihapus dengan aman
    let delete_cred_now =
        CredentialService::delete_credential(&db, &store, &cred_summary.id, false);
    assert!(delete_cred_now.is_ok());
}
