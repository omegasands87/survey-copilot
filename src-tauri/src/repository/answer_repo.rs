use rusqlite::{params, Connection};
use serde_json::json;

use crate::domain::answer_engine::{
    AnswerHistoryItem, EvidenceStatus, ReviewAction, ValidatedAnswerRecommendation,
};
use crate::domain::error::AppError;
use crate::domain::evidence::{EvidenceConfidence, RetrievedEvidenceItem, ValidityStatus};
use crate::domain::fact::{Sensitivity, VerificationStatus};

pub struct AnswerRepository;

impl AnswerRepository {
    /// Menyimpan hasil rekomendasi jawaban AI beserta sumber-sumber buktinya
    /// Dilakukan dalam transaksi atomik dengan penegakan kepemilikan persona mutlak.
    pub fn save_recommendation(
        conn: &Connection,
        rec: &ValidatedAnswerRecommendation,
    ) -> Result<(), AppError> {
        let question_id = match rec.question_id.as_deref() {
            Some(qid) if !qid.trim().is_empty() => qid.trim(),
            _ => return Ok(()), // Jika pertanyaan ad-hoc/tanpa question_id di database, skip persistensi tabel relasional
        };

        let tx = conn
            .unchecked_transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        // 1. Simpan rekaman answer_recommendations
        tx.execute(
            "INSERT INTO answer_recommendations (
                id, question_id, persona_id, status, draft_text, user_answer, review_status,
                review_reason, reviewed_at, interpretation, rationale, missing_information_json,
                conflicts_json, assumptions_json, clarifying_question, confidence,
                rejected_citations_json, provider_config_id, model_id, prompt_version, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            params![
                rec.id,
                question_id,
                rec.persona_id,
                rec.status.as_str(),
                rec.draft_answer,
                rec.user_answer,
                rec.review_status.as_deref().unwrap_or("draft"),
                rec.review_reason,
                rec.reviewed_at,
                rec.interpretation,
                rec.rationale,
                json!(rec.missing_information).to_string(),
                json!(rec.conflicts).to_string(),
                json!(rec.assumptions).to_string(),
                rec.clarifying_question,
                rec.confidence.as_str(),
                json!(rec.rejected_citations).to_string(),
                rec.provider_config_id,
                rec.model_id,
                rec.prompt_version,
                rec.created_at,
            ],
        )
        .map_err(|e| AppError::database(format!("Gagal menyimpan answer_recommendations: {e}")))?;

        // 2. Simpan sumber rujukan bukti ke recommendation_sources
        // Hanya fakta yang lolos allowlist dan terbukti milik persona aktif yang disimpan
        for fact in &rec.supporting_facts {
            tx.execute(
                "INSERT OR IGNORE INTO recommendation_sources (
                    recommendation_id, fact_id, persona_id, support_note
                ) VALUES (?1, ?2, ?3, ?4)",
                params![rec.id, fact.fact_id, rec.persona_id, fact.selection_reason,],
            )
            .map_err(|e| {
                AppError::database(format!("Gagal menyimpan recommendation_sources: {e}"))
            })?;
        }

        tx.commit()
            .map_err(|e| AppError::database(format!("Gagal commit transaksi: {e}")))?;

