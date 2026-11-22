/**
 * Status kesiapan kredensial di Windows Credential Manager
 */
export type CredentialStatus =
  | 'configured'
  | 'unconfigured'
  | 'needs_update'
  | 'inaccessible';

export type ProviderType =
  | 'gemini'
  | 'openai_compatible'
  | 'anthropic_compatible'
  | 'custom_http';

/**
 * Ringkasan publik kredensial yang aman diterima oleh frontend.
 * TIDAK PERNAH memuat nilai kunci API asli atau token rahasia.
 */
export interface CredentialSummary {
  id: string;
  providerType: ProviderType;
  label: string;
  credentialRef: string;
  status: CredentialStatus;
  usageCount: number;
  createdAt: string;
  updatedAt: string;
}

/**
 * Payload simpan kredensial baru.
 * Nilai `secret` segera dibersihkan dari memori UI setelah dikirim ke backend.
 */
export interface SaveCredentialInput {
  id?: string;
  providerType: string;
  label: string;
  secret: string;
}

/**
 * Payload pembaruan kredensial.
 */
export interface UpdateCredentialInput {
  id: string;
  label?: string;
  secret?: string;
}

/**
 * Hasil operasi penghapusan kredensial.
 */
export interface DeleteCredentialResult {
  id: string;
  credentialRef: string;
  deletedFromStore: boolean;
  deletedFromDatabase: boolean;
  message: string;
}

/**
 * Hasil pengujian keberadaan dan keterbacaan kredensial di Windows Credential Manager.
 */
export interface TestCredentialResult {
  id: string;
  credentialRef: string;
  isValid: boolean;
  status: CredentialStatus;
  message: string;
  testedAt: string;
}
