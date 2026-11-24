use chrono::Utc;
use rusqlite::Connection;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::survey::{
    compute_question_fingerprint, NextQuestionIntelligenceResult, NextQuestionPrediction,
    PredictNextQuestionInput, PredictionStatus, SurveyBranchTransition,
    SurveyQuestion, SurveyQuestionPredictionRecord, TransitionType,
};
use crate::repository::{PersonaRepository, SurveyRepository};

pub struct NextQuestionService;

impl NextQuestionService {
    // ==========================================
    // NEXT QUESTION INTELLIGENCE ENGINE (SC-054)
    // ==========================================

    /// Memprediksi kemungkinan pertanyaan berikutnya berdasarkan riwayat transisi percabangan nyata (AT-065)
    pub fn predict_next_questions(
        conn: &Connection,
        input: &PredictNextQuestionInput,
    ) -> Result<NextQuestionIntelligenceResult, AppError> {
        // 1. Validasi keberadaan persona aktif (Isolasi Persona)
        PersonaRepository::find_by_id(conn, &input.persona_id)?
            .ok_or_else(|| AppError::not_found("Profil responden aktif tidak ditemukan"))?;

        // 2. Ambil sesi survei
        let session = SurveyRepository::get_session(conn, &input.persona_id, &input.session_id)?
            .ok_or_else(|| {
                AppError::not_found("Sesi survei tidak ditemukan atau bukan milik profil aktif ini")
            })?;

        // 3. Ambil pertanyaan saat ini
        let current_question = SurveyRepository::get_question(conn, &input.persona_id, &input.current_question_id)?
            .ok_or_else(|| {
                AppError::not_found("Pertanyaan survei aktif tidak ditemukan pada sesi ini")
            })?;

        let now = Utc::now().to_rfc3339();

        // 4. Sinkronkan transisi yang belum tercatat pada sesi ini untuk memperkaya data observasi nyata
        let _ = Self::sync_session_transitions(conn, &input.persona_id, &input.session_id);

        // 5. Tentukan jawaban aktif saat ini (prioritas: input manual/draft/confirmed)
        let active_answer = input
            .current_answer
            .as_ref()
            .cloned()
            .or_else(|| current_question.user_answer_confirmed.clone())
            .or_else(|| current_question.user_answer_draft.clone())
            .map(|ans| ans.trim().to_lowercase())
            .filter(|s| !s.is_empty());

        // 6. Ambil transisi percabangan historis yang pernah diobservasi dari pertanyaan ini
        // Pertama, cari di dalam riwayat persona aktif (prioritas tertinggi)
        let mut transitions = SurveyRepository::get_transitions_for_question(
            conn,
            &input.persona_id,
            &current_question.id,
            current_question.question_fingerprint.as_deref(),
        )?;

        // Jika riwayat pada persona aktif belum ada, periksa referensi struktur survei jika sesi ini
        // mengenali atau cocok secara struktural dengan sesi survei lain (Struktur saja, tanpa data jawaban pribadi)
        let is_structural_fallback = if transitions.is_empty() {
            if let Some(ref fp) = current_question.question_fingerprint {
                let structural = SurveyRepository::get_structural_transitions_for_fingerprint(conn, fp)?;
                if !structural.is_empty() {
                    transitions = structural;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        // 7. AT-065 & Non-negotiable Rule 5: Jika riwayat tidak cukup, tampilkan insufficient evidence dan jangan berspekulasi
        if transitions.is_empty() {
            let result = NextQuestionIntelligenceResult {
                session_id: input.session_id.clone(),
                persona_id: input.persona_id.clone(),
                current_question_id: input.current_question_id.clone(),
                current_question_text: current_question.question_text_original.clone(),
                current_user_answer: active_answer,
                status: PredictionStatus::InsufficientEvidence,
                predictions: Vec::new(),
                observed_transitions_count: 0,
                explanation: "Belum ada riwayat transisi atau struktur survei yang teramati untuk pertanyaan ini. Sesuai prinsip kehati-hatian, sistem tidak membuat prediksi spekulatif.".to_string(),
                evaluated_at: now,
            };

            return Ok(result);
        }

        // 8. Evaluasi dan ranking kandidat percabangan berdasarkan kecocokan jawaban dan frekuensi
        let observed_count = transitions.len();
        let predictions = Self::evaluate_candidate_predictions(
            &transitions,
            active_answer.as_deref(),
            is_structural_fallback,
            session.version,
        );

        // Simpan log prediksi baru ke database
        for pred in &predictions {
            let record = SurveyQuestionPredictionRecord {
                id: pred.id.clone(),
                session_id: input.session_id.clone(),
                persona_id: input.persona_id.clone(),
                current_question_id: input.current_question_id.clone(),
                predicted_question_text: pred.candidate_question_text.clone(),
                predicted_question_type: pred.question_type,
                predicted_options: pred.options.clone(),
                confidence_score: pred.confidence,
                evidence_source: pred.evidence_source.clone(),
                explanation: pred.explanation.clone(),
                leading_condition: pred.leading_condition.clone(),
                status: PredictionStatus::PredictedNotDetected,
                matched_question_id: None,
                created_at: now.clone(),
                updated_at: now.clone(),
            };
            let _ = SurveyRepository::save_prediction_record(conn, &record);
        }

        let explanation = format!(
            "Ditemukan {} observasi transisi historis. Menghasilkan {} kandidat prediksi pertanyaan berikutnya.",
            observed_count,
            predictions.len()
        );

        Ok(NextQuestionIntelligenceResult {
            session_id: input.session_id.clone(),
            persona_id: input.persona_id.clone(),
            current_question_id: input.current_question_id.clone(),
            current_question_text: current_question.question_text_original,
            current_user_answer: active_answer,
            status: PredictionStatus::PredictedNotDetected,
            predictions,
            observed_transitions_count: observed_count,
            explanation,
            evaluated_at: now,
        })
    }

    /// Evaluasi dan ranking kandidat prediksi pertanyaan
    fn evaluate_candidate_predictions(
        transitions: &[SurveyBranchTransition],
        active_answer: Option<&str>,
        is_structural_fallback: bool,
        _session_version: i64,
    ) -> Vec<NextQuestionPrediction> {
        // Kelompokkan transisi berdasarkan teks target pertanyaan yang dinormalisasi
        let mut grouped: HashMap<String, Vec<&SurveyBranchTransition>> = HashMap::new();
        for trans in transitions {
            let key = trans.to_question_text.trim().to_lowercase();
            grouped.entry(key).or_default().push(trans);
        }

        let mut candidate_list = Vec::new();
        let total_transitions = transitions.len() as f64;

        for (_key, group) in grouped {
            let representative = group[0];
            let count = group.len() as f64;
            let frequency_ratio = count / total_transitions;

            // Evaluasi kondisi percabangan dan hitung skor keyakinan (confidence)
            let mut best_cond: Option<String> = None;
            let mut answer_matched = false;
            let mut is_alternative = false;

            for t in &group {
                if let Some(ref cond_val) = t.from_answer_value {
                    let cond_norm = cond_val.trim().to_lowercase();
                    if let Some(ans) = active_answer {
                        if ans.contains(&cond_norm) || cond_norm.contains(ans) {
                            answer_matched = true;
                            best_cond = Some(format!("Jawaban cocok: '{}'", cond_val));
                            break;
                        } else {
                            is_alternative = true;
                            best_cond = Some(format!("Cabang jika jawaban: '{}'", cond_val));
                        }
                    } else {
                        best_cond = Some(format!("Kondisi cabang: '{}'", cond_val));
                    }
                } else if let Some(ref bc) = t.branch_condition {
                    best_cond = Some(bc.clone());
                }
            }

            // Hitung skor confidence berdasarkan bukti dan kondisi
            let (confidence, explanation, leading_condition) = match (active_answer, answer_matched, is_alternative) {
                // Jawaban saat ini cocok persis dengan cabang percabangan teramati
                (Some(ans), true, _) => {
                    let conf = (0.85 + (frequency_ratio * 0.10)).min(0.95);
                    (
                        conf,
                        format!(
                            "Kandidat terkuat: Jawaban responden '{}' cocok dengan kondisi percabangan historis (teramati {} kali).",
                            ans, count as usize
                        ),
                        best_cond.or_else(|| Some(format!("Kondisi: Jawaban '{}'", ans))),
                    )
                }
                // Jawaban saat ini berbeda, ini merupakan cabang alternatif
                (Some(ans), false, true) => {
                    let conf = (0.30 + (frequency_ratio * 0.15)).min(0.49);
                    (
                        conf,
                        format!(
                            "Cabang alternatif: Teramati pada riwayat untuk pilihan jawaban berbeda (jawaban aktif responden: '{}').",
                            ans
                        ),
                        best_cond.or_else(|| Some("Cabang alternatif".to_string())),
                    )
                }
                // Pengguna belum menjawab, tapi ada cabang kondisional
                (None, _, true) | (None, false, false) if best_cond.is_some() => {
                    let conf = (0.65 + (frequency_ratio * 0.15)).min(0.85);
                    (
                        conf,
                        format!(
                            "Kemungkinan percabangan teramati {} kali dalam riwayat survei.",
                            count as usize
                        ),
                        best_cond,
                    )
                }
                // Transisi sekuensial normal tanpa syarat khusus
                _ => {
                    let conf = (0.80 + (frequency_ratio * 0.10)).min(0.90);
                    (
                        conf,
                        format!(
                            "Alur sekuensial teramati {} kali pada urutan pertanyaan ini.",
                            count as usize
                        ),
                        Some("Alur sekuensial normal".to_string()),
                    )
                }
            };

            let evidence_source = if is_structural_fallback {
                "survey_structure".to_string()
            } else if representative.transition_type == TransitionType::ConditionalBranch {
                "observed_branch".to_string()
            } else {
                "same_survey_history".to_string()
            };

            candidate_list.push(NextQuestionPrediction {
                id: Uuid::new_v4().to_string(),
                candidate_question_text: representative.to_question_text.clone(),
                question_type: representative.to_question_type,
                options: representative.to_options.clone(),
                evidence_source,
                confidence: (confidence * 100.0).round() / 100.0,
                explanation,
                leading_condition,
                // Non-negotiable AT-065: selalu berlabel tegas "Predicted — not yet detected"
                is_predicted_not_detected: true,
            });
        }

        // Urutkan kandidat berdasarkan confidence tertinggi
        candidate_list.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        candidate_list
    }

    /// Sinkronisasi transisi percabangan yang diamati pada suatu sesi survei
    pub fn sync_session_transitions(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<usize, AppError> {
        let questions = SurveyRepository::list_questions_by_session(conn, persona_id, session_id)?;
        if questions.len() < 2 {
            return Ok(0);
        }

        let existing_transitions =
            SurveyRepository::list_branch_transitions_by_session(conn, persona_id, session_id)?;

        let mut recorded = 0;
        let now = Utc::now().to_rfc3339();

        for i in 0..questions.len() - 1 {
            let from_q = &questions[i];
            let to_q = &questions[i + 1];

            // Cek apakah transisi dari from_q ke to_q sudah pernah dicatat
            let already_exists = existing_transitions
                .iter()
                .any(|t| t.from_question_id == from_q.id && t.to_question_id.as_deref() == Some(&to_q.id));

            if !already_exists {
                let order_diff = to_q.question_order - from_q.question_order;
                let (transition_type, branch_condition) = if order_diff > 1 {
                    (
                        TransitionType::SkipJump,
                        Some(format!("Lompat {} pertanyaan", order_diff - 1)),
                    )
                } else if from_q.user_answer_confirmed.is_some() || from_q.branch_logic.is_some() {
                    let cond = from_q
                        .user_answer_confirmed
                        .as_ref()
                        .map(|ans| format!("Jawaban: {}", ans))
                        .or_else(|| from_q.branch_logic.clone());
                    (TransitionType::ConditionalBranch, cond)
                } else {
                    (TransitionType::Sequential, Some("Sekuensial".to_string()))
                };

                let transition = SurveyBranchTransition {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    persona_id: persona_id.to_string(),
                    from_question_id: from_q.id.clone(),
                    from_question_fingerprint: from_q.question_fingerprint.clone(),
                    from_question_order: from_q.question_order,
                    from_answer_value: from_q.user_answer_confirmed.as_ref().map(|s| s.trim().to_lowercase()),
                    to_question_id: Some(to_q.id.clone()),
                    to_question_fingerprint: to_q.question_fingerprint.clone(),
                    to_question_text: to_q.question_text_original.clone(),
                    to_question_type: to_q.question_type,
                    to_options: to_q.options.clone(),
                    to_question_order: to_q.question_order,
                    branch_condition,
                    transition_type,
                    observed_at: now.clone(),
                    created_at: now.clone(),
                };

                SurveyRepository::record_branch_transition(conn, &transition)?;
                recorded += 1;
            }
        }

        Ok(recorded + existing_transitions.len())
    }

    /// Dipanggil saat pertanyaan baru terdeteksi (melalui OCR atau manual) untuk
    /// memperbarui status prediksi terdahulu (Matched vs Diverged) dan merekam transisi nyata (AT-065).
    pub fn on_question_detected(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
        detected_question: &SurveyQuestion,
    ) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();

        // 1. Jika ada preceding_question_id, cari prediksi tertunda untuk pertanyaan pendahulu tersebut
        if let Some(ref pred_q_id) = detected_question.preceding_question_id {
            let pending_predictions =
                SurveyRepository::find_pending_predictions_for_question(conn, persona_id, pred_q_id)?;

            let detected_text_norm = detected_question
                .question_text_original
                .trim()
                .to_lowercase();
            let detected_fp = detected_question.question_fingerprint.as_deref().unwrap_or("");

            for pred in pending_predictions {
                let pred_text_norm = pred.predicted_question_text.trim().to_lowercase();
                let pred_fp = compute_question_fingerprint(
                    &pred.predicted_question_text,
                    pred.predicted_question_type,
                    &pred.predicted_options,
                );

                // Periksa kecocokan prediksi dengan observasi aktual yang baru saja terkonfirmasi
                let is_match = pred_text_norm == detected_text_norm
                    || (!detected_fp.is_empty() && detected_fp == pred_fp)
                    || (detected_text_norm.len() >= 10 && pred_text_norm.contains(&detected_text_norm))
                    || (pred_text_norm.len() >= 10 && detected_text_norm.contains(&pred_text_norm));

                let new_status = if is_match {
                    PredictionStatus::DetectedMatched
                } else {
                    PredictionStatus::DetectedDiverged
                };

                let matched_id = if is_match {
                    Some(detected_question.id.as_str())
                } else {
                    None
                };

                let _ = SurveyRepository::update_prediction_status(
                    conn,
                    persona_id,
                    &pred.id,
                    new_status,
                    matched_id,
                    &now,
                );
            }
        }

        // 2. Sinkronkan transisi percabangan sesi
        let _ = Self::sync_session_transitions(conn, persona_id, session_id);

        Ok(())
    }

    /// Mengambil seluruh riwayat prediksi sesi survei
    pub fn get_session_predictions(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyQuestionPredictionRecord>, AppError> {
        PersonaRepository::find_by_id(conn, persona_id)?
            .ok_or_else(|| AppError::not_found("Profil responden tidak ditemukan"))?;

        SurveyRepository::list_predictions_by_session(conn, persona_id, session_id)
    }

    /// Mengambil seluruh riwayat transisi percabangan yang diobservasi pada sesi
    pub fn get_session_branch_transitions(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyBranchTransition>, AppError> {
        PersonaRepository::find_by_id(conn, persona_id)?
            .ok_or_else(|| AppError::not_found("Profil responden tidak ditemukan"))?;

        SurveyRepository::list_branch_transitions_by_session(conn, persona_id, session_id)
    }
}