        Ok(())
    }

    /// Mengambil rekomendasi jawaban terbaru untuk pertanyaan tertentu dengan isolasi persona ketat
    pub fn get_latest_recommendation_by_question(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
    ) -> Result<Option<ValidatedAnswerRecommendation>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, question_id, persona_id, status, draft_text, user_answer, review_status,
                        review_reason, reviewed_at, interpretation, rationale, missing_information_json,
                        conflicts_json, assumptions_json, clarifying_question, confidence,
                        rejected_citations_json, provider_config_id, model_id, prompt_version, created_at
                 FROM answer_recommendations
                 WHERE question_id = ?1 AND persona_id = ?2
                 ORDER BY created_at DESC
                 LIMIT 1",
            )
            .map_err(|e| AppError::database(format!("Gagal menyiapkan query: {e}")))?;

        let mut rows = stmt
            .query(params![question_id, persona_id])
            .map_err(|e| AppError::database(format!("Gagal query rekomendasi: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| AppError::database(format!("Gagal membaca row rekomendasi: {e}")))?
        {
            Ok(Some(Self::map_row_to_recommendation(conn, row)?))
        } else {
            Ok(None)
        }
    }

    /// Mengambil satu rekomendasi berdasarkan ID dengan isolasi persona ketat
    pub fn get_recommendation_by_id(
        conn: &Connection,
        persona_id: &str,
        recommendation_id: &str,
    ) -> Result<Option<ValidatedAnswerRecommendation>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, question_id, persona_id, status, draft_text, user_answer, review_status,
                        review_reason, reviewed_at, interpretation, rationale, missing_information_json,
                        conflicts_json, assumptions_json, clarifying_question, confidence,
                        rejected_citations_json, provider_config_id, model_id, prompt_version, created_at
                 FROM answer_recommendations
                 WHERE id = ?1 AND persona_id = ?2
                 LIMIT 1",
            )
            .map_err(|e| AppError::database(format!("Gagal menyiapkan query get by id: {e}")))?;

        let mut rows = stmt
            .query(params![recommendation_id, persona_id])
            .map_err(|e| AppError::database(format!("Gagal query rekomendasi by id: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| AppError::database(format!("Gagal membaca row rekomendasi: {e}")))?
        {
            Ok(Some(Self::map_row_to_recommendation(conn, row)?))
        } else {
            Ok(None)
        }
    }

    /// Mengambil seluruh riwayat revisi rekomendasi untuk satu pertanyaan dengan isolasi persona ketat
    pub fn list_recommendations_by_question(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
    ) -> Result<Vec<ValidatedAnswerRecommendation>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, question_id, persona_id, status, draft_text, user_answer, review_status,
                        review_reason, reviewed_at, interpretation, rationale, missing_information_json,
                        conflicts_json, assumptions_json, clarifying_question, confidence,
                        rejected_citations_json, provider_config_id, model_id, prompt_version, created_at
                 FROM answer_recommendations
                 WHERE question_id = ?1 AND persona_id = ?2
                 ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::database(format!("Gagal menyiapkan query list: {e}")))?;

        let mut rows = stmt
            .query(params![question_id, persona_id])
            .map_err(|e| AppError::database(format!("Gagal query list rekomendasi: {e}")))?;

        let mut recs = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| AppError::database(format!("Gagal baca row list: {e}")))?
        {
            recs.push(Self::map_row_to_recommendation(conn, row)?);
        }

        Ok(recs)
    }

    /// Memperbarui status peninjauan dan jawaban aktual pengguna pada rekaman rekomendasi
    pub fn update_recommendation_review(
        conn: &Connection,
        persona_id: &str,
        recommendation_id: &str,
        user_answer: Option<&str>,
        review_status: &str,
        review_reason: Option<&str>,
        reviewed_at: &str,
    ) -> Result<bool, AppError> {
        let rows_affected = conn
            .execute(
                "UPDATE answer_recommendations
                 SET user_answer = ?1, review_status = ?2, review_reason = ?3, reviewed_at = ?4
                 WHERE id = ?5 AND persona_id = ?6",
                params![
                    user_answer,
                    review_status,
                    review_reason,
                    reviewed_at,
                    recommendation_id,
                    persona_id
                ],
            )
            .map_err(|e| {
                AppError::database(format!(
                    "Gagal memperbarui review answer_recommendations: {e}"
                ))
            })?;

        Ok(rows_affected > 0)
    }

    /// Menyimpan rekaman riwayat perubahan jawaban survei (SC-034)
    pub fn save_history(conn: &Connection, item: &AnswerHistoryItem) -> Result<(), AppError> {
        conn.execute(
            "INSERT INTO answer_history (
                id, persona_id, question_id, recommendation_id, action, draft_answer,
                actual_answer, review_status, provider_id, model_id, prompt_version,
                evidence_sources_json, reason, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                item.id,
                item.persona_id,
                item.question_id,
                item.recommendation_id,
                item.action.as_str(),
                item.draft_answer,
                item.actual_answer,
                item.review_status,
                item.provider_id,
                item.model_id,
                item.prompt_version,
                item.evidence_sources_json,
                item.reason,
                item.created_at,
            ],
        )
        .map_err(|e| AppError::database(format!("Gagal menyimpan answer_history: {e}")))?;

        Ok(())
    }

    /// Mengambil daftar riwayat perubahan jawaban survei dengan isolasi persona ketat
    pub fn list_history(
        conn: &Connection,
        persona_id: &str,
        question_id: Option<&str>,
        action: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<AnswerHistoryItem>, AppError> {
        let mut query = String::from(
            "SELECT id, persona_id, question_id, recommendation_id, action, draft_answer,
                    actual_answer, review_status, provider_id, model_id, prompt_version,
                    evidence_sources_json, reason, created_at
             FROM answer_history
             WHERE persona_id = ?1",
        );

        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(persona_id.to_string())];

        if let Some(qid) = question_id {
            if !qid.trim().is_empty() {
                param_values.push(Box::new(qid.trim().to_string()));
                query.push_str(&format!(" AND question_id = ?{}", param_values.len()));
            }
        }

        if let Some(act) = action {
            if !act.trim().is_empty() {
                param_values.push(Box::new(act.trim().to_string()));
                query.push_str(&format!(" AND action = ?{}", param_values.len()));
            }
        }

        query.push_str(" ORDER BY created_at DESC");

        let lim = limit.unwrap_or(100).min(500);
        param_values.push(Box::new(lim));
        query.push_str(&format!(" LIMIT ?{}", param_values.len()));

        if let Some(off) = offset {
            param_values.push(Box::new(off));
            query.push_str(&format!(" OFFSET ?{}", param_values.len()));
        }

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| AppError::database(format!("Gagal menyiapkan query riwayat: {e}")))?;

        let params_slice: Vec<&dyn rusqlite::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();

        let rows = stmt
            .query_map(params_slice.as_slice(), |row| {
                let id: String = row.get(0)?;
                let persona_id: String = row.get(1)?;
                let question_id: String = row.get(2)?;
                let recommendation_id: Option<String> = row.get(3)?;
                let action_str: String = row.get(4)?;
                let draft_answer: Option<String> = row.get(5)?;
                let actual_answer: String = row.get(6)?;
                let review_status: String = row.get(7)?;
                let provider_id: Option<String> = row.get(8)?;
                let model_id: Option<String> = row.get(9)?;
                let prompt_version: Option<String> = row.get(10)?;
                let evidence_sources_json: String = row.get(11)?;
                let reason: Option<String> = row.get(12)?;
                let created_at: String = row.get(13)?;

                Ok(AnswerHistoryItem {
                    id,
                    persona_id,
                    question_id,
                    recommendation_id,
                    action: ReviewAction::from_str_loose(&action_str),
                    draft_answer,
                    actual_answer,
                    review_status,
                    provider_id,
                    model_id,
                    prompt_version,
                    evidence_sources_json,
                    reason,
                    created_at,
                })
            })
            .map_err(|e| AppError::database(format!("Gagal membaca query riwayat: {e}")))?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(|e| AppError::database(format!("Error row riwayat: {e}")))?);
        }

        Ok(items)
    }

    /// Helper privat untuk mapping satu row SQLite ke ValidatedAnswerRecommendation
    fn map_row_to_recommendation(
        conn: &Connection,
        row: &rusqlite::Row,
    ) -> Result<ValidatedAnswerRecommendation, AppError> {
        let rec_id: String = row.get(0)?;
        let q_id: String = row.get(1)?;
        let p_id: String = row.get(2)?;
        let status_str: String = row.get(3)?;
        let draft_answer: Option<String> = row.get(4)?;
        let user_answer: Option<String> = row.get(5)?;
        let review_status: Option<String> = row.get(6)?;
        let review_reason: Option<String> = row.get(7)?;
        let reviewed_at: Option<String> = row.get(8)?;
        let interpretation: String = row.get(9)?;
        let rationale: Option<String> = row.get(10)?;
        let missing_info_raw: String = row.get(11)?;
        let conflicts_raw: String = row.get(12)?;
        let assumptions_raw: String = row.get(13)?;
        let clarifying_question: Option<String> = row.get(14)?;
        let confidence_str: String = row.get(15)?;
        let rejected_raw: String = row.get(16)?;
        let provider_config_id: Option<String> = row.get(17)?;
        let model_id: Option<String> = row.get(18)?;
        let prompt_version: Option<String> = row.get(19)?;
        let created_at: String = row.get(20)?;

        let missing_information: Vec<String> =
            serde_json::from_str(&missing_info_raw).unwrap_or_default();
        let conflicts: Vec<String> = serde_json::from_str(&conflicts_raw).unwrap_or_default();
        let assumptions: Vec<String> = serde_json::from_str(&assumptions_raw).unwrap_or_default();
        let rejected_citations: Vec<String> =
            serde_json::from_str(&rejected_raw).unwrap_or_default();

        // Ambil bukti rujukan terhubung
        let supporting_facts = Self::list_sources_for_recommendation(conn, &rec_id, &p_id)?;

        let confidence = match confidence_str.as_str() {
            "high" => EvidenceConfidence::High,
            "medium" => EvidenceConfidence::Medium,
            "low" => EvidenceConfidence::Low,
            _ => EvidenceConfidence::Uncertain,
        };

        Ok(ValidatedAnswerRecommendation {
            id: rec_id,
            question_id: Some(q_id),
            session_id: None,
            persona_id: p_id,
            status: EvidenceStatus::from_str_loose(&status_str),
            question_intent: "survey_question".to_string(),
            answer_format: "standard".to_string(),
            interpretation,
            draft_answer,
            user_answer,
            review_status,
            review_reason,
            reviewed_at,
            supporting_facts,
            rejected_citations,
            rationale,
            missing_information,
            conflicts,
            assumptions,
            clarifying_question,
            confidence,
            prompt_version: prompt_version.unwrap_or_else(|| "1.0".to_string()),
            provider_config_id,
            model_id,
            created_at,
        })
    }

    /// Mengambil daftar fakta pendukung untuk rekomendasi dengan isolasi persona ketat
    fn list_sources_for_recommendation(
        conn: &Connection,
        recommendation_id: &str,
        persona_id: &str,
    ) -> Result<Vec<RetrievedEvidenceItem>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT pf.id, pf.attribute, pf.category, pf.value_json, pf.normalized_value,
                        pf.verification_status, pf.sensitivity, pf.valid_from, pf.valid_until,
                        rs.support_note
                 FROM recommendation_sources rs
                 JOIN profile_facts pf ON pf.id = rs.fact_id AND pf.persona_id = rs.persona_id
                 WHERE rs.recommendation_id = ?1 AND rs.persona_id = ?2",
            )
            .map_err(|e| AppError::database(format!("Gagal menyiapkan query sources: {e}")))?;

        let rows = stmt
            .query_map(params![recommendation_id, persona_id], |row| {
                let fact_id: String = row.get(0)?;
                let attribute: String = row.get(1)?;
                let category: String = row.get(2)?;
                let val_json: String = row.get(3)?;
                let normalized_value: String = row.get(4)?;
                let ver_status_str: String = row.get(5)?;
                let sens_str: String = row.get(6)?;
                let valid_from: Option<String> = row.get(7)?;
                let valid_until: Option<String> = row.get(8)?;
                let support_note: Option<String> = row.get(9)?;

                let value: serde_json::Value =
                    serde_json::from_str(&val_json).unwrap_or(serde_json::Value::Null);

                let verification_status = match ver_status_str.as_str() {
                    "confirmed" => VerificationStatus::Confirmed,
                    "disputed" => VerificationStatus::Disputed,
                    "outdated" => VerificationStatus::Outdated,
                    _ => VerificationStatus::Unverified,
                };

                let sensitivity = match sens_str.as_str() {
                    "sensitive" => Sensitivity::Sensitive,
                    "highly_sensitive" => Sensitivity::HighlySensitive,
                    _ => Sensitivity::Ordinary,
                };

                Ok(RetrievedEvidenceItem {
                    fact_id: fact_id.clone(),
                    source_id: None,
                    fact_key: format!("{}:{}", category, attribute),
                    attribute,
                    category,
                    value,
                    normalized_value,
                    unit: None,
                    verification_status,
                    validity_status: ValidityStatus::Valid,
                    sensitivity,
                    relevance_score: 1.0,
                    selection_reason: support_note
                        .unwrap_or_else(|| "Sumber terverifikasi".to_string()),
                    source_type: Some("profile_fact".to_string()),
                    source_title: None,
                    source_description: None,
                    source_date: None,
                    valid_from,
                    valid_until,
                })
            })
            .map_err(|e| AppError::database(format!("Gagal membaca query sources: {e}")))?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(|e| AppError::database(format!("Error row source: {e}")))?);
        }

        Ok(items)
    }
}
