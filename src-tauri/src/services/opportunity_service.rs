use rusqlite::Connection;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::opportunity::{
    OpportunityCriterion, OpportunityDetail, OpportunityProvenance, OpportunityStatus,
    SaveOpportunityWithCriteriaInput, SurveyOpportunity, UpdateCriteriaInput,
    UpdateOpportunityInput,
};
use crate::repository::opportunity_repo::OpportunityRepository;
use crate::services::criteria_extractor::CriteriaExtractorService;

pub struct OpportunityService;

impl OpportunityService {
    /// Mengekstrak peluang survei dan kriteria dari teks mentah (pratinjau sebelum simpan)
    pub fn extract_from_text(
        raw_text: &str,
        platform_label: Option<&str>,
    ) -> Result<OpportunityDetail, AppError> {
        let trimmed = raw_text.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(
                "rawText",
                "Teks sumber peluang survei tidak boleh kosong",
            ));
        }

        let (opportunity, criteria) = CriteriaExtractorService::build_opportunity_with_criteria(
            trimmed,
            platform_label,
            OpportunityProvenance::ManualEntry,
            None,
            None,
        );

        Ok(OpportunityDetail {
            opportunity,
            criteria,
        })
    }

    /// Mengekstrak peluang survei dari hasil tangkapan layar dan OCR
    pub fn extract_from_capture(
        conn: &Connection,
        capture_id: &str,
        ocr_id: Option<&str>,
    ) -> Result<OpportunityDetail, AppError> {
        if capture_id.trim().is_empty() {
            return Err(AppError::validation(
                "captureId",
                "ID tangkapan layar tidak boleh kosong",
            ));
        }

        // Cari data teks OCR terkait
        let raw_text_opt: Option<(String, Option<String>)> = if let Some(oid) = ocr_id {
            conn.query_row(
                "SELECT raw_ocr_text, id FROM ocr_extractions WHERE id = ?1",
                rusqlite::params![oid],
                |row| Ok((row.get(0)?, Some(row.get(1)?))),
            )
            .ok()
        } else {
            conn.query_row(
                "SELECT raw_ocr_text, id FROM ocr_extractions WHERE capture_id = ?1 ORDER BY created_at DESC LIMIT 1",
                rusqlite::params![capture_id],
                |row| Ok((row.get(0)?, Some(row.get(1)?))),
            )
            .ok()
        };

        let (raw_ocr_text, found_ocr_id) = match raw_text_opt {
            Some(data) => data,
            None => {
                return Err(AppError::not_found(format!(
                    "Tidak ditemukan data ekstraksi OCR untuk capture ID '{capture_id}'"
                )));
            }
        };

        if raw_ocr_text.trim().is_empty() {
            return Err(AppError::validation(
                "rawOcrText",
                "Hasil pembacaan teks OCR kosong",
            ));
        }

        let (opportunity, criteria) = CriteriaExtractorService::build_opportunity_with_criteria(
            &raw_ocr_text,
            None,
            OpportunityProvenance::OcrDetected,
            Some(capture_id.to_string()),
            found_ocr_id,
        );

        Ok(OpportunityDetail {
            opportunity,
            criteria,
        })
    }

    /// Menyimpan peluang survei beserta seluruh kriterianya yang telah ditinjau/dikoreksi
    pub fn save_opportunity(
        conn: &mut Connection,
        input: SaveOpportunityWithCriteriaInput,
    ) -> Result<OpportunityDetail, AppError> {
        let trimmed_text = input.opportunity.raw_text.trim();
        if trimmed_text.is_empty() {
            return Err(AppError::validation(
                "rawText",
                "Teks sumber peluang survei wajib diisi",
            ));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let opp_id = Uuid::new_v4().to_string();

        let opp = SurveyOpportunity {
            id: opp_id.clone(),
            title: input.opportunity.title.filter(|t| !t.trim().is_empty()),
            platform_label: input
                .opportunity
                .platform_label
                .filter(|p| !p.trim().is_empty()),
            topic: input.opportunity.topic.filter(|t| !t.trim().is_empty()),
            description: input
                .opportunity
                .description
                .filter(|d| !d.trim().is_empty()),
            raw_text: Some(trimmed_text.to_string()),
            provenance: input
                .opportunity
                .provenance
                .unwrap_or(OpportunityProvenance::ManualEntry),
            status: OpportunityStatus::Confirmed,
            source_capture_id: input.opportunity.source_capture_id,
            source_ocr_id: input.opportunity.source_ocr_id,
            captured_at: input.opportunity.captured_at.unwrap_or_else(|| now.clone()),
            created_at: now.clone(),
            updated_at: now,
        };

        let criteria: Vec<OpportunityCriterion> = input
            .criteria
            .into_iter()
            .enumerate()
            .map(|(i, c)| OpportunityCriterion {
                id: c.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                opportunity_id: opp_id.clone(),
                criterion_text: c.criterion_text,
                raw_text: c.raw_text,
                criterion_type: c.criterion_type,
                normalized_attribute: c.normalized_attribute,
                expected_value_json: c.expected_value_json,
                operator: c.operator,
                criterion_order: i as i64,
            })
            .collect();

        OpportunityRepository::save_opportunity_with_criteria(conn, &opp, &criteria)
            .map_err(|e| AppError::database(format!("Gagal menyimpan peluang survei: {e}")))?;

        Ok(OpportunityDetail {
            opportunity: opp,
            criteria,
        })
    }

    /// Mengambil detail peluang survei berdasarkan ID
    pub fn get_opportunity(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<OpportunityDetail>, AppError> {
        OpportunityRepository::get_opportunity_detail(conn, id)
            .map_err(|e| AppError::database(format!("Gagal mengambil peluang survei: {e}")))
    }

    /// Mengambil daftar peluang survei terurut
    pub fn list_opportunities(
        conn: &Connection,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<SurveyOpportunity>, AppError> {
        let l = limit.unwrap_or(50).clamp(1, 200);
        let o = offset.unwrap_or(0).max(0);

        OpportunityRepository::list_opportunities(conn, l, o)
            .map_err(|e| AppError::database(format!("Gagal mengambil daftar peluang survei: {e}")))
    }

    /// Memperbarui metadata peluang survei
    pub fn update_opportunity(
        conn: &Connection,
        input: UpdateOpportunityInput,
    ) -> Result<SurveyOpportunity, AppError> {
        let existing = OpportunityRepository::get_opportunity_by_id(conn, &input.id)
            .map_err(|e| AppError::database(format!("Gagal memeriksa peluang survei: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Peluang survei dengan ID '{}' tidak ditemukan",
                    input.id
                ))
            })?;

        let now = chrono::Utc::now().to_rfc3339();
        let updated = SurveyOpportunity {
            id: existing.id,
            title: input.title.or(existing.title),
            platform_label: input.platform_label.or(existing.platform_label),
            topic: input.topic.or(existing.topic),
            description: input.description.or(existing.description),
            raw_text: existing.raw_text,
            provenance: existing.provenance,
            status: input.status.unwrap_or(existing.status),
            source_capture_id: existing.source_capture_id,
            source_ocr_id: existing.source_ocr_id,
            captured_at: existing.captured_at,
            created_at: existing.created_at,
            updated_at: now,
        };

        OpportunityRepository::save_opportunity(conn, &updated)
            .map_err(|e| AppError::database(format!("Gagal memperbarui peluang survei: {e}")))?;

        Ok(updated)
    }

    /// Memperbarui kriteria kelayakan peluang survei
    pub fn update_criteria(
        conn: &mut Connection,
        input: UpdateCriteriaInput,
    ) -> Result<Vec<OpportunityCriterion>, AppError> {
        let _existing = OpportunityRepository::get_opportunity_by_id(conn, &input.opportunity_id)
            .map_err(|e| AppError::database(format!("Gagal memeriksa peluang survei: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Peluang survei dengan ID '{}' tidak ditemukan",
                    input.opportunity_id
                ))
            })?;

        let criteria: Vec<OpportunityCriterion> = input
            .criteria
            .into_iter()
            .enumerate()
            .map(|(i, c)| OpportunityCriterion {
                id: c.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                opportunity_id: input.opportunity_id.clone(),
                criterion_text: c.criterion_text,
                raw_text: c.raw_text,
                criterion_type: c.criterion_type,
                normalized_attribute: c.normalized_attribute,
                expected_value_json: c.expected_value_json,
                operator: c.operator,
                criterion_order: i as i64,
            })
            .collect();

        let tx = conn
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal membuka transaksi: {e}")))?;

        OpportunityRepository::delete_criteria_by_opportunity_id(&tx, &input.opportunity_id)
            .map_err(|e| AppError::database(format!("Gagal membersihkan kriteria lama: {e}")))?;

        OpportunityRepository::save_criteria(&tx, &criteria)
            .map_err(|e| AppError::database(format!("Gagal menyimpan kriteria baru: {e}")))?;

        tx.commit()
            .map_err(|e| AppError::database(format!("Gagal menerapkan perubahan kriteria: {e}")))?;

        Ok(criteria)
    }

    /// Menghapus peluang survei beserta seluruh kriterianya
    pub fn delete_opportunity(conn: &Connection, id: &str) -> Result<bool, AppError> {
        OpportunityRepository::delete_opportunity(conn, id)
            .map_err(|e| AppError::database(format!("Gagal menghapus peluang survei: {e}")))
    }
}
