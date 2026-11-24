import { invoke } from '@tauri-apps/api/core';
import type {
  AttemptFilter,
  CreateAttemptInput,
  HistoricalLearningSummary,
  RecordOutcomeEventInput,
  SurveyAttemptDetail,
} from '../types/attempt';

export const attemptService = {
  /**
   * Mencatat percobaan survei baru (SC-052)
   */
  async recordAttempt(input: CreateAttemptInput): Promise<SurveyAttemptDetail> {
    return invoke<SurveyAttemptDetail>('survey_attempt_record', { input });
  },

  /**
   * Menambahkan peristiwa hasil observasi baru pada percobaan survei (SC-052)
   */
  async recordOutcomeEvent(input: RecordOutcomeEventInput): Promise<SurveyAttemptDetail> {
    return invoke<SurveyAttemptDetail>('survey_attempt_record_outcome', { input });
  },

  /**
   * Mengambil detail lengkap suatu percobaan survei
   */
  async getAttemptDetail(id: string, personaId?: string): Promise<SurveyAttemptDetail> {
    return invoke<SurveyAttemptDetail>('survey_attempt_get', { id, personaId });
  },

  /**
   * Mengambil daftar riwayat percobaan survei dengan filter dinamis
   */
  async listAttempts(filter: AttemptFilter = {}): Promise<SurveyAttemptDetail[]> {
    return invoke<SurveyAttemptDetail[]>('survey_attempt_list', { filter });
  },

  /**
   * Menghapus catatan percobaan survei (isolasi persona)
   */
  async deleteAttempt(id: string, personaId: string): Promise<boolean> {
    return invoke<boolean>('survey_attempt_delete', { id, personaId });
  },

  /**
   * Mengambil ringkasan pembelajaran historis (AT-063: sparse data labeled insufficient)
   */
  async getLearningSummary(
    personaId: string,
    category?: string,
    sourcePlatform?: string
  ): Promise<HistoricalLearningSummary> {
    return invoke<HistoricalLearningSummary>('survey_attempt_get_learning_summary', {
      personaId,
      category,
      sourcePlatform,
    });
  },
};
