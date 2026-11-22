import { invokeCommand } from "./ipcClient";
import type {
  ConfirmAnswerInput,
  CreateQuestionInput,
  CreateSessionInput,
  SaveQuestionDraftInput,
  SurveyQuestion,
  SurveySession,
  SurveySessionDetail,
  SurveySessionSummary,
  UpdateQuestionInput,
  UpdateSessionInput,
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
};
