use rusqlite::Connection;
use uuid::Uuid;

use crate::db::Database;
use crate::domain::answer_engine::{AnswerHistoryItem, ReviewAction, ReviewStatus};
use crate::domain::classifier::{ClassifyQuestionInput, QuestionClassificationResult};
use crate::domain::error::AppError;
use crate::domain::evidence::{EvidencePackage, EvidenceRetrievalInput};
use crate::domain::survey::{
    compute_question_fingerprint, extract_domain_from_url, AnswerFormat, ConfirmAnswerInput,
    ConfirmationStatus, CreateQuestionInput, CreateSessionInput, NextQuestionIntelligenceResult,
    PredictNextQuestionInput, Provenance, QuestionStatus, QuestionType, RecognizeSurveyInput,
    SaveQuestionDraftInput, SessionStatus, SurveyBranchTransition, SurveyMemorySessionDetail,
    SurveyQuestion, SurveyQuestionPredictionRecord, SurveyRecognitionEvaluation, SurveySession,
    SurveySessionDetail, SurveySessionSummary, UpdateQuestionInput, UpdateSessionInput,
};
use crate::repository::{AnswerRepository, PersonaRepository, SurveyRepository};
use crate::services::{
    EvidenceEngineService, NextQuestionService, QuestionClassifierService, SurveyRecognitionService,
};

pub struct SurveyService;

impl SurveyService {
    // ==========================================
    // SURVEY SESSION MANAGEMENT
    // ==========================================

    /// Membuat sesi survei baru yang terikat pada persona aktif
    pub fn create_session(
        conn: &Connection,
        input: CreateSessionInput,
    ) -> Result<SurveySession, AppError> {
        let trimmed_title = input.title.trim();
        if trimmed_title.is_empty() {
            return Err(AppError::validation(
                "title",
                "Judul survei tidak boleh kosong",
            ));
        }

        // Verifikasi bahwa persona ada dan aktif/tidak terhapus
        Self::validate_persona_exists(conn, &input.persona_id)?;

        let now = chrono::Utc::now().to_rfc3339();
        let domain = input.source_url.as_deref().and_then(extract_domain_from_url);
        let session = SurveySession {
            id: Uuid::new_v4().to_string(),
            persona_id: input.persona_id,
            title: trimmed_title.to_string(),
            platform_label: input
                .platform_label
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            source_url: input
                .source_url
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            source_domain: domain,
            external_survey_id: input
                .external_survey_id
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            status: SessionStatus::Active,
            language: input.language.unwrap_or_else(|| "id".to_string()),
            category: input
                .category
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            fingerprint: None,
            version: 1,
            recognized_from_id: None,
            notes: input
                .notes
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            created_at: now.clone(),
            updated_at: now,
            archived_at: None,
        };

        SurveyRepository::create_session(conn, &session)
    }

    /// Mengambil detail satu sesi survei
    pub fn get_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<SurveySession, AppError> {
        Self::validate_persona_exists(conn, persona_id)?;

        SurveyRepository::get_session(conn, persona_id, session_id)?.ok_or_else(|| {
            AppError::not_found("Sesi survei tidak ditemukan atau bukan milik persona ini")
        })
    }

    /// Mengambil seluruh sesi survei untuk persona tertentu
    pub fn list_sessions(
        conn: &Connection,
        persona_id: &str,
        include_archived: bool,
    ) -> Result<Vec<SurveySessionSummary>, AppError> {
        Self::validate_persona_exists(conn, persona_id)?;

        SurveyRepository::list_sessions(conn, persona_id, include_archived)
    }

