use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::attempt::{
    AttemptFilter, OutcomeConfidence, OutcomeLabel, SurveyAttempt, SurveyAttemptDetail,
    SurveyOutcomeEvent,
};
use crate::repository::opportunity_repo::OpportunityRepository;

pub struct AttemptRepository;

impl AttemptRepository {
    /// Menyimpan atau memperbarui entitas percobaan survei
    pub fn save_attempt(conn: &Connection, attempt: &SurveyAttempt) -> Result<(), rusqlite::Error> {
        conn.execute(
            r#"
            INSERT INTO survey_attempts (
                id, opportunity_id, persona_id, session_id,
                started_at, ended_at, current_outcome,
                created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7,
                ?8, ?9
            )
            ON CONFLICT(id) DO UPDATE SET
                opportunity_id = excluded.opportunity_id,
                session_id = excluded.session_id,
                ended_at = excluded.ended_at,
                current_outcome = excluded.current_outcome,
                updated_at = excluded.updated_at
            "#,
            params![
                attempt.id,
                attempt.opportunity_id,
                attempt.persona_id,
                attempt.session_id,
                attempt.started_at,
                attempt.ended_at,
                attempt.current_outcome.as_str(),
                attempt.created_at,
                attempt.updated_at,
            ],
        )?;

        Ok(())
    }

    /// Menyimpan peristiwa hasil observasi survei
    pub fn save_outcome_event(
        conn: &Connection,
        event: &SurveyOutcomeEvent,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            r#"
            INSERT INTO survey_outcome_events (
                id, attempt_id, outcome_label, evidence_source,
                evidence_note, confidence, observed_at, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7, ?8
            )
            ON CONFLICT(id) DO UPDATE SET
                outcome_label = excluded.outcome_label,
                evidence_source = excluded.evidence_source,
                evidence_note = excluded.evidence_note,
                confidence = excluded.confidence,
                observed_at = excluded.observed_at
            "#,
            params![
                event.id,
                event.attempt_id,
                event.outcome_label.as_str(),
                event.evidence_source,
                event.evidence_note,
                event.confidence.as_str(),
                event.observed_at,
                event.created_at,
            ],
        )?;

