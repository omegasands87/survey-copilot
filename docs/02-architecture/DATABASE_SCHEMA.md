# Database Schema

**Status:** Normative logical schema for v1.0. Exact SQL migrations are implementation deliverables.

## 1. Conventions

- UUID text primary keys.
- UTC timestamps in ISO-8601.
- Enable SQLite foreign keys on every connection.
- Parameterized SQL only.
- Every persona-owned record includes persona_id directly or through a parent relationship that is checked in the service layer.
- Never store API key values in SQLite.

## 2. Tables

### app_meta
- key TEXT PRIMARY KEY
- value TEXT NOT NULL

Stores schema-independent app metadata and migration version.

### personas
- id TEXT PRIMARY KEY
- display_name TEXT NOT NULL
- description TEXT NULL
- is_active INTEGER NOT NULL DEFAULT 0
- status TEXT NOT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL
- archived_at TEXT NULL

Only one active persona at a time in v1.0. Persona records represent real respondents.

### profile_facts
- id TEXT PRIMARY KEY
- persona_id TEXT NOT NULL REFERENCES personas(id)
- category TEXT NOT NULL
- attribute TEXT NOT NULL
- value_json TEXT NOT NULL
- normalized_value TEXT NULL
- verification_status TEXT NOT NULL
- sensitivity TEXT NOT NULL
- source_date TEXT NULL
- valid_from TEXT NULL
- valid_until TEXT NULL
- notes TEXT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL
- archived_at TEXT NULL

Statuses: confirmed, unverified, disputed, outdated. Sensitivity: ordinary, sensitive, highly_sensitive.

### fact_sources
- id TEXT PRIMARY KEY
- persona_id TEXT NOT NULL REFERENCES personas(id)
- fact_id TEXT NOT NULL REFERENCES profile_facts(id) ON DELETE CASCADE
- source_type TEXT NOT NULL
- description TEXT NOT NULL
- reference_uri TEXT NULL
- observed_at TEXT NULL
- created_at TEXT NOT NULL

Source links must belong to the same persona as the fact.

### persona_preferences
- id TEXT PRIMARY KEY
- persona_id TEXT NOT NULL REFERENCES personas(id)
- preference_key TEXT NOT NULL
- value_json TEXT NOT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL
- UNIQUE(persona_id, preference_key)

Preferences and opinions must not be treated as objective facts.

### survey_sessions
- id TEXT PRIMARY KEY
- persona_id TEXT NOT NULL REFERENCES personas(id)
- title TEXT NULL
- platform_label TEXT NULL
- source_url TEXT NULL
- source_domain TEXT NULL
- version INTEGER NOT NULL DEFAULT 1
- recognized_from_id TEXT NULL REFERENCES survey_sessions(id) ON DELETE SET NULL
- status TEXT NOT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL
- archived_at TEXT NULL

A session belongs to one persona. Changing active persona must never reassign an existing session.

### survey_recognitions (Migration 0011)
- id TEXT PRIMARY KEY
- session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE
- persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE
- matched_session_id TEXT NULL REFERENCES survey_sessions(id) ON DELETE SET NULL
- match_status TEXT NOT NULL
- confidence_score REAL NOT NULL
- structural_similarity REAL NOT NULL
- title_similarity REAL NOT NULL
- matched_question_count INTEGER NOT NULL DEFAULT 0
- total_question_count INTEGER NOT NULL DEFAULT 0
- differences_json TEXT NOT NULL DEFAULT '[]'
- provenance TEXT NOT NULL DEFAULT 'system_heuristic'
- recognized_at TEXT NOT NULL
- created_at TEXT NOT NULL

Stores survey recognition engine evaluations, structural/title similarity scores, version change differences, and provenance without leaking cross-persona data.

### screen_captures
- id TEXT PRIMARY KEY
- session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE
- capture_mode TEXT NOT NULL
- captured_at TEXT NOT NULL
- application_label TEXT NULL
- temporary_image_path TEXT NULL
- image_deleted_at TEXT NULL
- created_at TEXT NOT NULL

