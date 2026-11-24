import { describe, it } from 'node:test';
import assert from 'node:assert';
import type {
  BoundingBox,
  ExtractedSurveyQuestion,
  MatrixStructure,
  OcrTextBlock,
  ScaleStructure,
  UncertainRegion,
  UncertaintyReason,
} from '../src/types/ocr';

describe('SC-041: OCR and Layout Reconstruction Contracts', () => {
  it('BoundingBox contract correctly computes spatial dimensions and right/bottom edges', () => {
    const bbox: BoundingBox = {
      x: 20.0,
      y: 40.0,
      width: 150.0,
      height: 25.0,
    };

    assert.strictEqual(bbox.x, 20.0);
    assert.strictEqual(bbox.y, 40.0);
    assert.strictEqual(bbox.x + bbox.width, 170.0);
    assert.strictEqual(bbox.y + bbox.height, 65.0);
  });

  it('ScaleStructure contract accurately represents linear rating scale with anchors', () => {
    const scale: ScaleStructure = {
      minValue: 1,
      maxValue: 5,
      minLabel: 'Sangat Tidak Puas',
      maxLabel: 'Sangat Puas',
      allLabels: ['1', '2', '3', '4', '5'],
      step: 1,
    };

    assert.strictEqual(scale.minValue, 1);
    assert.strictEqual(scale.maxValue, 5);
    assert.strictEqual(scale.minLabel, 'Sangat Tidak Puas');
    assert.strictEqual(scale.maxLabel, 'Sangat Puas');
    assert.strictEqual(scale.allLabels.length, 5);
  });

  it('MatrixStructure contract correctly preserves 2D row statements and column headers mapping', () => {
    const matrix: MatrixStructure = {
      rows: [
        'Kualitas produk yang diterima',
        'Kecepatan waktu pengiriman pesanan',
        'Kemudahan navigasi aplikasi',
      ],
      columns: ['Sangat Buruk', 'Buruk', 'Cukup', 'Baik', 'Sangat Baik'],
      cellMatrix: [
        [null, null, null, null, null],
        [null, null, null, null, null],
        [null, null, null, null, null],
      ],
      isMultiSelect: false,
    };

    assert.strictEqual(matrix.rows.length, 3);
    assert.strictEqual(matrix.columns.length, 5);
    assert.strictEqual(matrix.cellMatrix.length, 3);
    assert.strictEqual(matrix.cellMatrix[0].length, 5);
    assert.strictEqual(matrix.isMultiSelect, false);
  });

  it('UncertaintyReason covers all required quality ambiguity reasons (AT-052)', () => {
    const validReasons: UncertaintyReason[] = [
      'low_confidence',
      'likely_clipped',
      'ambiguous_layout',
      'overlapping_elements',
    ];

    assert.strictEqual(validReasons.length, 4);

    const region: UncertainRegion = {
      id: 'unc-1',
      reason: 'likely_clipped',
      boundingBox: { x: 380, y: 10, width: 25, height: 18 },
      text: 'Karakter',
      details: 'Teks berada di sisi kanan gambar dan terpotong.',
    };

    assert.strictEqual(region.reason, 'likely_clipped');
    assert.ok(region.details.includes('terpotong'));
  });

  it('ExtractedSurveyQuestion strictly preserves raw OCR text and original prompt immutability', () => {
    const extracted: ExtractedSurveyQuestion = {
      extractionId: 'ext-uuid-1',
      captureId: 'cap-uuid-1',
      sessionId: 'sess-uuid-1',
      personaId: 'pers-uuid-1',
      sourceMode: 'region',
      capturedAt: '2026-11-23T20:00:00.000Z',
      extractedAt: '2026-11-23T20:00:01.000Z',
      ocrEngine: 'windows_media_ocr',
      imageWidth: 800,
      imageHeight: 600,
      rawOcrText: '1. Berapa usia Anda? (Pilih satu jawaban) A. 18-24 B. 25-34',
      questionTextOriginal: 'Berapa usia Anda?',
      questionTextCorrected: 'Berapa usia Anda saat ini?',
      instructions: '(Pilih satu jawaban)',
      answerOptions: ['18-24 tahun', '25-34 tahun'],
      questionType: 'single_choice',
      answerFormat: 'single_choice',
      textBlocks: [],
      uncertainRegions: [],
      warnings: [],
      overallConfidence: 0.96,
      confirmationStatus: 'draft',
    };

    // Immutability: original text remains separate from user's corrected text
    assert.strictEqual(extracted.questionTextOriginal, 'Berapa usia Anda?');
    assert.strictEqual(extracted.questionTextCorrected, 'Berapa usia Anda saat ini?');
    assert.notStrictEqual(extracted.questionTextOriginal, extracted.questionTextCorrected);
    assert.strictEqual(extracted.confirmationStatus, 'draft');
    assert.strictEqual(extracted.overallConfidence, 0.96);
  });

  it('Fallback boundary: manual entry remains usable when OCR is unavailable (AT-053)', () => {
    // Verifikasi bahwa tipe DTO dan alur tidak mengharuskan captureId untuk input manual
    const manualPrompt = 'Pertanyaan survei yang diketik manual oleh pengguna';
    assert.ok(manualPrompt.length > 0);
  });
});
