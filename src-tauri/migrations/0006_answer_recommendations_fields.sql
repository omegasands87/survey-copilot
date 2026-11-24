-- Migration 0006: Answer Recommendations Enhancements
-- Adds persona_id, clarifying_question, confidence, and rejected_citations_json
-- to support structured AI generation, local evidence allowlist, and abstention reporting.

ALTER TABLE answer_recommendations ADD COLUMN persona_id TEXT REFERENCES personas(id) ON DELETE CASCADE;
ALTER TABLE answer_recommendations ADD COLUMN clarifying_question TEXT;
ALTER TABLE answer_recommendations ADD COLUMN confidence TEXT NOT NULL DEFAULT 'uncertain';
ALTER TABLE answer_recommendations ADD COLUMN rejected_citations_json TEXT NOT NULL DEFAULT '[]';

-- Backfill persona_id from parent survey_questions
UPDATE answer_recommendations
SET persona_id = (SELECT persona_id FROM survey_questions WHERE survey_questions.id = answer_recommendations.question_id)
WHERE persona_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_recommendations_persona ON answer_recommendations(persona_id, status);
