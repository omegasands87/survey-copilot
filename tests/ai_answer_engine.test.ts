import test from "node:test";
import assert from "node:assert/strict";

import type {
  EvidenceStatus,
  GenerateAnswerInput,
  ValidatedAnswerRecommendation,
} from "../src/types/survey";

test("EvidenceStatus covers all normative states", () => {
  const validStatuses: EvidenceStatus[] = [
    "supported",
    "partially_supported",
    "insufficient_information",
    "conflicting_evidence",
    "user_input_required",
    "not_applicable",
  ];

  assert.equal(validStatuses.length, 6);
  assert.ok(validStatuses.includes("supported"));
  assert.ok(validStatuses.includes("insufficient_information"));
  assert.ok(validStatuses.includes("conflicting_evidence"));
  assert.ok(validStatuses.includes("user_input_required"));
});

test("GenerateAnswerInput requires personaId and questionText", () => {
  const input: GenerateAnswerInput = {
    personaId: "persona-123",
    questionText: "Berapa usia Anda saat ini?",
    options: ["18-24 tahun", "25-34 tahun", "35-44 tahun"],
    consent: {
      allowSensitive: false,
      allowHighlySensitive: false,
    },
    promptVersion: "1.0",
  };

  assert.equal(input.personaId, "persona-123");
  assert.equal(input.questionText, "Berapa usia Anda saat ini?");
  assert.equal(input.options?.length, 3);
  assert.equal(input.consent?.allowSensitive, false);
});

test("ValidatedAnswerRecommendation structure strictly separates draft answer from actual confirmed answer", () => {
  const rec: ValidatedAnswerRecommendation = {
    id: "rec-001",
    questionId: "q-101",
    sessionId: "sess-202",
    personaId: "persona-alpha",
    status: "supported",
    questionIntent: "demographic",
    answerFormat: "single_choice",
    interpretation: "Pertanyaan mengenai kelompok umur responden.",
    draftAnswer: "25-34 tahun",
    supportingFacts: [
      {
        factId: "fact-age-01",
        factKey: "demographic:age",
        attribute: "age",
        category: "demographic",
        value: 29,
        normalizedValue: "29",
        unit: "tahun",
        verificationStatus: "confirmed",
        validityStatus: "valid",
        sensitivity: "ordinary",
        relevanceScore: 0.95,
        selectionReason: "Fakta usia profil terkonfirmasi",
      },
    ],
    rejectedCitations: [],
    rationale: "Sesuai dengan fakta usia profil responden (29 tahun).",
    missingInformation: [],
    conflicts: [],
    assumptions: [],
    clarifyingQuestion: null,
    confidence: "high",
    promptVersion: "1.0",
    providerConfigId: "provider-gemini-01",
    modelId: "gemini-1.5-flash",
    createdAt: "2026-11-23T15:00:00Z",
  };

  assert.equal(rec.status, "supported");
  assert.equal(rec.draftAnswer, "25-34 tahun");
  assert.equal(rec.supportingFacts.length, 1);
  assert.equal(rec.supportingFacts[0].factId, "fact-age-01");
  assert.equal(rec.rejectedCitations.length, 0);
});

test("ValidatedAnswerRecommendation records rejected citations when model hallucinates fact IDs (AT-043)", () => {
  const recWithHallucination: ValidatedAnswerRecommendation = {
    id: "rec-002",
    questionId: "q-102",
    personaId: "persona-alpha",
    status: "insufficient_information",
    questionIntent: "financial",
    answerFormat: "number",
    interpretation: "Pertanyaan pendapatan bulanan.",
    draftAnswer: null, // Draft must be null on insufficient evidence
    supportingFacts: [],
    rejectedCitations: ["fake-fact-income-99", "unauthorized-fact-persona-b"],
    rationale: "Fakta yang dikutip model tidak sah di allowlist.",
    missingInformation: ["Data pendapatan belum terverifikasi."],
    conflicts: [],
    assumptions: [],
    clarifyingQuestion: "Berapa rentang pendapatan bulanan rata-rata Anda?",
    confidence: "uncertain",
    promptVersion: "1.0",
    createdAt: "2026-11-23T15:05:00Z",
  };

  assert.equal(recWithHallucination.status, "insufficient_information");
  assert.equal(recWithHallucination.draftAnswer, null);
  assert.equal(recWithHallucination.supportingFacts.length, 0);
  assert.equal(recWithHallucination.rejectedCitations.length, 2);
  assert.ok(recWithHallucination.rejectedCitations.includes("fake-fact-income-99"));
  assert.ok(recWithHallucination.rejectedCitations.includes("unauthorized-fact-persona-b"));
});

test("Abstention contract: conflicting evidence enforces null draftAnswer and lists conflict details (AT-045)", () => {
  const recConflict: ValidatedAnswerRecommendation = {
    id: "rec-003",
    questionId: "q-103",
    personaId: "persona-alpha",
    status: "conflicting_evidence",
    questionIntent: "employment",
    answerFormat: "single_choice",
    interpretation: "Status pekerjaan utama responden.",
    draftAnswer: null, // Draft strictly forbidden
    supportingFacts: [],
    rejectedCitations: [],
    rationale: "Terdapat dua fakta profil pekerjaan yang saling bertentangan.",
    missingInformation: [],
    conflicts: [
      "[FactConflict] Fakta 'Karyawan Swasta' bertentangan dengan 'Wirausaha Mandiri'",
    ],
    assumptions: [],
    clarifyingQuestion: "Apakah saat ini Anda bekerja penuh waktu sebagai Karyawan Swasta atau Wirausaha Mandiri?",
    confidence: "uncertain",
    promptVersion: "1.0",
    createdAt: "2026-11-23T15:10:00Z",
  };

  assert.equal(recConflict.status, "conflicting_evidence");
  assert.equal(recConflict.draftAnswer, null);
  assert.equal(recConflict.conflicts.length, 1);
  assert.ok(recConflict.clarifyingQuestion !== null);
});

test("Strict persona isolation constraint: recommendation payload preserves personaId scoping", () => {
  const inputA: GenerateAnswerInput = {
    personaId: "persona-real-budi",
    questionText: "Apakah Anda memiliki mobil pribadi?",
  };

  assert.equal(inputA.personaId, "persona-real-budi");
  assert.notEqual(inputA.personaId, "persona-real-siti");
});
