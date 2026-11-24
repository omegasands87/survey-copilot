import type { FactHistoryEntry, FactSource } from "./fact";

export type ExportScope = "full_application" | "single_persona";

export interface ExportSummary {
  exportScope: ExportScope;
  targetPersonaId: string | null;
  targetPersonaName: string | null;
  totalPersonas: number;
  totalFacts: number;
  totalSources: number;
  totalHistoryEntries: number;
  totalPreferences: number;
  totalSessions: number;
  totalQuestions: number;
  totalRecommendations: number;
  totalAnswers: number;
  totalFitResults: number;
  totalAttempts: number;
  sensitiveFactsCount: number;
  highlySensitiveFactsCount: number;
  hasSensitiveData: boolean;
  estimatedRecordsCount: number;
  warningMessage: string | null;
}

export interface ExportPersona {
  id: string;
  displayName: string;
  description: string | null;
  isActive: boolean;
  status: string;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
}

export interface ExportFact {
  id: string;
  personaId: string;
  category: string;
  factKey: string;
  attribute: string;
  dataType: string;
  value: unknown;
  normalizedValue: string;
  unit: string | null;
  verificationStatus: string;
  sensitivity: string;
  sourceDate: string | null;
  validFrom: string | null;
  validUntil: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
}

export interface ExportDataCollections {
  personas: ExportPersona[];
  facts: ExportFact[];
  factSources: FactSource[];
  factHistory: FactHistoryEntry[];
  personaPreferences: unknown[];
  surveySessions: unknown[];
  screenCaptures: unknown[];
  surveyQuestions: unknown[];
  answerRecommendations: unknown[];
  answerHistory: unknown[];
  surveyOpportunities: unknown[];
  surveyOpportunityCriteria: unknown[];
  surveyFitResults: unknown[];
  surveyAttempts: unknown[];
  surveyOutcomeEvents: unknown[];
  surveyMemory: unknown[];
}

export interface ExportPayload {
  $schema: string;
  schemaVersion: string;
  appVersion: string;
  exportTimestamp: string;
  exportScope: ExportScope;
  targetPersonaId: string | null;
  targetPersonaName: string | null;
  summary: ExportSummary;
  data: ExportDataCollections;
}

export interface AllRecordCounts {
  personas: number;
  facts: number;
  sources: number;
  history: number;
  preferences: number;
  sessions: number;
  captures: number;
  questions: number;
  recommendations: number;
  answers: number;
  opportunities: number;
  criteria: number;
  fitResults: number;
  attempts: number;
  outcomes: number;
  providerConfigs: number;
  aiUsage: number;
  appSettings: number;
  credentials: number;
  ocrExtractions: number;
  recognitions: number;
  branchTransitions: number;
  questionPredictions: number;
  diagnosticLogs: number;
  recommendationSources: number;
  providerHealth: number;
  total: number;
}

export interface WipeReport {
  deletedPersonasCount: number;
  deletedFactsCount: number;
  deletedSourcesCount: number;
  deletedHistoryCount: number;
  deletedSessionsCount: number;
  deletedQuestionsCount: number;
  deletedRecommendationsCount: number;
  deletedAnswersCount: number;
  deletedPreferencesCount: number;
  deletedFitResultsCount: number;
  deletedAttemptsCount: number;
  deletedProviderConfigsCount: number;
  deletedCredentialsCount: number;
  deletedOcrExtractionsCount: number;
  deletedRecognitionsCount: number;
  deletedBranchTransitionsCount: number;
  deletedQuestionPredictionsCount: number;
  deletedDiagnosticLogsCount: number;
  deletedRecommendationSourcesCount: number;
  deletedProviderHealthCount: number;
  deletedTempFilesCount: number;
  failedTempFilesCount: number;
  cleanupWarnings: string[];
  walCheckpointStatus: string;
  vacuumStatus: string;
  wipedAt: string;
  unaffectedExternalItems: string[];
}

export interface RetentionPolicyInfo {
  activeFactsPolicy: string;
  archivedFactsPolicy: string;
  screenCapturePolicy: string;
  ocrDataPolicy: string;
  surveyHistoryPolicy: string;
  exportFilePolicy: string;
  logsAndDiagnosticsPolicy: string;
  deletionScopeDisclaimer: string;
}
