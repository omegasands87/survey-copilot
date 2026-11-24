import { invoke } from '@tauri-apps/api/core';
import type {
  OpportunityCriterion,
  OpportunityDetail,
  SaveOpportunityWithCriteriaInput,
  SurveyOpportunity,
  UpdateCriteriaInput,
  UpdateOpportunityInput,
} from '../types/opportunity';

export const opportunityService = {
  /**
   * Mengekstrak informasi peluang survei dan kriteria dari teks mentah (SC-050)
   */
  async extractFromText(
    rawText: string,
    platformLabel?: string
  ): Promise<OpportunityDetail> {
    return invoke<OpportunityDetail>('opportunity_extract_from_text', {
      rawText,
      platformLabel: platformLabel || null,
    });
  },

  /**
   * Mengekstrak informasi peluang survei dari hasil tangkapan layar / OCR
   */
  async extractFromCapture(
    captureId: string,
    ocrId?: string
  ): Promise<OpportunityDetail> {
    return invoke<OpportunityDetail>('opportunity_extract_from_capture', {
      captureId,
      ocrId: ocrId || null,
    });
  },

  /**
   * Menyimpan peluang survei beserta seluruh kriteria hasil tinjauan/koreksi pengguna
   */
  async saveOpportunity(
    input: SaveOpportunityWithCriteriaInput
  ): Promise<OpportunityDetail> {
    return invoke<OpportunityDetail>('opportunity_save', { input });
  },

  /**
   * Mengambil detail peluang survei berdasarkan ID
   */
  async getOpportunity(id: string): Promise<OpportunityDetail | null> {
    return invoke<OpportunityDetail | null>('opportunity_get', { id });
  },

  /**
   * Mengambil daftar peluang survei yang tersimpan
   */
  async listOpportunities(
    limit?: number,
    offset?: number
  ): Promise<SurveyOpportunity[]> {
    return invoke<SurveyOpportunity[]>('opportunity_list', {
      limit: limit ?? null,
      offset: offset ?? null,
    });
  },

  /**
   * Memperbarui metadata peluang survei
   */
  async updateOpportunity(
    input: UpdateOpportunityInput
  ): Promise<SurveyOpportunity> {
    return invoke<SurveyOpportunity>('opportunity_update', { input });
  },

  /**
   * Memperbarui kriteria kelayakan suatu peluang survei
   */
  async updateCriteria(
    input: UpdateCriteriaInput
  ): Promise<OpportunityCriterion[]> {
    return invoke<OpportunityCriterion[]>('opportunity_update_criteria', {
      input,
    });
  },

  /**
   * Menghapus peluang survei beserta kriterianya
   */
  async deleteOpportunity(id: string): Promise<boolean> {
    return invoke<boolean>('opportunity_delete', { id });
  },
};
