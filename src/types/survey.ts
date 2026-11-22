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
  externalSurveyId?: string | null;
  status: SessionStatus;
  language: string;
  category?: string | null;
  fingerprint?: string | null;
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
  status: SessionStatus;
  language: string;
  category?: string | null;
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
