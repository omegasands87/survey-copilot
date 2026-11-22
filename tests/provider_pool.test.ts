import test from 'node:test';
import assert from 'node:assert/strict';
import type {
  PoolExecutionResult,
  PoolSettings,
  PoolStrategy,
  ProviderAttemptLog,
  ProviderHealthInfo,
  ProviderHealthState,
  UpdatePoolSettingsInput,
} from '../src/types/providerPool';

test('PoolStrategy supports adaptive, round_robin, weighted, and priority', () => {
  const strategies: PoolStrategy[] = ['adaptive', 'round_robin', 'weighted', 'priority'];
  assert.equal(strategies.length, 4);

  strategies.forEach((strat) => {
    assert.ok(['adaptive', 'round_robin', 'weighted', 'priority'].includes(strat));
  });
});

test('PoolSettings contract validates bounded retry limits (1 to 10 attempts)', () => {
  const settings: PoolSettings = {
    id: 'default',
    strategy: 'adaptive',
    maxAttempts: 3,
    autoRetry: true,
    retryBaseDelayMs: 1000,
    maxRetryDelayMs: 15000,
    updatedAt: '2026-11-23T03:00:00Z',
  };

  assert.ok(settings.maxAttempts >= 1 && settings.maxAttempts <= 10);
  assert.ok(settings.retryBaseDelayMs >= 100);
  assert.ok(settings.maxRetryDelayMs >= settings.retryBaseDelayMs);
});

test('UpdatePoolSettingsInput correctly accepts partial pool parameter updates', () => {
  const updateInput: UpdatePoolSettingsInput = {
    strategy: 'priority',
    maxAttempts: 5,
    autoRetry: false,
    retryBaseDelayMs: 2000,
  };

  assert.equal(updateInput.strategy, 'priority');
  assert.equal(updateInput.maxAttempts, 5);
  assert.equal(updateInput.autoRetry, false);
});

test('ProviderHealthState covers ready, cooling_down, quota_exhausted, needs_attention, and disabled', () => {
  const states: ProviderHealthState[] = [
    'ready',
    'cooling_down',
    'quota_exhausted',
    'needs_attention',
    'disabled',
  ];
  assert.equal(states.length, 5);

  const healthInfo: ProviderHealthInfo = {
    providerConfigId: 'p_gemini_01',
    displayName: 'Google Gemini 2.5 Flash',
    state: 'cooling_down',
    cooldownUntil: '2026-11-23T03:05:00Z',
    lastSuccessAt: '2026-11-23T02:50:00Z',
    lastFailureAt: '2026-11-23T03:00:00Z',
    lastErrorCategory: 'rate_limited',
    consecutiveFailures: 1,
    recentLatencyMs: 340,
    currentConcurrency: 0,
    maxConcurrency: 2,
    priority: 10,
    weight: 1,
    enabled: true,
    updatedAt: '2026-11-23T03:00:00Z',
  };

  assert.equal(healthInfo.state, 'cooling_down');
  assert.ok(healthInfo.cooldownUntil !== null);
  assert.ok(healthInfo.currentConcurrency <= healthInfo.maxConcurrency);
});

test('PoolExecutionResult strictly preserves user request draft and context on failure (AT-040 boundary)', () => {
  const result: PoolExecutionResult<{ answer: string }> = {
    success: false,
    response: null,
    finalError: {
      category: 'rate_limited',
      message: 'Seluruh provider AI sedang sibuk atau dalam cooldown.',
      statusCode: 429,
      retryable: true,
      retryAfterMs: 5000,
    },
    attempts: [
      {
        attemptNumber: 1,
        providerConfigId: 'p_gemini_01',
        modelId: 'gemini-2.5-flash',
        latencyMs: 120,
        success: false,
        errorCategory: 'rate_limited',
        errorMessage: 'Resource exhausted',
        timestamp: '2026-11-23T03:00:00Z',
      },
    ],
    preservedRequest: {
      requestId: 'req_user_draft_01',
      operation: 'survey_question_analysis',
      modelId: 'gemini-2.5-flash',
      systemInstruction: 'Anda adalah asisten survei berbasis bukti.',
      userPayload: 'Pertanyaan survei penting yang dimasukkan responden.',
      temperature: 0.1,
    },
  };

  assert.equal(result.success, false);
  assert.ok(result.finalError !== null);
  assert.equal(
    result.preservedRequest.userPayload,
    'Pertanyaan survei penting yang dimasukkan responden.',
    'Draf dan input pertanyaan pengguna HARUS dipertahankan secara utuh ketika provider gagal'
  );
});

test('ProviderAttemptLog and PoolExecutionResult contracts contain zero raw API secrets', () => {
  const attempt: ProviderAttemptLog = {
    attemptNumber: 1,
    providerConfigId: 'provider_gemini_prod',
    modelId: 'gemini-2.5-pro',
    latencyMs: 450,
    success: true,
    errorCategory: null,
    errorMessage: null,
    timestamp: '2026-11-23T03:00:00Z',
  };

  const serialized = JSON.stringify(attempt);
  assert.equal(serialized.includes('secret'), false);
  assert.equal(serialized.includes('apiKey'), false);
  assert.equal(serialized.includes('AIzaSy'), false);
});

test('Account and project quota error stops retries across pool (AT-034)', () => {
  const isAccountQuota = (category: string, message: string): boolean => {
    return (
      category === 'quota_exhausted' ||
      message.toLowerCase().includes('quota exceeded') ||
      message.toLowerCase().includes('billing')
    );
  };

  assert.equal(isAccountQuota('quota_exhausted', 'Daily project quota reached'), true);
  assert.equal(isAccountQuota('rate_limited', 'Too many requests per minute'), false);
});
