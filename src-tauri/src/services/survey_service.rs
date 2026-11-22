use rusqlite::Connection;
use uuid::Uuid;

use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::survey::{
    AnswerFormat, ConfirmAnswerInput, ConfirmationStatus, CreateQuestionInput, CreateSessionInput,
    Provenance, QuestionStatus, QuestionType, SaveQuestionDraftInput, SessionStatus,
    SurveyQuestion, SurveySession, SurveySessionDetail, SurveySessionSummary, UpdateQuestionInput,
    UpdateSessionInput,
};
use crate::repository::{PersonaRepository, SurveyRepository};

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
            _ => SurveyRepository::get_next_question_order(conn, &session.id)?,
        };

        let cleaned_options: Vec<String> = input
            .options
            .unwrap_or_default()
            .into_iter()
            .map(|opt| opt.trim().to_string())
            .filter(|opt| !opt.is_empty())
            .collect();

        let now = chrono::Utc::now().to_rfc3339();
        let question = SurveyQuestion {
            id: Uuid::new_v4().to_string(),
            session_id: session.id,
            persona_id: input.persona_id,
            capture_id: None,
            question_order: order,
            question_text_original: original_text.to_string(),
            question_text_corrected: None,
            instructions: input
                .instructions
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            options: cleaned_options,
            structure_json: None,
            extraction_status: "manual".to_string(),
            question_type: input.question_type.unwrap_or(QuestionType::Unknown),
            answer_format: input.answer_format.unwrap_or(AnswerFormat::Unknown),
            provenance: input.provenance.unwrap_or(Provenance::UserEntered),
            observation_type: QuestionStatus::Detected,
            confirmation_status: ConfirmationStatus::Draft,
            user_answer_draft: None,
            user_answer_confirmed: None,
            confirmed_at: None,
            preceding_question_id: input.preceding_question_id,
            branch_logic: input.branch_logic,
            created_at: now.clone(),
            updated_at: now,
        };

        SurveyRepository::create_question(conn, &question)
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

        question.updated_at = chrono::Utc::now().to_rfc3339();

        SurveyRepository::update_question(conn, &question)
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

        let now = chrono::Utc::now().to_rfc3339();
        SurveyRepository::confirm_question_answer(
            conn,
            &input.persona_id,
            &input.question_id,
            trimmed_answer,
            &now,
            &now,
        )
    }

    /// Menghapus pertanyaan survei
    pub fn delete_question(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
    ) -> Result<(), AppError> {
        Self::validate_persona_exists(conn, persona_id)?;
        let _ = Self::get_question(conn, persona_id, question_id)?;

        let deleted = SurveyRepository::delete_question(conn, persona_id, question_id)?;
        if !deleted {
            return Err(AppError::not_found(
                "Pertanyaan survei tidak ditemukan atau bukan milik persona ini",
            ));
        }

        Ok(())
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