Default retention stores no screenshot after OCR review; temporary path is nullable and must be deleted after processing.

### ocr_extractions (Migration 0008)
- id TEXT PRIMARY KEY
- capture_id TEXT NULL REFERENCES screen_captures(id) ON DELETE SET NULL
- session_id TEXT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE
- persona_id TEXT NULL REFERENCES personas(id) ON DELETE CASCADE
- ocr_engine TEXT NOT NULL
- raw_ocr_text TEXT NOT NULL
- question_text_original TEXT NOT NULL
- question_text_corrected TEXT NULL
- instructions TEXT NULL
- options_json TEXT NOT NULL DEFAULT '[]'
- structure_json TEXT NULL
- text_blocks_json TEXT NOT NULL DEFAULT '[]'
- uncertain_regions_json TEXT NOT NULL DEFAULT '[]'
- warnings_json TEXT NOT NULL DEFAULT '[]'
- overall_confidence REAL NOT NULL DEFAULT 0.0
- confirmation_status TEXT NOT NULL DEFAULT 'draft'
- extracted_at TEXT NOT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL

Preserves raw OCR output, structured layout reconstruction, spatial bounding boxes, confidence scoring, uncertainty/clipping detection, and user confirmation state (SC-041).

### survey_questions
- id TEXT PRIMARY KEY
- session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE
- capture_id TEXT NULL REFERENCES screen_captures(id) ON DELETE SET NULL
- question_text_original TEXT NOT NULL
- question_text_corrected TEXT NULL
- instructions TEXT NULL
- options_json TEXT NOT NULL DEFAULT '[]'
- structure_json TEXT NULL
- extraction_status TEXT NOT NULL
- question_order INTEGER NOT NULL
- question_fingerprint TEXT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL

### survey_opportunities
- id TEXT PRIMARY KEY
- title TEXT NULL
- platform_label TEXT NULL
- topic TEXT NULL
- description TEXT NULL
- source_capture_id TEXT NULL REFERENCES screen_captures(id) ON DELETE SET NULL
- captured_at TEXT NOT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL

An opportunity is a visible survey offer/description, not an account or survey-platform integration.

### survey_opportunity_criteria
- id TEXT PRIMARY KEY
- opportunity_id TEXT NOT NULL REFERENCES survey_opportunities(id) ON DELETE CASCADE
- criterion_text TEXT NOT NULL
- normalized_attribute TEXT NULL
- expected_value_json TEXT NULL
- criterion_order INTEGER NOT NULL

### survey_fit_results
- id TEXT PRIMARY KEY
- opportunity_id TEXT NOT NULL REFERENCES survey_opportunities(id) ON DELETE CASCADE
- persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE
- criterion_id TEXT NOT NULL REFERENCES survey_opportunity_criteria(id) ON DELETE CASCADE
- match_state TEXT NOT NULL
- supporting_fact_ids_json TEXT NOT NULL
- explanation TEXT NOT NULL
- evaluated_at TEXT NOT NULL
- UNIQUE(opportunity_id, persona_id, criterion_id)

Match states: match, mismatch, unknown, stale, conflict. The application computes final states deterministically from persona facts; model extraction alone cannot mark a criterion as satisfied.

### survey_attempts
- id TEXT PRIMARY KEY
- opportunity_id TEXT NOT NULL REFERENCES survey_opportunities(id) ON DELETE CASCADE
- persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE
- session_id TEXT NULL REFERENCES survey_sessions(id) ON DELETE SET NULL
- started_at TEXT NOT NULL
- ended_at TEXT NULL
- current_outcome TEXT NOT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL

### survey_outcome_events
- id TEXT PRIMARY KEY
- attempt_id TEXT NOT NULL REFERENCES survey_attempts(id) ON DELETE CASCADE
- outcome_label TEXT NOT NULL
- evidence_source TEXT NOT NULL
- evidence_note TEXT NULL
- confidence TEXT NOT NULL
- observed_at TEXT NOT NULL
- created_at TEXT NOT NULL

