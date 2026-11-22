use crate::domain::export::{
    ExportAnswerHistory, ExportAttempt, ExportCaptureMetadata, ExportFact, ExportFitResult,
    ExportOpportunity, ExportOpportunityCriterion, ExportOutcomeEvent, ExportPersona,
    ExportPreference, ExportQuestion, ExportRecommendation, ExportSession,
};
use crate::domain::fact::{FactHistoryEntry, FactSource};
use rusqlite::Connection;

pub struct ExportRepository;

impl ExportRepository {
    pub fn get_personas(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportPersona>, rusqlite::Error> {
        let mut result = Vec::new();
        if let Some(id) = persona_id {
            let mut stmt = connection.prepare(
                "SELECT id, display_name, description, is_active, status, created_at, updated_at, archived_at
                 FROM personas WHERE id = ?1",
            )?;
            let rows = stmt.query_map([id], |row| {
                let is_active_int: i64 = row.get(3)?;
                Ok(ExportPersona {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    description: row.get(2)?,
                    is_active: is_active_int == 1,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                    archived_at: row.get(7)?,
                })
            })?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let mut stmt = connection.prepare(
                "SELECT id, display_name, description, is_active, status, created_at, updated_at, archived_at
                 FROM personas ORDER BY is_active DESC, created_at ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                let is_active_int: i64 = row.get(3)?;
                Ok(ExportPersona {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    description: row.get(2)?,
                    is_active: is_active_int == 1,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                    archived_at: row.get(7)?,
                })
            })?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_facts(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportFact>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT id, persona_id, category, fact_key, attribute, data_type, value_json, normalized_value,
                    unit, verification_status, sensitivity, source_date, valid_from, valid_until, notes,
                    created_at, updated_at, archived_at
             FROM profile_facts WHERE persona_id = ?1 ORDER BY category ASC, attribute ASC"
        } else {
            "SELECT id, persona_id, category, fact_key, attribute, data_type, value_json, normalized_value,
                    unit, verification_status, sensitivity, source_date, valid_from, valid_until, notes,
                    created_at, updated_at, archived_at
             FROM profile_facts ORDER BY persona_id ASC, category ASC, attribute ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            let value_str: String = row.get(6)?;
            let parsed_value: serde_json::Value =
                serde_json::from_str(&value_str).unwrap_or(serde_json::Value::String(value_str));

            Ok(ExportFact {
                id: row.get(0)?,
                persona_id: row.get(1)?,
                category: row.get(2)?,
                fact_key: row.get(3)?,
                attribute: row.get(4)?,
                data_type: row.get(5)?,
                value: parsed_value,
                normalized_value: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                unit: row.get(8)?,
                verification_status: row.get(9)?,
                sensitivity: row.get(10)?,
                source_date: row.get(11)?,
                valid_from: row.get(12)?,
                valid_until: row.get(13)?,
                notes: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
                archived_at: row.get(17)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_fact_sources(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<FactSource>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT id, persona_id, fact_id, source_type, title, description, reference_uri, observed_at, valid_from, valid_until, notes, created_at
             FROM fact_sources WHERE persona_id = ?1 ORDER BY created_at ASC"
        } else {
            "SELECT id, persona_id, fact_id, source_type, title, description, reference_uri, observed_at, valid_from, valid_until, notes, created_at
             FROM fact_sources ORDER BY persona_id ASC, created_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            Ok(FactSource {
                id: row.get(0)?,
                persona_id: row.get(1)?,
                fact_id: row.get(2)?,
                source_type: row.get(3)?,
                title: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                description: row.get(5)?,
                reference_uri: row.get(6)?,
                observed_at: row.get(7)?,
                valid_from: row.get(8)?,
                valid_until: row.get(9)?,
                notes: row.get(10)?,
                created_at: row.get(11)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_fact_history(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<FactHistoryEntry>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT id, persona_id, fact_id, previous_value_json, new_value_json, previous_status, new_status, change_reason, source_id, changed_at
             FROM fact_history WHERE persona_id = ?1 ORDER BY changed_at ASC"
        } else {
            "SELECT id, persona_id, fact_id, previous_value_json, new_value_json, previous_status, new_status, change_reason, source_id, changed_at
             FROM fact_history ORDER BY persona_id ASC, changed_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            Ok(FactHistoryEntry {
                id: row.get(0)?,
                persona_id: row.get(1)?,
                fact_id: row.get(2)?,
                previous_value_json: row.get(3)?,
                new_value_json: row.get(4)?,
                previous_status: row.get(5)?,
                new_status: row.get(6)?,
                change_reason: row.get(7)?,
                source_id: row.get(8)?,
                changed_at: row.get(9)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_preferences(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportPreference>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT id, persona_id, preference_key, value_json, created_at, updated_at
             FROM persona_preferences WHERE persona_id = ?1 ORDER BY preference_key ASC"
        } else {
            "SELECT id, persona_id, preference_key, value_json, created_at, updated_at
             FROM persona_preferences ORDER BY persona_id ASC, preference_key ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            let value_str: String = row.get(3)?;
            let parsed_value: serde_json::Value =
                serde_json::from_str(&value_str).unwrap_or(serde_json::Value::String(value_str));
            Ok(ExportPreference {
                id: row.get(0)?,
                persona_id: row.get(1)?,
                preference_key: row.get(2)?,
                value: parsed_value,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_sessions(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportSession>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT id, persona_id, title, platform_label, status, created_at, updated_at, archived_at
             FROM survey_sessions WHERE persona_id = ?1 ORDER BY created_at ASC"
        } else {
            "SELECT id, persona_id, title, platform_label, status, created_at, updated_at, archived_at
             FROM survey_sessions ORDER BY persona_id ASC, created_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            Ok(ExportSession {
                id: row.get(0)?,
                persona_id: row.get(1)?,
                title: row.get(2)?,
                platform_label: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                archived_at: row.get(7)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_captures_metadata(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportCaptureMetadata>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT sc.id, sc.session_id, sc.capture_mode, sc.captured_at, sc.application_label, sc.created_at
             FROM screen_captures sc
             JOIN survey_sessions ss ON sc.session_id = ss.id
             WHERE ss.persona_id = ?1 ORDER BY sc.created_at ASC"
        } else {
            "SELECT id, session_id, capture_mode, captured_at, application_label, created_at
             FROM screen_captures ORDER BY created_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            Ok(ExportCaptureMetadata {
                id: row.get(0)?,
                session_id: row.get(1)?,
                capture_mode: row.get(2)?,
                captured_at: row.get(3)?,
                application_label: row.get(4)?,
                created_at: row.get(5)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_questions(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportQuestion>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT sq.id, sq.session_id, sq.capture_id, sq.question_text_original, sq.question_text_corrected,
                    sq.instructions, sq.options_json, sq.structure_json, sq.extraction_status, sq.question_order,
                    sq.created_at, sq.updated_at
             FROM survey_questions sq
             JOIN survey_sessions ss ON sq.session_id = ss.id
             WHERE ss.persona_id = ?1 ORDER BY sq.session_id ASC, sq.question_order ASC"
        } else {
            "SELECT id, session_id, capture_id, question_text_original, question_text_corrected,
                    instructions, options_json, structure_json, extraction_status, question_order,
                    created_at, updated_at
             FROM survey_questions ORDER BY session_id ASC, question_order ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            let options_str: String = row.get(6)?;
            let parsed_options: serde_json::Value =
                serde_json::from_str(&options_str).unwrap_or(serde_json::json!([]));
            let structure_str: Option<String> = row.get(7)?;
            let parsed_structure: Option<serde_json::Value> =
                structure_str.and_then(|s| serde_json::from_str(&s).ok());

            Ok(ExportQuestion {
                id: row.get(0)?,
                session_id: row.get(1)?,
                capture_id: row.get(2)?,
                question_text_original: row.get(3)?,
                question_text_corrected: row.get(4)?,
                instructions: row.get(5)?,
                options: parsed_options,
                structure: parsed_structure,
                extraction_status: row.get(8)?,
                question_order: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_recommendations(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportRecommendation>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT ar.id, ar.question_id, ar.status, ar.draft_text, ar.interpretation, ar.rationale,
                    ar.missing_information_json, ar.conflicts_json, ar.assumptions_json, ar.model_id,
                    ar.prompt_version, ar.created_at
             FROM answer_recommendations ar
             JOIN survey_questions sq ON ar.question_id = sq.id
             JOIN survey_sessions ss ON sq.session_id = ss.id
             WHERE ss.persona_id = ?1 ORDER BY ar.created_at ASC"
        } else {
            "SELECT id, question_id, status, draft_text, interpretation, rationale,
                    missing_information_json, conflicts_json, assumptions_json, model_id,
                    prompt_version, created_at
             FROM answer_recommendations ORDER BY created_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            let missing_str: String = row.get(6)?;
            let conflicts_str: String = row.get(7)?;
            let assumptions_str: String = row.get(8)?;

            Ok(ExportRecommendation {
                id: row.get(0)?,
                question_id: row.get(1)?,
                status: row.get(2)?,
                draft_text: row.get(3)?,
                interpretation: row.get(4)?,
                rationale: row.get(5)?,
                missing_information: serde_json::from_str(&missing_str)
                    .unwrap_or(serde_json::json!([])),
                conflicts: serde_json::from_str(&conflicts_str).unwrap_or(serde_json::json!([])),
                assumptions: serde_json::from_str(&assumptions_str)
                    .unwrap_or(serde_json::json!([])),
                model_id: row.get(9)?,
                prompt_version: row.get(10)?,
                created_at: row.get(11)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_answer_history(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportAnswerHistory>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT ah.id, ah.question_id, ah.recommendation_id, ah.answer_text, ah.action, ah.created_at
             FROM answer_history ah
             JOIN survey_questions sq ON ah.question_id = sq.id
             JOIN survey_sessions ss ON sq.session_id = ss.id
             WHERE ss.persona_id = ?1 ORDER BY ah.created_at ASC"
        } else {
            "SELECT id, question_id, recommendation_id, answer_text, action, created_at
             FROM answer_history ORDER BY created_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            Ok(ExportAnswerHistory {
                id: row.get(0)?,
                question_id: row.get(1)?,
                recommendation_id: row.get(2)?,
                answer_text: row.get(3)?,
                action: row.get(4)?,
                created_at: row.get(5)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_fit_results(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportFitResult>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT id, opportunity_id, persona_id, criterion_id, match_state, supporting_fact_ids_json, explanation, evaluated_at
             FROM survey_fit_results WHERE persona_id = ?1 ORDER BY evaluated_at ASC"
        } else {
            "SELECT id, opportunity_id, persona_id, criterion_id, match_state, supporting_fact_ids_json, explanation, evaluated_at
             FROM survey_fit_results ORDER BY persona_id ASC, evaluated_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            let facts_str: String = row.get(5)?;
            Ok(ExportFitResult {
                id: row.get(0)?,
                opportunity_id: row.get(1)?,
                persona_id: row.get(2)?,
                criterion_id: row.get(3)?,
                match_state: row.get(4)?,
                supporting_fact_ids: serde_json::from_str(&facts_str)
                    .unwrap_or(serde_json::json!([])),
                explanation: row.get(6)?,
                evaluated_at: row.get(7)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_attempts(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportAttempt>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT id, opportunity_id, persona_id, session_id, started_at, ended_at, current_outcome, created_at, updated_at
             FROM survey_attempts WHERE persona_id = ?1 ORDER BY started_at ASC"
        } else {
            "SELECT id, opportunity_id, persona_id, session_id, started_at, ended_at, current_outcome, created_at, updated_at
             FROM survey_attempts ORDER BY persona_id ASC, started_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            Ok(ExportAttempt {
                id: row.get(0)?,
                opportunity_id: row.get(1)?,
                persona_id: row.get(2)?,
                session_id: row.get(3)?,
                started_at: row.get(4)?,
                ended_at: row.get(5)?,
                current_outcome: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_outcome_events(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<Vec<ExportOutcomeEvent>, rusqlite::Error> {
        let mut result = Vec::new();
        let query = if persona_id.is_some() {
            "SELECT oe.id, oe.attempt_id, oe.outcome_label, oe.evidence_source, oe.evidence_note, oe.confidence, oe.observed_at, oe.created_at
             FROM survey_outcome_events oe
             JOIN survey_attempts sa ON oe.attempt_id = sa.id
             WHERE sa.persona_id = ?1 ORDER BY oe.observed_at ASC"
        } else {
            "SELECT id, attempt_id, outcome_label, evidence_source, evidence_note, confidence, observed_at, created_at
             FROM survey_outcome_events ORDER BY observed_at ASC"
        };

        let mut stmt = connection.prepare(query)?;
        let row_mapper = |row: &rusqlite::Row| {
            Ok(ExportOutcomeEvent {
                id: row.get(0)?,
                attempt_id: row.get(1)?,
                outcome_label: row.get(2)?,
                evidence_source: row.get(3)?,
                evidence_note: row.get(4)?,
                confidence: row.get(5)?,
                observed_at: row.get(6)?,
                created_at: row.get(7)?,
            })
        };

        if let Some(id) = persona_id {
            let rows = stmt.query_map([id], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map([], row_mapper)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    pub fn get_opportunities(
        connection: &Connection,
    ) -> Result<Vec<ExportOpportunity>, rusqlite::Error> {
        let mut stmt = connection.prepare(
            "SELECT id, title, platform_label, topic, description, captured_at, created_at, updated_at
             FROM survey_opportunities ORDER BY captured_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ExportOpportunity {
                id: row.get(0)?,
                title: row.get(1)?,
                platform_label: row.get(2)?,
                topic: row.get(3)?,
                description: row.get(4)?,
                captured_at: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_opportunity_criteria(
        connection: &Connection,
    ) -> Result<Vec<ExportOpportunityCriterion>, rusqlite::Error> {
        let mut stmt = connection.prepare(
            "SELECT id, opportunity_id, criterion_text, normalized_attribute, expected_value_json, criterion_order
             FROM survey_opportunity_criteria ORDER BY opportunity_id ASC, criterion_order ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let val_str: Option<String> = row.get(4)?;
            let expected_val: Option<serde_json::Value> =
                val_str.and_then(|s| serde_json::from_str(&s).ok());
            Ok(ExportOpportunityCriterion {
                id: row.get(0)?,
                opportunity_id: row.get(1)?,
                criterion_text: row.get(2)?,
                normalized_attribute: row.get(3)?,
                expected_value: expected_val,
                criterion_order: row.get(5)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn count_sensitive_facts(
        connection: &Connection,
        persona_id: Option<&str>,
    ) -> Result<(i64, i64), rusqlite::Error> {
        let mut sensitive = 0i64;
        let mut highly_sensitive = 0i64;

        let query = if persona_id.is_some() {
            "SELECT sensitivity, COUNT(1) FROM profile_facts WHERE persona_id = ?1 GROUP BY sensitivity"
        } else {
            "SELECT sensitivity, COUNT(1) FROM profile_facts GROUP BY sensitivity"
        };

        let mut stmt = connection.prepare(query)?;
        let map_func = |row: &rusqlite::Row| {
            let sensitivity: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((sensitivity, count))
        };

        let rows = if let Some(id) = persona_id {
            stmt.query_map([id], map_func)?
        } else {
            stmt.query_map([], map_func)?
        };

        for item in rows {
            let (sens, cnt) = item?;
            if sens == "sensitive" {
                sensitive = cnt;
            } else if sens == "highly_sensitive" {
                highly_sensitive = cnt;
            }
        }

        Ok((sensitive, highly_sensitive))
    }
}
