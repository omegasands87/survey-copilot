pub mod prompts;
pub mod validator;

use std::sync::Arc;
use uuid::Uuid;

use crate::db::Database;
use crate::domain::answer_engine::{
    AnswerHistoryItem, ConfirmRecommendationInput, CopyAnswerInput, EditAnswerInput,
    EvidenceStatus, GenerateAnswerInput, ListAnswerHistoryInput, RawAnswerRecommendation,
    RegenerateAnswerInput, RejectAnswerInput, ReviewAction, ReviewStatus,
    ValidatedAnswerRecommendation,
};
use crate::domain::classifier::ClassifyQuestionInput;
use crate::domain::error::AppError;
use crate::domain::evidence::{EvidenceConfidence, EvidenceRecommendation, EvidenceRetrievalInput};
use crate::domain::pool::CancellationToken;
use crate::domain::provider::NormalizedRequest;
use crate::infrastructure::credentials::CredentialStore;
use crate::infrastructure::http::HttpTransport;
use crate::repository::{AnswerRepository, PersonaRepository, SurveyRepository};
use crate::services::provider_pool::ProviderPoolService;
use crate::services::{EvidenceEngineService, QuestionClassifierService};

pub use prompts::PROMPT_VERSION_V1_0;
pub use validator::validate_recommendation;

pub struct AnswerEngineService;

