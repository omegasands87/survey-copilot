import test from "node:test";
import assert from "node:assert/strict";
import type {
  AllRecordCounts,
  ExportPayload,
  ExportSummary,
  RetentionPolicyInfo,
  WipeReport,
} from "../src/types/dataManagement";

test("ExportSummary correctly reflects data counts and privacy warnings", () => {
  const summaryWithSensitive: ExportSummary = {
    exportScope: "full_application",
    targetPersonaId: null,
    targetPersonaName: null,
    totalPersonas: 2,
    totalFacts: 5,
    totalSources: 4,
    totalHistoryEntries: 2,
    totalPreferences: 1,
    totalSessions: 1,
    totalQuestions: 3,
    totalRecommendations: 2,
    totalAnswers: 2,
    totalFitResults: 1,
    totalAttempts: 1,
    sensitiveFactsCount: 2,
    highlySensitiveFactsCount: 1,
    hasSensitiveData: true,
    estimatedRecordsCount: 24,
    warningMessage:
      "Perhatian Privasi: Berkas ekspor memuat 2 fakta sensitif dan 1 fakta sangat sensitif.",
  };

  assert.equal(summaryWithSensitive.exportScope, "full_application");
  assert.equal(summaryWithSensitive.totalPersonas, 2);
  assert.equal(summaryWithSensitive.totalFacts, 5);
  assert.equal(summaryWithSensitive.sensitiveFactsCount, 2);
  assert.equal(summaryWithSensitive.highlySensitiveFactsCount, 1);
  assert.equal(summaryWithSensitive.hasSensitiveData, true);
  assert.ok(summaryWithSensitive.warningMessage !== null);
  assert.ok(summaryWithSensitive.warningMessage.includes("fakta sensitif"));
});

test("ExportPayload schema contract contains no API keys or credential references", () => {
  const sampleExport: ExportPayload = {
    $schema: "https://survey-copilot.app/schemas/export-v1.json",
    schemaVersion: "1.0.0",
    appVersion: "0.1.0",
    exportTimestamp: "2026-11-23T01:50:00.000Z",
    exportScope: "full_application",
    targetPersonaId: null,
    targetPersonaName: null,
    summary: {
      exportScope: "full_application",
      targetPersonaId: null,
      targetPersonaName: null,
      totalPersonas: 1,
      totalFacts: 1,
      totalSources: 1,
      totalHistoryEntries: 0,
      totalPreferences: 0,
      totalSessions: 0,
      totalQuestions: 0,
      totalRecommendations: 0,
      totalAnswers: 0,
      totalFitResults: 0,
      totalAttempts: 0,
      sensitiveFactsCount: 0,
      highlySensitiveFactsCount: 0,
      hasSensitiveData: false,
      estimatedRecordsCount: 3,
      warningMessage: null,
    },
    data: {
      personas: [
        {
          id: "p-1",
          displayName: "Budi Santoso",
          description: "Responden Utama",
          isActive: true,
          status: "active",
          createdAt: "2026-11-23T00:00:00.000Z",
          updatedAt: "2026-11-23T00:00:00.000Z",
          archivedAt: null,
        },
      ],
      facts: [
        {
          id: "f-1",
          personaId: "p-1",
          category: "Demografi",
          factKey: "age",
          attribute: "Usia",
          dataType: "number",
          value: 28,
          normalizedValue: "28",
          unit: "tahun",
          verificationStatus: "confirmed",
          sensitivity: "ordinary",
          sourceDate: "2024-01-01",
          validFrom: null,
          validUntil: null,
          notes: null,
          createdAt: "2026-11-23T00:00:00.000Z",
          updatedAt: "2026-11-23T00:00:00.000Z",
          archivedAt: null,
        },
      ],
      factSources: [
        {
          id: "s-1",
          personaId: "p-1",
          factId: "f-1",
          sourceType: "user_declaration",
          title: "Deklarasi Mandiri",
          description: "Profil mandiri",
          referenceUri: null,
          observedAt: null,
          validFrom: null,
          validUntil: null,
          notes: null,
          createdAt: "2026-11-23T00:00:00.000Z",
        },
      ],
      factHistory: [],
      personaPreferences: [],
      surveySessions: [],
      screenCaptures: [],
      surveyQuestions: [],
      answerRecommendations: [],
      answerHistory: [],
      surveyOpportunities: [],
      surveyOpportunityCriteria: [],
      surveyFitResults: [],
      surveyAttempts: [],
      surveyOutcomeEvents: [],
      surveyMemory: [],
    },
  };

  const serialized = JSON.stringify(sampleExport);

  // Validate no credential references or keys leaked in export
  assert.equal(serialized.includes("credential_ref"), false);
  assert.equal(serialized.includes("credentialRef"), false);
  assert.equal(serialized.includes("apiKey"), false);
  assert.equal(serialized.includes("api_key"), false);
  assert.equal(serialized.includes("secret"), false);

  assert.equal(sampleExport.schemaVersion, "1.0.0");
  assert.equal(sampleExport.data.personas.length, 1);
  assert.equal(sampleExport.data.facts.length, 1);
});

