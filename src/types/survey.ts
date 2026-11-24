export type SessionStatus = "active" | "archived" | "completed";

export type QuestionType =
  | "single_choice"
  | "multiple_choice"
  | "free_text"
  | "scale"
  | "matrix"
  | "numeric"
  | "date"
  | "other"
  | "unknown";

export type AnswerFormat =
  | "single_choice"
  | "multiple_selection"
  | "text"
  | "number"
  | "rating_scale"
  | "matrix"
  | "boolean"
  | "unknown";

export type Provenance =
  | "manual_entry"
  | "manually_confirmed"
  | "ocr_detected"
  | "ai_inferred";

export type QuestionStatus = "detected" | "predicted" | "historical" | "unknown";

export type ConfirmationStatus = "draft" | "confirmed" | "skipped" | "rejected";

export interface SurveySession {
  id: string;
  personaId: string;
  title: string;
  platformLabel?: string | null;
  sourceUrl?: string | null;
  sourceDomain?: string | null;
  externalSurveyId?: string | null;
  status: SessionStatus;
  language: string;
  category?: string | null;
  fingerprint?: string | null;
  version: number;
  recognizedFromId?: string | null;
  notes?: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt?: string | null;
}

export interface SurveySessionSummary {
  id: string;
  personaId: string;
  title: string;
  platformLabel?: string | null;
  sourceUrl?: string | null;
  sourceDomain?: string | null;
  status: SessionStatus;
  language: string;
  category?: string | null;
  fingerprint?: string | null;
  version: number;
  questionCount: number;
  confirmedAnswerCount: number;
  createdAt: string;
  updatedAt: string;
  archivedAt?: string | null;
}