Outcome labels: qualified, screened_out, quota_full, completed, quality_rejected, technical_failure, abandoned_by_user, pending, unknown. These are survey-fit observations only, not reward or income records.

### answer_recommendations
- id TEXT PRIMARY KEY
- question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE
- status TEXT NOT NULL
- draft_text TEXT NULL
- interpretation TEXT NOT NULL
- rationale TEXT NULL
- missing_information_json TEXT NOT NULL DEFAULT '[]'
- conflicts_json TEXT NOT NULL DEFAULT '[]'
- assumptions_json TEXT NOT NULL DEFAULT '[]'
- provider_config_id TEXT NULL REFERENCES provider_configs(id) ON DELETE SET NULL
- model_id TEXT NULL
- prompt_version TEXT NULL
- created_at TEXT NOT NULL

### recommendation_sources
- recommendation_id TEXT NOT NULL REFERENCES answer_recommendations(id) ON DELETE CASCADE
- fact_id TEXT NOT NULL REFERENCES profile_facts(id) ON DELETE RESTRICT
- persona_id TEXT NOT NULL REFERENCES personas(id)
- support_note TEXT NULL
- PRIMARY KEY(recommendation_id, fact_id)

Before insert, verify the recommendation's session persona, source fact persona, and source persona are identical.

### answer_history
- id TEXT PRIMARY KEY
- question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE
- recommendation_id TEXT NULL REFERENCES answer_recommendations(id) ON DELETE SET NULL
- answer_text TEXT NOT NULL
- action TEXT NOT NULL
- created_at TEXT NOT NULL

Actions: edited, confirmed, copied, rejected. A history record is not automatically a profile fact.

### provider_configs
- id TEXT PRIMARY KEY
- display_name TEXT NOT NULL
- provider_type TEXT NOT NULL
- base_url TEXT NULL
- adapter_type TEXT NOT NULL
- model_id TEXT NOT NULL
- credential_ref TEXT NOT NULL
- enabled INTEGER NOT NULL DEFAULT 1
- priority INTEGER NOT NULL DEFAULT 100
- weight INTEGER NOT NULL DEFAULT 1
- timeout_ms INTEGER NOT NULL
- max_concurrency INTEGER NOT NULL DEFAULT 1
- rate_limit_hint_json TEXT NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL

credential_ref is an opaque Windows Credential Manager reference, never the secret itself.

### provider_pool_settings
- id TEXT PRIMARY KEY
- strategy TEXT NOT NULL
- max_attempts INTEGER NOT NULL
- auto_retry INTEGER NOT NULL
- retry_base_delay_ms INTEGER NOT NULL
- max_retry_delay_ms INTEGER NOT NULL
- updated_at TEXT NOT NULL

Strategies: adaptive, round_robin, weighted, priority.

### provider_health
- provider_config_id TEXT PRIMARY KEY REFERENCES provider_configs(id) ON DELETE CASCADE
- state TEXT NOT NULL
- cooldown_until TEXT NULL
- last_success_at TEXT NULL
- last_failure_at TEXT NULL
- last_error_category TEXT NULL
- consecutive_failures INTEGER NOT NULL DEFAULT 0
- recent_latency_ms INTEGER NULL
- updated_at TEXT NOT NULL

Do not store API key or raw prompt in health data.

### ai_usage_events
- id TEXT PRIMARY KEY
- provider_config_id TEXT NULL REFERENCES provider_configs(id) ON DELETE SET NULL
- model_id TEXT NULL
- operation TEXT NOT NULL
- outcome TEXT NOT NULL
- error_category TEXT NULL
- input_tokens INTEGER NULL
- output_tokens INTEGER NULL
- duration_ms INTEGER NULL
- created_at TEXT NOT NULL

No raw prompts, answer text, screenshots, or secrets.

