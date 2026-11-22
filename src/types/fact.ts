export type DataType =
  | "string"
  | "number"
  | "boolean"
  | "date"
  | "json";

export type VerificationStatus =
  | "confirmed"
  | "unverified"
  | "disputed"
  | "outdated";

export type Sensitivity = "ordinary" | "sensitive" | "highly_sensitive";

export type SourceType =
  | "user_declaration"
  | "uploaded_document"
  | "manual_entry"
  | "observed_action"
  | "external_record";

export interface FactSource {
  id: string;
  personaId: string;
  factId: string;
  sourceType: SourceType | string;
  title: string;
  description: string;
  referenceUri?: string | null;
  observedAt?: string | null;
  validFrom?: string | null;
  validUntil?: string | null;
  notes?: string | null;
  createdAt: string;
}

export interface FactHistoryEntry {
  id: string;
  personaId: string;
  factId: string;
  previousValueJson?: string | null;
  newValueJson: string;
  previousStatus?: string | null;
  newStatus: string;
  changeReason?: string | null;
  sourceId?: string | null;
  changedAt: string;
}

export interface ConflictWarning {
  factKey: string;
  category: string;
  existingFactId: string;
  existingDisplayName: string;
  existingValue: string;
  existingStatus: string;
  conflictingFactId: string;
  conflictingDisplayName: string;
  conflictingValue: string;
  conflictingStatus: string;
  message: string;
}

export interface ProfileFact {
  id: string;
  personaId: string;
  category: string;
  factKey: string;
  attribute: string;
  dataType: DataType | string;
  value: string | number | boolean | Record<string, unknown>;
  normalizedValue: string;
  unit?: string | null;
  verificationStatus: VerificationStatus;
  sensitivity: Sensitivity;
  sourceDescription?: string | null;
  validFrom?: string | null;
  validUntil?: string | null;
  isValid: boolean;
  hasConflict: boolean;
  sourcesCount: number;
  createdAt: string;
  updatedAt: string;
  archivedAt?: string | null;
}

export interface ProfileFactDetail extends ProfileFact {
  sourceDate?: string | null;
  notes?: string | null;
  sources: FactSource[];
  history: FactHistoryEntry[];
}

export interface CreateFactInput {
  personaId: string;
  category: string;
  factKey?: string;
  attribute: string;
  dataType?: DataType | string;
  value: string;
  unit?: string;
  sourceType?: SourceType | string;
  sourceTitle?: string;
  sourceDescription: string;
  referenceUri?: string;
  observedAt?: string;
  validFrom?: string;
  validUntil?: string;
  notes?: string;
  verificationStatus?: VerificationStatus;
  sensitivity?: Sensitivity;
}

export interface UpdateFactInput {
  personaId: string;
  factId: string;
  category?: string;
  attribute?: string;
  dataType?: DataType | string;
  value?: string;
  unit?: string;
  validFrom?: string;
  validUntil?: string;
  notes?: string;
  changeReason?: string;
  verificationStatus?: VerificationStatus;
  sensitivity?: Sensitivity;
}

export interface AddFactSourceInput {
  personaId: string;
  factId: string;
  sourceType: string;
  title: string;
  description: string;
  referenceUri?: string;
  observedAt?: string;
  validFrom?: string;
  validUntil?: string;
  notes?: string;
}
