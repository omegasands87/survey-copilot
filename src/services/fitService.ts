import { invoke } from '@tauri-apps/api/core';
import type { PersonaFitComparisonItem, PersonaOpportunityFitDetail } from '../types/fit';

export const fitService = {
  /**
   * Mengevaluasi kesesuaian kriteria peluang survei terhadap persona secara deterministik (SC-051)
   */
  async evaluateOpportunityFit(
    opportunityId: string,
    personaId: string
  ): Promise<PersonaOpportunityFitDetail> {
    return invoke<PersonaOpportunityFitDetail>('survey_fit_evaluate', {
      opportunityId,
      personaId,
    });
  },

  /**
   * Mengambil hasil evaluasi kesesuaian yang tersimpan
   */
  async getOpportunityFit(
    opportunityId: string,
    personaId: string
  ): Promise<PersonaOpportunityFitDetail | null> {
    return invoke<PersonaOpportunityFitDetail | null>('survey_fit_get_results', {
      opportunityId,
      personaId,
    });
  },

  /**
   * Membandingkan kesesuaian peluang survei lintas seluruh persona (truthful respondent selection)
   */
  async comparePersonasFit(opportunityId: string): Promise<PersonaFitComparisonItem[]> {
    return invoke<PersonaFitComparisonItem[]>('survey_fit_compare_personas', {
      opportunityId,
    });
  },
};
