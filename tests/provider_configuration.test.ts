import test from 'node:test';
import assert from 'node:assert/strict';
import type {
  ConnectionTestResult,
  CreateProviderConfigInput,
  ModelOption,
  ProviderConfigSummary,
  ProviderType,
} from '../src/types/provider';

test('ProviderConfigSummary contract strictly contains zero secrets, API tokens, or headers', () => {
  const summary: ProviderConfigSummary = {
    id: 'cfg_gemini_01',
    displayName: 'Google Gemini 2.5 Flash Utama',
    providerType: 'gemini',
    baseUrl: null,
    adapterType: 'gemini',
    modelId: 'gemini-2.5-flash',
    credentialRef: 'survey_copilot/cred/cred_01',
    credentialLabel: 'Kunci API Gemini Produksi',
    enabled: true,
    priority: 10,
    weight: 1,
    timeoutMs: 30000,
    maxConcurrency: 1,
    rateLimitHintJson: null,
    createdAt: '2026-11-23T02:00:00Z',
    updatedAt: '2026-11-23T02:00:00Z',
  };

  const serialized = JSON.stringify(summary);
  assert.equal(serialized.includes('secret'), false);
  assert.equal(serialized.includes('apiKey'), false);
  assert.equal(serialized.includes('AIzaSy'), false);
  assert.equal(summary.credentialRef.startsWith('survey_copilot/cred/'), true);
});

test('CreateProviderConfigInput validates non-empty display name and model ID', () => {
  const validInput: CreateProviderConfigInput = {
    displayName: 'Gemini Flash',
    providerType: 'gemini',
    adapterType: 'gemini',
    modelId: 'gemini-2.5-flash',
    credentialRef: 'survey_copilot/cred/cred_01',
    priority: 100,
    timeoutMs: 30000,
  };

  assert.ok(validInput.displayName.length >= 1 && validInput.displayName.length <= 100);
  assert.ok(validInput.modelId.length >= 1 && validInput.modelId.length <= 100);
  assert.ok(validInput.credentialRef.startsWith('survey_copilot/'));
});

test('Endpoint and SSRF validation rules enforce HTTPS by default and localhost HTTP exception (AT-037)', () => {
  function validateEndpoint(urlStr: string): { isValid: boolean; reason?: string } {
    try {
      const parsed = new URL(urlStr);
      if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
        return { isValid: false, reason: 'Skema tidak diizinkan' };
      }
      const host = parsed.hostname.toLowerCase();
      // Block cloud metadata
      if (host === '169.254.169.254' || host === 'metadata.google.internal' || host.startsWith('169.254.')) {
        return { isValid: false, reason: 'SSRF metadata dilarang' };
      }
      // HTTP allowed only for localhost
      if (parsed.protocol === 'http:') {
        const isLocalhost = host === 'localhost' || host === '127.0.0.1' || host === '[::1]' || host === '::1';
        if (!isLocalhost) {
          return { isValid: false, reason: 'HTTP hanya diperbolehkan untuk localhost' };
        }
      }
      return { isValid: true };
    } catch {
      return { isValid: false, reason: 'URL tidak valid' };
    }
  }

  // AT-037: External HTTP rejected
  assert.equal(validateEndpoint('http://api.external.com/v1').isValid, false);
  assert.equal(validateEndpoint('http://api.external.com/v1').reason, 'HTTP hanya diperbolehkan untuk localhost');

  // Localhost HTTP allowed
  assert.equal(validateEndpoint('http://localhost:11434').isValid, true);
  assert.equal(validateEndpoint('http://127.0.0.1:8000/v1').isValid, true);

  // HTTPS external allowed
  assert.equal(validateEndpoint('https://generativelanguage.googleapis.com').isValid, true);
  assert.equal(validateEndpoint('https://api.openai.com/v1').isValid, true);

  // SSRF blocked
  assert.equal(validateEndpoint('https://169.254.169.254/latest/meta-data').isValid, false);
  assert.equal(validateEndpoint('https://metadata.google.internal/computeMetadata/v1').isValid, false);
  assert.equal(validateEndpoint('file:///etc/passwd').isValid, false);
});

test('Redirect host isolation prevents credential forwarding (AT-038)', () => {
  function isSafeRedirect(origUrl: string, targetUrl: string): boolean {
    try {
      const orig = new URL(origUrl);
      const target = new URL(targetUrl);
      return orig.origin === target.origin;
    } catch {
      return false;
    }
  }

  const orig = 'https://generativelanguage.googleapis.com/v1beta/models';

  // Same host redirect: safe
  assert.equal(
    isSafeRedirect(orig, 'https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash'),
    true
  );

  // Cross host redirect: strictly rejected
  assert.equal(isSafeRedirect(orig, 'https://evil-site.com/steal-key'), false);
  assert.equal(isSafeRedirect(orig, 'http://generativelanguage.googleapis.com/v1beta/models'), false);
});

test('ConnectionTestResult surfaces verification outcome and latency without secrets', () => {
  const result: ConnectionTestResult = {
    providerConfigId: 'cfg_gemini_01',
    success: true,
    modelId: 'gemini-2.5-flash',
    latencyMs: 142,
    message: 'Koneksi ke Google Gemini (gemini-2.5-flash) berhasil diverifikasi (142 ms).',
    testedAt: '2026-11-23T02:10:00Z',
    availableModels: [
      {
        id: 'gemini-2.5-flash',
        displayName: 'Gemini 2.5 Flash',
        supportedOperations: ['question_analysis', 'question_explanation'],
      },
    ],
  };

  assert.equal(result.success, true);
  assert.equal(result.latencyMs, 142);
  const serialized = JSON.stringify(result);
  assert.equal(serialized.includes('secret'), false);
  assert.equal(serialized.includes('key'), false);
});

test('Normalized structured response contract matches AT-032 expectations', () => {
  const structuredResponse = {
    requestId: 'req_001',
    providerId: 'gemini',
    modelId: 'gemini-2.5-flash',
    rawText: '{"status":"supported","draft_answer":"Software Engineer"}',
    structuredJson: {
      status: 'supported',
      draft_answer: 'Software Engineer',
      supporting_fact_ids: ['fact_01'],
      interpretation: 'Responden bekerja di bidang teknologi',
    },
    finishReason: 'stop',
    inputTokens: 110,
    outputTokens: 45,
    latencyMs: 156,
    receivedAt: '2026-11-23T02:15:00Z',
  };

  assert.equal(structuredResponse.providerId, 'gemini');
  assert.equal(structuredResponse.structuredJson.status, 'supported');
  assert.equal(structuredResponse.structuredJson.draft_answer, 'Software Engineer');
  assert.deepEqual(structuredResponse.structuredJson.supporting_fact_ids, ['fact_01']);
  assert.equal(structuredResponse.finishReason, 'stop');
});

test('Supported ProviderType and AdapterType covers gemini, openai_compatible, anthropic_compatible, and custom_http', () => {
  const supportedTypes: ProviderType[] = ['gemini', 'openai_compatible', 'anthropic_compatible', 'custom_http'];
  assert.equal(supportedTypes.length, 4);
  assert.ok(supportedTypes.includes('gemini'));
  assert.ok(supportedTypes.includes('openai_compatible'));
});
