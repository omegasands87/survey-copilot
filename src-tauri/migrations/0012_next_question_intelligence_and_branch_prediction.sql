-- Migration 0012: Next Question Intelligence and Branch Prediction (SC-054)
-- Models observed question sequences, conditional branch transitions,
-- and prediction audit history with explicit 'predicted_not_detected' status.

CREATE TABLE IF NOT EXISTS survey_branch_transitions (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE,
    persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    from_question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE,
    from_question_fingerprint TEXT,
    from_question_order INTEGER NOT NULL,
    from_answer_value TEXT,
    to_question_id TEXT REFERENCES survey_questions(id) ON DELETE SET NULL,
    to_question_fingerprint TEXT,
    to_question_text TEXT NOT NULL,
    to_question_type TEXT NOT NULL DEFAULT 'unknown',
    to_options_json TEXT NOT NULL DEFAULT '[]',
    to_question_order INTEGER NOT NULL,
    branch_condition TEXT,
    transition_type TEXT NOT NULL CHECK(transition_type IN ('sequential', 'conditional_branch', 'skip_jump')),
    observed_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS survey_question_predictions (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE,
    persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    current_question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE,
    predicted_question_text TEXT NOT NULL,
    predicted_question_type TEXT NOT NULL DEFAULT 'unknown',
    predicted_options_json TEXT NOT NULL DEFAULT '[]',
    confidence_score REAL NOT NULL,
    evidence_source TEXT NOT NULL,
    explanation TEXT NOT NULL,
    leading_condition TEXT,
    status TEXT NOT NULL CHECK(status IN ('predicted_not_detected', 'detected_matched', 'detected_diverged', 'insufficient_evidence')),
    matched_question_id TEXT REFERENCES survey_questions(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_branch_transitions_from_fp ON survey_branch_transitions(from_question_fingerprint);
CREATE INDEX IF NOT EXISTS idx_branch_transitions_session ON survey_branch_transitions(session_id);
CREATE INDEX IF NOT EXISTS idx_branch_transitions_persona ON survey_branch_transitions(persona_id);
CREATE INDEX IF NOT EXISTS idx_branch_transitions_from_q ON survey_branch_transitions(from_question_id);

CREATE INDEX IF NOT EXISTS idx_q_predictions_session ON survey_question_predictions(session_id);
CREATE INDEX IF NOT EXISTS idx_q_predictions_persona ON survey_question_predictions(persona_id);
CREATE INDEX IF NOT EXISTS idx_q_predictions_current_q ON survey_question_predictions(current_question_id);
CREATE INDEX IF NOT EXISTS idx_q_predictions_status ON survey_question_predictions(status);