    /// Memperbarui informasi sesi survei
    pub fn update_session(
        conn: &Connection,
        input: UpdateSessionInput,
    ) -> Result<SurveySession, AppError> {
        Self::validate_persona_exists(conn, &input.persona_id)?;

        let trimmed_title = input.title.trim();
        if trimmed_title.is_empty() {
            return Err(AppError::validation(
                "title",
                "Judul survei tidak boleh kosong",
            ));
        }

        let mut session = Self::get_session(conn, &input.persona_id, &input.session_id)?;
        session.title = trimmed_title.to_string();
        session.platform_label = input
            .platform_label
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        session.source_url = input
            .source_url
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        session.source_domain = session.source_url.as_deref().and_then(extract_domain_from_url);
        session.external_survey_id = input
            .external_survey_id
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(lang) = input.language {
            session.language = lang;
        }
        session.category = input
            .category
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        session.notes = input
            .notes
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        if let Some(status) = input.status {
            session.status = status;
            if status == SessionStatus::Archived && session.archived_at.is_none() {
                session.archived_at = Some(chrono::Utc::now().to_rfc3339());
            } else if status == SessionStatus::Active {
                session.archived_at = None;
            }
        }

        session.updated_at = chrono::Utc::now().to_rfc3339();

        SurveyRepository::update_session(conn, &session)
    }

    /// Mengarsipkan sesi survei
    pub fn archive_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<(), AppError> {
        Self::validate_persona_exists(conn, persona_id)?;
        let _ = Self::get_session(conn, persona_id, session_id)?;

        let now = chrono::Utc::now().to_rfc3339();
        SurveyRepository::set_session_status(
            conn,
            persona_id,
            session_id,
            SessionStatus::Archived,
            Some(&now),
            &now,
        )
    }

    /// Memulihkan sesi survei yang diarsipkan
    pub fn restore_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<(), AppError> {
        Self::validate_persona_exists(conn, persona_id)?;
        let _ = Self::get_session(conn, persona_id, session_id)?;

        let now = chrono::Utc::now().to_rfc3339();
        SurveyRepository::set_session_status(
            conn,
            persona_id,
            session_id,
            SessionStatus::Active,
            None,
            &now,
        )
    }

    /// Menghapus sesi survei dan seluruh relasi datanya
    pub fn delete_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<(), AppError> {
        Self::validate_persona_exists(conn, persona_id)?;
        let _ = Self::get_session(conn, persona_id, session_id)?;

        let deleted = SurveyRepository::delete_session(conn, persona_id, session_id)?;
        if !deleted {
            return Err(AppError::not_found(
                "Sesi survei tidak ditemukan atau bukan milik persona ini",
            ));
        }

        Ok(())
    }

