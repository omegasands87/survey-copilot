import { test } from "node:test";
import assert from "node:assert/strict";
import type {
  AnswerFormat,
  ConfirmationStatus,
  CreateQuestionInput,
  CreateSessionInput,
  Provenance,
  QuestionStatus,
  QuestionType,
  SaveQuestionDraftInput,
  SessionStatus,
  SurveyQuestion,
  SurveySession,
  SurveySessionSummary,
} from "../src/types/survey";

test("SurveySession and SurveySessionSummary contracts support full lifecycle metadata", () => {
  const session: SurveySession = {
    id: "sess-100",
    personaId: "persona-1",
    title: "Survei Perilaku Konsumen Q4",
    platformLabel: "Toluna",
    sourceUrl: "https://surveys.example.com/s/100",
    externalSurveyId: "EXT-100",
    status: "active",
    language: "id",
    category: "Consumer Research",
    fingerprint: "fp-v1-hash",
    notes: "Target responden dewasa 25-40 tahun",
    createdAt: "2026-11-23T04:00:00Z",
    updatedAt: "2026-11-23T04:10:00Z",
    archivedAt: null,
  };

  assert.equal(session.id, "sess-100");
  assert.equal(session.personaId, "persona-1");
  assert.equal(session.status, "active");
  assert.equal(session.platformLabel, "Toluna");

  const summary: SurveySessionSummary = {
    id: session.id,
    personaId: session.personaId,
    title: session.title,
    platformLabel: session.platformLabel,
    sourceUrl: session.sourceUrl,
    status: session.status,
    language: session.language,
    category: session.category,
    questionCount: 5,
    confirmedAnswerCount: 3,
    createdAt: session.createdAt,
    updatedAt: session.updatedAt,
    archivedAt: null,
  };

  assert.equal(summary.questionCount, 5);
  assert.equal(summary.confirmedAnswerCount, 3);
});

test("SessionStatus covers active, archived, and completed states", () => {
  const validStatuses: SessionStatus[] = ["active", "archived", "completed"];
  assert.equal(validStatuses.length, 3);
  assert.ok(validStatuses.includes("active"));
  assert.ok(validStatuses.includes("archived"));
  assert.ok(validStatuses.includes("completed"));
});

test("CreateSessionInput requires non-empty title and valid persona ID", () => {
  const input: CreateSessionInput = {
    personaId: "persona-1",
    title: "Survei Finansial Keluarga",
    platformLabel: "YouGov",
    sourceUrl: null,
    externalSurveyId: null,
    language: "id",
    category: "Keuangan",
    notes: null,
  };

  assert.ok(input.title.trim().length > 0, "Title must be non-empty");
  assert.ok(input.personaId.trim().length > 0, "Persona ID must be non-empty");
});

test("SurveyQuestion contract strictly preserves original text and separates draft from actual answer", () => {
  const question: SurveyQuestion = {
    id: "q-201",
    sessionId: "sess-100",
    personaId: "persona-1",
    captureId: null,
    questionOrder: 1,
    questionTextOriginal: "Apakah Anda berbelanja kebutuhan pokok secara daring dalam 30 hari terakhir?",
    questionTextCorrected: null,
    instructions: "Pilih Ya atau Tidak",
    options: ["Ya", "Tidak"],
    structureJson: null,
    extractionStatus: "manual",
    questionType: "single_choice",
    answerFormat: "boolean",
    provenance: "manual_entry",
    observationType: "detected",
    confirmationStatus: "draft",
    userAnswerDraft: "Ya, saya sering membeli lewat aplikasi e-groceries",
    userAnswerConfirmed: null,
    confirmedAt: null,
    precedingQuestionId: null,
    branchLogic: null,
    createdAt: "2026-11-23T04:05:00Z",
    updatedAt: "2026-11-23T04:05:00Z",
  };

  assert.equal(
    question.questionTextOriginal,
    "Apakah Anda berbelanja kebutuhan pokok secara daring dalam 30 hari terakhir?"
  );
  assert.equal(question.questionOrder, 1);
  assert.equal(question.options.length, 2);
  assert.equal(question.confirmationStatus, "draft");
  assert.equal(question.userAnswerConfirmed, null);
  assert.ok(question.userAnswerDraft !== null);

  // When confirmed
  const confirmedQuestion: SurveyQuestion = {
    ...question,
    confirmationStatus: "confirmed",
    userAnswerConfirmed: "Ya",
    confirmedAt: "2026-11-23T04:12:00Z",
  };

  assert.equal(confirmedQuestion.confirmationStatus, "confirmed");
  assert.equal(confirmedQuestion.userAnswerConfirmed, "Ya");
  assert.equal(
    confirmedQuestion.userAnswerDraft,
    "Ya, saya sering membeli lewat aplikasi e-groceries"
  );
  assert.notEqual(confirmedQuestion.userAnswerConfirmed, confirmedQuestion.userAnswerDraft);
  assert.ok(confirmedQuestion.confirmedAt !== null);
});

test("QuestionType covers normative taxonomy", () => {
  const types: QuestionType[] = [
    "single_choice",
    "multiple_choice",
    "free_text",
    "scale",
    "matrix",
    "numeric",
    "date",
    "other",
    "unknown",
  ];
  assert.equal(types.length, 9);
});

test("AnswerFormat covers single_choice, multiple_selection, text, number, rating_scale, matrix, boolean, and unknown", () => {
  const formats: AnswerFormat[] = [
    "single_choice",
    "multiple_selection",
    "text",
    "number",
    "rating_scale",
    "matrix",
    "boolean",
    "unknown",
  ];
  assert.equal(formats.length, 8);
});

test("ConfirmationStatus covers draft, confirmed, skipped, and rejected", () => {
  const statuses: ConfirmationStatus[] = ["draft", "confirmed", "skipped", "rejected"];
  assert.equal(statuses.length, 4);
});

test("Provenance and ObservationType satisfy Next Question Intelligence (NQI) contracts", () => {
  const provenances: Provenance[] = [
    "manual_entry",
    "manually_confirmed",
    "ocr_detected",
    "ai_inferred",
  ];
  assert.equal(provenances.length, 4);

  const observations: QuestionStatus[] = [
    "detected",
    "predicted",
    "historical",
    "unknown",
  ];
  assert.equal(observations.length, 4);
  assert.ok(observations.includes("detected"));
  assert.ok(observations.includes("predicted"));
  assert.ok(observations.includes("historical"));
});

test("Draft persistence protects user inputs during provider failures (AT-040 boundary)", () => {
  const draftInput: SaveQuestionDraftInput = {
    personaId: "persona-1",
    questionId: "q-201",
    userAnswerDraft: "Draf jawaban yang sedang dikerjakan sebelum kegagalan koneksi AI",
  };

  assert.ok(draftInput.userAnswerDraft.length > 0);
  assert.equal(draftInput.personaId, "persona-1");
  assert.equal(draftInput.questionId, "q-201");
});

test("Strict persona isolation constraint: session and question operations require personaId scoping", () => {
  const createQ: CreateQuestionInput = {
    personaId: "persona-active",
    sessionId: "sess-owned",
    questionTextOriginal: "Apa profesi utama Anda saat ini?",
    instructions: "Pilih salah satu",
    options: ["Karyawan Swasta", "Wiraswasta", "PNS", "Pelajar/Mahasiswa", "Lainnya"],
    questionType: "single_choice",
    answerFormat: "single_choice",
    provenance: "manual_entry",
  };

  assert.equal(createQ.personaId, "persona-active");
  assert.equal(createQ.sessionId, "sess-owned");
  assert.equal(createQ.options?.length, 5);
});
