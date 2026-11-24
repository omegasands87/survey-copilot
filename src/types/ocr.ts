import type { AnswerFormat, ConfirmationStatus, QuestionType } from './survey';

export interface BoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface OcrWord {
  text: string;
  boundingBox: BoundingBox;
  confidence: number;
}

export type BlockRole =
  | 'prompt'
  | 'instruction'
  | 'option'
  | 'scale_label'
  | 'scale_number'
  | 'matrix_row'
  | 'matrix_column'
  | 'unit'
  | 'required_indicator'
  | 'noise'
  | 'unknown';

export interface OcrTextBlock {
  id: string;
  text: string;
  boundingBox: BoundingBox;
  confidence: number;
  words: OcrWord[];
  role: BlockRole;
}

export type UncertaintyReason =
  | 'low_confidence'
  | 'likely_clipped'
  | 'ambiguous_layout'
  | 'overlapping_elements';

export interface UncertainRegion {
  id: string;
  reason: UncertaintyReason;
  boundingBox?: BoundingBox;
  text?: string;
  details: string;
}

export interface ScaleStructure {
  minValue: number;
  maxValue: number;
  minLabel?: string;
  maxLabel?: string;
  allLabels: string[];
  step?: number;
}

export interface MatrixStructure {
  rows: string[];
  columns: string[];
  cellMatrix: (string | null)[][];
  isMultiSelect: boolean;
}

export interface ExtractedSurveyQuestion {
  extractionId: string;
  captureId?: string;
  sessionId?: string;
  personaId?: string;
  sourceMode: string;
  capturedAt: string;
  extractedAt: string;
  ocrEngine: string;
  applicationLabel?: string;
  imageWidth: number;
  imageHeight: number;

  rawOcrText: string;
  questionTextOriginal: string;
  questionTextCorrected?: string;
  instructions?: string;
  answerOptions: string[];
  questionType: QuestionType;
  answerFormat: AnswerFormat;

  scale?: ScaleStructure;
  matrix?: MatrixStructure;

  visibleUnits?: string;
  isRequired?: boolean;

  textBlocks: OcrTextBlock[];
  uncertainRegions: UncertainRegion[];
  warnings: string[];
  overallConfidence: number;

  confirmationStatus: ConfirmationStatus;
}

export interface ExtractOcrInput {
  captureId: string;
  sessionId?: string;
  personaId?: string;
  engine?: string;
}

export interface ConfirmOcrExtractionInput {
  extractionId: string;
  sessionId: string;
  personaId: string;
  captureId?: string;
  questionTextCorrected: string;
  instructions?: string;
  options: string[];
  questionType?: QuestionType;
  answerFormat?: AnswerFormat;
  scaleMin?: number;
  scaleMax?: number;
  scaleLabels?: string[];
  matrixRows?: string[];
  matrixColumns?: string[];
  visibleUnits?: string;
}
