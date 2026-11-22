PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS personas (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL CHECK(length(display_name) BETWEEN 1 AND 80),
  description TEXT,
  is_active INTEGER NOT NULL DEFAULT 0 CHECK(is_active IN (0, 1)),
  status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'archived', 'disabled')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived_at TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_one_active_persona
  ON personas(is_active) WHERE is_active = 1;

CREATE TABLE IF NOT EXISTS profile_facts (
  id TEXT PRIMARY KEY,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  category TEXT NOT NULL,
  attribute TEXT NOT NULL,
  value_json TEXT NOT NULL,
  normalized_value TEXT,
  verification_status TEXT NOT NULL CHECK(verification_status IN ('confirmed', 'unverified', 'disputed', 'outdated')),
  sensitivity TEXT NOT NULL DEFAULT 'ordinary' CHECK(sensitivity IN ('ordinary', 'sensitive', 'highly_sensitive')),
  source_date TEXT,
  valid_from TEXT,
  valid_until TEXT,
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived_at TEXT
);

CREATE TABLE IF NOT EXISTS fact_sources (
  id TEXT PRIMARY KEY,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  fact_id TEXT NOT NULL REFERENCES profile_facts(id) ON DELETE CASCADE,
  source_type TEXT NOT NULL,
  description TEXT NOT NULL,
  reference_uri TEXT,
  observed_at TEXT,
  created_at TEXT NOT NULL,
  UNIQUE(id, persona_id)
);

CREATE TABLE IF NOT EXISTS persona_preferences (
  id TEXT PRIMARY KEY,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  preference_key TEXT NOT NULL,
  value_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(persona_id, preference_key)
);

CREATE TABLE IF NOT EXISTS survey_sessions (
  id TEXT PRIMARY KEY,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  title TEXT,
  platform_label TEXT,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived_at TEXT
);

