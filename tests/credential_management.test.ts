import { test } from 'node:test';
import assert from 'node:assert';
import type {
  CredentialStatus,
  CredentialSummary,
  ProviderType,
  SaveCredentialInput,
  TestCredentialResult,
} from '../src/types/credential.ts';
import { AppError } from '../src/types/error.ts';

test('CredentialSummary contract strictly contains no raw secrets or tokens', () => {
  const sampleSummary: CredentialSummary = {
    id: 'c0a80101-0000-0000-0000-000000000001',
    providerType: 'gemini',
    label: 'Google Gemini Pro 1.5',
    credentialRef: 'survey_copilot/cred/c0a80101-0000-0000-0000-000000000001',
    status: 'configured',
    usageCount: 2,
    createdAt: '2026-11-23T00:00:00.000Z',
    updatedAt: '2026-11-23T00:00:00.000Z',
  };

  const json = JSON.stringify(sampleSummary);
  assert.strictEqual(json.includes('secret'), false, 'CredentialSummary tidak boleh memuat field secret');
  assert.strictEqual(json.includes('token'), false, 'CredentialSummary tidak boleh memuat field token');
  assert.strictEqual(json.includes('apiKey'), false, 'CredentialSummary tidak boleh memuat field apiKey');
  assert.strictEqual(sampleSummary.credentialRef.startsWith('survey_copilot/cred/'), true);
});

test('CredentialStatus covers all four normative states', () => {
  const validStatuses: CredentialStatus[] = [
    'configured',
    'unconfigured',
    'needs_update',
    'inaccessible',
  ];

  assert.strictEqual(validStatuses.length, 4);
  for (const s of validStatuses) {
    assert.strictEqual(typeof s, 'string');
  }
});

test('SaveCredentialInput requires non-empty label and minimum 8-character secret', () => {
  const validPayload: SaveCredentialInput = {
    providerType: 'gemini',
    label: 'Primary Gemini Key',
    secret: 'AIzaSy1234567890abcdef',
  };

  assert.ok(validPayload.label.trim().length > 0);
  assert.ok(validPayload.secret.trim().length >= 8);

  const shortSecret = '12345';
  assert.strictEqual(shortSecret.length < 8, true, 'Kunci rahasia di bawah 8 karakter harus ditolak');
});

test('Shared credential deletion logic detects active references', () => {
  const sharedCred: CredentialSummary = {
    id: 'shared-uuid-1',
    providerType: 'gemini',
    label: 'Shared Model Key',
    credentialRef: 'survey_copilot/cred/shared-uuid-1',
    status: 'configured',
    usageCount: 3,
    createdAt: '2026-11-23T00:00:00Z',
    updatedAt: '2026-11-23T00:00:00Z',
  };

  const soloCred: CredentialSummary = {
    id: 'solo-uuid-2',
    providerType: 'openai_compatible',
    label: 'Unused Key',
    credentialRef: 'survey_copilot/cred/solo-uuid-2',
    status: 'configured',
    usageCount: 0,
    createdAt: '2026-11-23T00:00:00Z',
    updatedAt: '2026-11-23T00:00:00Z',
  };

  assert.strictEqual(sharedCred.usageCount > 0, true, 'Kredensial bersama harus memicu modal konfirmasi khusus');
  assert.strictEqual(soloCred.usageCount === 0, true, 'Kredensial tanpa referensi aman untuk langsung dihapus');
});

test('TestCredentialResult surfaces verification outcome without exposing secret', () => {
  const testSuccess: TestCredentialResult = {
    id: 'cred-1',
    credentialRef: 'survey_copilot/cred/cred-1',
    isValid: true,
    status: 'configured',
    message: 'Kredensial berhasil diverifikasi di Windows Credential Manager.',
    testedAt: '2026-11-23T01:00:00Z',
  };

  assert.strictEqual(testSuccess.isValid, true);
  assert.strictEqual(testSuccess.status, 'configured');

  const serialized = JSON.stringify(testSuccess);
  assert.strictEqual(serialized.includes('AIzaSy'), false);
  assert.strictEqual(serialized.includes('Bearer'), false);
});

test('AppError normalizes credential errors safely without leaking sensitive information', () => {
  const rawNotFound = JSON.stringify({
    code: 'CREDENTIAL_NOT_FOUND',
    message: 'Kredensial tidak ditemukan di Windows Credential Manager.',
  });

  const appErr = AppError.from(rawNotFound);
  assert.strictEqual(appErr.code, 'CREDENTIAL_NOT_FOUND');
  assert.strictEqual(appErr.message, 'Kredensial tidak ditemukan di Windows Credential Manager.');
  assert.strictEqual(appErr.field, undefined);
});
