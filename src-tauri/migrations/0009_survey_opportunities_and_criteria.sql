-- Migration 0009: Opportunity Capture and Criteria Extraction Enhancements (SC-050)
-- Adds raw source text, capture provenance, status, OCR extraction link, and structured criterion attributes.

ALTER TABLE survey_opportunities ADD COLUMN raw_text TEXT;
ALTER TABLE survey_opportunities ADD COLUMN provenance TEXT NOT NULL DEFAULT 'manual_entry';
ALTER TABLE survey_opportunities ADD COLUMN status TEXT NOT NULL DEFAULT 'captured';
ALTER TABLE survey_opportunities ADD COLUMN source_ocr_id TEXT REFERENCES ocr_extractions(id) ON DELETE SET NULL;

ALTER TABLE survey_opportunity_criteria ADD COLUMN raw_text TEXT;
ALTER TABLE survey_opportunity_criteria ADD COLUMN criterion_type TEXT NOT NULL DEFAULT 'general';
ALTER TABLE survey_opportunity_criteria ADD COLUMN operator TEXT NOT NULL DEFAULT 'equals';

CREATE INDEX IF NOT EXISTS idx_survey_opportunities_capture ON survey_opportunities(source_capture_id);
CREATE INDEX IF NOT EXISTS idx_survey_opportunities_ocr ON survey_opportunities(source_ocr_id);
CREATE INDEX IF NOT EXISTS idx_survey_opportunity_criteria_opp ON survey_opportunity_criteria(opportunity_id);