        Ok(())
    }

    /// Memperbarui status akhir/terkini dari percobaan survei
    pub fn update_attempt_outcome(
        conn: &Connection,
        attempt_id: &str,
        outcome: OutcomeLabel,
        ended_at: Option<&str>,
        updated_at: &str,
    ) -> Result<(), rusqlite::Error> {
        if let Some(ended) = ended_at {
            conn.execute(
                r#"
                UPDATE survey_attempts
                SET current_outcome = ?1,
                    ended_at = COALESCE(ended_at, ?2),
                    updated_at = ?3
                WHERE id = ?4
                "#,
                params![outcome.as_str(), ended, updated_at, attempt_id],
            )?;
        } else {
            conn.execute(
                r#"
                UPDATE survey_attempts
                SET current_outcome = ?1,
                    updated_at = ?2
                WHERE id = ?3
                "#,
                params![outcome.as_str(), updated_at, attempt_id],
            )?;
        }

        Ok(())
    }

    /// Mengambil percobaan survei berdasarkan ID
    pub fn get_attempt_by_id(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<SurveyAttempt>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, opportunity_id, persona_id, session_id,
                started_at, ended_at, current_outcome,
                created_at, updated_at
            FROM survey_attempts
            WHERE id = ?1
            "#,
        )?;

        let attempt = stmt
            .query_row(params![id], |row| {
                let outcome_str: String = row.get(6)?;
                Ok(SurveyAttempt {
                    id: row.get(0)?,
                    opportunity_id: row.get(1)?,
                    persona_id: row.get(2)?,
                    session_id: row.get(3)?,
                    started_at: row.get(4)?,
                    ended_at: row.get(5)?,
                    current_outcome: OutcomeLabel::from_str_loose(&outcome_str),
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .optional()?;

        Ok(attempt)
    }

    /// Mengambil seluruh events untuk sebuah percobaan dalam urutan kronologis
    pub fn get_events_for_attempt(
        conn: &Connection,
        attempt_id: &str,
    ) -> Result<Vec<SurveyOutcomeEvent>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, attempt_id, outcome_label, evidence_source,
                evidence_note, confidence, observed_at, created_at
            FROM survey_outcome_events
            WHERE attempt_id = ?1
            ORDER BY observed_at ASC, created_at ASC
            "#,
        )?;

        let rows = stmt.query_map(params![attempt_id], |row| {
            let label_str: String = row.get(2)?;
            let conf_str: String = row.get(5)?;
            Ok(SurveyOutcomeEvent {
                id: row.get(0)?,
                attempt_id: row.get(1)?,
                outcome_label: OutcomeLabel::from_str_loose(&label_str),
                evidence_source: row.get(3)?,
                evidence_note: row.get(4)?,
                confidence: OutcomeConfidence::from_str_loose(&conf_str),
                observed_at: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r?);
        }
        Ok(events)
    }

    /// Mengambil detail lengkap percobaan survei beserta relasi persona, peluang, kriteria, dan events
    pub fn get_attempt_detail(
        conn: &Connection,
        id: &str,
        persona_id: Option<&str>,
    ) -> Result<Option<SurveyAttemptDetail>, rusqlite::Error> {
        let sql = match persona_id {
            Some(_) => {
                r#"
                SELECT
                    a.id, a.opportunity_id, COALESCE(o.title, 'Survei Tanpa Judul'),
                    COALESCE(o.platform_label, 'Umum'), o.topic, o.description, o.raw_text,
                    a.persona_id, p.display_name, a.session_id,
                    a.started_at, a.ended_at, a.current_outcome,
                    a.created_at, a.updated_at
                FROM survey_attempts a
                JOIN personas p ON p.id = a.persona_id
                JOIN survey_opportunities o ON o.id = a.opportunity_id
                WHERE a.id = ?1 AND a.persona_id = ?2
                "#
            }
            None => {
                r#"
                SELECT
                    a.id, a.opportunity_id, COALESCE(o.title, 'Survei Tanpa Judul'),
                    COALESCE(o.platform_label, 'Umum'), o.topic, o.description, o.raw_text,
                    a.persona_id, p.display_name, a.session_id,
                    a.started_at, a.ended_at, a.current_outcome,
                    a.created_at, a.updated_at
                FROM survey_attempts a
                JOIN personas p ON p.id = a.persona_id
                JOIN survey_opportunities o ON o.id = a.opportunity_id
                WHERE a.id = ?1
                "#
            }
        };

        let mut stmt = conn.prepare(sql)?;
        let row_opt = if let Some(pid) = persona_id {
            stmt.query_row(params![id, pid], |row| Self::map_attempt_detail_row(row))
                .optional()?
        } else {
            stmt.query_row(params![id], |row| Self::map_attempt_detail_row(row))
                .optional()?
        };

        if let Some((mut detail, opp_id, att_id)) = row_opt {
            let criteria = OpportunityRepository::get_criteria_by_opportunity_id(conn, &opp_id)?;
            let events = Self::get_events_for_attempt(conn, &att_id)?;
            detail.criteria = criteria;
            detail.events = events;
            Ok(Some(detail))
        } else {
            Ok(None)
        }
    }

    /// Membaca satu baris hasil query detail
    fn map_attempt_detail_row(
        row: &rusqlite::Row,
    ) -> Result<(SurveyAttemptDetail, String, String), rusqlite::Error> {
        let att_id: String = row.get(0)?;
        let opp_id: String = row.get(1)?;
        let title: String = row.get(2)?;
        let platform: String = row.get(3)?;
        let category: Option<String> = row.get(4)?;
        let target_url: Option<String> = row.get(5)?;
        let raw_text: Option<String> = row.get(6)?;
        let persona_id: String = row.get(7)?;
        let persona_name: String = row.get(8)?;
        let session_id: Option<String> = row.get(9)?;
        let started_at: String = row.get(10)?;
        let ended_at: Option<String> = row.get(11)?;
        let outcome_str: String = row.get(12)?;
        let created_at: String = row.get(13)?;
        let updated_at: String = row.get(14)?;

        let detail = SurveyAttemptDetail {
            id: att_id.clone(),
            opportunity_id: opp_id.clone(),
            opportunity_title: title,
            source_platform: platform,
            category,
            target_url,
            raw_text,
            persona_id,
            persona_name,
            session_id,
            started_at,
            ended_at,
            current_outcome: OutcomeLabel::from_str_loose(&outcome_str),
            criteria: Vec::new(),
            events: Vec::new(),
            created_at,
            updated_at,
        };

        Ok((detail, opp_id, att_id))
    }

    /// Mengambil daftar percobaan survei berdasarkan kriteria filter dinamis
    pub fn list_attempts(
        conn: &Connection,
        filter: &AttemptFilter,
    ) -> Result<Vec<SurveyAttemptDetail>, rusqlite::Error> {
        let mut query = String::from(
            r#"
            SELECT
                a.id, a.opportunity_id, COALESCE(o.title, 'Survei Tanpa Judul'),
                COALESCE(o.platform_label, 'Umum'), o.topic, o.description, o.raw_text,
                a.persona_id, p.display_name, a.session_id,
                a.started_at, a.ended_at, a.current_outcome,
                a.created_at, a.updated_at
            FROM survey_attempts a
            JOIN personas p ON p.id = a.persona_id
            JOIN survey_opportunities o ON o.id = a.opportunity_id
            WHERE 1=1
            "#,
        );

        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref pid) = filter.persona_id {
            query.push_str(" AND a.persona_id = ?");
            param_values.push(Box::new(pid.clone()));
        }

        if let Some(ref cat) = filter.category {
            query.push_str(" AND LOWER(COALESCE(o.topic, '')) = LOWER(?)");
            param_values.push(Box::new(cat.clone()));
        }

        if let Some(ref plat) = filter.source_platform {
            query.push_str(" AND LOWER(COALESCE(o.platform_label, '')) = LOWER(?)");
            param_values.push(Box::new(plat.clone()));
        }

        if let Some(status) = filter.status {
            query.push_str(" AND a.current_outcome = ?");
            param_values.push(Box::new(status.as_str().to_string()));
        }

        if let Some(ref from) = filter.from_date {
            query.push_str(" AND a.started_at >= ?");
            param_values.push(Box::new(from.clone()));
        }

        if let Some(ref to) = filter.to_date {
            query.push_str(" AND a.started_at <= ?");
            param_values.push(Box::new(to.clone()));
        }

        query.push_str(" ORDER BY a.started_at DESC, a.created_at DESC");

        let mut stmt = conn.prepare(&query)?;
        let params_slice: Vec<&dyn rusqlite::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();

        let rows = stmt.query_map(params_slice.as_slice(), |row| {
            Self::map_attempt_detail_row(row)
        })?;

        let mut raw_items = Vec::new();
        for r in rows {
            raw_items.push(r?);
        }

        let mut results = Vec::new();
        for (mut detail, opp_id, att_id) in raw_items {
            let criteria = OpportunityRepository::get_criteria_by_opportunity_id(conn, &opp_id)?;
            let events = Self::get_events_for_attempt(conn, &att_id)?;
            detail.criteria = criteria;
            detail.events = events;
            results.push(detail);
        }

        Ok(results)
    }

    /// Mengambil percobaan historis untuk persona tertentu dalam urutan kronologis tertua ke terbaru
    pub fn get_historical_attempts_for_learning(
        conn: &Connection,
        persona_id: &str,
        category: Option<&str>,
        platform: Option<&str>,
    ) -> Result<Vec<SurveyAttempt>, rusqlite::Error> {
        let mut query = String::from(
            r#"
            SELECT
                a.id, a.opportunity_id, a.persona_id, a.session_id,
                a.started_at, a.ended_at, a.current_outcome,
                a.created_at, a.updated_at
            FROM survey_attempts a
            JOIN survey_opportunities o ON o.id = a.opportunity_id
            WHERE a.persona_id = ?1
            "#,
        );

        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(persona_id.to_string())];

        if let Some(cat) = category {
            if !cat.trim().is_empty() {
                query.push_str(" AND LOWER(COALESCE(o.topic, '')) = LOWER(?)");
                param_values.push(Box::new(cat.to_string()));
            }
        }

        if let Some(plat) = platform {
            if !plat.trim().is_empty() {
                query.push_str(" AND LOWER(COALESCE(o.platform_label, '')) = LOWER(?)");
                param_values.push(Box::new(plat.to_string()));
            }
        }

        // Urutan kronologis menaik untuk holdout testing
        query.push_str(" ORDER BY a.started_at ASC, a.created_at ASC");

        let mut stmt = conn.prepare(&query)?;
        let params_slice: Vec<&dyn rusqlite::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();

        let rows = stmt.query_map(params_slice.as_slice(), |row| {
            let outcome_str: String = row.get(6)?;
            Ok(SurveyAttempt {
                id: row.get(0)?,
                opportunity_id: row.get(1)?,
                persona_id: row.get(2)?,
                session_id: row.get(3)?,
                started_at: row.get(4)?,
                ended_at: row.get(5)?,
                current_outcome: OutcomeLabel::from_str_loose(&outcome_str),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut attempts = Vec::new();
        for r in rows {
            attempts.push(r?);
        }
        Ok(attempts)
    }

    /// Menghapus percobaan survei berdasarkan ID dan persona (isolasi persona)
    pub fn delete_attempt(
        conn: &Connection,
        id: &str,
        persona_id: &str,
    ) -> Result<bool, rusqlite::Error> {
        let rows_affected = conn.execute(
            "DELETE FROM survey_attempts WHERE id = ?1 AND persona_id = ?2",
            params![id, persona_id],
        )?;
        Ok(rows_affected > 0)
    }
}
