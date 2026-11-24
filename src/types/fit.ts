import type { CriterionOperator, CriterionType } from './opportunity';

export type MatchState = 'match' | 'mismatch' | 'unknown' | 'stale' | 'conflict';

export type OverallFitState =
  | 'eligible'
  | 'ineligible'
  | 'conflict'
  | 'stale'
  | 'incomplete'
  | 'no_criteria';

export interface FitCriterionResult {
  criterionId: string;
  criterionText: string;
  rawText?: string | null;
  criterionType: CriterionType;
  operator: CriterionOperator;
  matchState: MatchState;
  supportingFactIds: string[];
  supportingFactsSummary: string[];
  explanation: string;
}

export interface PersonaOpportunityFitDetail {
  opportunityId: string;
  personaId: string;
  personaName: string;
  overallFit: OverallFitState;
  matchCount: number;
  mismatchCount: number;
  unknownCount: number;
  staleCount: number;
  conflictCount: number;
  totalCriteria: number;
  criteriaResults: FitCriterionResult[];
  evaluatedAt: string;
}

export interface PersonaFitComparisonItem {
  personaId: string;
  personaName: string;
  overallFit: OverallFitState;
  matchCount: number;
  mismatchCount: number;
  unknownCount: number;
  staleCount: number;
  conflictCount: number;
  totalCriteria: number;
  keyReasons: string[];
  evaluatedAt?: string | null;
}
