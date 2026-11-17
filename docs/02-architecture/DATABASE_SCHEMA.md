# Database Schema

**Status:** Proposed logical schema. Exact SQL and migrations must be reviewed during implementation.

## 1. Conventions
- Primary identifiers: UUID stored as TEXT.
- Timestamps: UTC ISO-8601 strings.
- Foreign keys enabled on every connection.
- Use parameterized SQL exclusively.
- Use explicit migrations; never mutate schema ad hoc at runtime.
- Add indexes based on measured query patterns.

## 2. Core tables

### users
Single local profile owner for MVP: `id`, `display_name` (optional), `created_at`, `updated_at`.

### profile_facts
`id`, `user_id`, `category`, `attribute`, `value_json`, `normalized_value` (optional), `verification_status`, `sensitivity`, `valid_from`, `valid_until`, `created_at`, `updated_at`, `archived_at`.

Allowed verification statuses: `confirmed`, `unverified`, `disputed`, `outdated`. Store values with explicit types; do not coerce unknown values into strings where numeric/date semantics matter.

### fact_sources
`id`, `fact_id`, `source_type`, `description`, `reference_uri` (optional), `observed_at`, `created_at`. Avoid storing unnecessary document contents or credentials.

### user_preferences
`id`, `user_id`, `preference_key`, `value_json`, `updated_at`. Separate subjective preferences from factual profile claims.

### survey_sessions
`id`, `user_id`, `title`, `platform_label` (optional free text), `status`, `created_at`, `updated_at`, `archived_at`.

### survey_questions
`id`, `session_id`, `question_text`, `question_order`, `classification_json`, `created_at`, `updated_at`.

### answer_recommendations
`id`, `question_id`, `status`, `draft_text`, `rationale`, `assumptions_json`, `model_metadata_json` (redacted), `created_at`.

### answer_sources
`id`, `recommendation_id`, `fact_id`, `support_note`. The application verifies every referenced fact exists and belongs to the current local user.

### answer_history
`id`, `question_id`, `recommendation_id` (nullable), `answer_text`, `origin` (user_edited, user_confirmed, copied, rejected), `created_at`. A copied answer is not necessarily confirmed as fact.

### ai_usage_logs
`id`, `provider`, `model`, `operation`, `input_token_count` (optional), `output_token_count` (optional), `duration_ms`, `outcome`, `created_at`. Never store raw prompts, API keys, or full personal context by default.

### app_settings
`key`, `value_json`, `updated_at`. Secret values must not be stored here.

## 3. Relationships
- One user owns many facts and sessions.
- One fact may have multiple source records.
- One session contains ordered questions.
- One question may have multiple recommendations.
- One recommendation may cite multiple facts.
- History records may refer to a recommendation but remain independently retained according to user policy.

## 4. Integrity and deletion
Use foreign keys and explicit deletion behavior. Prefer soft archive for user-facing history where appropriate, but provide a true delete operation. Deleting a fact must remove or invalidate its source links and cause affected recommendations to display a missing-source state. Do not leave orphaned evidence references.

## 5. Indexes to consider
- `profile_facts(user_id, category, verification_status)`
- `survey_sessions(user_id, updated_at)`
- `survey_questions(session_id, question_order)`
- `answer_history(question_id, created_at)`
- FTS5 index for question text and user-authored notes only if needed.

## 6. Sensitive data and encryption
Classify sensitive attributes before implementation. Determine whether whole-database encryption is required for the release. If encryption is implemented, document key derivation, OS-backed key protection, backup/export behavior, and recovery limitations. Do not invent a cryptographic scheme; use maintained, reviewed libraries.

## 7. Migration requirements
- Sequential, immutable migration files.
- Schema version table.
- Backup before destructive migration.
- Transactional migration where supported.
- Test upgrade from every supported prior schema version.
- Recovery instructions for failed migration.
