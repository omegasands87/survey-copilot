use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::error::AppError;
use crate::domain::survey::{
    AnswerFormat, ConfirmationStatus, PredictionStatus, Provenance, QuestionStatus, QuestionType,
    SessionStatus, SurveyBranchTransition, SurveyMemoryQuestionItem, SurveyMemorySessionDetail,
    SurveyQuestion, SurveyQuestionPredictionRecord, SurveyRecognitionRecord, SurveyRecognitionStatus,
    SurveySession, SurveySessionDetail, SurveySessionSummary, TransitionType,
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
                id, persona_id, title, platform_label, source_url, source_domain, external_survey_id,
                status, language, category, fingerprint, version, recognized_from_id, notes, created_at, updated_at, archived_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                session.id,
                session.persona_id,
                session.title,
                session.platform_label,
                session.source_url,
                session.source_domain,
                session.external_survey_id,
                session.status.as_str(),
                session.language,
                session.category,
                session.fingerprint,
                session.version,
                session.recognized_from_id,
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
                "SELECT id, persona_id, title, platform_label, source_url, source_domain, external_survey_id,
                        status, language, category, fingerprint, version, recognized_from_id, notes, created_at, updated_at, archived_at
                 FROM survey_sessions
                 WHERE id = ?1 AND persona_id = ?2",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let session = stmt
            .query_row(params![session_id, persona_id], |row| {
                let status_str: String = row.get(7)?;
                Ok(SurveySession {
                    id: row.get(0)?,
                    persona_id: row.get(1)?,
                    title: row.get(2)?,
                    platform_label: row.get(3)?,
                    source_url: row.get(4)?,
                    source_domain: row.get(5)?,
                    external_survey_id: row.get(6)?,
                    status: SessionStatus::from_str_loose(&status_str),
                    language: row.get(8)?,
                    category: row.get(9)?,
                    fingerprint: row.get(10)?,
                    version: row.get(11)?,
                    recognized_from_id: row.get(12)?,
                    notes: row.get(13)?,
                    created_at: row.get(14)?,
                    updated_at: row.get(15)?,
                    archived_at: row.get(16)?,
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
            "SELECT s.id, s.persona_id, s.title, s.platform_label, s.source_url, s.source_domain, s.status,
                    s.language, s.category, s.fingerprint, s.version, s.created_at, s.updated_at, s.archived_at,
                    (SELECT COUNT(*) FROM survey_questions q WHERE q.session_id = s.id) as question_count,
                    (SELECT COUNT(*) FROM survey_questions q WHERE q.session_id = s.id AND q.confirmation_status = 'confirmed') as confirmed_count
             FROM survey_sessions s
             WHERE s.persona_id = ?1
             ORDER BY s.updated_at DESC"
        } else {
            "SELECT s.id, s.persona_id, s.title, s.platform_label, s.source_url, s.source_domain, s.status,
                    s.language, s.category, s.fingerprint, s.version, s.created_at, s.updated_at, s.archived_at,
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
                let status_str: String = row.get(6)?;
                Ok(SurveySessionSummary {
                    id: row.get(0)?,
                    persona_id: row.get(1)?,
                    title: row.get(2)?,
                    platform_label: row.get(3)?,
                    source_url: row.get(4)?,
                    source_domain: row.get(5)?,
                    status: SessionStatus::from_str_loose(&status_str),
                    language: row.get(7)?,
                    category: row.get(8)?,
                    fingerprint: row.get(9)?,
                    version: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    archived_at: row.get(13)?,
                    question_count: row.get(14)?,
                    confirmed_answer_count: row.get(15)?,
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
                    source_domain = ?4,
                    external_survey_id = ?5,
                    status = ?6,
                    language = ?7,
                    category = ?8,
                    fingerprint = ?9,
                    version = ?10,
                    recognized_from_id = ?11,
                    notes = ?12,
                    updated_at = ?13,
                    archived_at = ?14
                 WHERE id = ?15 AND persona_id = ?16",
                params![
                    session.title,
                    session.platform_label,
                    session.source_url,
                    session.source_domain,
                    session.external_survey_id,
                    session.status.as_str(),
                    session.language,
                    session.category,
                    session.fingerprint,
                    session.version,
                    session.recognized_from_id,
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

    /// Menghitung nomor urut pertanyaan berikutnya dalam sesi dengan isolasi persona ketat
    pub fn get_next_question_order(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<i64, AppError> {
        let max_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(question_order), 0) FROM survey_questions WHERE session_id = ?1 AND persona_id = ?2",
                params![session_id, persona_id],
                |row| row.get(0),
            )
            .map_err(|e| {
                AppError::database(format!("Gagal mengambil nomor urut pertanyaan: {e}"))
            })?;

        Ok(max_order + 1)
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
                preceding_question_id, branch_logic, question_fingerprint, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24
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
                question.question_fingerprint,
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
                        preceding_question_id, branch_logic, question_fingerprint, created_at, updated_at
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
                        preceding_question_id, branch_logic, question_fingerprint, created_at, updated_at
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
                    question_fingerprint = ?15,
                    updated_at = ?16
                 WHERE id = ?17 AND persona_id = ?18",
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
                    question.question_fingerprint,
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

        // Update session's updated_at timestamp
        let _ = conn.execute(
            "UPDATE survey_sessions SET updated_at = ?1 WHERE id = (SELECT session_id FROM survey_questions WHERE id = ?2) AND persona_id = ?3",
            params![updated_at, question_id, persona_id],
        );

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

        // Update session's updated_at timestamp
        let _ = conn.execute(
            "UPDATE survey_sessions SET updated_at = ?1 WHERE id = (SELECT session_id FROM survey_questions WHERE id = ?2) AND persona_id = ?3",
            params![updated_at, question_id, persona_id],
        );

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

    /// Mengambil seluruh pertanyaan survei yang telah dikonfirmasi jawabannya oleh persona ini.
    /// Wajib mengisolasi data secara ketat menggunakan WHERE persona_id = ?1.
    pub fn list_confirmed_answers_by_persona(
        conn: &Connection,
        persona_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<SurveyQuestion>, AppError> {
        let max_limit = limit.unwrap_or(200);
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, capture_id, question_order,
                        question_text_original, question_text_corrected, instructions,
                        options_json, structure_json, extraction_status, question_type,
                        answer_format, provenance, observation_type, confirmation_status,
                        user_answer_draft, user_answer_confirmed, confirmed_at,
                        preceding_question_id, branch_logic, question_fingerprint, created_at, updated_at
                 FROM survey_questions
                 WHERE persona_id = ?1
                   AND confirmation_status = 'confirmed'
                   AND user_answer_confirmed IS NOT NULL
                 ORDER BY confirmed_at DESC, created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(
                params![persona_id, max_limit as i64],
                Self::map_question_row,
            )
            .map_err(|e| {
                AppError::database(format!(
                    "Gagal mengambil riwayat jawaban terkonfirmasi: {e}"
                ))
            })?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        Ok(questions)
    }

    // ==========================================
    // SC-053: SURVEY MEMORY & RECOGNITION CRUD
    // ==========================================

    /// Memperbarui sidik jari struktural, nomor versi, dan referensi asal sesi
    pub fn update_session_fingerprint(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
        fingerprint: &str,
        version: i64,
        recognized_from_id: Option<&str>,
        updated_at: &str,
    ) -> Result<(), AppError> {
        let count = conn
            .execute(
                "UPDATE survey_sessions SET
                    fingerprint = ?1,
                    version = ?2,
                    recognized_from_id = ?3,
                    updated_at = ?4
                 WHERE id = ?5 AND persona_id = ?6",
                params![
                    fingerprint,
                    version,
                    recognized_from_id,
                    updated_at,
                    session_id,
                    persona_id
                ],
            )
            .map_err(|e| AppError::database(format!("Gagal memperbarui sidik jari sesi: {e}")))?;

        if count == 0 {
            return Err(AppError::not_found(
                "Sesi survei tidak ditemukan atau bukan milik persona ini".to_string(),
            ));
        }

        Ok(())
    }

    /// Menyimpan catatan hasil pengenalan survei untuk audit dan review
    pub fn save_recognition(
        conn: &Connection,
        rec: &SurveyRecognitionRecord,
    ) -> Result<SurveyRecognitionRecord, AppError> {
        let diffs_json = serde_json::to_string(&rec.differences).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO survey_recognitions (
                id, session_id, persona_id, matched_session_id, match_status,
                confidence_score, structural_similarity, title_similarity,
                matched_question_count, total_question_count, differences_json,
                provenance, recognized_at, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                rec.id,
                rec.session_id,
                rec.persona_id,
                rec.matched_session_id,
                rec.match_status.as_str(),
                rec.confidence_score,
                rec.structural_similarity,
                rec.title_similarity,
                rec.matched_question_count,
                rec.total_question_count,
                diffs_json,
                rec.provenance,
                rec.recognized_at,
                rec.created_at,
            ],
        )
        .map_err(|e| AppError::database(format!("Gagal menyimpan catatan pengenalan survei: {e}")))?;

        Ok(rec.clone())
    }

    /// Mengambil catatan pengenalan survei terbaru untuk sesi tertentu
    pub fn get_latest_recognition(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Option<SurveyRecognitionRecord>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, matched_session_id, match_status,
                        confidence_score, structural_similarity, title_similarity,
                        matched_question_count, total_question_count, differences_json,
                        provenance, recognized_at, created_at
                 FROM survey_recognitions
                 WHERE session_id = ?1 AND persona_id = ?2
                 ORDER BY created_at DESC
                 LIMIT 1",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rec = stmt
            .query_row(params![session_id, persona_id], |row| {
                let status_str: String = row.get(4)?;
                let diffs_json: String = row.get(10)?;
                let differences: Vec<String> = serde_json::from_str(&diffs_json).unwrap_or_default();
                Ok(SurveyRecognitionRecord {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    persona_id: row.get(2)?,
                    matched_session_id: row.get(3)?,
                    match_status: SurveyRecognitionStatus::from_str_loose(&status_str),
                    confidence_score: row.get(5)?,
                    structural_similarity: row.get(6)?,
                    title_similarity: row.get(7)?,
                    matched_question_count: row.get(8)?,
                    total_question_count: row.get(9)?,
                    differences,
                    provenance: row.get(11)?,
                    recognized_at: row.get(12)?,
                    created_at: row.get(13)?,
                })
            })
            .optional()
            .map_err(|e| AppError::database(format!("Gagal mengambil catatan pengenalan: {e}")))?;

        Ok(rec)
    }

    /// Mengambil daftar seluruh sesi historis yang berpotensi dicocokkan (eksklusif untuk persona_id yang sama, AT-066)
    pub fn list_candidate_sessions_for_recognition(
        conn: &Connection,
        persona_id: &str,
        exclude_session_id: &str,
    ) -> Result<Vec<SurveySessionDetail>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id FROM survey_sessions
                 WHERE persona_id = ?1 AND id != ?2 AND status != 'archived'
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let session_ids: Vec<String> = stmt
            .query_map(params![persona_id, exclude_session_id], |row| row.get(0))
            .map_err(|e| AppError::database(e.to_string()))?
            .filter_map(Result::ok)
            .collect();

        let mut details = Vec::new();
        for s_id in session_ids {
            if let Some(detail) = Self::get_session_detail(conn, persona_id, &s_id)? {
                details.push(detail);
            }
        }

        Ok(details)
    }

    /// Mengambil detail Survey Memory untuk sesi survei dengan pemisahan mutlak
    /// antara user_answer_confirmed (jawaban aktual) dan rekomendasi saran AI (AT-064)
    pub fn get_memory_session_detail(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Option<SurveyMemorySessionDetail>, AppError> {
        let session = match Self::get_session(conn, persona_id, session_id)? {
            Some(s) => s,
            None => return Ok(None),
        };

        // Query pertanyaan beserta rekomendasi AI terbaru yang terkait (LEFT JOIN)
        let mut stmt = conn
            .prepare(
                "SELECT q.id, q.session_id, q.persona_id, q.question_order,
                        q.question_text_original, q.instructions, q.options_json,
                        q.question_type, q.answer_format, q.provenance,
                        q.user_answer_confirmed, q.confirmed_at, q.user_answer_draft,
                        q.question_fingerprint,
                        ar.draft_text, ar.confidence, ar.rationale
                 FROM survey_questions q
                 LEFT JOIN (
                     SELECT r1.*
                     FROM answer_recommendations r1
                     INNER JOIN (
                         SELECT question_id, MAX(created_at) as max_created
                         FROM answer_recommendations
                         GROUP BY question_id
                     ) r2 ON r1.question_id = r2.question_id AND r1.created_at = r2.max_created
                 ) ar ON ar.question_id = q.id
                 WHERE q.session_id = ?1 AND q.persona_id = ?2
                 ORDER BY q.question_order ASC, q.created_at ASC",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(params![session_id, persona_id], |row| {
                let options_json: String = row.get(6)?;
                let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
                let qtype_str: String = row.get(7)?;
                let aformat_str: String = row.get(8)?;
                let prov_str: String = row.get(9)?;

                Ok(SurveyMemoryQuestionItem {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    persona_id: row.get(2)?,
                    question_order: row.get(3)?,
                    prompt_text: row.get(4)?,
                    instructions: row.get(5)?,
                    options,
                    question_type: QuestionType::from_str_loose(&qtype_str),
                    answer_format: AnswerFormat::from_str_loose(&aformat_str),
                    provenance: Provenance::from_str_loose(&prov_str),
                    user_answer_confirmed: row.get(10)?,
                    confirmed_at: row.get(11)?,
                    user_answer_draft: row.get(12)?,
                    question_fingerprint: row.get(13)?,
                    ai_recommendation_draft: row.get(14)?,
                    ai_confidence: row.get(15)?,
                    ai_rationale: row.get(16)?,
                })
            })
            .map_err(|e| AppError::database(format!("Gagal mengambil pertanyaan memori survei: {e}")))?;

        let mut memory_questions = Vec::new();
        for row in rows {
            memory_questions.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        let latest_recognition = Self::get_latest_recognition(conn, persona_id, session_id)?;

        Ok(Some(SurveyMemorySessionDetail {
            session,
            questions: memory_questions,
            latest_recognition,
        }))
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
            question_fingerprint: row.get(21)?,
            created_at: row.get(22)?,
            updated_at: row.get(23)?,
        })
    }

    // ==========================================
    // SC-054: BRANCH TRANSITIONS & PREDICTIONS
    // ==========================================

    /// Menyimpan atau mencatat transisi percabangan baru yang diobservasi
    pub fn record_branch_transition(
        conn: &Connection,
        transition: &SurveyBranchTransition,
    ) -> Result<(), AppError> {
        let options_json = serde_json::to_string(&transition.to_options)
            .unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO survey_branch_transitions (
                id, session_id, persona_id, from_question_id, from_question_fingerprint,
                from_question_order, from_answer_value, to_question_id, to_question_fingerprint,
                to_question_text, to_question_type, to_options_json, to_question_order,
                branch_condition, transition_type, observed_at, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                transition.id,
                transition.session_id,
                transition.persona_id,
                transition.from_question_id,
                transition.from_question_fingerprint,
                transition.from_question_order,
                transition.from_answer_value,
                transition.to_question_id,
                transition.to_question_fingerprint,
                transition.to_question_text,
                transition.to_question_type.as_str(),
                options_json,
                transition.to_question_order,
                transition.branch_condition,
                transition.transition_type.as_str(),
                transition.observed_at,
                transition.created_at,
            ],
        )
        .map_err(|e| AppError::database(format!("Gagal mencatat transisi percabangan: {e}")))?;

        Ok(())
    }

    /// Mengambil seluruh transisi percabangan yang diamati pada satu sesi survei
    pub fn list_branch_transitions_by_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyBranchTransition>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, from_question_id, from_question_fingerprint,
                        from_question_order, from_answer_value, to_question_id, to_question_fingerprint,
                        to_question_text, to_question_type, to_options_json, to_question_order,
                        branch_condition, transition_type, observed_at, created_at
                 FROM survey_branch_transitions
                 WHERE session_id = ?1 AND persona_id = ?2
                 ORDER BY from_question_order ASC, to_question_order ASC",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(params![session_id, persona_id], Self::map_branch_transition_row)
            .map_err(|e| AppError::database(format!("Gagal membaca transisi percabangan sesi: {e}")))?;

        let mut transitions = Vec::new();
        for row in rows {
            transitions.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        Ok(transitions)
    }

    /// Mengambil transisi yang pernah diobservasi dari pertanyaan tertentu (berdasarkan ID atau sidik jari)
    pub fn get_transitions_for_question(
        conn: &Connection,
        persona_id: &str,
        from_question_id: &str,
        from_question_fingerprint: Option<&str>,
    ) -> Result<Vec<SurveyBranchTransition>, AppError> {
        let fp = from_question_fingerprint.unwrap_or("");
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, from_question_id, from_question_fingerprint,
                        from_question_order, from_answer_value, to_question_id, to_question_fingerprint,
                        to_question_text, to_question_type, to_options_json, to_question_order,
                        branch_condition, transition_type, observed_at, created_at
                 FROM survey_branch_transitions
                 WHERE persona_id = ?1 AND (from_question_id = ?2 OR (from_question_fingerprint = ?3 AND ?3 != ''))
                 ORDER BY observed_at DESC",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(params![persona_id, from_question_id, fp], Self::map_branch_transition_row)
            .map_err(|e| AppError::database(format!("Gagal mencari transisi pertanyaan: {e}")))?;

        let mut transitions = Vec::new();
        for row in rows {
            transitions.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        Ok(transitions)
    }

    /// Mengambil transisi struktural dari sidik jari pertanyaan tanpa data jawaban pribadi (cross-persona structural reference)
    pub fn get_structural_transitions_for_fingerprint(
        conn: &Connection,
        from_question_fingerprint: &str,
    ) -> Result<Vec<SurveyBranchTransition>, AppError> {
        if from_question_fingerprint.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, from_question_id, from_question_fingerprint,
                        from_question_order, NULL as from_answer_value, to_question_id, to_question_fingerprint,
                        to_question_text, to_question_type, to_options_json, to_question_order,
                        branch_condition, transition_type, observed_at, created_at
                 FROM survey_branch_transitions
                 WHERE from_question_fingerprint = ?1
                 ORDER BY observed_at DESC",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(params![from_question_fingerprint], Self::map_branch_transition_row)
            .map_err(|e| AppError::database(format!("Gagal mencari transisi struktural: {e}")))?;

        let mut transitions = Vec::new();
        for row in rows {
            transitions.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        Ok(transitions)
    }

    /// Menyimpan catatan rekaman prediksi pertanyaan ke audit log
    pub fn save_prediction_record(
        conn: &Connection,
        record: &SurveyQuestionPredictionRecord,
    ) -> Result<(), AppError> {
        let options_json = serde_json::to_string(&record.predicted_options)
            .unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO survey_question_predictions (
                id, session_id, persona_id, current_question_id, predicted_question_text,
                predicted_question_type, predicted_options_json, confidence_score, evidence_source,
                explanation, leading_condition, status, matched_question_id, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                record.id,
                record.session_id,
                record.persona_id,
                record.current_question_id,
                record.predicted_question_text,
                record.predicted_question_type.as_str(),
                options_json,
                record.confidence_score,
                record.evidence_source,
                record.explanation,
                record.leading_condition,
                record.status.as_str(),
                record.matched_question_id,
                record.created_at,
                record.updated_at,
            ],
        )
        .map_err(|e| AppError::database(format!("Gagal menyimpan rekaman prediksi: {e}")))?;

        Ok(())
    }

    /// Mengambil daftar riwayat prediksi untuk sesi survei
    pub fn list_predictions_by_session(
        conn: &Connection,
        persona_id: &str,
        session_id: &str,
    ) -> Result<Vec<SurveyQuestionPredictionRecord>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, current_question_id, predicted_question_text,
                        predicted_question_type, predicted_options_json, confidence_score, evidence_source,
                        explanation, leading_condition, status, matched_question_id, created_at, updated_at
                 FROM survey_question_predictions
                 WHERE session_id = ?1 AND persona_id = ?2
                 ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(params![session_id, persona_id], Self::map_prediction_record_row)
            .map_err(|e| AppError::database(format!("Gagal mengambil riwayat prediksi: {e}")))?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        Ok(records)
    }

    /// Mengambil prediksi yang masih berstatus 'predicted_not_detected' untuk pertanyaan sebelumnya
    pub fn find_pending_predictions_for_question(
        conn: &Connection,
        persona_id: &str,
        current_question_id: &str,
    ) -> Result<Vec<SurveyQuestionPredictionRecord>, AppError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, persona_id, current_question_id, predicted_question_text,
                        predicted_question_type, predicted_options_json, confidence_score, evidence_source,
                        explanation, leading_condition, status, matched_question_id, created_at, updated_at
                 FROM survey_question_predictions
                 WHERE persona_id = ?1 AND current_question_id = ?2 AND status = 'predicted_not_detected'",
            )
            .map_err(|e| AppError::database(e.to_string()))?;

        let rows = stmt
            .query_map(params![persona_id, current_question_id], Self::map_prediction_record_row)
            .map_err(|e| AppError::database(format!("Gagal mengambil prediksi tertunda: {e}")))?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(|e| AppError::database(e.to_string()))?);
        }

        Ok(records)
    }

    /// Memperbarui status prediksi (misal: matched atau diverged saat pertanyaan aktual teramati)
    pub fn update_prediction_status(
        conn: &Connection,
        persona_id: &str,
        prediction_id: &str,
        new_status: PredictionStatus,
        matched_question_id: Option<&str>,
        updated_at: &str,
    ) -> Result<(), AppError> {
        conn.execute(
            "UPDATE survey_question_predictions SET
                status = ?1,
                matched_question_id = ?2,
                updated_at = ?3
             WHERE id = ?4 AND persona_id = ?5",
            params![
                new_status.as_str(),
                matched_question_id,
                updated_at,
                prediction_id,
                persona_id,
            ],
        )
        .map_err(|e| AppError::database(format!("Gagal memperbarui status prediksi: {e}")))?;

        Ok(())
    }

    fn map_branch_transition_row(row: &rusqlite::Row) -> rusqlite::Result<SurveyBranchTransition> {
        let q_type_str: String = row.get(10)?;
        let options_json: String = row.get(11)?;
        let to_options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
        let transition_type_str: String = row.get(14)?;

        Ok(SurveyBranchTransition {
            id: row.get(0)?,
            session_id: row.get(1)?,
            persona_id: row.get(2)?,
            from_question_id: row.get(3)?,
            from_question_fingerprint: row.get(4)?,
            from_question_order: row.get(5)?,
            from_answer_value: row.get(6)?,
            to_question_id: row.get(7)?,
            to_question_fingerprint: row.get(8)?,
            to_question_text: row.get(9)?,
            to_question_type: QuestionType::from_str_loose(&q_type_str),
            to_options,
            to_question_order: row.get(12)?,
            branch_condition: row.get(13)?,
            transition_type: TransitionType::from_str_loose(&transition_type_str),
            observed_at: row.get(15)?,
            created_at: row.get(16)?,
        })
    }

    fn map_prediction_record_row(row: &rusqlite::Row) -> rusqlite::Result<SurveyQuestionPredictionRecord> {
        let q_type_str: String = row.get(5)?;
        let options_json: String = row.get(6)?;
        let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
        let status_str: String = row.get(11)?;

        Ok(SurveyQuestionPredictionRecord {
            id: row.get(0)?,
            session_id: row.get(1)?,
            persona_id: row.get(2)?,
            current_question_id: row.get(3)?,
            predicted_question_text: row.get(4)?,
            predicted_question_type: QuestionType::from_str_loose(&q_type_str),
            predicted_options: options,
            confidence_score: row.get(7)?,
            evidence_source: row.get(8)?,
            explanation: row.get(9)?,
            leading_condition: row.get(10)?,
            status: PredictionStatus::from_str_loose(&status_str),
            matched_question_id: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }
}