### survey_branch_transitions
- id TEXT PRIMARY KEY
- session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE
- persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE
- from_question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE
- from_question_fingerprint TEXT NULL
- from_question_order INTEGER NOT NULL
- from_answer_value TEXT NULL
- to_question_id TEXT NULL REFERENCES survey_questions(id) ON DELETE SET NULL
- to_question_fingerprint TEXT NULL
- to_question_text TEXT NOT NULL
- to_question_type TEXT NOT NULL DEFAULT 'unknown'
- to_options_json TEXT NOT NULL DEFAULT '[]'
- to_question_order INTEGER NOT NULL
- branch_condition TEXT NULL
- transition_type TEXT NOT NULL ('sequential', 'conditional_branch', 'skip_jump')
- observed_at TEXT NOT NULL
- created_at TEXT NOT NULL

### survey_question_predictions
- id TEXT PRIMARY KEY
- session_id TEXT NOT NULL REFERENCES survey_sessions(id) ON DELETE CASCADE
- persona_id TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE
- current_question_id TEXT NOT NULL REFERENCES survey_questions(id) ON DELETE CASCADE
- predicted_question_text TEXT NOT NULL
- predicted_question_type TEXT NOT NULL DEFAULT 'unknown'
- predicted_options_json TEXT NOT NULL DEFAULT '[]'
- confidence_score REAL NOT NULL
- evidence_source TEXT NOT NULL
- explanation TEXT NOT NULL
- leading_condition TEXT NULL
- status TEXT NOT NULL ('predicted_not_detected', 'detected_matched', 'detected_diverged', 'insufficient_evidence')
- matched_question_id TEXT NULL REFERENCES survey_questions(id) ON DELETE SET NULL
- created_at TEXT NOT NULL
- updated_at TEXT NOT NULL

### app_settings
- key TEXT PRIMARY KEY
- value_json TEXT NOT NULL
- updated_at TEXT NOT NULL

Only non-secret preferences belong here.

## 3. Required indexes

- profile_facts(persona_id, verification_status, category)
- fact_sources(persona_id, fact_id)
- survey_sessions(persona_id, updated_at)
- screen_captures(session_id, captured_at)
- ocr_extractions(session_id, capture_id, persona_id)
- survey_questions(session_id, question_order)
- answer_recommendations(question_id, created_at)
- answer_history(question_id, created_at)
- survey_opportunities(captured_at)
- survey_fit_results(persona_id, opportunity_id)
- survey_attempts(persona_id, started_at)
- survey_attempts(opportunity_id)
- survey_attempts(current_outcome)
- survey_outcome_events(attempt_id, observed_at)
- survey_outcome_events(outcome_label)
- survey_outcome_events(confidence)
- provider_configs(enabled, priority)
- provider_health(state, cooldown_until)
- ai_usage_events(created_at, outcome)
- survey_sessions(source_domain)
- survey_sessions(fingerprint)
- survey_questions(question_fingerprint)
- survey_recognitions(session_id, persona_id)
- survey_recognitions(matched_session_id)
- survey_branch_transitions(from_question_fingerprint)
- survey_branch_transitions(session_id, persona_id)
- survey_branch_transitions(from_question_id)
- survey_question_predictions(session_id, persona_id)
- survey_question_predictions(current_question_id)
- survey_question_predictions(status)

## 4. Integrity rules

- Every fact, source, session, question, and recommendation is scoped to exactly one persona through its parent.
- Service layer checks persona equality for every relationship; do not rely solely on UI filtering.
- Deleting a persona requires explicit confirmation and deletes all owned records in a transaction.
- Deleting a fact must not silently rewrite historical answer text; remove or retain source references according to an explicit deletion policy and show evidence as unavailable.
- Deleting provider configuration removes its credential from Windows Credential Manager.
- A failed credential deletion must be surfaced, not hidden.

## 5. Migration requirements

- Sequential immutable migration files.
- Transactional migrations where supported.
- Backup before destructive changes.
- Test fresh install and upgrade from every released schema.
- Include migration rollback/recovery instructions.
- Never perform schema changes through ad-hoc runtime SQL.
