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

export type QuestionStatus =
  | "detected"
  | "predicted"
  | "historical"
  | "unknown";

export type AnswerState =
  | "draft"
  | "user_edited"
  | "user_confirmed"
  | "skipped"
  | "not_answered"
  | "unknown";

export type Provenance =
  | "user_entered"
  | "manually_confirmed"
  | "ocr_detected"
  | "ai_inferred";

export interface SurveyRecord {
  id: string;
  personaId: string;
  title: string;
  sourcePlatform?: string;
  sourceUrl?: string;
  providerSurveyId?: string;
  recognitionFingerprint?: string;
  firstSeenAt: string;
  lastSeenAt: string;
}

export interface SurveyQuestionRecord {
  id: string;
  surveyId: string;
  personaId: string;
  sequenceOrder: number;
  promptText: string;
  normalizedText?: string;
  instructions?: string;
  optionsJson?: unknown;
  questionType: QuestionType;
  questionStatus: QuestionStatus;
  provenance: Provenance;
  observedAt: string;
}

export interface ActualUserAnswer {
  id: string;
  questionId: string;
  personaId: string;
  answerText: string;
  answerState: AnswerState;
  confirmedAt: string;
}

export interface NextQuestionPrediction {
  candidateQuestionText: string;
  questionType: QuestionType;
  evidenceSource: string;
  confidence: number;
  explanation: string;
  leadingCondition?: string;
  isPredictedNotDetected: true;
}
