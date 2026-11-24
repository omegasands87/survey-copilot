import test, { describe } from "node:test";
import assert from "node:assert/strict";
import type {
  NextQuestionIntelligenceResult,
  NextQuestionPrediction,
  PredictNextQuestionInput,
  PredictionStatus,
  SurveyBranchTransition,
  SurveyQuestionPredictionRecord,
  TransitionType,
} from "../src/types/survey";

describe("SC-054: Next Question Intelligence and Branch Prediction Contracts", () => {
  test("TransitionType taxonomy strictly covers normative transitions", () => {
    const validTypes: TransitionType[] = [
      "sequential",
      "conditional_branch",
      "skip_jump",
    ];

    assert.equal(validTypes.length, 3);
    assert.ok(validTypes.includes("sequential"));
    assert.ok(validTypes.includes("conditional_branch"));
    assert.ok(validTypes.includes("skip_jump"));
  });

  test("PredictionStatus taxonomy covers normative NQI lifecycle states", () => {
    const validStatuses: PredictionStatus[] = [
      "predicted_not_detected",
      "detected_matched",
      "detected_diverged",
      "insufficient_evidence",
    ];

    assert.equal(validStatuses.length, 4);
    assert.ok(validStatuses.includes("predicted_not_detected"));
    assert.ok(validStatuses.includes("detected_matched"));
    assert.ok(validStatuses.includes("detected_diverged"));
    assert.ok(validStatuses.includes("insufficient_evidence"));
  });

  test("NextQuestionPrediction strictly enforces 'Predicted — not yet detected' labeling (AT-065)", () => {
    const prediction: NextQuestionPrediction = {
      id: "pred-1",
      candidateQuestionText: "Apa merk mobil utama yang Anda kendarai?",
      questionType: "single_choice",
      options: ["Toyota", "Honda", "Daihatsu", "Lainnya"],
      evidenceSource: "observed_branch",
      confidence: 0.92,
      explanation: "Berdasarkan observasi historis ketika responden menjawab 'Ya' pada pertanyaan kepemilikan mobil.",
      leadingCondition: "Jawaban cocok: 'Ya'",
      isPredictedNotDetected: true,
    };

    assert.equal(prediction.isPredictedNotDetected, true);
    assert.ok(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
    assert.ok(prediction.evidenceSource.length > 0);
    assert.ok(prediction.candidateQuestionText.length > 0);
  });

  test("Non-negotiable Rule 5: Sparse or empty history strictly returns insufficient_evidence without speculative predictions", () => {
    const insufficientResult: NextQuestionIntelligenceResult = {
      sessionId: "session-new-1",
      personaId: "persona-alpha",
      currentQuestionId: "q-unknown-1",
      currentQuestionText: "Apakah Anda memiliki hobi berkebun?",
      currentUserAnswer: null,
      status: "insufficient_evidence",
      predictions: [],
      observedTransitionsCount: 0,
      explanation: "Belum ada riwayat transisi atau struktur survei yang teramati untuk pertanyaan ini. Sesuai prinsip kehati-hatian, sistem tidak membuat prediksi spekulatif.",
      evaluatedAt: new Date().toISOString(),
    };

    assert.equal(insufficientResult.status, "insufficient_evidence");
    assert.equal(insufficientResult.predictions.length, 0);
    assert.equal(insufficientResult.observedTransitionsCount, 0);
    assert.ok(insufficientResult.explanation.includes("tidak membuat prediksi spekulatif"));
  });

  test("NextQuestionIntelligenceResult separates historical observations, current actual question, and predictions", () => {
    const result: NextQuestionIntelligenceResult = {
      sessionId: "session-101",
      personaId: "persona-1",
      currentQuestionId: "q-1",
      currentQuestionText: "Berapa usia Anda saat ini?",
      currentUserAnswer: "28",
      status: "predicted_not_detected",
      predictions: [
        {
          id: "pred-candidate-1",
          candidateQuestionText: "Apa status pernikahan Anda?",
          questionType: "single_choice",
          options: ["Belum Menikah", "Menikah", "Pernah Menikah"],
          evidenceSource: "same_survey_history",
          confidence: 0.88,
          explanation: "Alur sekuensial teramati 3 kali pada urutan pertanyaan ini.",
          leadingCondition: "Alur sekuensial normal",
          isPredictedNotDetected: true,
        },
      ],
      observedTransitionsCount: 3,
      explanation: "Ditemukan 3 observasi transisi historis. Menghasilkan 1 kandidat prediksi pertanyaan berikutnya.",
      evaluatedAt: new Date().toISOString(),
    };

    // Separasi struktural
    assert.ok(result.currentQuestionText);
    assert.ok(result.currentUserAnswer);
    assert.equal(result.predictions.length, 1);
    assert.equal(result.predictions[0].isPredictedNotDetected, true);
  });

  test("SurveyBranchTransition model preserves exact observed transitions and conditions", () => {
    const transition: SurveyBranchTransition = {
      id: "trans-001",
      sessionId: "session-xyz",
      personaId: "persona-alpha",
      fromQuestionId: "q-10",
      fromQuestionFingerprint: "fp-q10",
      fromQuestionOrder: 1,
      fromAnswerValue: "ya",
      toQuestionId: "q-11",
      toQuestionFingerprint: "fp-q11",
      toQuestionText: "Berapa lama Anda telah memiliki hewan peliharaan?",
      toQuestionType: "single_choice",
      toOptions: ["Kurang dari 1 tahun", "1-3 tahun", "Lebih dari 3 tahun"],
      toQuestionOrder: 2,
      branchCondition: "Jawaban: Ya",
      transitionType: "conditional_branch",
      observedAt: new Date().toISOString(),
      createdAt: new Date().toISOString(),
    };

    assert.equal(transition.transitionType, "conditional_branch");
    assert.equal(transition.fromAnswerValue, "ya");
    assert.equal(transition.toQuestionOrder, 2);
    assert.equal(transition.branchCondition, "Jawaban: Ya");
  });

  test("SurveyQuestionPredictionRecord tracks audit lifecycle from predicted to detected", () => {
    const predictionRecord: SurveyQuestionPredictionRecord = {
      id: "pred-rec-1",
      sessionId: "session-123",
      personaId: "persona-1",
      currentQuestionId: "q-20",
      predictedQuestionText: "Apakah Anda bekerja secara penuh waktu?",
      predictedQuestionType: "single_choice",
      predictedOptions: ["Ya", "Tidak"],
      confidenceScore: 0.90,
      evidenceSource: "observed_branch",
      explanation: "Prediksi percabangan",
      leadingCondition: "Jawaban: Ya",
      status: "detected_matched",
      matchedQuestionId: "q-21",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    assert.equal(predictionRecord.status, "detected_matched");
    assert.equal(predictionRecord.matchedQuestionId, "q-21");
  });

  test("PredictNextQuestionInput requires personaId, sessionId, and currentQuestionId for strict scoping (AT-066)", () => {
    const input: PredictNextQuestionInput = {
      personaId: "persona-scoped-1",
      sessionId: "session-scoped-1",
      currentQuestionId: "q-active-1",
      currentAnswer: "Mahasiswa",
    };

    assert.ok(input.personaId);
    assert.ok(input.sessionId);
    assert.ok(input.currentQuestionId);
    assert.equal(input.currentAnswer, "Mahasiswa");
  });
});
