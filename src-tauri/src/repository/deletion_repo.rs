use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllRecordCounts {
    pub personas: i64,
    pub facts: i64,
    pub sources: i64,
    pub history: i64,
    pub preferences: i64,
    pub sessions: i64,
    pub captures: i64,
    pub questions: i64,
    pub recommendations: i64,
    pub answers: i64,
    pub opportunities: i64,
    pub criteria: i64,
    pub fit_results: i64,
    pub attempts: i64,
    pub outcomes: i64,
    pub provider_configs: i64,
    pub ai_usage: i64,
    pub app_settings: i64,
    pub credentials: i64,
    pub ocr_extractions: i64,
    pub recognitions: i64,
    pub branch_transitions: i64,
    pub question_predictions: i64,
    pub diagnostic_logs: i64,
    pub recommendation_sources: i64,
    pub provider_health: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WipeTableCounts {
    pub personas: i64,
    pub facts: i64,
    pub sources: i64,
    pub history: i64,
    pub preferences: i64,
    pub sessions: i64,
    pub questions: i64,
    pub recommendations: i64,
    pub answers: i64,
    pub fit_results: i64,
    pub attempts: i64,
    pub provider_configs: i64,
    pub credentials: i64,
    pub ocr_extractions: i64,
    pub recognitions: i64,
    pub branch_transitions: i64,
    pub question_predictions: i64,
    pub diagnostic_logs: i64,
    pub recommendation_sources: i64,
    pub provider_health: i64,
}

pub struct DeletionRepository;

impl DeletionRepository {
    pub fn count_all_records(connection: &Connection) -> Result<AllRecordCounts, rusqlite::Error> {
        let count_table = |table: &str| -> Result<i64, rusqlite::Error> {
            let sql = format!("SELECT COUNT(1) FROM {table}");
            connection.query_row(&sql, [], |row| row.get(0))
        };

        let personas = count_table("personas")?;
        let facts = count_table("profile_facts")?;
        let sources = count_table("fact_sources")?;
        let history = count_table("fact_history")?;
        let preferences = count_table("persona_preferences")?;
        let sessions = count_table("survey_sessions")?;
        let captures = count_table("screen_captures")?;
        let questions = count_table("survey_questions")?;
        let recommendations = count_table("answer_recommendations")?;
        let answers = count_table("answer_history")?;
        let opportunities = count_table("survey_opportunities")?;
        let criteria = count_table("survey_opportunity_criteria")?;
        let fit_results = count_table("survey_fit_results")?;
        let attempts = count_table("survey_attempts")?;
        let outcomes = count_table("survey_outcome_events")?;
        let provider_configs = count_table("provider_configs")?;
        let ai_usage = count_table("ai_usage_events")?;
        let app_settings = count_table("app_settings")?;
        let credentials = count_table("app_credentials")?;
        let ocr_extractions = count_table("ocr_extractions")?;
        let recognitions = count_table("survey_recognitions")?;
        let branch_transitions = count_table("survey_branch_transitions")?;
        let question_predictions = count_table("survey_question_predictions")?;
        let diagnostic_logs = count_table("app_diagnostic_logs")?;
        let recommendation_sources = count_table("recommendation_sources")?;
        let provider_health = count_table("provider_health")?;

        let total = [
            personas,
            facts,
            sources,
            history,
            preferences,
            sessions,
            captures,
            questions,
            recommendations,
            answers,
            opportunities,
            criteria,
            fit_results,
            attempts,
            outcomes,
            provider_configs,
            ai_usage,
            app_settings,
            credentials,
            ocr_extractions,
            recognitions,
            branch_transitions,
            question_predictions,
            diagnostic_logs,
            recommendation_sources,
            provider_health,
        ]
        .iter()
        .sum();

        Ok(AllRecordCounts {
            personas,
            facts,
            sources,
            history,
            preferences,
            sessions,
            captures,
            questions,
            recommendations,
            answers,
            opportunities,
            criteria,
            fit_results,
            attempts,
            outcomes,
            provider_configs,
            ai_usage,
            app_settings,
            credentials,
            ocr_extractions,
            recognitions,
            branch_transitions,
            question_predictions,
            diagnostic_logs,
            recommendation_sources,
            provider_health,
            total,
        })
    }

    pub fn get_capture_image_paths_for_persona(
        connection: &Connection,
        persona_id: &str,
    ) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = connection.prepare(
            "SELECT sc.temporary_image_path
             FROM screen_captures sc
             JOIN survey_sessions ss ON sc.session_id = ss.id
             WHERE ss.persona_id = ?1 AND sc.temporary_image_path IS NOT NULL",
        )?;
        let rows = stmt.query_map([persona_id], |row| row.get::<_, String>(0))?;
        let mut paths = Vec::new();
        for path in rows {
            paths.push(path?);
        }
        Ok(paths)
    }

    pub fn get_all_capture_image_paths(
        connection: &Connection,
    ) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = connection.prepare(
            "SELECT temporary_image_path FROM screen_captures WHERE temporary_image_path IS NOT NULL",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut paths = Vec::new();
        for path in rows {
            paths.push(path?);
        }
        Ok(paths)
    }

    pub fn wipe_all_data(connection: &Connection) -> Result<WipeTableCounts, rusqlite::Error> {
        let before = Self::count_all_records(connection)?;

        // Every delete is checked. Missing tables or SQL errors abort the transaction.
        connection.execute("DELETE FROM survey_question_predictions", [])?;
        connection.execute("DELETE FROM survey_branch_transitions", [])?;
        connection.execute("DELETE FROM survey_recognitions", [])?;
        connection.execute("DELETE FROM ocr_extractions", [])?;
        connection.execute("DELETE FROM recommendation_sources", [])?;
        connection.execute("DELETE FROM answer_history", [])?;
        connection.execute("DELETE FROM answer_recommendations", [])?;
        connection.execute("DELETE FROM survey_questions", [])?;
        connection.execute("DELETE FROM screen_captures", [])?;
        connection.execute("DELETE FROM survey_sessions", [])?;
        connection.execute("DELETE FROM fact_history", [])?;
        connection.execute("DELETE FROM fact_sources", [])?;
        connection.execute("DELETE FROM profile_facts", [])?;
        connection.execute("DELETE FROM persona_preferences", [])?;
        connection.execute("DELETE FROM survey_outcome_events", [])?;
        connection.execute("DELETE FROM survey_attempts", [])?;
        connection.execute("DELETE FROM survey_fit_results", [])?;
        connection.execute("DELETE FROM survey_opportunity_criteria", [])?;
        connection.execute("DELETE FROM survey_opportunities", [])?;
        connection.execute("DELETE FROM ai_usage_events", [])?;
        connection.execute("DELETE FROM provider_health", [])?;
        connection.execute("DELETE FROM provider_configs", [])?;
        connection.execute("DELETE FROM app_settings", [])?;
        connection.execute("DELETE FROM app_diagnostic_logs", [])?;
        connection.execute("DELETE FROM app_credentials", [])?;
        connection.execute("DELETE FROM personas", [])?;

        // Provider pool settings are intentionally re-created with safe defaults.
        connection.execute("DELETE FROM provider_pool_settings", [])?;
        connection.execute(
            "INSERT INTO provider_pool_settings
              (id, strategy, max_attempts, auto_retry, retry_base_delay_ms, max_retry_delay_ms, updated_at)
             VALUES
              ('default', 'adaptive', 3, 1, 1000, 15000, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            [],
        )?;

        Ok(WipeTableCounts {
            personas: before.personas,
            facts: before.facts,
            sources: before.sources,
            history: before.history,
            preferences: before.preferences,
            sessions: before.sessions,
            questions: before.questions,
            recommendations: before.recommendations,
            answers: before.answers,
            fit_results: before.fit_results,
            attempts: before.attempts,
            provider_configs: before.provider_configs,
            credentials: before.credentials,
            ocr_extractions: before.ocr_extractions,
            recognitions: before.recognitions,
            branch_transitions: before.branch_transitions,
            question_predictions: before.question_predictions,
            diagnostic_logs: before.diagnostic_logs,
            recommendation_sources: before.recommendation_sources,
            provider_health: before.provider_health,
        })
    }

    pub fn wal_checkpoint_truncate(connection: &Connection) -> Result<String, rusqlite::Error> {
        let mut stmt = connection.prepare("PRAGMA wal_checkpoint(TRUNCATE)")?;
        let res = stmt.query_row([], |row| {
            let busy: i64 = row.get(0)?;
            let log: i64 = row.get(1)?;
            let checkpointed: i64 = row.get(2)?;
            Ok(format!(
                "busy={busy}, log={log}, checkpointed={checkpointed}"
            ))
        });
        match res {
            Ok(status) => Ok(format!("Berhasil (TRUNCATE: {status})")),
            Err(e) => Ok(format!("Peringatan non-fatal WAL checkpoint: {e}")),
        }
    }

    pub fn vacuum(connection: &Connection) -> Result<String, rusqlite::Error> {
        connection.execute("VACUUM", [])?;
        Ok("Berhasil (database dipadatkan)".to_string())
    }
}
