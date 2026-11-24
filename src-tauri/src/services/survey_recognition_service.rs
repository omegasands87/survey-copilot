use rusqlite::Connection;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::survey::{
    compute_question_fingerprint, compute_session_structural_fingerprint,
    RecognizeSurveyInput, RecognizedQuestionAnswer, SurveyMemorySessionDetail,
    SurveyQuestion, SurveyRecognitionEvaluation, SurveyRecognitionRecord, SurveyRecognitionStatus,
    SurveySession, SurveySessionDetail,
};
use crate::repository::{PersonaRepository, SurveyRepository};

pub struct SurveyRecognitionService;

impl SurveyRecognitionService {
    // ==========================================
    // SURVEY RECOGNITION ENGINE (SC-053)
    // ==========================================

    /// Mengevaluasi pengenalan survei baru terhadap memori riwayat sesi survei (AT-064 & AT-066)
    pub fn evaluate_recognition(
        conn: &Connection,
        input: &RecognizeSurveyInput,
    ) -> Result<SurveyRecognitionEvaluation, AppError> {
        // Validasi persona
        Self::validate_persona(conn, &input.persona_id)?;

        // Ambil sesi target
        let target_session = match SurveyRepository::get_session(conn, &input.persona_id, &input.session_id)? {
            Some(s) => s,
            None => {
                return Err(AppError::not_found(
                    "Sesi survei tidak ditemukan atau bukan milik profil aktif ini",
                ));
            }
        };

        // Ambil daftar seluruh pertanyaan sesi target
        let target_questions = SurveyRepository::list_questions_by_session(conn, &input.persona_id, &input.session_id)?;

        // Pastikan sidik jari sesi target terhitung dan tersimpan
        let target_fingerprint = Self::ensure_target_fingerprints(conn, &target_session, &target_questions)?;

        // Ambil kandidat sesi historis HANYA untuk persona yang sama (AT-066: isolasi persona mutlak)
        let candidates = SurveyRepository::list_candidate_sessions_for_recognition(
            conn,
            &input.persona_id,
            &input.session_id,
        )?;

        if candidates.is_empty() {
            let eval = SurveyRecognitionEvaluation {
                current_session_id: input.session_id.clone(),
                current_fingerprint: Some(target_fingerprint.clone()),
                match_status: SurveyRecognitionStatus::NoMatch,
                confidence_score: 0.0,
                structural_similarity: 0.0,
                title_similarity: 0.0,
                matched_session_id: None,
                matched_session_title: None,
                matched_platform: None,
                matched_version: None,
                matched_question_count: 0,
                total_question_count: target_questions.len() as i64,
                differences: vec!["Belum ada sesi survei historis tersimpan untuk profil responden ini.".to_string()],
                provenance: "structural_fingerprint".to_string(),
                explanation: "Survei baru — belum ada riwayat survei tersimpan pada profil ini.".to_string(),
                recognized_answers: Vec::new(),
                recognized_at: chrono::Utc::now().to_rfc3339(),
            };

            let record = SurveyRecognitionRecord {
                id: Uuid::new_v4().to_string(),
                session_id: input.session_id.clone(),
                persona_id: input.persona_id.clone(),
                matched_session_id: None,
                match_status: SurveyRecognitionStatus::NoMatch,
                confidence_score: 0.0,
                structural_similarity: 0.0,
                title_similarity: 0.0,
                matched_question_count: 0,
                total_question_count: target_questions.len() as i64,
                differences: eval.differences.clone(),
                provenance: eval.provenance.clone(),
                recognized_at: eval.recognized_at.clone(),
                created_at: eval.recognized_at.clone(),
            };
            let _ = SurveyRepository::save_recognition(conn, &record);

            return Ok(eval);
        }

        // Bandingkan sesi target dengan seluruh kandidat
        let mut best_eval: Option<SurveyRecognitionEvaluation> = None;
        let mut best_score = -1.0;

        for candidate in &candidates {
            let eval = Self::compare_sessions(&target_session, &target_questions, candidate);
            if eval.confidence_score > best_score {
                best_score = eval.confidence_score;
                best_eval = Some(eval);
            }
        }

        let final_eval = best_eval.unwrap_or_else(|| SurveyRecognitionEvaluation {
            current_session_id: input.session_id.clone(),
            current_fingerprint: Some(target_fingerprint.clone()),
            match_status: SurveyRecognitionStatus::NoMatch,
            confidence_score: 0.0,
            structural_similarity: 0.0,
            title_similarity: 0.0,
            matched_session_id: None,
            matched_session_title: None,
            matched_platform: None,
            matched_version: None,
            matched_question_count: 0,
            total_question_count: target_questions.len() as i64,
            differences: vec!["Tidak ditemukan kesamaan struktur dengan survei terdahulu.".to_string()],
            provenance: "structural_fingerprint".to_string(),
            explanation: "Tidak ada kecocokan survei historis yang signifikan.".to_string(),
            recognized_answers: Vec::new(),
            recognized_at: chrono::Utc::now().to_rfc3339(),
        });

        // Simpan audit catatan pengenalan survei
        let record = SurveyRecognitionRecord {
            id: Uuid::new_v4().to_string(),
            session_id: input.session_id.clone(),
            persona_id: input.persona_id.clone(),
            matched_session_id: final_eval.matched_session_id.clone(),
            match_status: final_eval.match_status,
            confidence_score: final_eval.confidence_score,
            structural_similarity: final_eval.structural_similarity,
            title_similarity: final_eval.title_similarity,
            matched_question_count: final_eval.matched_question_count,
            total_question_count: final_eval.total_question_count,
            differences: final_eval.differences.clone(),
            provenance: final_eval.provenance.clone(),
            recognized_at: final_eval.recognized_at.clone(),
            created_at: final_eval.recognized_at.clone(),
        };
        let _ = SurveyRepository::save_recognition(conn, &record);

        // Jika dikenali sebagai versi baru atau exact match, update metadata sesi
        if final_eval.match_status == SurveyRecognitionStatus::VersionChanged {
            let new_version = final_eval.matched_version.unwrap_or(1) + 1;
            let now = chrono::Utc::now().to_rfc3339();
            let _ = SurveyRepository::update_session_fingerprint(
                conn,
                &input.persona_id,
                &input.session_id,
                &target_fingerprint,
                new_version,
                final_eval.matched_session_id.as_deref(),
                &now,
            );
        } else if final_eval.match_status == SurveyRecognitionStatus::ExactMatch {
            let matched_version = final_eval.matched_version.unwrap_or(1);
            let now = chrono::Utc::now().to_rfc3339();
            let _ = SurveyRepository::update_session_fingerprint(
                conn,
                &input.persona_id,
                &input.session_id,
                &target_fingerprint,
                matched_version,
                final_eval.matched_session_id.as_deref(),
                &now,
            );
        }

        Ok(final_eval)
    }

