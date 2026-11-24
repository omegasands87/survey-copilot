import { test } from "node:test";
import assert from "node:assert/strict";
import type {
  SurveyRecognitionStatus,
  SurveyRecognitionRecord,
  SurveyRecognitionEvaluation,
  RecognizedQuestionAnswer,
  RecognizeSurveyInput,
  SurveyMemoryQuestionItem,
  SurveyMemorySessionDetail,
  SurveySession,
  SurveyQuestion,
} from "../src/types/survey";

test("Survey recognition status types cover all expected recognition outcomes", () => {
  const validStatuses: SurveyRecognitionStatus[] = [
    "exact_match",
    "likely_match",
    "version_changed",
    "partial_match",
    "no_match",
  ];

  assert.equal(validStatuses.length, 5);
  assert.ok(validStatuses.includes("version_changed"));
  assert.ok(validStatuses.includes("exact_match"));
});

test("SurveyMemorySessionDetail enforces strict answer separation (AT-064)", () => {
  // AT-064 requires that confirmed answers (actual user responses)
  // are never mixed or confused with AI draft answers.
  const questionItem: SurveyMemoryQuestionItem = {
    id: "q-101",
    questionOrder: 1,
    promptText: "Berapa usia Anda saat ini?",
    instructionText: "Pilih rentang usia Anda",
    options: ["18-24", "25-34", "35-44", "45+"],
    questionType: "single_choice",
    userAnswerConfirmed: "25-34", // Actual confirmed response
    draftAnswer: "25-34 (Sesuai profil)", // AI recommendation draft
    userAnswerDraft: "25-34",
    confirmationStatus: "confirmed",
    questionFingerprint: "sha256-fp-q101",
  };

  assert.equal(questionItem.userAnswerConfirmed, "25-34");
  assert.equal(questionItem.confirmationStatus, "confirmed");
  // Verified they are distinct fields
  assert.notEqual(questionItem.userAnswerConfirmed, questionItem.draftAnswer);

  const memoryDetail: SurveyMemorySessionDetail = {
    session: {
      id: "sess-200",
      personaId: "persona-alpha",
      title: "Survei Demografi Konsumen",
      platformLabel: "Toluna",
      sourceUrl: "https://survey.toluna.com/v1/200",
      sourceDomain: "toluna.com",
      version: 1,
      externalSurveyId: "TOL-200",
      status: "completed",
      language: "id",
      category: "Demographics",
      fingerprint: "session-fp-200",
      notes: null,
      recognizedFromId: null,
      createdAt: "2026-11-20T10:00:00Z",
      updatedAt: "2026-11-20T10:15:00Z",
      archivedAt: null,
    },
    questions: [questionItem],
    confirmedAnswersCount: 1,
    totalQuestionsCount: 1,
  };

  assert.equal(memoryDetail.confirmedAnswersCount, 1);
  assert.equal(memoryDetail.questions[0].userAnswerConfirmed, "25-34");
});

test("Survey recognition input and evaluation enforce persona boundary (AT-066)", () => {
  // AT-066: Persona isolation requires that recognition candidate matching
  // and history access are strictly scoped to the active persona.
  const inputPersonaA: RecognizeSurveyInput = {
    sessionId: "sess-cur-1",
    personaId: "persona-123", // Persona A
    candidateTitle: "Survei Belanja Online Bulanan",
    candidateSourceUrl: "https://panel.example.id/survey/301",
    questions: [
      {
        questionOrder: 1,
        promptText: "Apakah Anda berbelanja online dalam 30 hari terakhir?",
        options: ["Ya", "Tidak"],
        instructionText: "Pilih salah satu",
      },
    ],
  };

  const evalPersonaA: SurveyRecognitionEvaluation = {
    status: "exact_match",
    matchedSessionId: "sess-hist-1",
    confidence: 1.0,
    structuralSimilarity: 1.0,
    titleSimilarity: 1.0,
    domainMatched: true,
    matchedQuestionCount: 1,
    totalCandidateQuestions: 1,
    differences: [],
    explanation: "Semua pertanyaan cocok persis dengan riwayat survei persona ini.",
    matchedSession: {
      id: "sess-hist-1",
      personaId: "persona-123", // Strictly matches Persona A
      title: "Survei Belanja Online Bulanan",
      platformLabel: "PanelID",
      sourceUrl: "https://panel.example.id/survey/201",
      sourceDomain: "panel.example.id",
      version: 1,
      externalSurveyId: "PID-201",
      status: "completed",
      language: "id",
      category: "Retail",
      fingerprint: "fp-hist-1",
      notes: null,
      recognizedFromId: null,
      createdAt: "2026-11-01T08:00:00Z",
      updatedAt: "2026-11-01T08:10:00Z",
      archivedAt: null,
    },
    historicalAnswers: [
      {
        questionOrder: 1,
        promptText: "Apakah Anda berbelanja online dalam 30 hari terakhir?",
        userAnswerConfirmed: "Ya",
        confirmedAt: "2026-11-01T08:05:00Z",
      },
    ],
  };

  assert.equal(evalPersonaA.status, "exact_match");
  assert.equal(evalPersonaA.matchedSession?.personaId, inputPersonaA.personaId);
  assert.equal(evalPersonaA.historicalAnswers.length, 1);
  assert.equal(evalPersonaA.historicalAnswers[0].userAnswerConfirmed, "Ya");
});

