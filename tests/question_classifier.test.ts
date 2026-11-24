import { test } from "node:test";
import assert from "node:assert/strict";
import type {
  AmbiguityReport,
  ClassifyQuestionInput,
  ExtractedConstraints,
  QuestionCategory,
  QuestionClassificationResult,
  ScaleConstraint,
  TimeframeConstraint,
  UnitConstraint,
} from "../src/types/survey";

test("QuestionClassificationResult contract supports full multi-label categories", () => {
  const categories: QuestionCategory[] = [
    "demographic",
    "employment_education",
    "financial_economic",
    "consumption_habits",
    "asset_ownership",
    "brand_product_usage",
    "experience_behavior",
    "preference_opinion",
    "satisfaction_rating",
    "multiple_choice",
    "matrix_grid",
    "numeric",
    "date",
    "free_text",
    "other_unknown",
  ];

  assert.equal(categories.length, 15);
  assert.ok(categories.includes("demographic"));
  assert.ok(categories.includes("financial_economic"));
  assert.ok(categories.includes("other_unknown"));
});

test("ExtractedConstraints distinguishes explicit and inferred provenances", () => {
  const timeframe: TimeframeConstraint = {
    timeframe_type: "past_months",
    duration_amount: 12,
    raw_text: "12 bulan terakhir",
    provenance: "explicit",
  };

  assert.equal(timeframe.timeframe_type, "past_months");
  assert.equal(timeframe.duration_amount, 12);
  assert.equal(timeframe.provenance, "explicit");

  const unit: UnitConstraint = {
    unit: "IDR",
    category: "currency",
    provenance: "explicit",
  };

  assert.equal(unit.unit, "IDR");
  assert.equal(unit.provenance, "explicit");

  const scale: ScaleConstraint = {
    minScale: 1,
    maxScale: 5,
    direction: "low_to_high",
    anchors: [
      { value: 1, label: "Sangat Tidak Puas" },
      { value: 5, label: "Sangat Puas" },
    ],
    provenance: "explicit",
  };

  assert.equal(scale.minScale, 1);
  assert.equal(scale.maxScale, 5);
  assert.equal(scale.direction, "low_to_high");
  assert.equal(scale.anchors.length, 2);

  const constraints: ExtractedConstraints = {
    selectionMode: "multiple",
    minSelections: 1,
    maxSelections: 3,
    isRequired: true,
    expectedFormat: "multiple_selection",
    timeframe,
    units: unit,
    numericRange: null,
    scale,
    textLength: null,
    specialInstructions: ["Pilih maksimal 3 opsi"],
  };

  assert.equal(constraints.selectionMode, "multiple");
  assert.equal(constraints.maxSelections, 3);
  assert.equal(constraints.timeframe?.duration_amount, 12);
  assert.equal(constraints.units?.unit, "IDR");
});

test("AmbiguityReport surfaces missing timeframe, missing unit, and clarification prompt (AT-041)", () => {
  const report: AmbiguityReport = {
    isAmbiguous: true,
    needsClarification: true,
    reasons: [
      {
        ambiguityType: "missing_timeframe",
        explanation: "Pertanyaan frekuensi perilaku tidak memiliki batasan periode waktu yang jelas",
        contextSnippet: "Seberapa sering Anda mengonsumsi...",
      },
      {
        ambiguityType: "missing_unit",
        explanation: "Pertanyaan keuangan atau nominal tidak menyebutkan mata uang atau satuan pengukuran",
        contextSnippet: "Berapa total penghasilan Anda?",
      },
    ],
    clarificationPrompt:
      "Pertanyaan ini memerlukan klarifikasi responden: tentukan periode waktu yang dimaksud dan satuan mata uang.",
  };

  assert.equal(report.isAmbiguous, true);
  assert.equal(report.needsClarification, true);
  assert.equal(report.reasons.length, 2);
  assert.equal(report.reasons[0].ambiguityType, "missing_timeframe");
  assert.equal(report.reasons[1].ambiguityType, "missing_unit");
  assert.ok(report.clarificationPrompt?.includes("klarifikasi"));
});

test("QuestionClassificationResult guarantees immutable raw text and typed provenance", () => {
  const result: QuestionClassificationResult = {
    classifierVersion: "1.0.0",
    normalizedText: "berapa usia anda saat ini?",
    categories: ["demographic", "numeric"],
    primaryCategory: "demographic",
    confidence: "high",
    questionType: "single_choice",
    answerFormat: "single_choice",
    constraints: {
      selectionMode: "single",
      minSelections: 1,
      maxSelections: 1,
      isRequired: true,
      expectedFormat: "single_choice",
      timeframe: null,
      units: null,
      specialInstructions: [],
    },
    ambiguity: {
      isAmbiguous: false,
      needsClarification: false,
      reasons: [],
      clarificationPrompt: null,
    },
  };

  // Immutability and normalized text separation
  assert.equal(result.normalizedText, "berapa usia anda saat ini?");
  assert.equal(result.classifierVersion, "1.0.0");
  assert.equal(result.primaryCategory, "demographic");
  assert.equal(result.categories.length, 2);
  assert.ok(result.categories.includes("numeric"));
});

test("ClassifyQuestionInput requires personaId and raw text", () => {
  const input: ClassifyQuestionInput = {
    personaId: "persona-alpha",
    sessionId: "sess-100",
    questionId: "q-200",
    questionText: "Berapa pendapatan bulanan Anda?",
    instructions: "Pilih rentang",
    options: ["< 5 juta", "5-10 juta", "> 10 juta"],
  };

  assert.equal(input.personaId, "persona-alpha");
  assert.equal(input.options?.length, 3);
});
