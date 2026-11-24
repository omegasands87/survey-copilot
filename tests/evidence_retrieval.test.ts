import { test } from "node:test";
import assert from "node:assert/strict";
import type {
  ConflictSeverity,
  ConflictType,
  EvidenceConfidence,
  EvidenceConflictIndicator,
  EvidencePackage,
  EvidenceRecommendation,
  EvidenceRetrievalInput,
  RetrievedEvidenceItem,
  SensitivityConsent,
  ValidityStatus,
} from "../src/types/survey";

test("EvidencePackage contract matches SC-032 specification", () => {
  const item: RetrievedEvidenceItem = {
    factId: "fact-123",
    sourceId: "source-456",
    factKey: "pekerjaan",
    attribute: "Pekerjaan",
    category: "employment_education",
    value: "Software Engineer",
    normalizedValue: "software engineer",
    unit: null,
    verificationStatus: "confirmed",
    validityStatus: "valid",
    sensitivity: "ordinary",
    relevanceScore: 0.85,
    selectionReason: "Kategori dan nama atribut cocok dengan teks pertanyaan",
    sourceType: "uploaded_document",
    sourceTitle: "Kontrak Kerja",
    sourceDescription: "Perjanjian kerja waktu tidak tertentu",
    sourceDate: "2026-01-01",
    validFrom: "2026-01-01",
    validUntil: null,
  };

  const conflict: EvidenceConflictIndicator = {
    conflictType: "fact_conflict",
    severity: "critical",
    description: "Dua fakta profil memiliki nilai yang bertentangan",
    primaryFactId: "fact-123",
    conflictingFactId: "fact-999",
    resolutionPrompt: "Pilih nilai yang benar",
  };

  const pkg: EvidencePackage = {
    questionId: "q-789",
    activePersonaId: "persona-abc",
    retrievedEvidence: [item],
    conflictIndicators: [conflict],
    missingInformation: [],
    evidenceConfidence: "high",
    recommendation: "supported",
    evaluationSummary: "1 bukti terkonfirmasi ditemukan dengan skor tinggi",
    evaluatedAt: "2026-11-23T14:00:00Z",
  };

  assert.equal(pkg.activePersonaId, "persona-abc");
  assert.equal(pkg.retrievedEvidence.length, 1);
  assert.equal(pkg.retrievedEvidence[0].factKey, "pekerjaan");
  assert.equal(pkg.retrievedEvidence[0].verificationStatus, "confirmed");
  assert.equal(pkg.conflictIndicators.length, 1);
  assert.equal(pkg.conflictIndicators[0].conflictType, "fact_conflict");
  assert.equal(pkg.recommendation, "supported");
});

test("EvidenceRecommendation covers supported, needs_clarification, and insufficient_evidence", () => {
  const recommendations: EvidenceRecommendation[] = [
    "supported",
    "needs_clarification",
    "insufficient_evidence",
  ];

  assert.equal(recommendations.length, 3);
  assert.ok(recommendations.includes("supported"));
  assert.ok(recommendations.includes("needs_clarification"));
  assert.ok(recommendations.includes("insufficient_evidence"));
});

test("ConflictType covers fact_conflict, historical_contradiction, timeframe_mismatch, value_change_over_time, unit_mismatch", () => {
  const conflictTypes: ConflictType[] = [
    "fact_conflict",
    "historical_contradiction",
    "timeframe_mismatch",
    "value_change_over_time",
    "unit_mismatch",
  ];

  assert.equal(conflictTypes.length, 5);
  assert.ok(conflictTypes.includes("historical_contradiction"));
  assert.ok(conflictTypes.includes("timeframe_mismatch"));
});

test("ValidityStatus covers valid, expired, future, unspecified", () => {
  const statuses: ValidityStatus[] = ["valid", "expired", "future", "unspecified"];
  assert.equal(statuses.length, 4);
  assert.ok(statuses.includes("expired"));
  assert.ok(statuses.includes("valid"));
});

test("SensitivityConsent distinguishes sensitive and highly_sensitive permissions", () => {
  const defaultConsent: SensitivityConsent = {
    allowSensitive: false,
    allowHighlySensitive: false,
  };

  assert.equal(defaultConsent.allowSensitive, false);
  assert.equal(defaultConsent.allowHighlySensitive, false);

  const grantedConsent: SensitivityConsent = {
    allowSensitive: true,
    allowHighlySensitive: false,
  };

  assert.equal(grantedConsent.allowSensitive, true);
  assert.equal(grantedConsent.allowHighlySensitive, false);
});

test("EvidenceRetrievalInput structure guarantees persona isolation parameter", () => {
  const input: EvidenceRetrievalInput = {
    personaId: "persona-strict-1",
    questionText: "Berapa penghasilan bulanan Anda?",
    instructions: "Pilih salah satu",
    options: ["< 5 juta", "5-10 juta", "> 10 juta"],
    consent: {
      allowSensitive: true,
      allowHighlySensitive: false,
    },
    maxEvidenceItems: 5,
  };

  assert.equal(input.personaId, "persona-strict-1");
  assert.equal(input.questionText, "Berapa penghasilan bulanan Anda?");
  assert.equal(input.options?.length, 3);
  assert.equal(input.consent?.allowSensitive, true);
  assert.equal(input.maxEvidenceItems, 5);
});
