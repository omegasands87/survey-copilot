import { invoke } from '@tauri-apps/api/core';
import type {
  ConfirmOcrExtractionInput,
  ExtractOcrInput,
  ExtractedSurveyQuestion,
} from '../types/ocr';
import type { SurveyQuestion } from '../types/survey';

export const ocrService = {
  /**
   * Menjalankan ekstraksi OCR dan rekonstruksi layout pada gambar tangkapan layar (SC-041)
   */
  async extractCapture(input: ExtractOcrInput): Promise<ExtractedSurveyQuestion> {
    return invoke<ExtractedSurveyQuestion>('ocr_extract_capture', { input });
  },

  /**
   * Mengambil hasil ekstraksi OCR berdasarkan ID
   */
  async getExtraction(extractionId: string): Promise<ExtractedSurveyQuestion> {
    return invoke<ExtractedSurveyQuestion>('ocr_get_extraction', { extractionId });
  },

  /**
   * Mengambil hasil ekstraksi OCR terbaru untuk captureId tertentu
   */
  async getLatestForCapture(captureId: string): Promise<ExtractedSurveyQuestion | null> {
    return invoke<ExtractedSurveyQuestion | null>('ocr_get_latest_for_capture', { captureId });
  },

  /**
   * Mengonfirmasi dan mengoreksi hasil ekstraksi OCR serta mengintegrasikannya ke sesi survei (PR-008)
   */
  async confirmExtraction(input: ConfirmOcrExtractionInput): Promise<SurveyQuestion> {
    return invoke<SurveyQuestion>('ocr_confirm_extraction', { input });
  },
};
