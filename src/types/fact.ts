export type VerificationStatus =
  | "confirmed"
  | "unverified"
  | "disputed"
  | "outdated";

export type Sensitivity = "ordinary" | "sensitive" | "highly_sensitive";

export interface ProfileFact {
  id: string;
  category: string;
  attribute: string;
  value: string | number | boolean | Record<string, unknown>;
  verificationStatus: VerificationStatus | string;
  sensitivity: Sensitivity | string;
  sourceDescription: string | null;
}

export interface CreateFactInput {
  personaId: string;
  category: string;
  attribute: string;
  value: string;
  sourceDescription: string;
  verificationStatus: VerificationStatus;
  sensitivity: Sensitivity;
}