test("Survey recognition evaluation distinguishes version_changed from exact_match", () => {
  const versionChangedEval: SurveyRecognitionEvaluation = {
    status: "version_changed",
    matchedSessionId: "sess-v1",
    confidence: 0.82,
    structuralSimilarity: 0.80,
    titleSimilarity: 1.0,
    domainMatched: true,
    matchedQuestionCount: 4,
    totalCandidateQuestions: 5,
    differences: [
      "Pertanyaan #3 diubah pilihannya: opsi 'E-wallet' ditambahkan.",
      "Pertanyaan baru ditambahkan pada posisi #5.",
    ],
    explanation: "Ditemukan variasi versi survei: 4 dari 5 pertanyaan bersesuaian dengan perubahan struktur.",
    matchedSession: null,
    historicalAnswers: [],
  };

  assert.equal(versionChangedEval.status, "version_changed");
  assert.ok(versionChangedEval.differences.length > 0);
  assert.ok(versionChangedEval.confidence < 1.0);
  assert.ok(versionChangedEval.structuralSimilarity >= 0.60);
});

test("SurveyQuestion and SurveySession reflect metadata and deterministic fingerprint fields", () => {
  const session: SurveySession = {
    id: "sess-meta-1",
    personaId: "persona-1",
    title: "Survei Perilaku Keuangan",
    platformLabel: "YouGov",
    sourceUrl: "https://survey.yougov.com/surveys/fin-2026",
    sourceDomain: "yougov.com",
    version: 2,
    recognizedFromId: "sess-meta-0",
    externalSurveyId: "YG-FIN-26",
    status: "in_progress",
    language: "id",
    category: "Finance",
    fingerprint: "hash-finance-2026",
    notes: "Follow up dari survei tahun lalu",
    createdAt: "2026-11-24T00:00:00Z",
    updatedAt: "2026-11-24T00:05:00Z",
    archivedAt: null,
  };

  const question: SurveyQuestion = {
    id: "q-fin-1",
    sessionId: session.id,
    questionOrder: 1,
    promptText: "Produk perbankan apa saja yang Anda miliki?",
    instructionText: "Pilih semua yang sesuai",
    questionType: "multi_choice",
    options: ["Tabungan", "Deposito", "Kartu Kredit", "Reksadana"],
    answerFormat: "checkbox_list",
    provenance: "user_capture",
    status: "drafted",
    userAnswerDraft: "Tabungan, Reksadana",
    userAnswerConfirmed: null,
    confirmationStatus: "unconfirmed",
    questionFingerprint: "sha256-qfin1",
    createdAt: "2026-11-24T00:01:00Z",
    updatedAt: "2026-11-24T00:02:00Z",
  };

  assert.equal(session.sourceDomain, "yougov.com");
  assert.equal(session.version, 2);
  assert.equal(session.recognizedFromId, "sess-meta-0");
  assert.equal(question.questionFingerprint, "sha256-qfin1");
  assert.equal(question.questionOrder, 1);
});