CREATE TABLE IF NOT EXISTS screen_captures (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE,
  capture_mode TEXT NOT NULL CHECK(capture_mode IN ('region', 'window', 'clipboard_image')),
  captured_at TEXT NOT NULL,
  application_label TEXT,
  temporary_image_path TEXT,
  image_deleted_at TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS survey_questions (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE,
  capture_id TEXT REFERENCES screen_captures(id) ON DELETE SET NULL,
  question_text_original TEXT NOT NULL,
  question_text_corrected TEXT,
  instructions TEXT,
  options_json TEXT NOT NULL DEFAULT '[]',
  structure_json TEXT,
  extraction_status TEXT NOT NULL DEFAULT 'manual',
  question_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS survey_opportunities (
  id TEXT PRIMARY KEY,
  title TEXT,
  platform_label TEXT,
  topic TEXT,
  description TEXT,
  source_capture_id TEXT REFERENCES screen_captures(id) ON DELETE SET NULL,
  captured_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS survey_opportunity_criteria (
  id TEXT PRIMARY KEY,
  opportunity_id TEXT NOT NULL REFERENCES survey_opportunities(id) ON DELETE CASCADE,
  criterion_text TEXT NOT NULL,
  normalized_attribute TEXT,
  expected_value_json TEXT,
  criterion_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS survey_fit_results (
  id TEXT PRIMARY KEY,
  opportunity_id TEXT NOT NULL REFERENCES survey_opportunities(id) ON DELETE CASCADE,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  criterion_id TEXT NOT NULL REFERENCES survey_opportunity_criteria(id) ON DELETE CASCADE,
  match_state TEXT NOT NULL CHECK(match_state IN ('match', 'mismatch', 'unknown', 'stale', 'conflict')),
  supporting_fact_ids_json TEXT NOT NULL DEFAULT '[]',
  explanation TEXT NOT NULL,
  evaluated_at TEXT NOT NULL,
  UNIQUE(opportunity_id, persona_id, criterion_id)
);

CREATE TABLE IF NOT EXISTS survey_attempts (
  id TEXT PRIMARY KEY,
  opportunity_id TEXT NOT NULL REFERENCES survey_opportunities(id) ON DELETE CASCADE,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  session_id TEXT REFERENCES survey_sessions(id) ON DELETE SET NULL,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  current_outcome TEXT NOT NULL DEFAULT 'unknown'
    CHECK(current_outcome IN ('qualified', 'screened_out', 'quota_full', 'completed', 'quality_rejected', 'technical_failure', 'abandoned_by_user', 'pending', 'unknown')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS survey_outcome_events (
  id TEXT PRIMARY KEY,
  attempt_id TEXT NOT NULL REFERENCES survey_attempts(id) ON DELETE CASCADE,
  outcome_label TEXT NOT NULL
    CHECK(outcome_label IN ('qualified', 'screened_out', 'quota_full', 'completed', 'quality_rejected', 'technical_failure', 'abandoned_by_user', 'pending', 'unknown')),
  evidence_source TEXT NOT NULL,
  evidence_note TEXT,
  confidence TEXT NOT NULL DEFAULT 'observed'
    CHECK(confidence IN ('observed', 'user_confirmed', 'uncertain')),
  observed_at TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_configs (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  provider_type TEXT NOT NULL,
  base_url TEXT,
  adapter_type TEXT NOT NULL,
  model_id TEXT NOT NULL,
  credential_ref TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
  priority INTEGER NOT NULL DEFAULT 100,
  weight INTEGER NOT NULL DEFAULT 1 CHECK(weight > 0),
  timeout_ms INTEGER NOT NULL DEFAULT 30000,
  max_concurrency INTEGER NOT NULL DEFAULT 1,
  rate_limit_hint_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_pool_settings (
  id TEXT PRIMARY KEY,
  strategy TEXT NOT NULL DEFAULT 'adaptive' CHECK(strategy IN ('adaptive', 'round_robin', 'weighted', 'priority')),
  max_attempts INTEGER NOT NULL DEFAULT 3 CHECK(max_attempts BETWEEN 1 AND 10),
  auto_retry INTEGER NOT NULL DEFAULT 1 CHECK(auto_retry IN (0, 1)),
  retry_base_delay_ms INTEGER NOT NULL DEFAULT 1000,
  max_retry_delay_ms INTEGER NOT NULL DEFAULT 15000,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_health (
  provider_config_id TEXT PRIMARY KEY REFERENCES provider_configs(id) ON DELETE CASCADE,
  state TEXT NOT NULL DEFAULT 'ready',
  cooldown_until TEXT,
  last_success_at TEXT,
  last_failure_at TEXT,
  last_error_category TEXT,
  consecutive_failures INTEGER NOT NULL DEFAULT 0,
  recent_latency_ms INTEGER,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS answer_recommendations (
  id TEXT PRIMARY KEY,
  question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE,
  status TEXT NOT NULL,
  draft_text TEXT,
  interpretation TEXT NOT NULL,
  rationale TEXT,
  missing_information_json TEXT NOT NULL DEFAULT '[]',
  conflicts_json TEXT NOT NULL DEFAULT '[]',
  assumptions_json TEXT NOT NULL DEFAULT '[]',
  provider_config_id TEXT REFERENCES provider_configs(id) ON DELETE SET NULL,
  model_id TEXT,
  prompt_version TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS recommendation_sources (
  recommendation_id TEXT NOT NULL REFERENCES answer_recommendations(id) ON DELETE CASCADE,
  fact_id TEXT NOT NULL REFERENCES profile_facts(id) ON DELETE RESTRICT,
  persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
  support_note TEXT,
  PRIMARY KEY(recommendation_id, fact_id)
);

CREATE TABLE IF NOT EXISTS answer_history (
  id TEXT PRIMARY KEY,
  question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE,
  recommendation_id TEXT REFERENCES answer_recommendations(id) ON DELETE SET NULL,
  answer_text TEXT NOT NULL,
  action TEXT NOT NULL CHECK(action IN ('edited', 'confirmed', 'copied', 'rejected')),
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_usage_events (
  id TEXT PRIMARY KEY,
  provider_config_id TEXT REFERENCES provider_configs(id) ON DELETE SET NULL,
  model_id TEXT,
  operation TEXT NOT NULL,
  outcome TEXT NOT NULL,
  error_category TEXT,
  input_tokens INTEGER,
  output_tokens INTEGER,
  duration_ms INTEGER,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_facts_persona_status ON profile_facts(persona_id, verification_status, category);
CREATE INDEX IF NOT EXISTS idx_sources_persona_fact ON fact_sources(persona_id, fact_id);
CREATE INDEX IF NOT EXISTS idx_sessions_persona_updated ON survey_sessions(persona_id, updated_at);
CREATE INDEX IF NOT EXISTS idx_questions_session_order ON survey_questions(session_id, question_order);
CREATE INDEX IF NOT EXISTS idx_opportunity_captured ON survey_opportunities(captured_at);
CREATE INDEX IF NOT EXISTS idx_criteria_opportunity_order ON survey_opportunity_criteria(opportunity_id, criterion_order);
CREATE INDEX IF NOT EXISTS idx_fit_persona_opportunity ON survey_fit_results(persona_id, opportunity_id);
CREATE INDEX IF NOT EXISTS idx_attempt_persona_started ON survey_attempts(persona_id, started_at);
CREATE INDEX IF NOT EXISTS idx_outcome_attempt_observed ON survey_outcome_events(attempt_id, observed_at);
CREATE INDEX IF NOT EXISTS idx_recommendations_question_created ON answer_recommendations(question_id, created_at);
CREATE INDEX IF NOT EXISTS idx_history_question_created ON answer_history(question_id, created_at);
CREATE INDEX IF NOT EXISTS idx_provider_enabled_priority ON provider_configs(enabled, priority);
CREATE INDEX IF NOT EXISTS idx_provider_health_state ON provider_health(state, cooldown_until);
CREATE INDEX IF NOT EXISTS idx_usage_created_outcome ON ai_usage_events(created_at, outcome);

INSERT OR IGNORE INTO provider_pool_settings
  (id, strategy, max_attempts, auto_retry, retry_base_delay_ms, max_retry_delay_ms, updated_at)
VALUES
  ('default', 'adaptive', 3, 1, 1000, 15000, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT OR IGNORE INTO schema_migrations(version, applied_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
