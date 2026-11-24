use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::fit::{FitCriterionResult, MatchState, SurveyFitRecord};

pub struct FitRepository;

impl FitRepository {
    /// Menyimpan atau memperbarui hasil evaluasi kriteria kesesuaian survei untuk suatu persona
    pub fn save_fit_results(
        conn: &mut Connection,
        opportunity_id: &str,
        persona_id: &str,
        results: &[FitCriterionResult],
        evaluated_at: &str,
    ) -> Result<(), rusqlite::Error> {
        let tx = conn.transaction()?;

        // Hapus hasil evaluasi kriteria lama untuk persona & peluang ini agar selalu segar
        tx.execute(
            "DELETE FROM survey_fit_results WHERE opportunity_id = ?1 AND persona_id = ?2",
            params![opportunity_id, persona_id],
        )?;

        // Simpan setiap hasil kriteria secara atomik
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO survey_fit_results (
                    id, opportunity_id, persona_id, criterion_id,
                    match_state, supporting_fact_ids_json, explanation, evaluated_at
                ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, ?7, ?8
                )
                "#,
            )?;

            for item in results {
                let id = Uuid::new_v4().to_string();
                let supporting_ids_json = serde_json::to_string(&item.supporting_fact_ids)
                    .unwrap_or_else(|_| "[]".to_string());

                stmt.execute(params![
                    id,
                    opportunity_id,
                    persona_id,
                    item.criterion_id,
                    item.match_state.as_str(),
                    supporting_ids_json,
                    item.explanation,
                    evaluated_at,
                ])?;
            }
        }

        // Perbarui status peluang survei menjadi 'evaluated'
        tx.execute(
            r#"
            UPDATE survey_opportunities
            SET status = 'evaluated', updated_at = ?2
            WHERE id = ?1
            "#,
            params![opportunity_id, evaluated_at],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Mengambil daftar rekaman hasil evaluasi untuk peluang dan persona tertentu
    pub fn get_fit_results(
        conn: &Connection,
        opportunity_id: &str,
        persona_id: &str,
    ) -> Result<Vec<SurveyFitRecord>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, opportunity_id, persona_id, criterion_id,
                match_state, supporting_fact_ids_json, explanation, evaluated_at
            FROM survey_fit_results
            WHERE opportunity_id = ?1 AND persona_id = ?2
            ORDER BY rowid ASC
            "#,
        )?;

        let rows = stmt.query_map(params![opportunity_id, persona_id], |row| {
            let match_str: String = row.get(4)?;
            Ok(SurveyFitRecord {
                id: row.get(0)?,
                opportunity_id: row.get(1)?,
                persona_id: row.get(2)?,
                criterion_id: row.get(3)?,
                match_state: MatchState::from_str_loose(&match_str),
                supporting_fact_ids_json: row.get(5)?,
                explanation: row.get(6)?,
                evaluated_at: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    /// Mengambil seluruh hasil evaluasi untuk suatu peluang lintas seluruh persona
    pub fn list_fit_results_for_opportunity(
        conn: &Connection,
        opportunity_id: &str,
    ) -> Result<Vec<SurveyFitRecord>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, opportunity_id, persona_id, criterion_id,
                match_state, supporting_fact_ids_json, explanation, evaluated_at
            FROM survey_fit_results
            WHERE opportunity_id = ?1
            ORDER BY persona_id ASC, evaluated_at DESC
            "#,
        )?;

        let rows = stmt.query_map(params![opportunity_id], |row| {
            let match_str: String = row.get(4)?;
            Ok(SurveyFitRecord {
                id: row.get(0)?,
                opportunity_id: row.get(1)?,
                persona_id: row.get(2)?,
                criterion_id: row.get(3)?,
                match_state: MatchState::from_str_loose(&match_str),
                supporting_fact_ids_json: row.get(5)?,
                explanation: row.get(6)?,
                evaluated_at: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    /// Menghapus hasil evaluasi untuk peluang dan persona tertentu
    pub fn delete_fit_results(
        conn: &Connection,
        opportunity_id: &str,
        persona_id: &str,
    ) -> Result<usize, rusqlite::Error> {
        conn.execute(
            "DELETE FROM survey_fit_results WHERE opportunity_id = ?1 AND persona_id = ?2",
            params![opportunity_id, persona_id],
        )
    }
}