export interface SurveyQuestion {
  id: string;
  sessionId: string;
  personaId: string;
  captureId?: string | null;
  questionOrder: number;
  questionTextOriginal: string;
  questionTextCorrected?: string | null;
  instructions?: string | null;
  options: string[];
  structureJson?: Record<string, unknown> | null;
  extractionStatus: string;
  questionType: QuestionType;
  answerFormat: AnswerFormat;
  provenance: Provenance;
  observationType: QuestionStatus;
  confirmationStatus: ConfirmationStatus;
  userAnswerDraft?: string | null;
  userAnswerConfirmed?: string | null;
  confirmedAt?: string | null;
  precedingQuestionId?: string | null;
  branchLogic?: string | null;
  questionFingerprint?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface SurveySessionDetail {
  session: SurveySession;
  questions: SurveyQuestion[];
}

export interface CreateSessionInput {
  personaId: string;
  title: string;
  platformLabel?: string | null;
  sourceUrl?: string | null;
  externalSurveyId?: string | null;
  language?: string | null;
  category?: string | null;
  notes?: string | null;
}

export interface UpdateSessionInput {
  personaId: string;
  sessionId: string;
  title: string;
  platformLabel?: string | null;
  sourceUrl?: string | null;
  externalSurveyId?: string | null;
  language?: string | null;
  category?: string | null;
  notes?: string | null;
  status?: SessionStatus | null;
}

export interface CreateQuestionInput {
  personaId: string;
  sessionId: string;
  captureId?: string | null;
  questionTextOriginal: string;
  instructions?: string | null;
  options?: string[] | null;
  questionType?: QuestionType | null;
  answerFormat?: AnswerFormat | null;
  provenance?: Provenance | null;
  questionOrder?: number | null;
  precedingQuestionId?: string | null;
  branchLogic?: string | null;
}

export interface UpdateQuestionInput {
  personaId: string;
  questionId: string;
  questionTextOriginal?: string | null;
  questionTextCorrected?: string | null;
  instructions?: string | null;
  options?: string[] | null;
  questionType?: QuestionType | null;
  answerFormat?: AnswerFormat | null;
  confirmationStatus?: ConfirmationStatus | null;
  userAnswerDraft?: string | null;
  precedingQuestionId?: string | null;
  branchLogic?: string | null;
}

export interface SaveQuestionDraftInput {
  personaId: string;
  questionId: string;
  userAnswerDraft: string;
}

export interface ConfirmAnswerInput {
  personaId: string;
  questionId: string;
  userAnswer: string;
}

// ==========================================
// QUESTION CLASSIFIER & CONSTRAINT TYPES (SC-031)
// ==========================================

export type QuestionCategory =
  | "demographic"
  | "employment_education"
  | "financial_economic"
  | "consumption_habits"
  | "asset_ownership"
  | "brand_product_usage"
  | "experience_behavior"
  | "preference_opinion"
  | "satisfaction_rating"
  | "multiple_choice"
  | "matrix_grid"
  | "numeric"
  | "date"
  | "free_text"
  | "other_unknown";

export type ClassificationConfidence = "high" | "medium" | "low" | "uncertain";

export type ConstraintProvenance = "explicit" | "inferred";

export type SelectionMode = "single" | "multiple" | "unknown";

export type TimeframeType =
  | "past_days"
  | "past_weeks"
  | "past_months"
  | "past_year"
  | "custom_range"
  | "ongoing"
  | "ever"
  | "future";

export interface TimeframeConstraint {
  timeframe_type: TimeframeType;
  duration_amount?: number | null;
  raw_text: string;
  provenance: ConstraintProvenance;
}

export interface UnitConstraint {
  unit: string;
  category?: string | null;
  provenance: ConstraintProvenance;
}

export interface NumericRangeConstraint {
  min_value?: number | null;
  max_value?: number | null;
  step?: number | null;
  provenance: ConstraintProvenance;
}

export type ScaleDirection = "low_to_high" | "high_to_low" | "unknown";

export interface ScaleAnchor {
  value: number;
  label: string;
}

export interface ScaleConstraint {
  minScale: number;
  maxScale: number;
  direction: ScaleDirection;
  anchors: ScaleAnchor[];
  provenance: ConstraintProvenance;
}

export interface TextLengthConstraint {
  minChars?: number | null;
  maxChars?: number | null;
  maxWords?: number | null;
  provenance: ConstraintProvenance;
}

export interface ExtractedConstraints {
  units?: UnitConstraint | null;
  timeframe?: TimeframeConstraint | null;
  minSelections?: number | null;
  maxSelections?: number | null;
  selectionMode: SelectionMode;
  isRequired?: boolean | null;
  expectedFormat: AnswerFormat;
  numericRange?: NumericRangeConstraint | null;
  scale?: ScaleConstraint | null;
  specialInstructions: string[];
  textLength?: TextLengthConstraint | null;
}

export type AmbiguityType =
  | "incomplete_question"
  | "multiple_interpretations"
  | "conflicting_instructions"
  | "missing_timeframe"
  | "missing_unit"
  | "mismatched_options";

export interface AmbiguityReason {
  ambiguityType: AmbiguityType;
  explanation: string;
  contextSnippet?: string | null;
}

export interface AmbiguityReport {
  isAmbiguous: boolean;
  needsClarification: boolean;
  reasons: AmbiguityReason[];
  clarificationPrompt?: string | null;
}

export interface QuestionClassificationResult {
  classifierVersion: string;
  normalizedText: string;
  categories: QuestionCategory[];
  primaryCategory: QuestionCategory;
  confidence: ClassificationConfidence;
  questionType: QuestionType;
  answerFormat: AnswerFormat;
  constraints: ExtractedConstraints;
  ambiguity: AmbiguityReport;
}

export interface ClassifyQuestionInput {
  personaId?: string | null;
  sessionId?: string | null;
  questionId?: string | null;
  questionText: string;
  instructions?: string | null;
  options?: string[] | null;
}

// ==========================================
// SC-032 EVIDENCE RETRIEVAL & CONSISTENCY TYPES
// ==========================================

export type ValidityStatus = "valid" | "expired" | "future" | "unspecified";

export type ConflictType =
  | "fact_conflict"
  | "historical_contradiction"
  | "timeframe_mismatch"
  | "value_change_over_time"
  | "unit_mismatch";

export type ConflictSeverity = "critical" | "warning" | "note";

export interface EvidenceConflictIndicator {
  conflictType: ConflictType;
  severity: ConflictSeverity;
  description: string;
  primaryFactId?: string | null;
  primarySourceId?: string | null;
  conflictingFactId?: string | null;
  conflictingSourceId?: string | null;
  historicalQuestionId?: string | null;
  historicalQuestionText?: string | null;
  historicalAnswer?: string | null;
  resolutionPrompt?: string | null;
}

export type EvidenceConfidence = "high" | "medium" | "low" | "uncertain";

export type EvidenceRecommendation =
  | "supported"
  | "needs_clarification"
  | "insufficient_evidence";

export interface RetrievedEvidenceItem {
  factId: string;
  sourceId?: string | null;
  factKey: string;
  attribute: string;
  category: string;
  value: unknown;
  normalizedValue: string;
  unit?: string | null;
  verificationStatus: "confirmed" | "unverified" | "disputed" | "outdated";
  validityStatus: ValidityStatus;
  sensitivity: "ordinary" | "sensitive" | "highly_sensitive";
  relevanceScore: number;
  selectionReason: string;
  sourceType?: string | null;
  sourceTitle?: string | null;
  sourceDescription?: string | null;
  sourceDate?: string | null;
  validFrom?: string | null;
  validUntil?: string | null;
}

export interface SensitivityConsent {
  allowSensitive: boolean;
  allowHighlySensitive: boolean;
  allowExternalVision?: boolean;
}

export interface EvidenceRetrievalInput {
  personaId: string;
  sessionId?: string | null;
  questionId?: string | null;
  questionText: string;
  instructions?: string | null;
  options?: string[] | null;
  classification?: QuestionClassificationResult | null;
  consent?: SensitivityConsent | null;
  maxEvidenceItems?: number | null;
}

export interface EvidencePackage {
  questionId?: string | null;
  activePersonaId: string;
  retrievedEvidence: RetrievedEvidenceItem[];
  conflictIndicators: EvidenceConflictIndicator[];
  missingInformation: string[];
  evidenceConfidence: EvidenceConfidence;
  recommendation: EvidenceRecommendation;
  evaluationSummary: string;
  evaluatedAt: string;
}

// ==========================================
// STRUCTURED AI ANSWER ENGINE TYPES (SC-033)
// ==========================================

export type EvidenceStatus =
  | "supported"
  | "partially_supported"
  | "insufficient_information"
  | "conflicting_evidence"
  | "user_input_required"
  | "not_applicable";

export type ReviewAction =
  | "edited"
  | "confirmed"
  | "copied"
  | "rejected"
  | "regenerated";

export type ReviewStatus = "draft" | "confirmed" | "rejected" | "edited";

export interface ValidatedAnswerRecommendation {
  id: string;
  questionId?: string | null;
  sessionId?: string | null;
  personaId: string;
  status: EvidenceStatus;
  questionIntent: string;
  answerFormat: string;
  interpretation: string;
  draftAnswer?: string | null;
  userAnswer?: string | null;
  reviewStatus?: string | null;
  reviewReason?: string | null;
  reviewedAt?: string | null;
  supportingFacts: RetrievedEvidenceItem[];
  rejectedCitations: string[];
  rationale?: string | null;
  missingInformation: string[];
  conflicts: string[];
  assumptions: string[];
  clarifyingQuestion?: string | null;
  confidence: EvidenceConfidence;
  promptVersion: string;
  providerConfigId?: string | null;
  modelId?: string | null;
  createdAt: string;
}

export interface GenerateAnswerInput {
  personaId: string;
  sessionId?: string | null;
  questionId?: string | null;
  questionText: string;
  instructions?: string | null;
  options?: string[] | null;
  consent?: SensitivityConsent | null;
  providerConfigId?: string | null;
  promptVersion?: string | null;
}

// ==========================================
// REVIEW ACTIONS & HISTORY TYPES (SC-034)
// ==========================================

export interface AnswerHistoryItem {
  id: string;
  personaId: string;
  questionId: string;
  recommendationId?: string | null;
  action: ReviewAction;
  draftAnswer?: string | null;
  actualAnswer: string;
  reviewStatus: string;
  providerId?: string | null;
  modelId?: string | null;
  promptVersion?: string | null;
  evidenceSourcesJson: string;
  reason?: string | null;
  createdAt: string;
}

export interface EditAnswerInput {
  personaId: string;
  questionId: string;
  recommendationId?: string | null;
  actualAnswer: string;
  reason?: string | null;
}

export interface ConfirmRecommendationInput {
  personaId: string;
  questionId: string;
  recommendationId?: string | null;
  actualAnswer: string;
}

export interface CopyAnswerInput {
  personaId: string;
  questionId: string;
  recommendationId?: string | null;
  copiedText: string;
}

export interface RejectAnswerInput {
  personaId: string;
  questionId: string;
  recommendationId?: string | null;
  reason?: string | null;
}

export interface RegenerateAnswerInput {
  personaId: string;
  sessionId?: string | null;
  questionId: string;
  questionText: string;
  instructions?: string | null;
  options?: string[] | null;
  consent?: SensitivityConsent | null;
  providerConfigId?: string | null;
  promptVersion?: string | null;
  reason?: string | null;
}

export interface ListAnswerHistoryInput {
  personaId: string;
  questionId?: string | null;
  action?: string | null;
  limit?: number | null;
  offset?: number | null;
}

// ==========================================
// SC-053: SURVEY RECOGNITION & MEMORY TYPES
// ==========================================

export type SurveyRecognitionStatus =
  | "exact_match"
  | "version_changed"
  | "likely_match"
  | "possible_match"
  | "no_match";

export interface SurveyRecognitionRecord {
  id: string;
  sessionId: string;
  personaId: string;
  matchedSessionId?: string | null;
  matchStatus: SurveyRecognitionStatus;
  confidenceScore: number;
  structuralSimilarity: number;
  titleSimilarity: number;
  matchedQuestionCount: number;
  totalQuestionCount: number;
  differences: string[];
  provenance: string;
  recognizedAt: string;
  createdAt: string;
}

export interface RecognizedQuestionAnswer {
  questionOrder: number;
  questionText: string;
  matchedHistoricalQuestionId: string;
  confirmedAnswer: string;
  confirmedAt: string;
  provenance: Provenance;
}

export interface SurveyRecognitionEvaluation {
  currentSessionId: string;
  currentFingerprint?: string | null;
  matchStatus: SurveyRecognitionStatus;
  confidenceScore: number;
  structuralSimilarity: number;
  titleSimilarity: number;
  matchedSessionId?: string | null;
  matchedSessionTitle?: string | null;
  matchedPlatform?: string | null;
  matchedVersion?: number | null;
  matchedQuestionCount: number;
  totalQuestionCount: number;
  differences: string[];
  provenance: string;
  explanation: string;
  recognizedAnswers: RecognizedQuestionAnswer[];
  recognizedAt: string;
}

export interface RecognizeSurveyInput {
  personaId: string;
  sessionId: string;
}

export interface SurveyMemoryQuestionItem {
  id: string;
  sessionId: string;
  personaId: string;
  questionOrder: number;
  promptText: string;
  instructions?: string | null;
  options: string[];
  questionType: QuestionType;
  answerFormat: AnswerFormat;
  provenance: Provenance;
  userAnswerConfirmed?: string | null;
  confirmedAt?: string | null;
  userAnswerDraft?: string | null;
  aiRecommendationDraft?: string | null;
  aiConfidence?: string | null;
  aiRationale?: string | null;
  questionFingerprint?: string | null;
}

export interface SurveyMemorySessionDetail {
  session: SurveySession;
  questions: SurveyMemoryQuestionItem[];
  latestRecognition?: SurveyRecognitionRecord | null;
}

// ==========================================
// SC-054: NEXT QUESTION INTELLIGENCE TYPES
// ==========================================

export type TransitionType = "sequential" | "conditional_branch" | "skip_jump";

export type PredictionStatus =
  | "predicted_not_detected"
  | "detected_matched"
  | "detected_diverged"
  | "insufficient_evidence";

export interface SurveyBranchTransition {
  id: string;
  sessionId: string;
  personaId: string;
  fromQuestionId: string;
  fromQuestionFingerprint?: string | null;
  fromQuestionOrder: number;
  fromAnswerValue?: string | null;
  toQuestionId?: string | null;
  toQuestionFingerprint?: string | null;
  toQuestionText: string;
  toQuestionType: QuestionType;
  toOptions: string[];
  toQuestionOrder: number;
  branchCondition?: string | null;
  transitionType: TransitionType;
  observedAt: string;
  createdAt: string;
}

export interface SurveyQuestionPredictionRecord {
  id: string;
  sessionId: string;
  personaId: string;
  currentQuestionId: string;
  predictedQuestionText: string;
  predictedQuestionType: QuestionType;
  predictedOptions: string[];
  confidenceScore: number;
  evidenceSource: string;
  explanation: string;
  leadingCondition?: string | null;
  status: PredictionStatus;
  matchedQuestionId?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface NextQuestionPrediction {
  id: string;
  candidateQuestionText: string;
  questionType: QuestionType;
  options: string[];
  evidenceSource: string;
  confidence: number;
  explanation: string;
  leadingCondition?: string | null;
  isPredictedNotDetected: boolean;
}

export interface NextQuestionIntelligenceResult {
  sessionId: string;
  personaId: string;
  currentQuestionId: string;
  currentQuestionText: string;
  currentUserAnswer?: string | null;
  status: PredictionStatus;
  predictions: NextQuestionPrediction[];
  observedTransitionsCount: number;
  explanation: string;
  evaluatedAt: string;
}

export interface PredictNextQuestionInput {
  personaId: string;
  sessionId: string;
  currentQuestionId: string;
  currentAnswer?: string | null;
}
