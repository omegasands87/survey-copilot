use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::error::AppError;
use crate::domain::survey::{
    AnswerFormat, ConfirmationStatus, Provenance, QuestionStatus, QuestionType, SessionStatus,
    SurveyQuestion, SurveySession, SurveySessionDetail, SurveySessionSummary,
};

pub struct SurveyRepository;

impl SurveyRepository {
    // ==========================================
    // SURVEY SESSION CRUD
    // ==========================================

    /// Menyimpan sesi survei baru ke basis data
    pub fn create_session(
        conn: &Connection,
        session: &SurveySession,
    ) -> Result<SurveySession, AppError> {
        conn.execute(
            "INSERT INTO survey_sessions (
                id, persona_id, title, platform_label, source_url, external_survey_id,
                status, language, category, fingerprint, notes, created_at, updated_at, archived_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                session.id,
                session.persona_id,
                session.title,
                session.platform_label,
                session.source_url,
                session.external_survey_id,
                session.status.as_str(),
                session.language,
                session.category,
                session.fingerprint,
                session.notes,
                session.created_at,
                session.updated_at,
                session.archived_at,
            ],
        )
        .map_err(|e| AppError::database(format!("Gagal membuat sesi survei: {e}")))?;

        Ok(session.clone())
    }

    /// Mengambil sesi survei berdasarkan ID dan persona_id
    pub fn get_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Option<SurveySession>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, persona_id, title, platform_label, source_url, external_survey_id,
                        status, language, category, fingerprint, notes, created_at, updated_at, archived_at
                 FROM survey_sessions
                 WHERE id = ?1 AND persona_id = ?2",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let session = stmt
            .query_row(params![session_id, persona_id], |row| {
                let status_str: String = row.get(6)?;
                Ok(SurveySession {
                    id: row.get(0)?,
                    persona_id: row.get(1)?,
                    title: row.get(2)?,
                    platform_label: row.get(3)?,
                    source_url: row.get(4)?,
                    external_survey_id: row.get(5)?,
                    status: SessionStatus::from_str_loose(&status_str),
                    language: row.get(7)?,
                    category: row.get(8)?,
                    fingerprint: row.get(9)?,
                    notes: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    archived_at: row.get(13)?,
                })
            })
            .optional()
            .map_err(|e| AppError::database(format!("Gagal mengambil sesi survei: {e}")))?;

        Ok(session)
    }

    /// Mengambil daftar ringkasan sesi survei untuk persona tertentu
    pub fn list_sessions(
        conn: &Connection,
        persona_id: &str,
        include_archived: bool,
    ) -> Result<Vec<SurveySessionSummary>, AppError> {
        let sql = if include_archived {
            "SELECT s.id, s.persona_id, s.title, s.platform_label, s.source_url, s.status,
                    s.language, s.category, s.created_at, s.updated_at, s.archived_at,
                    (SELECT COUNT(*) FROM survey_questions q WHERE q.session_id = s.id) as question_count,
                    (SELECT COUNT(*) FROM survey_questions q WHERE q.session_id = s.id AND q.confirmation_status = 'confirmed') as confirmed_count
             FROM survey_sessions s
             WHERE s.persona_id = ?1
             ORDER BY s.updated_at DESC"
        } else {
            "SELECT s.id, s.persona_id, s.title, s.platform_label, s.source_url, s.status,
                    s.language, s.category, s.created_at, s.updated_at, s.archived_at,
                    (SELECT COUNT(*) FROM survey_questions q WHERE q.session_id = s.id) as question_count,
                    (SELECT COUNT(*) FROM survey_questions q WHERE q.session_id = s.id AND q.confirmation_status = 'confirmed') as confirmed_count
             FROM survey_sessions s
             WHERE s.persona_id = ?1 AND s.status != 'archived'
             ORDER BY s.updated_at DESC"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(params![persona_id], |row| {
                let status_str: String = row.get(5)?;
                Ok(SurveySessionSummary {
                    id: row.get(0)?,
                    persona_id: row.get(1)?,
                    title: row.get(2)?,
                    platform_label: row.get(3)?,
                    source_url: row.get(4)?,
                    status: SessionStatus::from_str_loose(&status_str),
                    language: row.get(6)?,
                    category: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    archived_at: row.get(10)?,
                    question_count: row.get(11)?,
                    confirmed_answer_count: row.get(12)?,
                })
            })
            .map_err(|e| AppError::database(format!("Gagal mengambil daftar sesi survei: {e}")))?;

        let mut summaries = Vec::new();
        for row in rows {
            summaries.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        Ok(summaries)
    }

    /// Memperbarui informasi sesi survei
    pub fn update_session(
        conn: &Connection,
        session: &SurveySession,
    ) -> Result<SurveySession, AppError> {
        let count = conn
            .execute(
                "UPDATE survey_sessions SET
                    title = ?1,
                    platform_label = ?2,
                    source_url = ?3,
                    external_survey_id = ?4,
                    status = ?5,
                    language = ?6,
                    category = ?7,
                    fingerprint = ?8,
                    notes = ?9,
                    updated_at = ?10,
                    archived_at = ?11
                 WHERE id = ?12 AND persona_id = ?13",
                params![
                    session.title,
                    session.platform_label,
                    session.source_url,
                    session.external_survey_id,
                    session.status.as_str(),
                    session.language,
                    session.category,
                    session.fingerprint,
                    session.notes,
                    session.updated_at,
                    session.archived_at,
                    session.id,
                    session.persona_id,
                ],
            )
            .map_err(|e| AppError::database(format!("Gagal memperbarui sesi survei: {e}")))?;

        if count == 0 {
            return Err(AppError::not_found(
                "Sesi survei tidak ditemukan atau bukan milik persona ini".to_string(),
            ));
        }

        Ok(session.clone())
    }

    /// Mengubah status sesi survei (active, archived, completed)
    pub fn set_session_status(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
        status: SessionStatus,
        archived_at: Option<&str>,
        updated_at: &str,
    ) -> Result<(), AppError> {
        let count = conn
            .execute(
                "UPDATE survey_sessions SET
                    status = ?1,
                    archived_at = ?2,
                    updated_at = ?3
                 WHERE id = ?4 AND persona_id = ?5",
                params![
                    status.as_str(),
                    archived_at,
                    updated_at,
                    session_id,
                    persona_id
                ],
            )
            .map_err(|e| AppError::database(format!("Gagal mengubah status sesi: {e}")))?;

        if count == 0 {
            return Err(AppError::not_found(
                "Sesi survei tidak ditemukan atau bukan milik persona ini".to_string(),
            ));
        }

        Ok(())
    }

    /// Menghapus sesi survei dan seluruh entitas anaknya (cascade)
    pub fn delete_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<bool, AppError> {
        let count = conn
            .execute(
                "DELETE FROM survey_sessions WHERE id = ?1 AND persona_id = ?2",
                params![session_id, persona_id],
            )
            .map_err(|e| AppError::database(format!("Gagal menghapus sesi survei: {e}")))?;

        Ok(count > 0)
    }

    // ==========================================
    // SURVEY QUESTIONS CRUD & PERSISTENCE
    // ==========================================

    /// Menghitung nomor urut pertanyaan berikutnya dalam sesi
    pub fn get_next_question_order(conn: &Connection, session_id: &str) -> Result<i64, AppError> {
        let max_order: Option<i64> = conn
            .query_row(
                "SELECT MAX(question_order) FROM survey_questions WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| {
                AppError::database(format!("Gagal mengambil nomor urut pertanyaan: {e}"))
            })?;

        Ok(max_order.unwrap_or(0) + 1)
    }

    /// Menyimpan pertanyaan baru ke basis data
    pub fn create_question(
        conn: &Connection,
        question: &SurveyQuestion,
    ) -> Result<SurveyQuestion, AppError> {
        let options_json =
            serde_json::to_string(&question.options).unwrap_or_else(|_| "[]".to_string());
        let structure_json = question
            .structure_json
            .as_ref()
            .map(|s| serde_json::to_string(s).unwrap_or_default());

        conn.execute(
            "INSERT INTO survey_questions (
                id, session_id, persona_id, capture_id, question_order,
                question_text_original, question_text_corrected, instructions,
                options_json, structure_json, extraction_status, question_type,
                answer_format, provenance, observation_type, confirmation_status,
                user_answer_draft, user_answer_confirmed, confirmed_at,
                preceding_question_id, branch_logic, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
            )",
            params![
                question.id,
                question.session_id,
                question.persona_id,
                question.capture_id,
                question.question_order,
                question.question_text_original,
                question.question_text_corrected,
                question.instructions,
                options_json,
                structure_json,
                question.extraction_status,
                question.question_type.as_str(),
                question.answer_format.as_str(),
                question.provenance.as_str(),
                question.observation_type.as_str(),
                question.confirmation_status.as_str(),
                question.user_answer_draft,
                question.user_answer_confirmed,
                question.confirmed_at,
                question.preceding_question_id,
                question.branch_logic,
                question.created_at,
                question.updated_at,
            ],
        )
        .map_err(|e| AppError::database(format!("Gagal menyimpan pertanyaan survei: {e}")))?;

        // Update session's updated_at timestamp
        let _ = conn.execute(
            "UPDATE survey_sessions SET updated_at = ?1 WHERE id = ?2 AND persona_id = ?3",
            params![
                question.updated_at,
                question.session_id,
                question.persona_id
            ],
        );

        Ok(question.clone())
    }

    /// Mengambil pertanyaan survei berdasarkan ID dan persona_id
    pub fn get_question(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
    ) -> Result<Option<SurveyQuestion>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, capture_id, question_order,
                        question_text_original, question_text_corrected, instructions,
                        options_json, structure_json, extraction_status, question_type,
                        answer_format, provenance, observation_type, confirmation_status,
                        user_answer_draft, user_answer_confirmed, confirmed_at,
                        preceding_question_id, branch_logic, created_at, updated_at
                 FROM survey_questions
                 WHERE id = ?1 AND persona_id = ?2",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let question = stmt
            .query_row(params![question_id, persona_id], Self::map_question_row)
            .optional()
            .map_err(|e| AppError::database(format!("Gagal mengambil pertanyaan survei: {e}")))?;

        Ok(question)
    }

    /// Mengambil daftar seluruh pertanyaan dalam sesi terurut secara sekuensial
    pub fn list_questions_by_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyQuestion>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, capture_id, question_order,
                        question_text_original, question_text_corrected, instructions,
                        options_json, structure_json, extraction_status, question_type,
                        answer_format, provenance, observation_type, confirmation_status,
                        user_answer_draft, user_answer_confirmed, confirmed_at,
                        preceding_question_id, branch_logic, created_at, updated_at
                 FROM survey_questions
                 WHERE session_id = ?1 AND persona_id = ?2
                 ORDER BY question_order ASC, created_at ASC",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(params![session_id, persona_id], Self::map_question_row)
            .map_err(|e| {
                AppError::database(format!("Gagal mengambil daftar pertanyaan sesi: {e}"))
            })?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        Ok(questions)
    }

    /// Memperbarui atribut pertanyaan survei
    pub fn update_question(
        conn: &Connection,
        question: &SurveyQuestion,
    ) -> Result<SurveyQuestion, AppError> {
        let options_json =
            serde_json::to_string(&question.options).unwrap_or_else(|_| "[]".to_string());
        let structure_json = question
            .structure_json
            .as_ref()
            .map(|s| serde_json::to_string(s).unwrap_or_default());

        let count = conn
            .execute(
                "UPDATE survey_questions SET
                    question_text_original = ?1,
                    question_text_corrected = ?2,
                    instructions = ?3,
                    options_json = ?4,
                    structure_json = ?5,
                    question_type = ?6,
                    answer_format = ?7,
                    confirmation_status = ?8,
                    user_answer_draft = ?9,
                    user_answer_confirmed = ?10,
                    confirmed_at = ?11,
                    preceding_question_id = ?12,
                    branch_logic = ?13,
                    question_order = ?14,
                    updated_at = ?15
                 WHERE id = ?16 AND persona_id = ?17",
                params![
                    question.question_text_original,
                    question.question_text_corrected,
                    question.instructions,
                    options_json,
                    structure_json,
                    question.question_type.as_str(),
                    question.answer_format.as_str(),
                    question.confirmation_status.as_str(),
                    question.user_answer_draft,
                    question.user_answer_confirmed,
                    question.confirmed_at,
                    question.preceding_question_id,
                    question.branch_logic,
                    question.question_order,
                    question.updated_at,
                    question.id,
                    question.persona_id,
                ],
            )
            .map_err(|e| AppError::database(format!("Gagal memperbarui pertanyaan survei: {e}")))?;

        if count == 0 {
            return Err(AppError::not_found(
                "Pertanyaan survei tidak ditemukan atau bukan milik persona ini".to_string(),
            ));
        }

        // Update session's updated_at
        let _ = conn.execute(
            "UPDATE survey_sessions SET updated_at = ?1 WHERE id = ?2 AND persona_id = ?3",
            params![
                question.updated_at,
                question.session_id,
                question.persona_id
            ],
        );

        Ok(question.clone())
    }

    /// Menyimpan draf jawaban pengguna secara langsung (auto-save)
    pub fn save_question_draft(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
        draft_text: &str,
        updated_at: &str,
    ) -> Result<(), AppError> {
        let count = conn
            .execute(
                "UPDATE survey_questions SET
                    user_answer_draft = ?1,
                    updated_at = ?2
                 WHERE id = ?3 AND persona_id = ?4",
                params![draft_text, updated_at, question_id, persona_id],
            )
            .map_err(|e| AppError::database(format!("Gagal menyimpan draf pertanyaan: {e}")))?;

        if count == 0 {
            return Err(AppError::not_found(
                "Pertanyaan survei tidak ditemukan atau bukan milik persona ini".to_string(),
            ));
        }

        Ok(())
    }

    /// Mengkonfirmasi jawaban aktual pengguna secara terpisah dari saran AI
    pub fn confirm_question_answer(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
        confirmed_answer: &str,
        confirmed_at: &str,
        updated_at: &str,
    ) -> Result<(), AppError> {
        let count = conn
            .execute(
                "UPDATE survey_questions SET
                    user_answer_confirmed = ?1,
                    confirmation_status = 'confirmed',
                    confirmed_at = ?2,
                    updated_at = ?3
                 WHERE id = ?4 AND persona_id = ?5",
                params![
                    confirmed_answer,
                    confirmed_at,
                    updated_at,
                    question_id,
                    persona_id
                ],
            )
            .map_err(|e| {
                AppError::database(format!("Gagal mengkonfirmasi jawaban pertanyaan: {e}"))
            })?;

        if count == 0 {
            return Err(AppError::not_found(
                "Pertanyaan survei tidak ditemukan atau bukan milik persona ini".to_string(),
            ));
        }

        Ok(())
    }

    /// Menghapus pertanyaan survei
    pub fn delete_question(
        conn: &Connection,
        persona_id: &str,
        question_id: &str,
    ) -> Result<bool, AppError> {
        let count = conn
            .execute(
                "DELETE FROM survey_questions WHERE id = ?1 AND persona_id = ?2",
                params![question_id, persona_id],
            )
            .map_err(|e| AppError::database(format!("Gagal menghapus pertanyaan survei: {e}")))?;

        Ok(count > 0)
    }

    /// Mengambil detail sesi survei beserta seluruh daftar pertanyaannya
    pub fn get_session_detail(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Option<SurveySessionDetail>, AppError> {
        let session = match Self::get_session(conn, persona_id, session_id)? {
            Some(s) => s,
            None => return Ok(None),
        };

        let questions = Self::list_questions_by_session(conn, persona_id, session_id)?;

        Ok(Some(SurveySessionDetail { session, questions }))
    }

    // ==========================================
    // HELPER MAPPING
    // ==========================================

    fn map_question_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SurveyQuestion> {
        let options_json: String = row.get(8)?;
        let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();

        let structure_raw: Option<String> = row.get(9)?;
        let structure_json: Option<serde_json::Value> =
            structure_raw.and_then(|s| serde_json::from_str(&s).ok());

        let qtype_str: String = row.get(11)?;
        let aformat_str: String = row.get(12)?;
        let prov_str: String = row.get(13)?;
        let obs_str: String = row.get(14)?;
        let conf_str: String = row.get(15)?;

        Ok(SurveyQuestion {
            id: row.get(0)?,
            session_id: row.get(1)?,
            persona_id: row.get(2)?,
            capture_id: row.get(3)?,
            question_order: row.get(4)?,
            question_text_original: row.get(5)?,
            question_text_corrected: row.get(6)?,
            instructions: row.get(7)?,
            options,
            structure_json,
            extraction_status: row.get(10)?,
            question_type: QuestionType::from_str_loose(&qtype_str),
            answer_format: AnswerFormat::from_str_loose(&aformat_str),
            provenance: Provenance::from_str_loose(&prov_str),
            observation_type: QuestionStatus::from_str_loose(&obs_str),
            confirmation_status: ConfirmationStatus::from_str_loose(&conf_str),
            user_answer_draft: row.get(16)?,
            user_answer_confirmed: row.get(17)?,
            confirmed_at: row.get(18)?,
            preceding_question_id: row.get(19)?,
            branch_logic: row.get(20)?,
            created_at: row.get(21)?,
            updated_at: row.get(22)?,
        })
    }
}
