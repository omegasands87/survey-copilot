import { invokeCommand } from "./ipcClient";
import type {
  AnswerHistoryItem,
  ClassifyQuestionInput,
  ConfirmAnswerInput,
  ConfirmRecommendationInput,
  CopyAnswerInput,
  CreateQuestionInput,
  CreateSessionInput,
  EditAnswerInput,
  EvidencePackage,
  EvidenceRetrievalInput,
  GenerateAnswerInput,
  ListAnswerHistoryInput,
  NextQuestionIntelligenceResult,
  PredictNextQuestionInput,
  QuestionClassificationResult,
  RecognizeSurveyInput,
  RegenerateAnswerInput,
  RejectAnswerInput,
  SaveQuestionDraftInput,
  SurveyBranchTransition,
  SurveyMemorySessionDetail,
  SurveyQuestion,
  SurveyQuestionPredictionRecord,
  SurveyRecognitionEvaluation,
  SurveySession,
  SurveySessionDetail,
  SurveySessionSummary,
  UpdateQuestionInput,
  UpdateSessionInput,
  ValidatedAnswerRecommendation,
} from "../types/survey";

export const surveyService = {
  // ==========================================
  // SURVEY SESSIONS
  // ==========================================

  /**
   * Membuat sesi survei baru untuk persona tertentu
   */
  async createSession(input: CreateSessionInput): Promise<SurveySession> {
    return invokeCommand<SurveySession>("survey_session_create", { input });
  },

  /**
   * Mengambil daftar ringkasan sesi survei milik persona
   */
  async listSessions(
    personaId: string,
    includeArchived = false
  ): Promise<SurveySessionSummary[]> {
    return invokeCommand<SurveySessionSummary[]>("survey_session_list", {
      personaId,
      includeArchived,
    });
  },

  /**
   * Mengambil satu sesi survei
   */
  async getSession(
    personaId: string,
    sessionId: string
  ): Promise<SurveySession> {
    return invokeCommand<SurveySession>("survey_session_get", {
      personaId,
      sessionId,
    });
  },

  /**
   * Memperbarui sesi survei
   */
  async updateSession(input: UpdateSessionInput): Promise<SurveySession> {
    return invokeCommand<SurveySession>("survey_session_update", { input });
  },

  /**
   * Mengarsipkan sesi survei
   */
  async archiveSession(
    personaId: string,
    sessionId: string
  ): Promise<void> {
    return invokeCommand<void>("survey_session_archive", {
      personaId,
      sessionId,
    });
  },

  /**
   * Memulihkan sesi survei yang diarsipkan
   */
  async restoreSession(
    personaId: string,
    sessionId: string
  ): Promise<void> {
    return invokeCommand<void>("survey_session_restore", {
      personaId,
      sessionId,
    });
  },

  /**
   * Menghapus sesi survei dan seluruh entitas anaknya
   */
  async deleteSession(
    personaId: string,
    sessionId: string
  ): Promise<void> {
    return invokeCommand<void>("survey_session_delete", {
      personaId,
      sessionId,
    });
  },

  /**
   * Mengambil detail lengkap sesi survei beserta seluruh daftar pertanyaannya
   */
  async getSessionDetail(
    personaId: string,
    sessionId: string
  ): Promise<SurveySessionDetail> {
    return invokeCommand<SurveySessionDetail>("survey_session_get_detail", {
      personaId,
      sessionId,
    });
  },

  // ==========================================
  // SURVEY QUESTIONS
  // ==========================================

  /**
   * Membuat pertanyaan survei manual dalam sesi
   */
  async createQuestion(input: CreateQuestionInput): Promise<SurveyQuestion> {
    return invokeCommand<SurveyQuestion>("survey_question_create", { input });
  },

  /**
   * Mengambil satu pertanyaan survei
   */
  async getQuestion(
    personaId: string,
    questionId: string
  ): Promise<SurveyQuestion> {
    return invokeCommand<SurveyQuestion>("survey_question_get", {
      personaId,
      questionId,
    });
  },

  /**
   * Mengambil seluruh pertanyaan dalam satu sesi survei
   */
  async listQuestions(
    personaId: string,
    sessionId: string
  ): Promise<SurveyQuestion[]> {
    return invokeCommand<SurveyQuestion[]>("survey_question_list", {
      personaId,
      sessionId,
    });
  },

  /**
   * Memperbarui pertanyaan survei
   */
  async updateQuestion(input: UpdateQuestionInput): Promise<SurveyQuestion> {
    return invokeCommand<SurveyQuestion>("survey_question_update", { input });
  },

  /**
   * Menyimpan draf jawaban pengguna secara langsung dan aman (auto-save)
   */
  async saveQuestionDraft(input: SaveQuestionDraftInput): Promise<void> {
    return invokeCommand<void>("survey_question_save_draft", { input });
  },

  /**
   * Mengkonfirmasi jawaban aktual pengguna (terpisah dari AI draft)
   */
  async confirmQuestionAnswer(input: ConfirmAnswerInput): Promise<void> {
    return invokeCommand<void>("survey_question_confirm_answer", { input });
  },

  /**
   * Menghapus pertanyaan survei
   */
  async deleteQuestion(
    personaId: string,
    questionId: string
  ): Promise<void> {
    return invokeCommand<void>("survey_question_delete", {
      personaId,
      questionId,
    });
  },

  /**
   * Mengklasifikasikan pertanyaan dan mengekstrak batasan serta ambiguitas (SC-031)
   */
  async classifyQuestion(
    input: ClassifyQuestionInput
  ): Promise<QuestionClassificationResult> {
    return invokeCommand<QuestionClassificationResult>(
      "survey_question_classify",
      { input }
    );
  },

  /**
   * Mengambil bukti fakta yang relevan dan mengevaluasi konsistensi (SC-032)
   */
  async evaluateEvidence(
    input: EvidenceRetrievalInput
  ): Promise<EvidencePackage> {
    return invokeCommand<EvidencePackage>(
      "survey_evidence_evaluate",
      { input }
    );
  },

  /**
   * Menghasilkan rekomendasi jawaban terstruktur yang evidence-grounded (SC-033)
   */
  async generateAnswer(
    input: GenerateAnswerInput
  ): Promise<ValidatedAnswerRecommendation> {
    return invokeCommand<ValidatedAnswerRecommendation>(
      "survey_answer_generate",
      { input }
    );
  },

  /**
   * Mengambil rekomendasi jawaban AI terbaru untuk pertanyaan survei tertentu (SC-033)
   */
  async getLatestAnswerRecommendation(
    personaId: string,
    questionId: string
  ): Promise<ValidatedAnswerRecommendation | null> {
    return invokeCommand<ValidatedAnswerRecommendation | null>(
      "survey_answer_get_latest",
      { personaId, questionId }
    );
  },

  // ==========================================
  // REVIEW ACTIONS & HISTORY (SC-034)
  // ==========================================

  /**
   * Mengedit actual_answer untuk rekomendasi/pertanyaan dan mencatat riwayat perubahan (SC-034)
   */
  async editAnswer(input: EditAnswerInput): Promise<AnswerHistoryItem> {
    return invokeCommand<AnswerHistoryItem>("survey_answer_edit", { input });
  },

  /**
   * Mengkonfirmasi rekomendasi atau draft pengguna secara internal (SC-034)
   * CATATAN: Konfirmasi adalah persetujuan internal; tidak pernah otomatis kirim ke survei atau dijadikan profil faktual.
   */
  async confirmAnswer(
    input: ConfirmRecommendationInput
  ): Promise<AnswerHistoryItem> {
    return invokeCommand<AnswerHistoryItem>("survey_answer_confirm", { input });
  },

  /**
   * Mencatat aksi penyalinan jawaban ke clipboard ke riwayat audit (SC-034)
   */
  async copyAnswer(input: CopyAnswerInput): Promise<AnswerHistoryItem> {
    return invokeCommand<AnswerHistoryItem>("survey_answer_copy", { input });
  },

  /**
   * Menolak rekomendasi jawaban AI dengan alasan opsional (SC-034)
   */
  async rejectAnswer(input: RejectAnswerInput): Promise<AnswerHistoryItem> {
    return invokeCommand<AnswerHistoryItem>("survey_answer_reject", { input });
  },

  /**
   * Meregenerasi rekomendasi jawaban baru tanpa menghapus revisi sebelumnya (SC-034)
   */
  async regenerateAnswer(
    input: RegenerateAnswerInput
  ): Promise<ValidatedAnswerRecommendation> {
    return invokeCommand<ValidatedAnswerRecommendation>(
      "survey_answer_regenerate",
      { input }
    );
  },

  /**
   * Mengambil riwayat perubahan jawaban survei dengan filter dan isolasi persona (SC-034)
   */
  async listAnswerHistory(
    input: ListAnswerHistoryInput
  ): Promise<AnswerHistoryItem[]> {
    return invokeCommand<AnswerHistoryItem[]>("survey_answer_history_list", {
      input,
    });
  },

  /**
   * Mengambil daftar seluruh revisi rekomendasi untuk satu pertanyaan (SC-034)
   */
  async listRecommendationRevisions(
    personaId: string,
    questionId: string
  ): Promise<ValidatedAnswerRecommendation[]> {
    return invokeCommand<ValidatedAnswerRecommendation[]>(
      "survey_answer_revisions_list",
      { personaId, questionId }
    );
  },

  // ==========================================
  // SC-053: SURVEY MEMORY & RECOGNITION
  // ==========================================

  /**
   * Mengevaluasi pengenalan survei aktif terhadap riwayat Survey Memory (AT-064 & AT-066)
   */
  async recognizeSurvey(
    input: RecognizeSurveyInput
  ): Promise<SurveyRecognitionEvaluation> {
    return invokeCommand<SurveyRecognitionEvaluation>(
      "survey_memory_recognize",
      { input }
    );
  },

  /**
   * Mengambil detail memori survei dengan pemisahan tegas antara jawaban konfirmasi nyata
   * dan draf saran AI (AT-064)
   */
  async getMemorySessionDetail(
    personaId: string,
    sessionId: string
  ): Promise<SurveyMemorySessionDetail> {
    return invokeCommand<SurveyMemorySessionDetail>(
      "survey_memory_get_detail",
      { personaId, sessionId }
    );
  },

  // ==========================================
  // SC-054: NEXT QUESTION INTELLIGENCE (NQI)
  // ==========================================

  /**
   * Memprediksi kemungkinan pertanyaan berikutnya berdasarkan riwayat transisi percabangan nyata (AT-065)
   */
  async predictNextQuestions(
    input: PredictNextQuestionInput
  ): Promise<NextQuestionIntelligenceResult> {
    return invokeCommand<NextQuestionIntelligenceResult>(
      "survey_predict_next_questions",
      { input }
    );
  },

  /**
   * Mengambil seluruh catatan riwayat prediksi pertanyaan untuk sesi survei
   */
  async getSessionPredictions(
    personaId: string,
    sessionId: string
  ): Promise<SurveyQuestionPredictionRecord[]> {
    return invokeCommand<SurveyQuestionPredictionRecord[]>(
      "survey_get_session_predictions",
      { personaId, sessionId }
    );
  },

  /**
   * Mengambil seluruh observasi transisi percabangan yang diamati pada sesi
   */
  async getSessionBranchTransitions(
    personaId: string,
    sessionId: string
  ): Promise<SurveyBranchTransition[]> {
    return invokeCommand<SurveyBranchTransition[]>(
      "survey_get_session_branch_transitions",
      { personaId, sessionId }
    );
  },
};
