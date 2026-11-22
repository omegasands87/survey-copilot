-- Migration 0005: Survey Sessions and Questions Enhancement
-- Adds metadata for Survey Memory, Next Question Intelligence (NQI),
-- user drafts, confirmed answers, and strict persona isolation.

ALTER TABLE survey_sessions ADD COLUMN source_url TEXT;
ALTER TABLE survey_sessions ADD COLUMN external_survey_id TEXT;
ALTER TABLE survey_sessions ADD COLUMN language TEXT NOT NULL DEFAULT 'id';
ALTER TABLE survey_sessions ADD COLUMN category TEXT;
ALTER TABLE survey_sessions ADD COLUMN fingerprint TEXT;
ALTER TABLE survey_sessions ADD COLUMN notes TEXT;

ALTER TABLE survey_questions ADD COLUMN persona_id TEXT REFERENCES personas(id) ON DELETE CASCADE;
ALTER TABLE survey_questions ADD COLUMN question_type TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE survey_questions ADD COLUMN answer_format TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE survey_questions ADD COLUMN provenance TEXT NOT NULL DEFAULT 'manual_entry';
ALTER TABLE survey_questions ADD COLUMN observation_type TEXT NOT NULL DEFAULT 'detected';
ALTER TABLE survey_questions ADD COLUMN confirmation_status TEXT NOT NULL DEFAULT 'draft';
ALTER TABLE survey_questions ADD COLUMN user_answer_draft TEXT;
ALTER TABLE survey_questions ADD COLUMN user_answer_confirmed TEXT;
ALTER TABLE survey_questions ADD COLUMN confirmed_at TEXT;
ALTER TABLE survey_questions ADD COLUMN preceding_question_id TEXT REFERENCES survey_questions(id) ON DELETE SET NULL;
ALTER TABLE survey_questions ADD COLUMN branch_logic TEXT;

-- Backfill persona_id from parent session for existing records
UPDATE survey_questions 
SET persona_id = (SELECT persona_id FROM survey_sessions WHERE survey_sessions.id = survey_questions.session_id)
WHERE persona_id IS NULL;

-- High-performance indexes for persona isolation, ordering, and status queries
CREATE INDEX IF NOT EXISTS idx_survey_sessions_persona ON survey_sessions(persona_id, status);
CREATE INDEX IF NOT EXISTS idx_survey_questions_session_order ON survey_questions(session_id, question_order);
CREATE INDEX IF NOT EXISTS idx_survey_questions_persona ON survey_questions(persona_id);
CREATE INDEX IF NOT EXISTS idx_survey_questions_confirmation ON survey_questions(confirmation_status);