    /// Menghitung dan memperbarui sidik jari sesi jika ada perubahan pertanyaan
    pub fn refresh_session_fingerprint(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<String, AppError> {
        let session = match SurveyRepository::get_session(conn, persona_id, session_id)? {
            Some(s) => s,
            None => return Err(AppError::not_found("Sesi survei tidak ditemukan")),
        };

        let questions = SurveyRepository::list_questions_by_session(conn, persona_id, session_id)?;
        Self::ensure_target_fingerprints(conn, &session, &questions)
    }

    /// Mengambil detail Survey Memory untuk sesi survei dengan pemisahan mutlak
    /// antara user_answer_confirmed (jawaban aktual) dan rekomendasi saran AI (AT-064)
    pub fn get_memory_session_detail(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<SurveyMemorySessionDetail, AppError> {
        Self::validate_persona(conn, persona_id)?;

        SurveyRepository::get_memory_session_detail(conn, persona_id, session_id)?.ok_or_else(|| {
            AppError::not_found("Sesi survei tidak ditemukan atau bukan milik profil aktif ini")
        })
    }

    // ==========================================
    // LOGIKA PERBANDINGAN DAN SIMILARITAS
    // ==========================================

    fn compare_sessions(
        target: &SurveySession,
        target_questions: &[SurveyQuestion],
        candidate: &SurveySessionDetail,
    ) -> SurveyRecognitionEvaluation {
        let now = chrono::Utc::now().to_rfc3339();
        let target_len = target_questions.len();
        let cand_len = candidate.questions.len();

        let title_sim = compute_title_similarity(&target.title, &candidate.session.title);

        let platform_match = match (&target.platform_label, &candidate.session.platform_label) {
            (Some(tp), Some(cp)) => tp.trim().eq_ignore_ascii_case(cp.trim()),
            _ => false,
        };

        let domain_match = match (&target.source_domain, &candidate.session.source_domain) {
            (Some(td), Some(cd)) => td.trim().eq_ignore_ascii_case(cd.trim()),
            _ => false,
        };

        // Jika tidak ada pertanyaan pada salah satu sesi
        if target_len == 0 || cand_len == 0 {
            // ATURAN NON-NEGOTIABLE: Kemiripan judul saja tidak boleh menghasilkan match pasti!
            let confidence = if title_sim >= 0.9 && (platform_match || domain_match) {
                0.25
            } else {
                0.0
            };

            let status = if confidence > 0.0 {
                SurveyRecognitionStatus::PossibleMatch
            } else {
                SurveyRecognitionStatus::NoMatch
            };

            return SurveyRecognitionEvaluation {
                current_session_id: target.id.clone(),
                current_fingerprint: target.fingerprint.clone(),
                match_status: status,
                confidence_score: confidence,
                structural_similarity: 0.0,
                title_similarity: title_sim,
                matched_session_id: Some(candidate.session.id.clone()),
                matched_session_title: Some(candidate.session.title.clone()),
                matched_platform: candidate.session.platform_label.clone(),
                matched_version: Some(candidate.session.version),
                matched_question_count: 0,
                total_question_count: target_len as i64,
                differences: vec![
                    "Belum ada cukup pertanyaan untuk membandingkan struktur survei.".to_string(),
                    "Kemiripan judul semata tidak cukup untuk memvalidasi kesamaan survei (SC-053).".to_string(),
                ],
                provenance: "title_only_heuristics".to_string(),
                explanation: "Hanya terdapat kemiripan judul atau platform tanpa data pertanyaan yang memadai.".to_string(),
                recognized_answers: Vec::new(),
                recognized_at: now,
            };
        }

        // Pengecekan Exact Fingerprint
        let exact_fingerprint_match = match (&target.fingerprint, &candidate.session.fingerprint) {
            (Some(tf), Some(cf)) => tf == cf && !tf.is_empty() && tf != "empty",
            _ => false,
        };

        // Pemetaan kesamaan pertanyaan antar sesi
        let mut exact_matches = 0usize;
        let mut modified_matches = 0usize;
        let mut recognized_answers: Vec<RecognizedQuestionAnswer> = Vec::new();
        let mut differences: Vec<String> = Vec::new();

        // Indeks pertanyaan kandidat berdasarkan fingerprint dan urutan
        for target_q in target_questions {
            let mut matched_cand_q: Option<&SurveyQuestion> = None;
            let mut is_exact = false;

            // 1. Cek exact fingerprint match pada pertanyaan kandidat
            if let Some(ref t_fp) = target_q.question_fingerprint {
                for cand_q in &candidate.questions {
                    if let Some(ref c_fp) = cand_q.question_fingerprint {
                        if t_fp == c_fp {
                            matched_cand_q = Some(cand_q);
                            is_exact = true;
                            break;
                        }
                    }
                }
            }

            // 2. Jika tidak exact fingerprint, cek kesamaan teks prompt tinggi
            if matched_cand_q.is_none() {
                for cand_q in &candidate.questions {
                    let text_sim = compute_text_similarity(
                        &target_q.question_text_original,
                        &cand_q.question_text_original,
                    );
                    if text_sim >= 0.85 {
                        matched_cand_q = Some(cand_q);
                        is_exact = false;
                        break;
                    }
                }
            }

            if let Some(cand_q) = matched_cand_q {
                if is_exact {
                    exact_matches += 1;
                } else {
                    modified_matches += 1;
                    differences.push(format!(
                        "Pertanyaan #{}: teks atau opsi jawaban sedikit berbeda dari sesi '{}' (#{}).",
                        target_q.question_order, candidate.session.title, cand_q.question_order
                    ));
                }

                // Ambil jawaban terkonfirmasi nyata pengguna dari sesi historis jika ada (AT-064)
                if let Some(ref confirmed_ans) = cand_q.user_answer_confirmed {
                    recognized_answers.push(RecognizedQuestionAnswer {
                        question_order: target_q.question_order,
                        question_text: target_q.question_text_original.clone(),
                        matched_historical_question_id: cand_q.id.clone(),
                        confirmed_answer: confirmed_ans.clone(),
                        confirmed_at: cand_q.confirmed_at.clone().unwrap_or_default(),
                        provenance: cand_q.provenance,
                    });
                }
            } else {
                differences.push(format!(
                    "Pertanyaan #{}: pertanyaan baru yang tidak terdapat pada sesi '{}'.",
                    target_q.question_order, candidate.session.title
                ));
            }
        }

        // Cek pertanyaan kandidat yang tidak ada pada target (pertanyaan terhapus/berkurang)
        if cand_len > target_len {
            differences.push(format!(
                "Jumlah pertanyaan berkurang (target: {}, versi historis: {}).",
                target_len, cand_len
            ));
        } else if target_len > cand_len {
            differences.push(format!(
                "Jumlah pertanyaan bertambah (target: {}, versi historis: {}).",
                target_len, cand_len
            ));
        }

        let total_matched = exact_matches + modified_matches;
        let max_len = target_len.max(cand_len) as f64;
        let structural_similarity = (exact_matches as f64 * 1.0 + modified_matches as f64 * 0.7) / max_len;

        // ATURAN KRITIS NON-NEGOTIABLE:
        // "Jangan menyatakan cocok secara pasti jika hanya berdasarkan kemiripan judul."
        // Jika tidak ada kecocokan struktur pertanyaan sama sekali, batas skor maksimal 0.20
        if total_matched == 0 {
            let conf = if title_sim >= 0.8 && (platform_match || domain_match) {
                0.20
            } else {
                0.05
            };
            return SurveyRecognitionEvaluation {
                current_session_id: target.id.clone(),
                current_fingerprint: target.fingerprint.clone(),
                match_status: SurveyRecognitionStatus::NoMatch,
                confidence_score: conf,
                structural_similarity: 0.0,
                title_similarity: title_sim,
                matched_session_id: Some(candidate.session.id.clone()),
                matched_session_title: Some(candidate.session.title.clone()),
                matched_platform: candidate.session.platform_label.clone(),
                matched_version: Some(candidate.session.version),
                matched_question_count: 0,
                total_question_count: target_len as i64,
                differences: vec![
                    "Tidak ada satu pun pertanyaan yang cocok secara struktural.".to_string(),
                    "Judul mungkin serupa, tetapi struktur pertanyaan sama sekali berbeda.".to_string(),
                ],
                provenance: "structural_fingerprint".to_string(),
                explanation: "Survei berbeda: tidak ada pertanyaan yang cocok dengan riwayat meskipun judul memiliki kemiripan.".to_string(),
                recognized_answers: Vec::new(),
                recognized_at: now,
            };
        }

        // Penentuan Status dan Skor Keyakinan
        let (status, confidence, explanation) = if exact_fingerprint_match && exact_matches == target_len && target_len == cand_len {
            // SAMA PERSIS (Exact Match)
            let conf = 0.98;
            let expl = format!(
                "Survei Dikenali (100% Sama Persis): Seluruh {} pertanyaan dan sidik jari struktural cocok dengan sesi '{}' ({})",
                target_len, candidate.session.title, candidate.session.platform_label.as_deref().unwrap_or("Platform tidak tercantum")
            );
            (SurveyRecognitionStatus::ExactMatch, conf, expl)
        } else if total_matched >= 2 && structural_similarity >= 0.60 {
            // VERSI BERUBAH (Version Changed / Modified Survey)
            let conf = (structural_similarity * 0.85 + title_sim * 0.15).min(0.92);
            let expl = format!(
                "Survei Dikenali (Versi Berubah): {} dari {} pertanyaan cocok dengan sesi '{}' (v{}). Terdeteksi perubahan urutan atau pertanyaan.",
                total_matched, target_len, candidate.session.title, candidate.session.version
            );
            (SurveyRecognitionStatus::VersionChanged, conf, expl)
        } else if structural_similarity >= 0.45 {
            // LIKELY MATCH
            let conf = (structural_similarity * 0.75 + title_sim * 0.25).min(0.80);
            let expl = format!(
                "Kemiripan Kuat: {} pertanyaan cocok dengan sesi '{}'.",
                total_matched, candidate.session.title
            );
            (SurveyRecognitionStatus::LikelyMatch, conf, expl)
        } else if structural_similarity >= 0.25 || (total_matched >= 1 && title_sim >= 0.60) {
            // POSSIBLE MATCH
            let conf = (structural_similarity * 0.60 + title_sim * 0.40).min(0.55);
            let expl = format!(
                "Kemiripan Sebagian: {} pertanyaan cocok dengan sesi '{}'.",
                total_matched, candidate.session.title
            );
            (SurveyRecognitionStatus::PossibleMatch, conf, expl)
        } else {
            // NO MATCH
            (
                SurveyRecognitionStatus::NoMatch,
                0.15,
                "Hanya sedikit kemiripan acak yang terdeteksi.".to_string(),
            )
        };

        SurveyRecognitionEvaluation {
            current_session_id: target.id.clone(),
            current_fingerprint: target.fingerprint.clone(),
            match_status: status,
            confidence_score: confidence,
            structural_similarity,
            title_similarity: title_sim,
            matched_session_id: Some(candidate.session.id.clone()),
            matched_session_title: Some(candidate.session.title.clone()),
            matched_platform: candidate.session.platform_label.clone(),
            matched_version: Some(candidate.session.version),
            matched_question_count: total_matched as i64,
            total_question_count: target_len as i64,
            differences,
            provenance: "structural_fingerprint".to_string(),
            explanation,
            recognized_answers,
            recognized_at: now,
        }
    }

    fn ensure_target_fingerprints(
        conn: &Connection,
        target_session: &SurveySession,
        target_questions: &[SurveyQuestion],
    ) -> Result<String, AppError> {
        let mut question_tuples: Vec<(i64, &str, crate::domain::survey::QuestionType, &[String])> = Vec::new();

        for q in target_questions {
            let q_fp = compute_question_fingerprint(
                &q.question_text_original,
                q.question_type,
                &q.options,
            );

            // Update jika belum ada atau berbeda
            if q.question_fingerprint.as_deref() != Some(&q_fp) {
                let _ = conn.execute(
                    "UPDATE survey_questions SET question_fingerprint = ?1 WHERE id = ?2 AND persona_id = ?3",
                    rusqlite::params![q_fp, q.id, q.persona_id],
                );
            }

            question_tuples.push((q.question_order, &q.question_text_original, q.question_type, &q.options));
        }

        let session_fp = compute_session_structural_fingerprint(&question_tuples);

        if target_session.fingerprint.as_deref() != Some(&session_fp) {
            let now = chrono::Utc::now().to_rfc3339();
            let _ = SurveyRepository::update_session_fingerprint(
                conn,
                &target_session.persona_id,
                &target_session.id,
                &session_fp,
                target_session.version,
                target_session.recognized_from_id.as_deref(),
                &now,
            );
        }

        Ok(session_fp)
    }

    fn validate_persona(conn: &Connection, persona_id: &str) -> Result<(), AppError> {
        let persona = PersonaRepository::find_by_id(conn, persona_id)
            .map_err(|e| AppError::database(format!("Gagal mencari persona: {e}")))?;
        match persona {
            Some(p) if p.status != "archived" => Ok(()),
            Some(_) => Err(AppError::validation(
                "persona_id",
                "Persona ini telah diarsipkan dan tidak dapat digunakan",
            )),
            None => Err(AppError::not_found("Persona tidak ditemukan")),
        }
    }
}

// ==========================================
// PENGHITUNG SIMILARITAS TEKS MURNI
// ==========================================

/// Menghitung kemiripan judul berbasis token Jaccard & panjang karakter
fn compute_title_similarity(t1: &str, t2: &str) -> f64 {
    let clean1 = normalize_for_similarity(t1);
    let clean2 = normalize_for_similarity(t2);

    if clean1.is_empty() && clean2.is_empty() {
        return 1.0;
    }
    if clean1.is_empty() || clean2.is_empty() {
        return 0.0;
    }
    if clean1 == clean2 {
        return 1.0;
    }

    let tokens1: std::collections::HashSet<&str> = clean1.split_whitespace().collect();
    let tokens2: std::collections::HashSet<&str> = clean2.split_whitespace().collect();

    let intersection = tokens1.intersection(&tokens2).count();
    let union = tokens1.union(&tokens2).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}

/// Menghitung kemiripan teks prompt pertanyaan
fn compute_text_similarity(s1: &str, s2: &str) -> f64 {
    let clean1 = normalize_for_similarity(s1);
    let clean2 = normalize_for_similarity(s2);

    if clean1 == clean2 {
        return 1.0;
    }

    let tokens1: std::collections::HashSet<&str> = clean1.split_whitespace().collect();
    let tokens2: std::collections::HashSet<&str> = clean2.split_whitespace().collect();

    let intersection = tokens1.intersection(&tokens2).count();
    let union = tokens1.union(&tokens2).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}

fn normalize_for_similarity(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
