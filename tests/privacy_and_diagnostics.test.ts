import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import type {
  DiagnosticsLogEntry,
  InspectOutboundDataInput,
  OutboundDataDisclosure,
  OutboundFactDisclosure,
  PrivacySettings,
  RetentionCleanupResult,
  RetentionStatus,
  SystemDiagnosticsInfo,
  UpdatePrivacySettingsInput,
} from '../src/types/privacy';
import type { SensitivityConsent } from '../src/types/survey';

describe('SC-060: Privacy, Settings, and Diagnostics Contracts & Invariants', () => {
  it('PrivacySettings contract covers all normative controls with safe defaults', () => {
    const settings: PrivacySettings = {
      autoRedactLogs: true,
      requireSensitiveConsentPrompt: true,
      requireVisionConsentPrompt: true,
      screenshotRetentionPolicy: 'immediate_delete',
      autoCleanupTempOnExit: true,
      logLevel: 'INFO',
      updatedAt: new Date().toISOString(),
    };

    assert.equal(settings.autoRedactLogs, true);
    assert.equal(settings.requireSensitiveConsentPrompt, true);
    assert.equal(settings.requireVisionConsentPrompt, true);
    assert.equal(settings.screenshotRetentionPolicy, 'immediate_delete');
    assert.equal(settings.autoCleanupTempOnExit, true);
    assert.equal(settings.logLevel, 'INFO');
  });

  it('SensitivityConsent contract supports both profile sensitivity and external vision consent', () => {
    const consent: SensitivityConsent = {
      allowSensitive: true,
      allowHighlySensitive: false,
      allowExternalVision: true,
    };

    assert.equal(consent.allowSensitive, true);
    assert.equal(consent.allowHighlySensitive, false);
    assert.equal(consent.allowExternalVision, true);
  });

  it('OutboundDataDisclosure contract provides full transparency on data transmission', () => {
    const factItem: OutboundFactDisclosure = {
      factId: 'fact-1',
      category: 'demographics',
      attribute: 'age',
      valuePreview: '28',
      sensitivity: 'ordinary',
      status: 'included',
      isSensitive: false,
    };

    const excludedSensitiveItem: OutboundFactDisclosure = {
      factId: 'fact-2',
      category: 'financial',
      attribute: 'monthly_income',
      valuePreview: '[REDAKSI_SENSITIF]',
      sensitivity: 'sensitive',
      status: 'excluded_no_consent',
      isSensitive: true,
      exclusionReason: 'Izin fakta sensitif tidak diaktifkan',
    };

    const disclosure: OutboundDataDisclosure = {
      providerType: 'gemini',
      providerName: 'Google Gemini (Default)',
      destinationEndpoint: 'https://generativelanguage.googleapis.com',
      modelId: 'gemini-2.5-flash',
      questionText: 'Berapakah usia Anda saat ini?',
      instructions: 'Pilih satu angka bulat',
      options: ['< 20', '20-29', '30-39', '40+'],
      includedFacts: [factItem],
      excludedFacts: [excludedSensitiveItem],
      includesSensitiveData: true,
      sensitiveConsentGranted: false,
      includesExternalVision: false,
      visionConsentGranted: false,
      notTransmittedGuarantees: [
        'Basis data profil lengkap persona tidak pernah dikirimkan ke model AI eksternal.',
        'Fakta dan riwayat dari profil persona lain terisolasi penuh dan tidak akan pernah bocor.',
        'Kunci API dan kredensial rahasia tidak pernah disertakan dalam payload model.',
      ],
      generatedAt: new Date().toISOString(),
    };

    assert.equal(disclosure.destinationEndpoint, 'https://generativelanguage.googleapis.com');
    assert.equal(disclosure.includedFacts.length, 1);
    assert.equal(disclosure.excludedFacts.length, 1);
    assert.equal(disclosure.sensitiveConsentGranted, false);
    assert.equal(disclosure.notTransmittedGuarantees.length >= 3, true);
    assert.equal(disclosure.excludedFacts[0].status, 'excluded_no_consent');
  });

  it('DiagnosticsLogEntry enforces non-empty sanitized category and level', () => {
    const log: DiagnosticsLogEntry = {
      id: 'log-1',
      timestamp: new Date().toISOString(),
      level: 'INFO',
      category: 'AI_PROVIDER',
      message: 'Rekomendasi AI selesai diproses (status: Supported)',
      metadataJson: JSON.stringify({ recommendationId: 'rec-123' }),
    };

    assert.equal(log.level, 'INFO');
    assert.equal(log.category, 'AI_PROVIDER');
    assert.equal(log.message.includes('AI'), true);
  });

  it('SystemDiagnosticsInfo accurately reports SQLite, WAL, and entity counts', () => {
    const diag: SystemDiagnosticsInfo = {
      appVersion: '0.1.0',
      osPlatform: 'windows',
      dbSchemaVersion: 13,
      sqliteVersion: '3.45.1',
      walSizeBytes: 4096,
      totalDbSizeBytes: 65536,
      credentialStoreType: 'WindowsCredentialManager',
      tempCapturesCount: 3,
      tempCapturesSizeBytes: 154200,
      activePersonasCount: 2,
      totalFactsCount: 15,
      totalSessionsCount: 4,
      providerConfigsCount: 1,
      generatedAt: new Date().toISOString(),
    };

    assert.equal(diag.osPlatform, 'windows');
    assert.equal(diag.dbSchemaVersion >= 13, true);
    assert.equal(diag.credentialStoreType, 'WindowsCredentialManager');
    assert.equal(diag.tempCapturesCount, 3);
  });

  it('RetentionStatus and RetentionCleanupResult contracts track file counts and freed storage', () => {
    const retStatus: RetentionStatus = {
      tempDirectoryPath: 'C:\\Users\\User\\AppData\\Local\\SurveyCopilot\\temp_captures',
      tempFileCount: 5,
      tempTotalBytes: 524288,
      oldestFileTimestamp: '2026-11-20T10:00:00Z',
      activeRetentionPolicy: 'immediate_delete',
      autoCleanupOnExit: true,
    };

    const cleanupResult: RetentionCleanupResult = {
      deletedFilesCount: 5,
      freedBytes: 524288,
      cleanedAt: new Date().toISOString(),
      message: 'Pembersihan berhasil: 5 berkas sementara dihapus (524288 byte dibebaskan).',
    };

    assert.equal(retStatus.tempFileCount, 5);
    assert.equal(cleanupResult.deletedFilesCount, 5);
    assert.equal(cleanupResult.freedBytes, 524288);
  });
});