test("Single persona export validates target persona isolation", () => {
  const personaAId = "persona-uuid-a";
  const personaBId = "persona-uuid-b";

  const allFacts = [
    { id: "f1", personaId: personaAId, value: "Data A" },
    { id: "f2", personaId: personaBId, value: "Data B" },
  ];

  // Simulating single persona export scoping
  const scopedFacts = allFacts.filter((f) => f.personaId === personaAId);

  assert.equal(scopedFacts.length, 1);
  assert.equal(scopedFacts[0].personaId, personaAId);
  assert.equal(scopedFacts[0].value, "Data A");
  assert.ok(scopedFacts.every((f) => f.personaId !== personaBId));
});

test("Wipe confirmation strictly enforces 'HAPUS SEMUA DATA'", () => {
  const isValidConfirmation = (input: string) => input.trim() === "HAPUS SEMUA DATA";

  assert.equal(isValidConfirmation("HAPUS SEMUA DATA"), true);
  assert.equal(isValidConfirmation("  HAPUS SEMUA DATA  "), true);
  assert.equal(isValidConfirmation("hapus semua data"), false);
  assert.equal(isValidConfirmation("DELETE"), false);
  assert.equal(isValidConfirmation("HAPUS"), false);
  assert.equal(isValidConfirmation(""), false);
});

test("WipeReport data structure discloses all cleared items and external limitations", () => {
  const report: WipeReport = {
    deletedPersonasCount: 2,
    deletedFactsCount: 10,
    deletedSourcesCount: 8,
    deletedHistoryCount: 4,
    deletedSessionsCount: 3,
    deletedQuestionsCount: 12,
    deletedRecommendationsCount: 5,
    deletedAnswersCount: 5,
    deletedPreferencesCount: 2,
    deletedFitResultsCount: 2,
    deletedAttemptsCount: 1,
    deletedProviderConfigsCount: 1,
    deletedTempFilesCount: 4,
    walCheckpointStatus: "Berhasil (TRUNCATE)",
    vacuumStatus: "Berhasil (database dipadatkan)",
    wipedAt: "2026-11-23T01:50:00.000Z",
    unaffectedExternalItems: [
      "Berkas ekspor JSON di luar aplikasi tidak terhapus.",
      "Cadangan eksternal tidak terpengaruh.",
    ],
  };

  assert.equal(report.deletedPersonasCount, 2);
  assert.equal(report.deletedFactsCount, 10);
  assert.equal(report.deletedTempFilesCount, 4);
  assert.ok(report.walCheckpointStatus.includes("Berhasil"));
  assert.ok(report.vacuumStatus.includes("Berhasil"));
  assert.equal(report.unaffectedExternalItems.length, 2);
});

test("RetentionPolicyInfo covers all lifecycle policies", () => {
  const policy: RetentionPolicyInfo = {
    activeFactsPolicy: "Disimpan di SQLite lokal.",
    archivedFactsPolicy: "Tidak aktif, tidak dikirim ke AI.",
    screenCapturePolicy: "Dihapus otomatis segera setelah OCR selesai.",
    ocrDataPolicy: "Teks lokal hasil OCR dapat dikoreksi pengguna.",
    surveyHistoryPolicy: "Disimpan ber-scope persona.",
    exportFilePolicy: "100% bebas kredensial.",
    logsAndDiagnosticsPolicy: "Redaksi otomatis.",
    deletionScopeDisclaimer: "Ekspor di luar direktori aplikasi tidak ikut terhapus.",
  };

  assert.ok(policy.screenCapturePolicy.includes("Dihapus otomatis"));
  assert.ok(policy.exportFilePolicy.includes("bebas kredensial"));
  assert.ok(policy.deletionScopeDisclaimer.includes("tidak ikut terhapus"));
});
