import type { OpportunityCriterion } from './opportunity';

export type OutcomeLabel =
  | 'qualified'
  | 'screened_out'
  | 'quota_full'
  | 'completed'
  | 'quality_rejected'
  | 'technical_failure'
  | 'abandoned_by_user'
  | 'pending'
  | 'unknown';

export type OutcomeConfidence = 'observed' | 'user_confirmed' | 'uncertain';

export type LearningDataStatus = 'insufficient_data' | 'sufficient_data';

export interface SurveyAttempt {
  id: string;
  opportunityId: string;
  personaId: string;
  sessionId?: string | null;
  startedAt: string;
  endedAt?: string | null;
  currentOutcome: OutcomeLabel;
  createdAt: string;
  updatedAt: string;
}

export interface SurveyOutcomeEvent {
  id: string;
  attemptId: string;
  outcomeLabel: OutcomeLabel;
  evidenceSource: string;
  evidenceNote?: string | null;
  confidence: OutcomeConfidence;
  observedAt: string;
  createdAt: string;
}

export interface SurveyAttemptDetail {
  id: string;
  opportunityId: string;
  opportunityTitle: string;
  sourcePlatform: string;
  category?: string | null;
  targetUrl?: string | null;
  rawText?: string | null;
  personaId: string;
  personaName: string;
  sessionId?: string | null;
  startedAt: string;
  endedAt?: string | null;
  currentOutcome: OutcomeLabel;
  criteria: OpportunityCriterion[];
  events: SurveyOutcomeEvent[];
  createdAt: string;
  updatedAt: string;
}

export interface AttemptFilter {
  personaId?: string | null;
  category?: string | null;
  sourcePlatform?: string | null;
  status?: OutcomeLabel | null;
  fromDate?: string | null;
  toDate?: string | null;
}

export interface CreateAttemptInput {
  personaId: string;
  opportunityId?: string | null;
  opportunityTitle?: string | null;
  sourcePlatform?: string | null;
  category?: string | null;
  targetUrl?: string | null;
  rawText?: string | null;
  sessionId?: string | null;
  startedAt?: string | null;
  initialOutcome?: OutcomeLabel | null;
  evidenceSource?: string | null;
  evidenceNote?: string | null;
  confidence?: OutcomeConfidence | null;
}

export interface RecordOutcomeEventInput {
  attemptId: string;
  personaId: string;
  outcomeLabel: OutcomeLabel;
  evidenceSource: string;
  evidenceNote?: string | null;
  confidence?: OutcomeConfidence | null;
  observedAt?: string | null;
}

export interface HistoricalLearningSummary {
  personaId: string;
  personaName: string;
  category?: string | null;
  sourcePlatform?: string | null;
  sampleSize: number;
  minimumRequiredSample: number;
  status: LearningDataStatus;
  observationWindowStart?: string | null;
  observationWindowEnd?: string | null;
  countsByOutcome: Record<string, number>;
  completionRate?: number | null;
  qualificationRate?: number | null;
  validationMethod: string;
  holdoutSampleSize: number;
  holdoutConsistencyRate?: number | null;
  message: string;
  uncertaintyDisclosure: string;
  disclaimers: string[];
}
