import test from "node:test";
import assert from "node:assert/strict";

import type {
  ReviewAction,
  ReviewStatus,
  AnswerHistoryItem,
  EditAnswerInput,
  ConfirmRecommendationInput,
  CopyAnswerInput,
  RejectAnswerInput,
  RegenerateAnswerInput,
  ListAnswerHistoryInput,
  ValidatedAnswerRecommendation,
} from "../src/types/survey";

test("ReviewAction enum covers all required review lifecycle actions", () => {
  const actions: ReviewAction[] = [
    "edited",
    "confirmed",
    "copied",
    "rejected",
    "regenerated",
  ];

  assert.equal(actions.length, 5);
  assert.ok(actions.includes("edited"));
  assert.ok(actions.includes("confirmed"));
  assert.ok(actions.includes("copied"));
  assert.ok(actions.includes("rejected"));
  assert.ok(actions.includes("regenerated"));
});

test("ReviewStatus enum covers all normative status states", () => {
  const statuses: ReviewStatus[] = [
    "draft",
    "confirmed",
    "rejected",
    "edited",
  ];

  assert.equal(statuses.length, 4);
  assert.ok(statuses.includes("draft"));
  assert.ok(statuses.includes("confirmed"));
  assert.ok(statuses.includes("rejected"));
  assert.ok(statuses.includes("edited"));
});

test("Permanent separation between draft_answer and actual_answer in ValidatedAnswerRecommendation", () => {
  const rec: ValidatedAnswerRecommendation = {
    id: "rec-rev-1",
    questionId: "q-1",
    personaId: "persona-1",
    status: "supported",
    questionIntent: "education",
    answerFormat: "free_text",
    interpretation: "Pendidikan terakhir responden",
    draftAnswer: "S1",
    userAnswer: "S1 Teknik Elektro",
    reviewStatus: "edited",
    reviewReason: "Menambahkan jurusan secara mendetail",
    reviewedAt: "2026-11-23T10:00:00Z",
    supportingFacts: [],
    missingInformation: [],
    conflicts: [],
    assumptions: [],
    rejectedCitations: [],
    promptVersion: "1.0",
    confidenceScore: 0.95,
  };

  // Draft asli dari AI tidak pernah ditimpa oleh jawaban aktual pengguna
  assert.equal(rec.draftAnswer, "S1");
  assert.equal(rec.userAnswer, "S1 Teknik Elektro");
  assert.equal(rec.reviewStatus, "edited");
  assert.notEqual(rec.draftAnswer, rec.userAnswer);
});

test("AnswerHistoryItem accurately tracks revision lifecycle and provenance", () => {
  const historyItem: AnswerHistoryItem = {
    id: "hist-001",
    personaId: "persona-alpha",
    questionId: "q-101",
    recommendationId: "rec-001",
    action: "edited",
    draftAnswer: "25 tahun",
    actualAnswer: "27 tahun (koreksi manual)",
    reviewStatus: "edited",
    providerId: "provider-gemini-pro",
    modelId: "gemini-2.0-flash",
    promptVersion: "v1.2",
    evidenceSourcesJson: JSON.stringify([{ factId: "f-1", value: "27" }]),
    reason: "Menyesuaikan dengan usia saat ini pada tahun 2026",
    createdAt: "2026-11-23T11:00:00Z",
  };

  assert.equal(historyItem.personaId, "persona-alpha");
  assert.equal(historyItem.action, "edited");
  assert.equal(historyItem.draftAnswer, "25 tahun");
  assert.equal(historyItem.actualAnswer, "27 tahun (koreksi manual)");
  assert.equal(historyItem.reviewStatus, "edited");
  assert.equal(historyItem.providerId, "provider-gemini-pro");
  assert.equal(historyItem.reason, "Menyesuaikan dengan usia saat ini pada tahun 2026");
});

test("Review input contracts validate required personaId and targets", () => {
  const editInput: EditAnswerInput = {
    personaId: "persona-1",
    questionId: "q-1",
    recommendationId: "rec-1",
    actualAnswer: "Kopi Hitam",
    reason: "Preferensi spesifik pengguna",
  };
  assert.equal(editInput.personaId, "persona-1");
  assert.equal(editInput.actualAnswer, "Kopi Hitam");

  const confirmInput: ConfirmRecommendationInput = {
    personaId: "persona-1",
    questionId: "q-1",
    recommendationId: "rec-1",
    actualAnswer: "Kopi Hitam",
  };
  assert.equal(confirmInput.personaId, "persona-1");

  const copyInput: CopyAnswerInput = {
    personaId: "persona-1",
    questionId: "q-1",
    recommendationId: "rec-1",
    copiedText: "Kopi Hitam",
  };
  assert.equal(copyInput.copiedText, "Kopi Hitam");

  const rejectInput: RejectAnswerInput = {
    personaId: "persona-1",
    questionId: "q-1",
    recommendationId: "rec-1",
    reason: "Tidak relevan dengan kebiasaan",
  };
  assert.equal(rejectInput.reason, "Tidak relevan dengan kebiasaan");

  const regenerateInput: RegenerateAnswerInput = {
    personaId: "persona-1",
    questionId: "q-1",
    reason: "Butuh variasi jawaban alternatif",
  };
  assert.equal(regenerateInput.reason, "Butuh variasi jawaban alternatif");

  const listHistoryInput: ListAnswerHistoryInput = {
    personaId: "persona-1",
    questionId: "q-1",
    action: "edited",
  };
  assert.equal(listHistoryInput.action, "edited");
});

test("Strict product boundary: confirmation is purely internal review approval without auto-submit", () => {
  const confirmPayload: ConfirmRecommendationInput = {
    personaId: "p-secure",
    questionId: "q-secure",
    recommendationId: "rec-secure",
    actualAnswer: "Ya, saya setuju.",
  };

  // Pastikan payload tidak memiliki field atau instruksi pengiriman external/browser
  assert.equal("autoSubmit" in confirmPayload, false);
  assert.equal("browserContext" in confirmPayload, false);
  assert.equal("domTarget" in confirmPayload, false);
  assert.equal("autoPromoteToFact" in confirmPayload, false);
});
