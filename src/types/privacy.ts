/**
 * Tipe dan kontrak data untuk Pengaturan Privasi, Retensi, dan Diagnostik (SC-060)
 */

export interface PrivacySettings {
  autoRedactLogs: boolean;
  requireSensitiveConsentPrompt: boolean;
  requireVisionConsentPrompt: boolean;
  screenshotRetentionPolicy: 'immediate_delete' | 'retain_1_hour' | 'retain_24_hours' | 'retain_manual_only';
  autoCleanupTempOnExit: boolean;
  logLevel: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
  updatedAt: string;
}

export interface UpdatePrivacySettingsInput {
  autoRedactLogs?: boolean;
  requireSensitiveConsentPrompt?: boolean;
  requireVisionConsentPrompt?: boolean;
  screenshotRetentionPolicy?: string;
  autoCleanupTempOnExit?: boolean;
  logLevel?: string;
}

export interface OutboundFactDisclosure {
  factId: string;
  category: string;
  attribute: string;
  valuePreview: string;
  sensitivity: 'ordinary' | 'sensitive' | 'highly_sensitive';
  status: 'included' | 'excluded_no_consent' | 'excluded_not_relevant' | 'excluded_archived';
  isSensitive: boolean;
  exclusionReason?: string;
}

export interface OutboundDataDisclosure {
  providerType: string;
  providerName: string;
  destinationEndpoint: string;
  modelId: string;
  questionText: string;
  instructions?: string;
  options?: string[];
  includedFacts: OutboundFactDisclosure[];
  excludedFacts: OutboundFactDisclosure[];
  includesSensitiveData: boolean;
  sensitiveConsentGranted: boolean;
  includesExternalVision: boolean;
  visionConsentGranted: boolean;
  notTransmittedGuarantees: string[];
  generatedAt: string;
}

export interface InspectOutboundDataInput {
  personaId: string;
  questionText: string;
  instructions?: string;
  options?: string[];
  allowSensitive?: boolean;
  allowHighlySensitive?: boolean;
  allowExternalVision?: boolean;
  hasImageAttachment?: boolean;
}

export interface DiagnosticsLogEntry {
  id: string;
  timestamp: string;
  level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
  category: 'AI_PROVIDER' | 'SCREEN_CAPTURE' | 'DATA_RETENTION' | 'SECURITY' | 'SYSTEM';
  message: string;
  metadataJson?: string;
}

export interface RecordLogInput {
  level: string;
  category: string;
  message: string;
  metadataJson?: string;
}

export interface SystemDiagnosticsInfo {
  appVersion: string;
  osPlatform: string;
  dbSchemaVersion: number;
  sqliteVersion: string;
  walSizeBytes: number;
  totalDbSizeBytes: number;
  credentialStoreType: string;
  tempCapturesCount: number;
  tempCapturesSizeBytes: number;
  activePersonasCount: number;
  totalFactsCount: number;
  totalSessionsCount: number;
  providerConfigsCount: number;
  generatedAt: string;
}

export interface RetentionStatus {
  tempDirectoryPath: string;
  tempFileCount: number;
  tempTotalBytes: number;
  oldestFileTimestamp?: string;
  activeRetentionPolicy: string;
  autoCleanupOnExit: boolean;
}

export interface RetentionCleanupResult {
  deletedFilesCount: number;
  freedBytes: number;
  cleanedAt: string;
  message: string;
}

export interface CleanupTempFilesInput {
  maxAgeSeconds?: number;
}
