use rusqlite::Connection;

use crate::domain::error::AppError;
use crate::domain::fit::{
    FitCriterionResult, MatchState, OverallFitState, PersonaFitComparisonItem,
    PersonaOpportunityFitDetail,
};
use crate::domain::opportunity::OpportunityDetail;
use crate::repository::fact_repo::FactRepository;
use crate::repository::fit_repo::FitRepository;
use crate::repository::opportunity_repo::OpportunityRepository;
use crate::repository::persona_repo::PersonaRepository;
use crate::services::fit_evaluator::FitEvaluator;

pub struct FitService;

impl FitService {
    /// Mengevaluasi kesesuaian kriteria peluang survei terhadap profil persona secara deterministik (SC-051)
    pub fn evaluate_opportunity_fit(
        conn: &mut Connection,
        opportunity_id: &str,
        persona_id: &str,
    ) -> Result<PersonaOpportunityFitDetail, AppError> {
        let opp_id_trimmed = opportunity_id.trim();
        let persona_id_trimmed = persona_id.trim();

        if opp_id_trimmed.is_empty() {
            return Err(AppError::validation(
                "opportunityId",
                "ID peluang survei tidak boleh kosong",
            ));
        }
        if persona_id_trimmed.is_empty() {
            return Err(AppError::validation(
                "personaId",
                "ID persona tidak boleh kosong",
            ));
        }

        // 1. Verifikasi persona
        let persona = PersonaRepository::find_by_id(conn, persona_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil data persona: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!("Persona dengan ID '{persona_id_trimmed}' tidak ditemukan"))
            })?;

        // 2. Verifikasi peluang survei dan ambil seluruh kriterianya
        let opp_detail: OpportunityDetail = OpportunityRepository::get_opportunity_detail(conn, opp_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil detail peluang survei: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Peluang survei dengan ID '{opp_id_trimmed}' tidak ditemukan"
                ))
            })?;

        // 3. Ambil fakta profil terkonfirmasi persona
        let now = chrono::Utc::now().to_rfc3339();
        let facts = FactRepository::list_by_persona(conn, persona_id_trimmed, false, &now)
            .map_err(|e| AppError::database(format!("Gagal mengambil fakta persona: {e}")))?;

        // 4. Evaluasi deterministik murni di Rust (tanpa model inference)
        let criteria_results = FitEvaluator::evaluate_criteria(&opp_detail.criteria, &facts);
        let overall_fit = FitEvaluator::compute_overall_fit(&criteria_results);

        // 5. Simpan hasil evaluasi ke tabel survey_fit_results
        FitRepository::save_fit_results(
            conn,
            opp_id_trimmed,
            persona_id_trimmed,
            &criteria_results,
            &now,
        )
        .map_err(|e| AppError::database(format!("Gagal menyimpan hasil kesesuaian survei: {e}")))?;

        // 6. Hitung rekap metrik
        let match_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Match)
            .count();
        let mismatch_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Mismatch)
            .count();
        let unknown_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Unknown)
            .count();
        let stale_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Stale)
            .count();
        let conflict_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Conflict)
            .count();
        let total_criteria = criteria_results.len();

        Ok(PersonaOpportunityFitDetail {
            opportunity_id: opp_id_trimmed.to_string(),
            persona_id: persona_id_trimmed.to_string(),
            persona_name: persona.display_name,
            overall_fit,
            match_count,
            mismatch_count,
            unknown_count,
            stale_count,
            conflict_count,
            total_criteria,
            criteria_results,
            evaluated_at: now,
        })
    }

    /// Mengambil hasil evaluasi yang tersimpan untuk peluang survei dan persona tertentu
    pub fn get_opportunity_fit(
        conn: &Connection,
        opportunity_id: &str,
        persona_id: &str,
    ) -> Result<Option<PersonaOpportunityFitDetail>, AppError> {
        let opp_id_trimmed = opportunity_id.trim();
        let persona_id_trimmed = persona_id.trim();

        let persona = match PersonaRepository::find_by_id(conn, persona_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil data persona: {e}")))?
        {
            Some(p) => p,
            None => return Ok(None),
        };

        let opp_detail = match OpportunityRepository::get_opportunity_detail(conn, opp_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil data peluang: {e}")))?
        {
            Some(od) => od,
            None => return Ok(None),
        };

        let records = FitRepository::get_fit_results(conn, opp_id_trimmed, persona_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil catatan kesesuaian: {e}")))?;

        if records.is_empty() {
            return Ok(None);
        }

        let evaluated_at = records[0].evaluated_at.clone();

        // Rekonstruksi FitCriterionResult dengan menggabungkan data kriteria asli
        let mut criteria_results = Vec::new();
        for rec in &records {
            let crit_opt = opp_detail
                .criteria
                .iter()
                .find(|c| c.id == rec.criterion_id);

            let supporting_fact_ids: Vec<String> =
                serde_json::from_str(&rec.supporting_fact_ids_json).unwrap_or_default();

            if let Some(crit) = crit_opt {
                criteria_results.push(FitCriterionResult {
                    criterion_id: rec.criterion_id.clone(),
                    criterion_text: crit.criterion_text.clone(),
                    raw_text: crit.raw_text.clone(),
                    criterion_type: crit.criterion_type,
                    operator: crit.operator,
                    match_state: rec.match_state,
                    supporting_fact_ids,
                    supporting_facts_summary: vec![],
                    explanation: rec.explanation.clone(),
                });
            }
        }

        let overall_fit = FitEvaluator::compute_overall_fit(&criteria_results);
        let match_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Match)
            .count();
        let mismatch_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Mismatch)
            .count();
        let unknown_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Unknown)
            .count();
        let stale_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Stale)
            .count();
        let conflict_count = criteria_results
            .iter()
            .filter(|r| r.match_state == MatchState::Conflict)
            .count();
        let total_criteria = criteria_results.len();

        Ok(Some(PersonaOpportunityFitDetail {
            opportunity_id: opp_id_trimmed.to_string(),
            persona_id: persona_id_trimmed.to_string(),
            persona_name: persona.display_name,
            overall_fit,
            match_count,
            mismatch_count,
            unknown_count,
            stale_count,
            conflict_count,
            total_criteria,
            criteria_results,
            evaluated_at,
        }))
    }

    /// Membandingkan kesesuaian peluang survei lintas seluruh persona nyata untuk truthful respondent selection
    pub fn compare_personas_fit(
        conn: &mut Connection,
        opportunity_id: &str,
    ) -> Result<Vec<PersonaFitComparisonItem>, AppError> {
        let opp_id_trimmed = opportunity_id.trim();
        if opp_id_trimmed.is_empty() {
            return Err(AppError::validation(
                "opportunityId",
                "ID peluang survei tidak boleh kosong",
            ));
        }

        // Pastikan peluang survei ada
        let _opp = OpportunityRepository::get_opportunity_by_id(conn, opp_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil peluang survei: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Peluang survei dengan ID '{opp_id_trimmed}' tidak ditemukan"
                ))
            })?;

        // Ambil seluruh persona yang tidak diarsipkan
        let personas = PersonaRepository::list_all(conn, false)
            .map_err(|e| AppError::database(format!("Gagal mengambil daftar persona: {e}")))?;

        let mut comparison_items = Vec::new();

        for persona in personas {
            // Evaluasi fit untuk setiap persona agar selalu berbasis fakta terbaru
            let fit_detail = match Self::evaluate_opportunity_fit(conn, opp_id_trimmed, &persona.id)
            {
                Ok(detail) => detail,
                Err(_) => continue,
            };

            let mut key_reasons = Vec::new();
            match fit_detail.overall_fit {
                OverallFitState::Eligible => {
                    key_reasons.push(format!(
                        "Memenuhi seluruh {} kriteria yang disyaratkan",
                        fit_detail.total_criteria
                    ));
                }
                OverallFitState::Ineligible => {
                    for r in fit_detail
                        .criteria_results
                        .iter()
                        .filter(|r| r.match_state == MatchState::Mismatch)
                        .take(2)
                    {
                        key_reasons.push(r.explanation.clone());
                    }
                }
                OverallFitState::Incomplete => {
                    for r in fit_detail
                        .criteria_results
                        .iter()
                        .filter(|r| r.match_state == MatchState::Unknown)
                        .take(2)
                    {
                        key_reasons.push(r.explanation.clone());
                    }
                }
                OverallFitState::Stale => {
                    for r in fit_detail
                        .criteria_results
                        .iter()
                        .filter(|r| r.match_state == MatchState::Stale)
                        .take(2)
                    {
                        key_reasons.push(r.explanation.clone());
                    }
                }
                OverallFitState::Conflict => {
                    for r in fit_detail
                        .criteria_results
                        .iter()
                        .filter(|r| r.match_state == MatchState::Conflict)
                        .take(2)
                    {
                        key_reasons.push(r.explanation.clone());
                    }
                }
                OverallFitState::NoCriteria => {
                    key_reasons.push("Tidak ada kriteria eksplisit tertangkap".to_string());
                }
            }

            comparison_items.push(PersonaFitComparisonItem {
                persona_id: persona.id,
                persona_name: persona.display_name,
                overall_fit: fit_detail.overall_fit,
                match_count: fit_detail.match_count,
                mismatch_count: fit_detail.mismatch_count,
                unknown_count: fit_detail.unknown_count,
                stale_count: fit_detail.stale_count,
                conflict_count: fit_detail.conflict_count,
                total_criteria: fit_detail.total_criteria,
                key_reasons,
                evaluated_at: Some(fit_detail.evaluated_at),
            });
        }

        // Urutkan: Eligible paling atas, diikuti Incomplete, Stale, Conflict, Ineligible
        comparison_items.sort_by_key(|item| match item.overall_fit {
            OverallFitState::Eligible => 0,
            OverallFitState::Incomplete => 1,
            OverallFitState::NoCriteria => 2,
            OverallFitState::Stale => 3,
            OverallFitState::Conflict => 4,
            OverallFitState::Ineligible => 5,
        });

        Ok(comparison_items)
    }
}
