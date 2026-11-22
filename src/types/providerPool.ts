/**
 * Tipe dan kontrak antarmuka Provider Pool dan Strategi Seleksi (SC-022)
 */

export type PoolStrategy = 'adaptive' | 'round_robin' | 'weighted' | 'priority';

export type ProviderHealthState =
  | 'ready'
  | 'cooling_down'
  | 'quota_exhausted'
  | 'needs_attention'
  | 'disabled';

export interface PoolSettings {
  id: string;
  strategy: PoolStrategy;
  maxAttempts: number;
  autoRetry: boolean;
  retryBaseDelayMs: number;
  maxRetryDelayMs: number;
  updatedAt: string;
}

export interface UpdatePoolSettingsInput {
  strategy?: PoolStrategy | string;
  maxAttempts?: number;
  autoRetry?: boolean;
  retryBaseDelayMs?: number;
  maxRetryDelayMs?: number;
}

export interface ProviderHealthInfo {
  providerConfigId: string;
  displayName: string;
  state: ProviderHealthState;
  cooldownUntil: string | null;
  lastSuccessAt: string | null;
  lastFailureAt: string | null;
  lastErrorCategory: string | null;
  consecutiveFailures: number;
  recentLatencyMs: number | null;
  currentConcurrency: number;
  maxConcurrency: number;
  priority: number;
  weight: number;
  enabled: boolean;
  updatedAt: string;
}

export interface ProviderAttemptLog {
  attemptNumber: number;
  providerConfigId: string;
  modelId: string;
  latencyMs: number;
  success: boolean;
  errorCategory: string | null;
  errorMessage: string | null;
  timestamp: string;
}

export interface PoolExecutionResult<T = unknown> {
  success: boolean;
  response: {
    requestId: string;
    providerRequestId?: string | null;
    providerId: string;
    modelId: string;
    rawText?: string | null;
    structuredJson?: T | null;
    finishReason: 'stop' | 'max_tokens' | 'safety' | string;
    inputTokens?: number | null;
    outputTokens?: number | null;
    latencyMs: number;
    receivedAt: string;
  } | null;
  finalError: {
    category: string;
    message: string;
    statusCode?: number | null;
    retryable: boolean;
    retryAfterMs?: number | null;
    providerRequestId?: string | null;
  } | null;
  attempts: ProviderAttemptLog[];
  preservedRequest: {
    requestId: string;
    operation: string;
    modelId: string;
    systemInstruction?: string | null;
    userPayload: string;
    responseSchema?: unknown;
    temperature?: number | null;
    maxTokens?: number | null;
    timeoutMs?: number | null;
  };
}
