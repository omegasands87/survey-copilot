-- Migration 0007: Review Actions and History Enhancement
-- Adds support for review actions (edit, confirm, copy, reject, regenerate),
-- distinct draft_answer vs actual_answer separation, persona isolation,
-- prompt metadata, evidence references, and reason tracking.

ALTER TABLE answer_recommendations ADD COLUMN user_answer TEXT;
ALTER TABLE answer_recommendations ADD COLUMN review_status TEXT NOT NULL DEFAULT 'draft';
ALTER TABLE answer_recommendations ADD COLUMN review_reason TEXT;
ALTER TABLE answer_recommendations ADD COLUMN reviewed_at TEXT;

CREATE TABLE IF NOT EXISTS answer_history_new (
  id TEXT PRIMARY KEY,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE,
  recommendation_id TEXT REFERENCES answer_recommendations(id) ON DELETE SET NULL,
  action TEXT NOT NULL CHECK(action IN ('edited', 'confirmed', 'copied', 'rejected', 'regenerated')),
  draft_answer TEXT,
  actual_answer TEXT NOT NULL,
  review_status TEXT NOT NULL DEFAULT 'draft',
  provider_id TEXT,
  model_id TEXT,
  prompt_version TEXT,
  evidence_sources_json TEXT NOT NULL DEFAULT '[]',
  reason TEXT,
  created_at TEXT NOT NULL
);

INSERT INTO answer_history_new (
  id, persona_id, question_id, recommendation_id, action, draft_answer, actual_answer, review_status, provider_id, model_id, prompt_version, evidence_sources_json, reason, created_at
)
SELECT 
  ah.id,
  COALESCE(sq.persona_id, 'p-default'),
  ah.question_id,
  ah.recommendation_id,
  ah.action,
  ar.draft_text,
  ah.answer_text,
  CASE WHEN ah.action = 'confirmed' THEN 'confirmed' WHEN ah.action = 'rejected' THEN 'rejected' ELSE 'draft' END,
  ar.provider_config_id,
  ar.model_id,
  ar.prompt_version,
  '[]',
  NULL,
  ah.created_at
FROM answer_history ah
LEFT JOIN survey_questions sq ON sq.id = ah.question_id
LEFT JOIN answer_recommendations ar ON ar.id = ah.recommendation_id;

DROP TABLE answer_history;
ALTER TABLE answer_history_new RENAME TO answer_history;

CREATE INDEX IF NOT EXISTS idx_history_persona_created ON answer_history(persona_id, created_at);
CREATE INDEX IF NOT EXISTS idx_history_question_created ON answer_history(question_id, created_at);
CREATE INDEX IF NOT EXISTS idx_history_recommendation ON answer_history(recommendation_id);
