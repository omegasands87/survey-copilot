use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::export::{ExportDataCollections, ExportPayload, ExportScope, ExportSummary};
use crate::repository::{ExportRepository, PersonaRepository};

pub struct ExportService;

impl ExportService {
    /// Menghasilkan ringkasan data sebelum berkas ekspor disimpan.
    pub fn get_summary(
        database: &Database,
        persona_id: Option<&str>,
    ) -> Result<ExportSummary, AppError> {
        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let mut target_persona_name: Option<String> = None;

        if let Some(id) = persona_id {
            let id = id.trim();
            if id.is_empty() {
                return Err(AppError::validation("personaId", "ID profil wajib diisi."));
            }
            let persona = PersonaRepository::find_by_id(&connection, id)
                .map_err(AppError::from)?
                .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))?;
            target_persona_name = Some(persona.display_name);
        }

        let personas =
            ExportRepository::get_personas(&connection, persona_id).map_err(AppError::from)?;
        let facts = ExportRepository::get_facts(&connection, persona_id).map_err(AppError::from)?;
        let sources =
            ExportRepository::get_fact_sources(&connection, persona_id).map_err(AppError::from)?;
        let history =
            ExportRepository::get_fact_history(&connection, persona_id).map_err(AppError::from)?;
        let preferences =
            ExportRepository::get_preferences(&connection, persona_id).map_err(AppError::from)?;
        let sessions =
            ExportRepository::get_sessions(&connection, persona_id).map_err(AppError::from)?;
        let questions =
            ExportRepository::get_questions(&connection, persona_id).map_err(AppError::from)?;
        let recommendations = ExportRepository::get_recommendations(&connection, persona_id)
            .map_err(AppError::from)?;
        let answers = ExportRepository::get_answer_history(&connection, persona_id)
            .map_err(AppError::from)?;
        let fit_results =
            ExportRepository::get_fit_results(&connection, persona_id).map_err(AppError::from)?;
        let attempts =
            ExportRepository::get_attempts(&connection, persona_id).map_err(AppError::from)?;

        let (sensitive_facts_count, highly_sensitive_facts_count) =
            ExportRepository::count_sensitive_facts(&connection, persona_id)
                .map_err(AppError::from)?;

        let total_personas = personas.len() as i64;
        let total_facts = facts.len() as i64;
        let total_sources = sources.len() as i64;
        let total_history_entries = history.len() as i64;
        let total_preferences = preferences.len() as i64;
        let total_sessions = sessions.len() as i64;
        let total_questions = questions.len() as i64;
        let total_recommendations = recommendations.len() as i64;
        let total_answers = answers.len() as i64;
        let total_fit_results = fit_results.len() as i64;
        let total_attempts = attempts.len() as i64;

        let estimated_records_count = total_personas
            + total_facts
            + total_sources
            + total_history_entries
            + total_preferences
            + total_sessions
            + total_questions
            + total_recommendations
            + total_answers
            + total_fit_results
            + total_attempts;

        let has_sensitive_data = sensitive_facts_count > 0 || highly_sensitive_facts_count > 0;
        let warning_message = if has_sensitive_data {
            Some(format!(
                "Perhatian Privasi: Berkas ekspor memuat {} fakta sensitif dan {} fakta sangat sensitif. Pastikan berkas disimpan di tempat aman.",
                sensitive_facts_count, highly_sensitive_facts_count
            ))
        } else {
            None
        };

        let scope_str = if persona_id.is_some() {
            ExportScope::SinglePersona.as_str().to_string()
        } else {
            ExportScope::FullApplication.as_str().to_string()
        };

        Ok(ExportSummary {
            export_scope: scope_str,
            target_persona_id: persona_id.map(|s| s.to_string()),
            target_persona_name,
            total_personas,
            total_facts,
            total_sources,
            total_history_entries,
            total_preferences,
            total_sessions,
            total_questions,
            total_recommendations,
            total_answers,
            total_fit_results,
            total_attempts,
            sensitive_facts_count,
            highly_sensitive_facts_count,
            has_sensitive_data,
            estimated_records_count,
            warning_message,
        })
    }

    /// Menghasilkan seluruh struktur dokumen ekspor terstruktur.
    pub fn export_data(
        database: &Database,
        persona_id: Option<&str>,
    ) -> Result<ExportPayload, AppError> {
        let summary = Self::get_summary(database, persona_id)?;

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let personas =
            ExportRepository::get_personas(&connection, persona_id).map_err(AppError::from)?;
        let facts = ExportRepository::get_facts(&connection, persona_id).map_err(AppError::from)?;
        let fact_sources =
            ExportRepository::get_fact_sources(&connection, persona_id).map_err(AppError::from)?;
        let fact_history =
            ExportRepository::get_fact_history(&connection, persona_id).map_err(AppError::from)?;
        let persona_preferences =
            ExportRepository::get_preferences(&connection, persona_id).map_err(AppError::from)?;
        let survey_sessions =
            ExportRepository::get_sessions(&connection, persona_id).map_err(AppError::from)?;
        let screen_captures = ExportRepository::get_captures_metadata(&connection, persona_id)
            .map_err(AppError::from)?;
        let survey_questions =
            ExportRepository::get_questions(&connection, persona_id).map_err(AppError::from)?;
        let answer_recommendations = ExportRepository::get_recommendations(&connection, persona_id)
            .map_err(AppError::from)?;
        let answer_history = ExportRepository::get_answer_history(&connection, persona_id)
            .map_err(AppError::from)?;
        let survey_fit_results =
            ExportRepository::get_fit_results(&connection, persona_id).map_err(AppError::from)?;
        let survey_attempts =
            ExportRepository::get_attempts(&connection, persona_id).map_err(AppError::from)?;
        let survey_outcome_events = ExportRepository::get_outcome_events(&connection, persona_id)
            .map_err(AppError::from)?;

        let survey_opportunities = if persona_id.is_none() {
            ExportRepository::get_opportunities(&connection).map_err(AppError::from)?
        } else {
            // Untuk persona tunggal, hanya sertakan opportunity yang relevan dengan attempts/fit_results milik persona tersebut
            let opp_ids: std::collections::HashSet<String> = survey_fit_results
                .iter()
                .map(|f| f.opportunity_id.clone())
                .chain(survey_attempts.iter().map(|a| a.opportunity_id.clone()))
                .collect();

            ExportRepository::get_opportunities(&connection)
                .map_err(AppError::from)?
                .into_iter()
                .filter(|o| opp_ids.contains(&o.id))
                .collect()
        };

        let opportunity_ids: std::collections::HashSet<String> =
            survey_opportunities.iter().map(|o| o.id.clone()).collect();

        let survey_opportunity_criteria = ExportRepository::get_opportunity_criteria(&connection)
            .map_err(AppError::from)?
            .into_iter()
            .filter(|c| opportunity_ids.contains(&c.opportunity_id))
            .collect();

        let data = ExportDataCollections {
            personas,
            facts,
            fact_sources,
            fact_history,
            persona_preferences,
            survey_sessions,
            screen_captures,
            survey_questions,
            answer_recommendations,
            answer_history,
            survey_opportunities,
            survey_opportunity_criteria,
            survey_fit_results,
            survey_attempts,
            survey_outcome_events,
            survey_memory: Vec::new(),
        };

        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let payload = ExportPayload {
            schema_url: "https://survey-copilot.app/schemas/export-v1.json".to_string(),
            schema_version: "1.0.0".to_string(),
            app_version: "0.1.0".to_string(),
            export_timestamp: now,
            export_scope: summary.export_scope.clone(),
            target_persona_id: summary.target_persona_id.clone(),
            target_persona_name: summary.target_persona_name.clone(),
            summary,
            data,
        };

        Ok(payload)
    }

    /// Menghasilkan teks JSON terformat rapi untuk diunduh langsung oleh pengguna.
    pub fn export_data_as_json_string(
        database: &Database,
        persona_id: Option<&str>,
    ) -> Result<String, AppError> {
        let payload = Self::export_data(database, persona_id)?;

        let json_string = serde_json::to_string_pretty(&payload)
            .map_err(|e| AppError::internal(format!("Gagal membuat format JSON ekspor: {e}")))?;

        // Audit privasi ketat: pastikan tidak ada kebocoran kunci rahasia
        if json_string.contains("credential_ref") || json_string.contains("api_key") {
            return Err(AppError::internal(
                "Pelanggaran keamanan: Ekspor dibatalkan karena terdeteksi kunci atau referensi kredensial.",
            ));
        }

        Ok(json_string)
    }
}
