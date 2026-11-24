use std::path::Path;
use survey_copilot_lib::db::Database;
use survey_copilot_lib::repository::PersonaRepository;
use survey_copilot_lib::services::ProfileImportService;

fn setup_test_db() -> Database {
    Database::initialize(Path::new(":memory:")).expect("in-memory database should migrate")
}

fn sample_single_persona_export() -> String {
    serde_json::json!({
        "$schema": "https://survey-copilot.app/schemas/export-v1.json",
        "schemaVersion": "1.0.0",
        "appVersion": "0.1.0",
        "exportTimestamp": "2026-11-24T09:00:00.000Z",
        "exportScope": "single_persona",
        "targetPersonaId": "source-persona",
        "targetPersonaName": "Profil Responden Asli",
        "summary": {
            "exportScope": "single_persona",
            "targetPersonaId": "source-persona",
            "targetPersonaName": "Profil Responden Asli",
            "totalPersonas": 1,
            "totalFacts": 1,
            "totalSources": 2,
            "totalHistoryEntries": 1,
            "totalPreferences": 0,
            "totalSessions": 0,
            "totalQuestions": 0,
            "totalRecommendations": 0,
            "totalAnswers": 0,
            "totalFitResults": 0,
            "totalAttempts": 0,
            "sensitiveFactsCount": 0,
            "highlySensitiveFactsCount": 0,
            "hasSensitiveData": false,
            "estimatedRecordsCount": 4,
            "warningMessage": null
        },
        "data": {
            "personas": [{
                "id": "source-persona",
                "displayName": "Profil Responden Asli",
                "description": "Profil yang diekspor",
                "isActive": true,
                "status": "active",
                "createdAt": "2026-11-01T00:00:00.000Z",
                "updatedAt": "2026-11-01T00:00:00.000Z",
                "archivedAt": null
            }],
            "facts": [{
                "id": "source-fact",
                "personaId": "source-persona",
                "category": "Demografi",
                "factKey": "age",
                "attribute": "Usia",
                "dataType": "number",
                "value": 32,
                "normalizedValue": "32",
                "unit": "tahun",
                "verificationStatus": "confirmed",
                "sensitivity": "ordinary",
                "sourceDate": null,
                "validFrom": null,
                "validUntil": null,
                "notes": null,
                "createdAt": "2026-11-01T00:00:00.000Z",
                "updatedAt": "2026-11-01T00:00:00.000Z",
                "archivedAt": null
            }],
            "factSources": [
                {
                    "id": "source-evidence-1",
                    "personaId": "source-persona",
                    "factId": "source-fact",
                    "sourceType": "user_declaration",
                    "title": "Pernyataan pengguna",
                    "description": "Dinyatakan langsung oleh pemilik profil.",
                    "referenceUri": null,
                    "observedAt": "2026-11-01T00:00:00.000Z",
                    "validFrom": null,
                    "validUntil": null,
                    "notes": null,
                    "createdAt": "2026-11-01T00:00:00.000Z"
                },
                {
                    "id": "source-evidence-2",
                    "personaId": "source-persona",
                    "factId": "source-fact",
                    "sourceType": "manual_entry",
                    "title": "Catatan tambahan",
                    "description": "Catatan yang ditambahkan pemilik profil.",
                    "referenceUri": null,
                    "observedAt": null,
                    "validFrom": null,
                    "validUntil": null,
                    "notes": null,
                    "createdAt": "2026-11-01T00:00:00.000Z"
                }
            ],
            "factHistory": [],
            "personaPreferences": [],
            "surveySessions": [],
            "screenCaptures": [],
            "surveyQuestions": [],
            "answerRecommendations": [],
            "answerHistory": [],
            "surveyOpportunities": [],
            "surveyOpportunityCriteria": [],
            "surveyFitResults": [],
            "surveyAttempts": [],
            "surveyOutcomeEvents": [],
            "surveyMemory": []
        }
    })
    .to_string()
}

#[test]
fn imports_single_persona_as_new_inactive_profile_with_fact_sources() {
    let database = setup_test_db();
    let json = sample_single_persona_export();

    let result = ProfileImportService::import_single_persona_json(
        &database,
        &json,
        Some("Salinan Profil Responden"),
    )
    .expect("valid single-persona export should import");

    assert_eq!(result.persona.display_name, "Salinan Profil Responden");
    assert!(!result.persona.is_active);
    assert_eq!(result.imported_facts, 1);
    assert_eq!(result.imported_sources, 2);

    let connection = database.connection.lock().unwrap();
    let fact_count: i64 = connection
        .query_row(
            "SELECT COUNT(1) FROM profile_facts WHERE persona_id = ?1",
            [&result.persona.id],
            |row| row.get(0),
        )
        .unwrap();
    let source_count: i64 = connection
        .query_row(
            "SELECT COUNT(1) FROM fact_sources WHERE persona_id = ?1",
            [&result.persona.id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(fact_count, 1);
    assert_eq!(source_count, 2);
    assert!(
        PersonaRepository::find_by_id(&connection, "source-persona")
            .unwrap()
            .is_none(),
        "source IDs must not be reused in the destination database"
    );
}

#[test]
fn rejects_full_application_export_without_creating_persona() {
    let database = setup_test_db();
    let mut payload: serde_json::Value =
        serde_json::from_str(&sample_single_persona_export()).unwrap();
    payload["exportScope"] = serde_json::json!("full_application");
    payload["summary"]["exportScope"] = serde_json::json!("full_application");

    let error =
        ProfileImportService::import_single_persona_json(&database, &payload.to_string(), None)
            .expect_err("full application exports must not be accepted");

    assert_eq!(error.code, "VALIDATION_ERROR");
    let connection = database.connection.lock().unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(1) FROM personas", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn rolls_back_persona_when_a_fact_value_fails_validation() {
    let database = setup_test_db();
    let mut payload: serde_json::Value =
        serde_json::from_str(&sample_single_persona_export()).unwrap();
    payload["data"]["facts"][0]["dataType"] = serde_json::json!("number");
    payload["data"]["facts"][0]["value"] = serde_json::json!("not-a-number");

    let error =
        ProfileImportService::import_single_persona_json(&database, &payload.to_string(), None)
            .expect_err("invalid fact value should cancel the whole import");

    assert_eq!(error.code, "VALIDATION_ERROR");
    let connection = database.connection.lock().unwrap();
    let personas: i64 = connection
        .query_row("SELECT COUNT(1) FROM personas", [], |row| row.get(0))
        .unwrap();
    let facts: i64 = connection
        .query_row("SELECT COUNT(1) FROM profile_facts", [], |row| row.get(0))
        .unwrap();
    assert_eq!(personas, 0);
    assert_eq!(facts, 0);
}

#[test]
fn rejects_json_larger_than_five_megabytes_before_parsing() {
    let database = setup_test_db();
    let oversized = " ".repeat(5 * 1024 * 1024 + 1);
    let error = ProfileImportService::import_single_persona_json(&database, &oversized, None)
        .expect_err("oversized files must be rejected");

    assert_eq!(error.code, "VALIDATION_ERROR");
}