impl AnswerEngineService {
    /// Menghasilkan rekomendasi jawaban terstruktur yang evidence-grounded
    /// Alur: Validasi Persona -> Klasifikasi Pertanyaan -> Retrieval Bukti -> AI Provider -> Local Validation -> Persistensi
    pub fn generate_recommendation(
        database: &Database,
        credential_store: &dyn CredentialStore,
        transport: Arc<dyn HttpTransport>,
        provider_pool: &ProviderPoolService,
        input: GenerateAnswerInput,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<ValidatedAnswerRecommendation, AppError> {
        let trimmed_text = input.question_text.trim();
        if trimmed_text.is_empty() {
            return Err(AppError::validation(
                "question_text",
                "Teks pertanyaan survei tidak boleh kosong",
            ));
        }

        let (classification, evidence, prompt_version) = {
            let conn = database
                .connection
                .lock()
                .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

            // 1. Verifikasi kepemilikan dan eksistensi Persona (Isolasi Persona Mutlak)
            let persona = PersonaRepository::find_by_id(&conn, &input.persona_id)
                .map_err(|e| AppError::database(format!("Gagal mencari persona: {e}")))?;
            match persona {
                Some(p) if p.status != "archived" => (),
                Some(_) => {
                    return Err(AppError::validation(
                        "persona_id",
                        "Persona ini telah diarsipkan dan tidak dapat digunakan untuk rekomendasi jawaban",
                    ));
                }
                None => return Err(AppError::not_found("Persona tidak ditemukan")),
            }

            // Jika session_id diberikan, pastikan sesi milik persona ini
            if let Some(ref sid) = input.session_id {
                if !sid.trim().is_empty() {
                    let session = SurveyRepository::get_session(&conn, &input.persona_id, sid)?
                        .ok_or_else(|| {
                            AppError::not_found(
                                "Sesi survei tidak ditemukan atau bukan milik persona aktif",
                            )
                        })?;
                    let _ = session;
                }
            }

            // Jika question_id diberikan, pastikan pertanyaan milik persona ini
            if let Some(ref qid) = input.question_id {
                if !qid.trim().is_empty() {
                    let question = SurveyRepository::get_question(&conn, &input.persona_id, qid)?
                        .ok_or_else(|| {
                        AppError::not_found(
                            "Pertanyaan survei tidak ditemukan atau bukan milik persona aktif",
                        )
                    })?;
                    let _ = question;
                }
            }

            // 2. SC-031 Klasifikasi Pertanyaan & Ekstraksi Batasan
            let classification = QuestionClassifierService::classify(&ClassifyQuestionInput {
                persona_id: Some(input.persona_id.clone()),
                session_id: input.session_id.clone(),
                question_id: input.question_id.clone(),
                question_text: trimmed_text.to_string(),
                instructions: input.instructions.clone(),
                options: input.options.clone(),
            });

            // 3. SC-032 Pengambilan Bukti & Evaluasi Konsistensi
            let evidence = EvidenceEngineService::evaluate_evidence(
                &conn,
                &EvidenceRetrievalInput {
                    persona_id: input.persona_id.clone(),
                    session_id: input.session_id.clone(),
                    question_id: input.question_id.clone(),
                    question_text: trimmed_text.to_string(),
                    instructions: input.instructions.clone(),
                    options: input.options.clone(),
                    classification: Some(classification.clone()),
                    consent: input.consent,
                    max_evidence_items: Some(10),
                },
            )?;

            let prompt_version = input
                .prompt_version
                .unwrap_or_else(|| PROMPT_VERSION_V1_0.to_string());

            (classification, evidence, prompt_version)
        };

        // 4. Konstruksi Prompt Terstruktur Versi 1.0 dengan Pertahanan Prompt Injection (AT-044)
        let system_instruction = prompts::build_system_instruction(&prompt_version);
        let user_payload = prompts::build_user_payload(
            &prompt_version,
            trimmed_text,
            input.instructions.as_deref(),
            input.options.as_deref(),
            Some(&classification),
            &evidence,
        );

        // 5. Normalisasi Request untuk AI Provider Pool (JSON Schema Output Enforced)
        let normalized_request = NormalizedRequest {
            request_id: Uuid::new_v4().to_string(),
            operation: "answer_engine_generate".to_string(),
            model_id: String::new(),
            system_instruction: Some(system_instruction),
            user_payload,
            response_schema: Some(prompts::get_structured_response_schema()),
            temperature: Some(0.1),
            max_tokens: Some(1024),
            timeout_ms: Some(30000),
        };

        // 6. Eksekusi melalui Provider Pool dengan Retry, Fallback, dan Sanitasi Kredensial
        let pool_result = provider_pool.execute_pooled_request(
            database,
            credential_store,
            transport,
            normalized_request,
            cancellation_token,
        );

        let validated_rec = if let (true, Some(resp)) = (pool_result.success, pool_result.response)
        {
            let raw_recommendation: RawAnswerRecommendation =
                if let Some(ref val) = resp.structured_json {
                    serde_json::from_value(val.clone()).unwrap_or_default()
                } else if let Some(ref text) = resp.raw_text {
                    serde_json::from_str(text).unwrap_or_else(|err| RawAnswerRecommendation {
                        status: "insufficient_information".to_string(),
                        rationale: Some(format!(
                            "Model menghasilkan output yang tidak dapat diparsing: {err}"
                        )),
                        ..Default::default()
                    })
                } else {
                    RawAnswerRecommendation::default()
                };

            // Validasi Lokal (Allowlist sitasi, tipe, schema)
            validate_recommendation(
                raw_recommendation,
                &evidence,
                &input.persona_id,
                input.question_id.clone(),
                input.session_id.clone(),
                &prompt_version,
                Some(resp.provider_id),
                Some(resp.model_id),
            )?
        } else {
            // GRACEFUL LOCAL FALLBACK (Provider gagal, kuota habis, atau offline)
            // Preservasi teks dan draf pengguna mutlak (AT-040 boundary)
            Self::create_deterministic_fallback_recommendation(
                &evidence,
                &classification,
                &input.persona_id,
                input.question_id.clone(),
                input.session_id.clone(),
                &prompt_version,
                input.options.as_deref(),
                pool_result.final_error.as_ref(),
            )
        };

        // 7. Simpan Rekomendasi ke Database SQLite
        {
            let conn = database
                .connection
                .lock()
                .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
            AnswerRepository::save_recommendation(&conn, &validated_rec)?;

            // 8. Catat log diagnostik teredaksi aman (SC-060)
            let _ = crate::repository::PrivacyRepository::insert_diagnostic_log(
                &conn,
                &crate::domain::privacy::DiagnosticsLogEntry {
                    id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    level: "INFO".to_string(),
                    category: "AI_PROVIDER".to_string(),
                    message: format!(
                        "Rekomendasi AI selesai diproses (status: {:?}, supporting_facts: {})",
                        validated_rec.status,
                        validated_rec.supporting_facts.len()
                    ),
                    metadata_json: Some(
                        serde_json::to_string(&serde_json::json!({
                            "recommendationId": validated_rec.id,
                            "personaId": validated_rec.persona_id,
                            "hasDraftAnswer": validated_rec.draft_answer.is_some(),
                        }))
                        .unwrap_or_default(),
                    ),
                },
            );
        }

        Ok(validated_rec)
    }

    /// Menghasilkan rekomendasi fallback lokal deterministik ketika AI provider tidak tersedia
    /// Memastikan sistem tetap evidence-grounded dan tidak pernah mengarang bebas
    #[allow(clippy::too_many_arguments)]
    fn create_deterministic_fallback_recommendation(
        evidence: &crate::domain::evidence::EvidencePackage,
        classification: &crate::domain::classifier::QuestionClassificationResult,
        persona_id: &str,
        question_id: Option<String>,
        session_id: Option<String>,
        prompt_version: &str,
        options: Option<&[String]>,
        error: Option<&crate::domain::provider::NormalizedError>,
    ) -> ValidatedAnswerRecommendation {
        let err_desc = error
            .map(|e| format!("{:?}: {}", e.category, e.message))
            .unwrap_or_else(|| "Provider AI sedang tidak tersedia".to_string());

        // Jika terdeteksi konflik kritis di data responden
        if evidence.recommendation == EvidenceRecommendation::NeedsClarification
            || !evidence.conflict_indicators.is_empty()
        {
            let conflict_messages = evidence
                .conflict_indicators
                .iter()
                .map(|c| format!("[{:?}] {}", c.conflict_type, c.description))
                .collect::<Vec<_>>();

            let prompt = evidence
                .conflict_indicators
                .first()
                .and_then(|c| c.resolution_prompt.clone())
                .unwrap_or_else(|| {
                    "Terdapat data profil yang bertentangan. Harap periksa profil Anda.".to_string()
                });

            return ValidatedAnswerRecommendation {
                id: Uuid::new_v4().to_string(),
                question_id,
                session_id,
                persona_id: persona_id.to_string(),
                status: EvidenceStatus::ConflictingEvidence,
                question_intent: classification.primary_category.as_str().to_string(),
                answer_format: classification.answer_format.as_str().to_string(),
                interpretation: "Terdeteksi konflik data pada profil responden.".to_string(),
                draft_answer: None,
                user_answer: None,
                review_status: Some("draft".to_string()),
                review_reason: None,
                reviewed_at: None,
                supporting_facts: vec![],
                rejected_citations: vec![],
                rationale: Some(format!(
                    "Analisis lokal mendeteksi konflik bukti. ({err_desc})"
                )),
                missing_information: vec![],
                conflicts: conflict_messages,
                assumptions: vec![],
                clarifying_question: Some(prompt),
                confidence: EvidenceConfidence::Uncertain,
                prompt_version: prompt_version.to_string(),
                provider_config_id: None,
                model_id: Some("local-fallback-engine".to_string()),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
        }

        // Jika bukti tidak ada
        if evidence.retrieved_evidence.is_empty()
            || evidence.recommendation == EvidenceRecommendation::InsufficientEvidence
        {
            let clarifying = evidence
                .missing_information
                .first()
                .cloned()
                .unwrap_or_else(|| "Informasi yang diperlukan untuk menjawab pertanyaan ini belum tersedia di profil Anda.".to_string());

            return ValidatedAnswerRecommendation {
                id: Uuid::new_v4().to_string(),
                question_id,
                session_id,
                persona_id: persona_id.to_string(),
                status: EvidenceStatus::InsufficientInformation,
                question_intent: classification.primary_category.as_str().to_string(),
                answer_format: classification.answer_format.as_str().to_string(),
                interpretation: "Tidak ditemukan fakta profil responden yang relevan untuk pertanyaan ini.".to_string(),
                draft_answer: None,
                user_answer: None,
                review_status: Some("draft".to_string()),
                review_reason: None,
                reviewed_at: None,
                supporting_facts: vec![],
                rejected_citations: vec![],
                rationale: Some(format!("Bukti profil tidak mencukupi untuk merumuskan jawaban secara bertanggung jawab. ({err_desc})")),
                missing_information: evidence.missing_information.clone(),
                conflicts: vec![],
                assumptions: vec![],
                clarifying_question: Some(clarifying),
                confidence: EvidenceConfidence::Uncertain,
                prompt_version: prompt_version.to_string(),
                provider_config_id: None,
                model_id: Some("local-fallback-engine".to_string()),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
        }

        // Jika ada bukti terkonfirmasi yang relevan (Deterministic offline answer matching)
        let top_fact = &evidence.retrieved_evidence[0];
        let mut chosen_draft = top_fact.normalized_value.clone();

        // Jika survei memiliki opsi jawaban pilihan ganda, cari opsi yang paling cocok secara deterministik
        if let Some(opts) = options {
            let fact_val_lower = top_fact.normalized_value.to_lowercase();
            if let Some(matched_opt) = opts.iter().find(|o| {
                let o_lower = o.to_lowercase();
                o_lower == fact_val_lower
                    || o_lower.contains(&fact_val_lower)
                    || fact_val_lower.contains(&o_lower)
            }) {
                chosen_draft = matched_opt.clone();
            }
        }

        ValidatedAnswerRecommendation {
            id: Uuid::new_v4().to_string(),
            question_id,
            session_id,
            persona_id: persona_id.to_string(),
            status: EvidenceStatus::Supported,
            question_intent: classification.primary_category.as_str().to_string(),
            answer_format: classification.answer_format.as_str().to_string(),
            interpretation: format!(
                "Draf jawaban dirumuskan berdasarkan fakta profil: {} = {}.",
                top_fact.attribute, top_fact.normalized_value
            ),
            draft_answer: Some(chosen_draft),
            user_answer: None,
            review_status: Some("draft".to_string()),
            review_reason: None,
            reviewed_at: None,
            supporting_facts: vec![top_fact.clone()],
            rejected_citations: vec![],
            rationale: Some(format!(
                "Dihasilkan secara deterministik oleh engine lokal berdasarkan bukti terverifikasi karena provider AI sedang tidak tersedia ({}).",
                err_desc
            )),
            missing_information: vec![],
            conflicts: vec![],
            assumptions: vec![],
            clarifying_question: None,
            confidence: evidence.evidence_confidence,
            prompt_version: prompt_version.to_string(),
            provider_config_id: None,
            model_id: Some("local-fallback-engine".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    // ==========================================
    // REVIEW ACTIONS & HISTORY (SC-034)
    // ==========================================

    /// Aksi Edit Jawaban (SC-034): Memperbarui actual_answer dan menyimpan riwayat perubahan
    pub fn edit_answer(
        database: &Database,
        input: EditAnswerInput,
    ) -> Result<AnswerHistoryItem, AppError> {
        let trimmed_answer = input.actual_answer.trim();
        if trimmed_answer.is_empty() {
            return Err(AppError::validation(
                "actual_answer",
                "Jawaban hasil edit tidak boleh kosong",
            ));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        Self::validate_persona_active(&conn, &input.persona_id)?;
        let question =
            SurveyRepository::get_question(&conn, &input.persona_id, &input.question_id)?
                .ok_or_else(|| {
                    AppError::not_found(
                        "Pertanyaan survei tidak ditemukan atau bukan milik persona ini",
                    )
                })?;

        let now = chrono::Utc::now().to_rfc3339();

        let (draft_answer, provider_id, model_id, prompt_version, evidence_sources_json) =
            if let Some(ref rec_id) = input.recommendation_id {
                if let Some(rec) =
                    AnswerRepository::get_recommendation_by_id(&conn, &input.persona_id, rec_id)?
                {
                    (
                        rec.draft_answer,
                        rec.provider_config_id,
                        rec.model_id,
                        Some(rec.prompt_version),
                        serde_json::to_string(&rec.supporting_facts)
                            .unwrap_or_else(|_| "[]".to_string()),
                    )
                } else {
                    (
                        question.user_answer_draft.clone(),
                        None,
                        None,
                        None,
                        "[]".to_string(),
                    )
                }
            } else {
                (
                    question.user_answer_draft.clone(),
                    None,
                    None,
                    None,
                    "[]".to_string(),
                )
            };

        // 1. Simpan user_answer_draft di survey_questions (terpisah dari AI draft)
        SurveyRepository::save_question_draft(
            &conn,
            &input.persona_id,
            &input.question_id,
            trimmed_answer,
            &now,
        )?;

        // 2. Perbarui record recommendation jika ada
        if let Some(ref rec_id) = input.recommendation_id {
            AnswerRepository::update_recommendation_review(
                &conn,
                &input.persona_id,
                rec_id,
                Some(trimmed_answer),
                ReviewStatus::Edited.as_str(),
                input.reason.as_deref(),
                &now,
            )?;
        }

        // 3. Simpan riwayat perubahan ke answer_history
        let history_item = AnswerHistoryItem {
            id: Uuid::new_v4().to_string(),
            persona_id: input.persona_id,
            question_id: input.question_id,
            recommendation_id: input.recommendation_id,
            action: ReviewAction::Edited,
            draft_answer,
            actual_answer: trimmed_answer.to_string(),
            review_status: ReviewStatus::Edited.as_str().to_string(),
            provider_id,
            model_id,
            prompt_version,
            evidence_sources_json,
            reason: input.reason,
            created_at: now,
        };

        AnswerRepository::save_history(&conn, &history_item)?;

        Ok(history_item)
    }

    /// Aksi Konfirmasi Jawaban (SC-034): Pengguna menyetujui draft/jawaban secara internal
    /// CRITICAL NON-NEGOTIABLE BOUNDARY:
    /// - Tidak boleh melakukan auto-fill, auto-click, atau auto-submit ke situs survei.
    /// - Tidak boleh otomatis mempromosikan jawaban survei menjadi profile fact terverifikasi!
    pub fn confirm_answer(
        database: &Database,
        input: ConfirmRecommendationInput,
    ) -> Result<AnswerHistoryItem, AppError> {
        let trimmed_answer = input.actual_answer.trim();
        if trimmed_answer.is_empty() {
            return Err(AppError::validation(
                "actual_answer",
                "Jawaban yang dikonfirmasi tidak boleh kosong",
            ));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        Self::validate_persona_active(&conn, &input.persona_id)?;
        let question =
            SurveyRepository::get_question(&conn, &input.persona_id, &input.question_id)?
                .ok_or_else(|| {
                    AppError::not_found(
                        "Pertanyaan survei tidak ditemukan atau bukan milik persona ini",
                    )
                })?;

        let now = chrono::Utc::now().to_rfc3339();

        let (draft_answer, provider_id, model_id, prompt_version, evidence_sources_json) =
            if let Some(ref rec_id) = input.recommendation_id {
                if let Some(rec) =
                    AnswerRepository::get_recommendation_by_id(&conn, &input.persona_id, rec_id)?
                {
                    (
                        rec.draft_answer,
                        rec.provider_config_id,
                        rec.model_id,
                        Some(rec.prompt_version),
                        serde_json::to_string(&rec.supporting_facts)
                            .unwrap_or_else(|_| "[]".to_string()),
                    )
                } else {
                    (
                        question.user_answer_draft.clone(),
                        None,
                        None,
                        None,
                        "[]".to_string(),
                    )
                }
            } else {
                (
                    question.user_answer_draft.clone(),
                    None,
                    None,
                    None,
                    "[]".to_string(),
                )
            };

        // 1. Simpan user_answer_confirmed di survey_questions (status confirmed)
        SurveyRepository::confirm_question_answer(
            &conn,
            &input.persona_id,
            &input.question_id,
            trimmed_answer,
            &now,
            &now,
        )?;

        // 2. Perbarui record recommendation jika ada
        if let Some(ref rec_id) = input.recommendation_id {
            AnswerRepository::update_recommendation_review(
                &conn,
                &input.persona_id,
                rec_id,
                Some(trimmed_answer),
                ReviewStatus::Confirmed.as_str(),
                None,
                &now,
            )?;
        }

        // 3. Simpan riwayat perubahan ke answer_history
        let history_item = AnswerHistoryItem {
            id: Uuid::new_v4().to_string(),
            persona_id: input.persona_id,
            question_id: input.question_id,
            recommendation_id: input.recommendation_id,
            action: ReviewAction::Confirmed,
            draft_answer,
            actual_answer: trimmed_answer.to_string(),
            review_status: ReviewStatus::Confirmed.as_str().to_string(),
            provider_id,
            model_id,
            prompt_version,
            evidence_sources_json,
            reason: None,
            created_at: now,
        };

        AnswerRepository::save_history(&conn, &history_item)?;

        // PENTING: Jawaban survei yang dikonfirmasi TIDAK PERNAH dimasukkan ke tabel profile_facts.
        // Data profil responden hanya dapat dibuat melalui menu Profil secara eksplisit.

        Ok(history_item)
    }

    /// Aksi Salin Jawaban (SC-034): Mencatat riwayat penyalinan jawaban ke clipboard
    pub fn copy_answer(
        database: &Database,
        input: CopyAnswerInput,
    ) -> Result<AnswerHistoryItem, AppError> {
        let trimmed_text = input.copied_text.trim();
        if trimmed_text.is_empty() {
            return Err(AppError::validation(
                "copied_text",
                "Teks yang disalin tidak boleh kosong",
            ));
        }

        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        Self::validate_persona_active(&conn, &input.persona_id)?;
        let question =
            SurveyRepository::get_question(&conn, &input.persona_id, &input.question_id)?
                .ok_or_else(|| {
                    AppError::not_found(
                        "Pertanyaan survei tidak ditemukan atau bukan milik persona ini",
                    )
                })?;

        let now = chrono::Utc::now().to_rfc3339();

        let (
            draft_answer,
            provider_id,
            model_id,
            prompt_version,
            evidence_sources_json,
            review_status,
        ) = if let Some(ref rec_id) = input.recommendation_id {
            if let Some(rec) =
                AnswerRepository::get_recommendation_by_id(&conn, &input.persona_id, rec_id)?
            {
                (
                    rec.draft_answer,
                    rec.provider_config_id,
                    rec.model_id,
                    Some(rec.prompt_version),
                    serde_json::to_string(&rec.supporting_facts)
                        .unwrap_or_else(|_| "[]".to_string()),
                    rec.review_status.unwrap_or_else(|| "draft".to_string()),
                )
            } else {
                (
                    question.user_answer_draft.clone(),
                    None,
                    None,
                    None,
                    "[]".to_string(),
                    "draft".to_string(),
                )
            }
        } else {
            (
                question.user_answer_draft.clone(),
                None,
                None,
                None,
                "[]".to_string(),
                "draft".to_string(),
            )
        };

        let history_item = AnswerHistoryItem {
            id: Uuid::new_v4().to_string(),
            persona_id: input.persona_id,
            question_id: input.question_id,
            recommendation_id: input.recommendation_id,
            action: ReviewAction::Copied,
            draft_answer,
            actual_answer: trimmed_text.to_string(),
            review_status,
            provider_id,
            model_id,
            prompt_version,
            evidence_sources_json,
            reason: None,
            created_at: now,
        };

        AnswerRepository::save_history(&conn, &history_item)?;

        Ok(history_item)
    }

    /// Aksi Tolak Rekomendasi (SC-034): Menolak rekomendasi AI dengan alasan opsional
    pub fn reject_answer(
        database: &Database,
        input: RejectAnswerInput,
    ) -> Result<AnswerHistoryItem, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        Self::validate_persona_active(&conn, &input.persona_id)?;
        let mut question =
            SurveyRepository::get_question(&conn, &input.persona_id, &input.question_id)?
                .ok_or_else(|| {
                    AppError::not_found(
                        "Pertanyaan survei tidak ditemukan atau bukan milik persona ini",
                    )
                })?;

        let now = chrono::Utc::now().to_rfc3339();

        let (draft_answer, provider_id, model_id, prompt_version, evidence_sources_json) =
            if let Some(ref rec_id) = input.recommendation_id {
                if let Some(rec) =
                    AnswerRepository::get_recommendation_by_id(&conn, &input.persona_id, rec_id)?
                {
                    (
                        rec.draft_answer,
                        rec.provider_config_id,
                        rec.model_id,
                        Some(rec.prompt_version),
                        serde_json::to_string(&rec.supporting_facts)
                            .unwrap_or_else(|_| "[]".to_string()),
                    )
                } else {
                    (
                        question.user_answer_draft.clone(),
                        None,
                        None,
                        None,
                        "[]".to_string(),
                    )
                }
            } else {
                (
                    question.user_answer_draft.clone(),
                    None,
                    None,
                    None,
                    "[]".to_string(),
                )
            };

        // 1. Update status pertanyaan menjadi rejected
        question.confirmation_status = crate::domain::survey::ConfirmationStatus::Rejected;
        question.updated_at = now.clone();
        SurveyRepository::update_question(&conn, &question)?;

        // 2. Update recommendation review status jika ada
        if let Some(ref rec_id) = input.recommendation_id {
            AnswerRepository::update_recommendation_review(
                &conn,
                &input.persona_id,
                rec_id,
                None,
                ReviewStatus::Rejected.as_str(),
                input.reason.as_deref(),
                &now,
            )?;
        }

        let actual_answer = question.user_answer_draft.unwrap_or_default();

        let history_item = AnswerHistoryItem {
            id: Uuid::new_v4().to_string(),
            persona_id: input.persona_id,
            question_id: input.question_id,
            recommendation_id: input.recommendation_id,
            action: ReviewAction::Rejected,
            draft_answer,
            actual_answer,
            review_status: ReviewStatus::Rejected.as_str().to_string(),
            provider_id,
            model_id,
            prompt_version,
            evidence_sources_json,
            reason: input.reason,
            created_at: now,
        };

        AnswerRepository::save_history(&conn, &history_item)?;

        Ok(history_item)
    }

    /// Aksi Regenerasi Rekomendasi (SC-034):
    /// Menghasilkan versi draf baru tanpa menghilangkan draf dan riwayat sebelumnya.
    /// Preservasi draf dan suntingan pengguna mutlak dipertahankan.
    pub fn regenerate_answer(
        database: &Database,
        credential_store: &dyn CredentialStore,
        transport: Arc<dyn HttpTransport>,
        provider_pool: &ProviderPoolService,
        input: RegenerateAnswerInput,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<ValidatedAnswerRecommendation, AppError> {
        let generate_input = GenerateAnswerInput {
            persona_id: input.persona_id.clone(),
            session_id: input.session_id,
            question_id: Some(input.question_id.clone()),
            question_text: input.question_text,
            instructions: input.instructions,
            options: input.options,
            consent: input.consent,
            provider_config_id: input.provider_config_id,
            prompt_version: input.prompt_version,
        };

        // Rekomendasi baru dibuat dengan ID UUID baru
        // Rekomendasi sebelumnya di tabel answer_recommendations tetap aman tersimpan
        let new_rec = Self::generate_recommendation(
            database,
            credential_store,
            transport,
            provider_pool,
            generate_input,
            cancellation_token,
        )?;

        // Ambil draf aktual pengguna saat ini dari database untuk preservasi draf
        let (user_draft, evidence_sources_json) = {
            let conn = database
                .connection
                .lock()
                .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
            let q = SurveyRepository::get_question(&conn, &input.persona_id, &input.question_id)?;
            let draft = q.and_then(|q| q.user_answer_draft).unwrap_or_default();
            let sources = serde_json::to_string(&new_rec.supporting_facts)
                .unwrap_or_else(|_| "[]".to_string());
            (draft, sources)
        };

        let now = chrono::Utc::now().to_rfc3339();
        let history_item = AnswerHistoryItem {
            id: Uuid::new_v4().to_string(),
            persona_id: input.persona_id,
            question_id: input.question_id,
            recommendation_id: Some(new_rec.id.clone()),
            action: ReviewAction::Regenerated,
            draft_answer: new_rec.draft_answer.clone(),
            actual_answer: user_draft,
            review_status: ReviewStatus::Draft.as_str().to_string(),
            provider_id: new_rec.provider_config_id.clone(),
            model_id: new_rec.model_id.clone(),
            prompt_version: Some(new_rec.prompt_version.clone()),
            evidence_sources_json,
            reason: input.reason,
            created_at: now,
        };

        {
            let conn = database
                .connection
                .lock()
                .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;
            AnswerRepository::save_history(&conn, &history_item)?;
        }

        Ok(new_rec)
    }

    /// Query daftar riwayat perubahan jawaban dengan isolasi persona ketat (SC-034)
    pub fn list_history(
        database: &Database,
        input: ListAnswerHistoryInput,
    ) -> Result<Vec<AnswerHistoryItem>, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        Self::validate_persona_active(&conn, &input.persona_id)?;

        AnswerRepository::list_history(
            &conn,
            &input.persona_id,
            input.question_id.as_deref(),
            input.action.as_deref(),
            input.limit,
            input.offset,
        )
    }

    /// Query seluruh revisi rekomendasi untuk satu pertanyaan dengan isolasi persona ketat (SC-034)
    pub fn list_recommendation_revisions(
        database: &Database,
        persona_id: &str,
        question_id: &str,
    ) -> Result<Vec<ValidatedAnswerRecommendation>, AppError> {
        let conn = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

        Self::validate_persona_active(&conn, persona_id)?;

        AnswerRepository::list_recommendations_by_question(&conn, persona_id, question_id)
    }

    // ==========================================
    // VALIDASI PRIVAT
    // ==========================================

    fn validate_persona_active(
        conn: &rusqlite::Connection,
        persona_id: &str,
    ) -> Result<(), AppError> {
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
