export type OpportunityStatus =
  | 'captured'
  | 'criteria_extracted'
  | 'confirmed'
  | 'evaluated'
  | 'archived';

export type OpportunityProvenance =
  | 'manual_entry'
  | 'screen_capture'
  | 'ocr_detected';

export type CriterionType =
  | 'age'
  | 'gender'
  | 'location'
  | 'occupation'
  | 'product_usage'
  | 'education'
  | 'income'
  | 'marital_status'
  | 'general';

export type CriterionOperator =
  | 'equals'
  | 'range'
  | 'greater_than_or_equal'
  | 'less_than_or_equal'
  | 'in'
  | 'contains'
  | 'not_equals';

export interface SurveyOpportunity {
  id: string;
  title?: string | null;
  platformLabel?: string | null;
  topic?: string | null;
  description?: string | null;
  rawText?: string | null;
  provenance: OpportunityProvenance;
  status: OpportunityStatus;
  sourceCaptureId?: string | null;
  sourceOcrId?: string | null;
  capturedAt: string;
  createdAt: string;
  updatedAt: string;
}

export interface OpportunityCriterion {
  id: string;
  opportunityId: string;
  criterionText: string;
  rawText?: string | null;
  criterionType: CriterionType;
  normalizedAttribute?: string | null;
  expectedValueJson?: Record<string, unknown> | null;
  operator: CriterionOperator;
  criterionOrder: number;
}

export interface OpportunityDetail {
  opportunity: SurveyOpportunity;
  criteria: OpportunityCriterion[];
}

export interface CreateOpportunityInput {
  title?: string | null;
  platformLabel?: string | null;
  topic?: string | null;
  description?: string | null;
  rawText: string;
  provenance?: OpportunityProvenance | null;
  sourceCaptureId?: string | null;
  sourceOcrId?: string | null;
  capturedAt?: string | null;
}

export interface UpdateOpportunityInput {
  id: string;
  title?: string | null;
  platformLabel?: string | null;
  topic?: string | null;
  description?: string | null;
  status?: OpportunityStatus | null;
}

export interface CriterionInput {
  id?: string | null;
  criterionText: string;
  rawText?: string | null;
  criterionType: CriterionType;
  normalizedAttribute?: string | null;
  expectedValueJson?: Record<string, unknown> | null;
  operator: CriterionOperator;
  criterionOrder: number;
}

export interface SaveOpportunityWithCriteriaInput {
  opportunity: CreateOpportunityInput;
  criteria: CriterionInput[];
}

export interface UpdateCriteriaInput {
  opportunityId: string;
  criteria: CriterionInput[];
}
