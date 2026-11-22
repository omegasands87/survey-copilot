/**
 * Tipe dan kontrak antarmuka konfigurasi AI Provider dan Adapter
 */

export type ProviderType = 'gemini' | 'openai_compatible' | 'anthropic_compatible' | 'custom_http';
export type AdapterType = 'gemini' | 'openai_compatible' | 'anthropic_compatible' | 'custom_http';

export interface ProviderConfigSummary {
  id: string;
  displayName: string;
  providerType: ProviderType;
  baseUrl: string | null;
  adapterType: AdapterType;
  modelId: string;
  credentialRef: string;
  credentialLabel?: string | null;
  enabled: boolean;
  priority: number;
  weight: number;
  timeoutMs: number;
  maxConcurrency: number;
  rateLimitHintJson?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateProviderConfigInput {
  id?: string;
  displayName: string;
  providerType: ProviderType;
  baseUrl?: string | null;
  adapterType: AdapterType;
  modelId: string;
  credentialRef: string;
  enabled?: boolean;
  priority?: number;
  weight?: number;
  timeoutMs?: number;
  maxConcurrency?: number;
  rateLimitHintJson?: string | null;
}

export interface UpdateProviderConfigInput {
  id: string;
  displayName?: string;
  baseUrl?: string | null;
  modelId?: string;
  credentialRef?: string;
  enabled?: boolean;
  priority?: number;
  weight?: number;
  timeoutMs?: number;
  maxConcurrency?: number;
  rateLimitHintJson?: string | null;
}

export interface ModelOption {
  id: string;
  displayName: string;
  description?: string | null;
  supportedOperations: string[];
}

export interface ConnectionTestResult {
  providerConfigId?: string | null;
  success: boolean;
  modelId: string;
  latencyMs: number;
  message: string;
  testedAt: string;
  availableModels?: ModelOption[];
}