    /// Mengambil detail lengkap sesi survei beserta seluruh daftar pertanyaan terurut
    pub fn get_session_detail(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<SurveySessionDetail, AppError> {
        Self::validate_persona_exists(conn, persona_id)?;

        SurveyRepository::get_session_detail(conn, persona_id, session_id)?.ok_or_else(|| {
            AppError::not_found("Sesi survei tidak ditemukan atau bukan milik persona ini")
        })
    }

    // ==========================================
    // MANUAL QUESTION INPUT & PERSISTENCE
    // ==========================================

    /// Menambahkan pertanyaan survei manual ke dalam sesi
    pub fn create_question(
        conn: &Connection,
        input: CreateQuestionInput,
    ) -> Result<SurveyQuestion, AppError> {
        Self::validate_persona_exists(conn, &input.persona_id)?;
        let session = Self::get_session(conn, &input.persona_id, &input.session_id)?;

        let original_text = input.question_text_original.trim();
        if original_text.is_empty() {
            return Err(AppError::validation(
                "question_text_original",
                "Teks pertanyaan tidak boleh kosong",
            ));
        }

        // Tentukan nomor urutan pertanyaan
        let order = match input.question_order {
            Some(o) if o > 0 => o,
            _ => SurveyRepository::get_next_question_order(conn, &input.persona_id, &session.id)?,
        };

        let cleaned_options: Vec<String> = input
            .options
            .unwrap_or_default()
            .into_iter()
            .map(|opt| opt.trim().to_string())
            .filter(|opt| !opt.is_empty())
            .collect();

        let classification = QuestionClassifierService::classify_raw(
            original_text,
            input.instructions.as_deref(),
            Some(&cleaned_options),
        );

        let q_type = match input.question_type {
            Some(t) if t != QuestionType::Unknown => t,
            _ => classification.question_type,
        };

        let a_format = match input.answer_format {
            Some(f) if f != AnswerFormat::Unknown => f,
            _ => classification.answer_format,
        };

        let structure_val = serde_json::to_value(&classification).ok();

        let now = chrono::Utc::now().to_rfc3339();
        let q_fp = compute_question_fingerprint(original_text, q_type, &cleaned_options);
        let question = SurveyQuestion {
            id: Uuid::new_v4().to_string(),
            session_id: session.id,
            persona_id: input.persona_id,
            capture_id: input.capture_id,
            question_order: order,
            question_text_original: original_text.to_string(),
            question_text_corrected: None,
            instructions: input
                .instructions
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            options: cleaned_options,
            structure_json: structure_val,
            extraction_status: "classified".to_string(),
            question_type: q_type,
            answer_format: a_format,
            provenance: input.provenance.unwrap_or(Provenance::UserEntered),
            observation_type: QuestionStatus::Detected,
            confirmation_status: ConfirmationStatus::Draft,
            user_answer_draft: None,
            user_answer_confirmed: None,
            confirmed_at: None,
            preceding_question_id: input.preceding_question_id,
            branch_logic: input.branch_logic,
            question_fingerprint: Some(q_fp),
            created_at: now.clone(),
            updated_at: now,
        };

        let created = SurveyRepository::create_question(conn, &question)?;
        let _ = SurveyRecognitionService::refresh_session_fingerprint(
            conn,
            &created.persona_id,
            &created.session_id,
        );
        let _ = crate::services::NextQuestionService::on_question_detected(
            conn,
            &created.persona_id,
            &created.session_id,
            &created,
        );
        Ok(created)
    }

    /// Mengambil satu pertanyaan survei berdasarkan ID
    pub fn get_question(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
    ) -> Result<SurveyQuestion, AppError> {
        Self::validate_persona_exists(conn, persona_id)?;

        SurveyRepository::get_question(conn, persona_id, question_id)?.ok_or_else(|| {
            AppError::not_found("Pertanyaan survei tidak ditemukan atau bukan milik persona ini")
        })
    }

    /// Mengambil seluruh pertanyaan dalam sesi terurut sekuensial
    pub fn list_questions(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyQuestion>, AppError> {
        Self::validate_persona_exists(conn, persona_id)?;
        let _ = Self::get_session(conn, persona_id, session_id)?;

        SurveyRepository::list_questions_by_session(conn, persona_id, session_id)
    }

    /// Memperbarui informasi pertanyaan survei
    pub fn update_question(
        conn: &Connection,
        input: UpdateQuestionInput,
    ) -> Result<SurveyQuestion, AppError> {
        Self::validate_persona_exists(conn, &input.persona_id)?;
        let mut question = Self::get_question(conn, &input.persona_id, &input.question_id)?;

        let has_text_change = input.question_text_original.is_some()
            || input.instructions.is_some()
            || input.options.is_some();

        if let Some(orig) = input.question_text_original {
            let trimmed = orig.trim();
            if trimmed.is_empty() {
                return Err(AppError::validation(
                    "question_text_original",
                    "Teks pertanyaan asli tidak boleh kosong",
                ));
            }
            question.question_text_original = trimmed.to_string();
        }

        if let Some(corr) = input.question_text_corrected {
            question.question_text_corrected =
                Some(corr.trim().to_string()).filter(|s| !s.is_empty());
        }

        if let Some(inst) = input.instructions {
            question.instructions = Some(inst.trim().to_string()).filter(|s| !s.is_empty());
        }

        if let Some(opts) = input.options {
            question.options = opts
                .into_iter()
                .map(|opt| opt.trim().to_string())
                .filter(|opt| !opt.is_empty())
                .collect();
        }

        if has_text_change {
            let reclassified = QuestionClassifierService::classify_raw(
                &question.question_text_original,
                question.instructions.as_deref(),
                Some(&question.options),
            );
            question.structure_json = serde_json::to_value(&reclassified).ok();
            question.extraction_status = "classified".to_string();
            if input.question_type.is_none() {
                question.question_type = reclassified.question_type;
            }
            if input.answer_format.is_none() {
                question.answer_format = reclassified.answer_format;
            }
        }

        if let Some(qtype) = input.question_type {
            question.question_type = qtype;
        }

        if let Some(aformat) = input.answer_format {
            question.answer_format = aformat;
        }

        if let Some(status) = input.confirmation_status {
            question.confirmation_status = status;
        }

        if let Some(draft) = input.user_answer_draft {
            question.user_answer_draft = Some(draft);
        }

        if input.preceding_question_id.is_some() {
            question.preceding_question_id = input.preceding_question_id;
        }

        if input.branch_logic.is_some() {
            question.branch_logic = input.branch_logic;
        }

        let q_fp = compute_question_fingerprint(
            &question.question_text_original,
            question.question_type,
            &question.options,
        );
        question.question_fingerprint = Some(q_fp);
        question.updated_at = chrono::Utc::now().to_rfc3339();

        let updated = SurveyRepository::update_question(conn, &question)?;
        let _ = SurveyRecognitionService::refresh_session_fingerprint(
            conn,
            &updated.persona_id,
            &updated.session_id,
        );
        Ok(updated)
    }

    /// Menyimpan draf jawaban pengguna secara langsung dan aman (auto-save)
    pub fn save_question_draft(
        conn: &Connection,
        input: SaveQuestionDraftInput,
    ) -> Result<(), AppError> {
        Self::validate_persona_exists(conn, &input.persona_id)?;
        let _ = Self::get_question(conn, &input.persona_id, &input.question_id)?;

        let now = chrono::Utc::now().to_rfc3339();
        SurveyRepository::save_question_draft(
            conn,
            &input.persona_id,
            &input.question_id,
            &input.user_answer_draft,
            &now,
        )
    }

    /// Mengkonfirmasi jawaban aktual pengguna (terpisah dari AI draft)
    pub fn confirm_question_answer(
        conn: &Connection,
        input: ConfirmAnswerInput,
    ) -> Result<(), AppError> {
        Self::validate_persona_exists(conn, &input.persona_id)?;
        let _ = Self::get_question(conn, &input.persona_id, &input.question_id)?;

        let trimmed_answer = input.user_answer.trim();
        if trimmed_answer.is_empty() {
            return Err(AppError::validation(
                "user_answer",
                "Jawaban yang dikonfirmasi tidak boleh kosong",
            ));
        }

        let question = Self::get_question(conn, &input.persona_id, &input.question_id)?;
        let now = chrono::Utc::now().to_rfc3339();
        SurveyRepository::confirm_question_answer(
            conn,
            &input.persona_id,
            &input.question_id,
            trimmed_answer,
            &now,
            &now,
        )?;

        // Sinkronisasi transisi percabangan sesi dengan jawaban aktual terkonfirmasi
        let _ = crate::services::NextQuestionService::sync_session_transitions(
            conn,
            &input.persona_id,
            &question.session_id,
        );

        // Catat riwayat audit konfirmasi langsung (SC-034 audit trail)
        let history_item = AnswerHistoryItem {
            id: Uuid::new_v4().to_string(),
            persona_id: input.persona_id,
            question_id: input.question_id,
            recommendation_id: None,
            action: ReviewAction::Confirmed,
            draft_answer: None,
            actual_answer: trimmed_answer.to_string(),
            review_status: ReviewStatus::Confirmed.as_str().to_string(),
            provider_id: None,
            model_id: None,
            prompt_version: None,
            evidence_sources_json: "[]".to_string(),
            reason: None,
            created_at: now,
        };
        let _ = AnswerRepository::save_history(conn, &history_item);

        Ok(())
    }

    /// Menghapus pertanyaan survei
    pub fn delete_question(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
    ) -> Result<(), AppError> {
        Self::validate_persona_exists(conn, persona_id)?;
        let q = Self::get_question(conn, persona_id, question_id)?;

        let deleted = SurveyRepository::delete_question(conn, persona_id, question_id)?;
        if !deleted {
            return Err(AppError::not_found(
                "Pertanyaan survei tidak ditemukan atau bukan milik persona ini",
            ));
        }

        let _ = SurveyRecognitionService::refresh_session_fingerprint(
            conn,
            persona_id,
            &q.session_id,
        );

        Ok(())
    }

    /// Menjalankan klasifikasi deterministik dan ekstraksi batasan pertanyaan (SC-031)
    pub fn classify_question(
        conn: &Connection,
        input: ClassifyQuestionInput,
    ) -> Result<QuestionClassificationResult, AppError> {
        if let Some(ref persona_id) = input.persona_id {
            Self::validate_persona_exists(conn, persona_id)?;
        }

        if let (Some(ref s_id), Some(ref p_id)) = (&input.session_id, &input.persona_id) {
            let _ = Self::get_session(conn, p_id, s_id)?;
        }

        let classification = QuestionClassifierService::classify(&input);

        // Jika question_id dan persona_id diberikan, validasi kepemilikan dan perbarui basis data
        if let (Some(ref q_id), Some(ref p_id)) = (&input.question_id, &input.persona_id) {
            match SurveyRepository::get_question(conn, p_id, q_id)? {
                Some(mut q) => {
                    q.structure_json = serde_json::to_value(&classification).ok();
                    q.question_type = classification.question_type;
                    q.answer_format = classification.answer_format;
                    q.extraction_status = "classified".to_string();
                    q.updated_at = chrono::Utc::now().to_rfc3339();
                    SurveyRepository::update_question(conn, &q)?;
                }
                None => {
                    return Err(AppError::not_found(
                        "Pertanyaan tidak ditemukan atau bukan milik persona yang aktif",
                    ));
                }
            }
        }

        Ok(classification)
    }

    /// Menjalankan pencarian bukti dan evaluasi konsistensi untuk pertanyaan survei (SC-032)
    pub fn evaluate_evidence(
        conn: &Connection,
        input: EvidenceRetrievalInput,
    ) -> Result<EvidencePackage, AppError> {
        Self::validate_persona_exists(conn, &input.persona_id)?;

        if let Some(ref s_id) = input.session_id {
            let _ = Self::get_session(conn, &input.persona_id, s_id)?;
        }

        if let Some(ref q_id) = input.question_id {
            let _ = Self::get_question(conn, &input.persona_id, q_id)?;
        }

        EvidenceEngineService::evaluate_evidence(conn, &input)
    }

    // ==========================================
    // DATABASE WRAPPER METHODS (FOR COMMANDS)
    // ==========================================

    pub fn db_create_session(
        db: &Database,
        input: CreateSessionInput,
    ) -> Result<SurveySession, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::create_session(&conn, input)
    }

    pub fn db_get_session(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<SurveySession, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::get_session(&conn, persona_id, session_id)
    }

    pub fn db_list_sessions(
        db: &Database,
        persona_id: &str,
        include_archived: bool,
    ) -> Result<Vec<SurveySessionSummary>, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::list_sessions(&conn, persona_id, include_archived)
    }

