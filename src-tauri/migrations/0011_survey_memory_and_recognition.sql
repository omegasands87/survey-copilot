-- Migration 0011: Survey Memory and Recognition Enhancement (SC-053)
-- Adds support for structural question sequence fingerprinting, versioning,
-- survey recognition auditing, and strict persona-scoped memory retrieval.

ALTER TABLE survey_sessions ADD COLUMN source_domain TEXT;
ALTER TABLE survey_sessions ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE survey_sessions ADD COLUMN recognized_from_id TEXT REFERENCES survey_sessions(id) ON DELETE SET NULL;

ALTER TABLE survey_questions ADD COLUMN question_fingerprint TEXT;

CREATE TABLE IF NOT EXISTS survey_recognitions (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE,
    persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    matched_session_id TEXT REFERENCES survey_sessions(id) ON DELETE SET NULL,
    match_status TEXT NOT NULL CHECK(match_status IN ('exact_match', 'version_changed', 'likely_match', 'possible_match', 'no_match')),
    confidence_score REAL NOT NULL,
    structural_similarity REAL NOT NULL,
    title_similarity REAL NOT NULL,
    matched_question_count INTEGER NOT NULL DEFAULT 0,
    total_question_count INTEGER NOT NULL DEFAULT 0,
    differences_json TEXT NOT NULL DEFAULT '[]',
    provenance TEXT NOT NULL DEFAULT 'structural_fingerprint',
    recognized_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_recognitions_session ON survey_recognitions(session_id);
CREATE INDEX IF NOT EXISTS idx_recognitions_persona ON survey_recognitions(persona_id);
CREATE INDEX IF NOT EXISTS idx_recognitions_matched ON survey_recognitions(matched_session_id);
CREATE INDEX IF NOT EXISTS idx_survey_sessions_fingerprint ON survey_sessions(fingerprint);
CREATE INDEX IF NOT EXISTS idx_survey_sessions_domain ON survey_sessions(source_domain);
CREATE INDEX IF NOT EXISTS idx_survey_questions_fingerprint ON survey_questions(question_fingerprint);
CREATE INDEX IF NOT EXISTS idx_survey_questions_session_persona ON survey_questions(session_id, persona_id, question_order);
