-- Migration 0008: OCR Extractions table
-- Stores structured OCR results, layout reconstruction provenance, bounding boxes,
-- confidence metrics, uncertain regions, and user confirmation state (SC-041).

CREATE TABLE IF NOT EXISTS ocr_extractions (
  id TEXT PRIMARY KEY,
  capture_id TEXT REFERENCES screen_captures(id) ON DELETE SET NULL,
  session_id TEXT REFERENCES survey_sessions(id) ON DELETE CASCADE,
  persona_id TEXT REFERENCES personas(id) ON DELETE CASCADE,
  ocr_engine TEXT NOT NULL,
  raw_ocr_text TEXT NOT NULL,
  question_text_original TEXT NOT NULL,
  question_text_corrected TEXT,
  instructions TEXT,
  options_json TEXT NOT NULL DEFAULT '[]',
  structure_json TEXT,
  text_blocks_json TEXT NOT NULL DEFAULT '[]',
  uncertain_regions_json TEXT NOT NULL DEFAULT '[]',
  warnings_json TEXT NOT NULL DEFAULT '[]',
  overall_confidence REAL NOT NULL DEFAULT 0.0,
  confirmation_status TEXT NOT NULL DEFAULT 'draft',
  extracted_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ocr_extractions_session ON ocr_extractions(session_id);
CREATE INDEX IF NOT EXISTS idx_ocr_extractions_capture ON ocr_extractions(capture_id);
CREATE INDEX IF NOT EXISTS idx_ocr_extractions_persona ON ocr_extractions(persona_id);