    pub fn db_update_session(
        db: &Database,
        input: UpdateSessionInput,
    ) -> Result<SurveySession, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::update_session(&conn, input)
    }

    pub fn db_archive_session(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<(), AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::archive_session(&conn, persona_id, session_id)
    }

    pub fn db_restore_session(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<(), AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::restore_session(&conn, persona_id, session_id)
    }

    pub fn db_delete_session(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<(), AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::delete_session(&conn, persona_id, session_id)
    }

    pub fn db_get_session_detail(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<SurveySessionDetail, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::get_session_detail(&conn, persona_id, session_id)
    }

    pub fn db_create_question(
        db: &Database,
        input: CreateQuestionInput,
    ) -> Result<SurveyQuestion, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::create_question(&conn, input)
    }

    pub fn db_get_question(
        db: &Database,
        persona_id: &str,
        question_id: &str,
    ) -> Result<SurveyQuestion, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::get_question(&conn, persona_id, question_id)
    }

    pub fn db_list_questions(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyQuestion>, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::list_questions(&conn, persona_id, session_id)
    }

    pub fn db_update_question(
        db: &Database,
        input: UpdateQuestionInput,
    ) -> Result<SurveyQuestion, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::update_question(&conn, input)
    }

    pub fn db_save_question_draft(
        db: &Database,
        input: SaveQuestionDraftInput,
    ) -> Result<(), AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::save_question_draft(&conn, input)
    }

    pub fn db_confirm_question_answer(
        db: &Database,
        input: ConfirmAnswerInput,
    ) -> Result<(), AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::confirm_question_answer(&conn, input)
    }

    pub fn db_delete_question(
        db: &Database,
        persona_id: &str,
        question_id: &str,
    ) -> Result<(), AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::delete_question(&conn, persona_id, question_id)
    }

    pub fn db_classify_question(
        db: &Database,
        input: ClassifyQuestionInput,
    ) -> Result<QuestionClassificationResult, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::classify_question(&conn, input)
    }

    pub fn db_evaluate_evidence(
        db: &Database,
        input: EvidenceRetrievalInput,
    ) -> Result<EvidencePackage, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::evaluate_evidence(&conn, input)
    }

    // ==========================================
    // SC-053: SURVEY MEMORY & RECOGNITION
    // ==========================================

    /// Mengevaluasi pengenalan survei aktif terhadap memori historis (AT-064 & AT-066)
    pub fn recognize_survey(
        conn: &Connection,
        input: &RecognizeSurveyInput,
    ) -> Result<SurveyRecognitionEvaluation, AppError> {
        Self::validate_persona_exists(conn, &input.persona_id)?;
        SurveyRecognitionService::evaluate_recognition(conn, input)
    }

    /// Mengambil detail Survey Memory dengan pemisahan tegas antara jawaban konfirmasi dan draf AI (AT-064)
    pub fn get_memory_session_detail(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<SurveyMemorySessionDetail, AppError> {
        Self::validate_persona_exists(conn, persona_id)?;
        SurveyRecognitionService::get_memory_session_detail(conn, persona_id, session_id)
    }

    pub fn db_recognize_survey(
        db: &Database,
        input: RecognizeSurveyInput,
    ) -> Result<SurveyRecognitionEvaluation, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::recognize_survey(&conn, &input)
    }

    pub fn db_get_memory_session_detail(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<SurveyMemorySessionDetail, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        Self::get_memory_session_detail(&conn, persona_id, session_id)
    }

    pub fn db_predict_next_questions(
        db: &Database,
        input: PredictNextQuestionInput,
    ) -> Result<NextQuestionIntelligenceResult, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        NextQuestionService::predict_next_questions(&conn, &input)
    }

    pub fn db_get_session_predictions(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyQuestionPredictionRecord>, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        NextQuestionService::get_session_predictions(&conn, persona_id, session_id)
    }

    pub fn db_get_session_branch_transitions(
        db: &Database,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyBranchTransition>, AppError> {
        let conn = db
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
        NextQuestionService::get_session_branch_transitions(&conn, persona_id, session_id)
    }

    // ==========================================
    // VALIDASI PRIVAT
    // ==========================================

    fn validate_persona_exists(conn: &Connection, persona_id: &str) -> Result<(), AppError> {
        let persona = PersonaRepository::find_by_id(conn, persona_id)
            .map_err(|e| AppError::database(format!("Gagal mencari persona: {e}")))?;
        match persona {
            Some(p) if p.status != "archived" => Ok(()),
            Some(_) => Err(AppError::validation(
                "persona_id",
                "Persona ini telah diarsipkan dan tidak dapat digunakan untuk sesi baru",
            )),
            None => Err(AppError::not_found("Persona tidak ditemukan")),
        }
    }
}
