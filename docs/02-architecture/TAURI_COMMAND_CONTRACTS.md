# Tauri Command Contracts

**Target:** Tauri 2 / Rust / Windows 11 x64  
**Status:** Normative implementation contract

## 1. Rules

- Expose only typed, allowlisted commands.
- Validate every input in Rust even if the frontend validates it.
- Return typed success/error values; do not leak internal paths, secrets, or stack traces.
- Never expose a generic SQL, shell, filesystem, or browser-control command.
- Every persona-owned command receives persona_id and verifies ownership.
- Cancellation must be supported for capture, OCR, and AI generation where practical.

## 2. Persona commands

| Command | Input | Output |
|---|---|---|
| persona_list | none | PersonaSummary[] |
| persona_create | display_name, optional description | Persona |
| persona_update | persona_id, patch | Persona |
| persona_set_active | persona_id | ActivePersona |
| persona_archive | persona_id | Persona |
| persona_delete | persona_id, confirmation_token | DeleteResult |

## 3. Fact commands

| Command | Input | Output |
|---|---|---|
| fact_list | persona_id, filters | ProfileFact[] |
| fact_create | persona_id, fact payload | ProfileFact |
| fact_update | persona_id, fact_id, patch | ProfileFact |
| fact_set_status | persona_id, fact_id, status | ProfileFact |
| fact_delete | persona_id, fact_id, confirmation_token | DeleteResult |
| fact_search | persona_id, query, filters | ProfileFact[] |

All fact queries must enforce persona_id in the repository query and service layer.

## 4. Capture and OCR commands

| Command | Input | Output |
|---|---|---|
| capture_region | explicit user invocation | CapturePreview |
| capture_window | selected window identifier | CapturePreview |
| capture_discard | capture_id | DiscardResult |
| ocr_extract | capture_id, language hints | ExtractionDraft |
| ocr_confirm | capture_id, corrected extraction | ConfirmedQuestionInput |

No command may click, type into, navigate, inspect DOM, or submit another application.

## 5. Survey and answer commands

| Command | Input | Output |
|---|---|---|
| survey_session_create | persona_id, optional title/platform | SurveySession |
| survey_session_list | persona_id, filters | SurveySession[] |
| survey_session_delete | persona_id, session_id | DeleteResult |
| question_save | session_id, confirmed question payload | SurveyQuestion |
| answer_analyze | session_id, question_id, persona_id | AnalysisJob |
| answer_cancel | job_id | CancelResult |
| answer_review | recommendation_id, action, optional edited text | AnswerHistory |

The backend verifies that session, question, and persona IDs belong together before retrieval or persistence.

## 6. Provider commands

| Command | Input | Output |
|---|---|---|
| provider_list | none | ProviderConfigSummary[] |
| provider_add | non-secret provider config + secret payload | ProviderConfigSummary |
| provider_update | provider_id, config patch, optional secret replacement | ProviderConfigSummary |
| provider_test | provider_id | ProviderTestResult |
| provider_enable | provider_id | ProviderConfigSummary |
| provider_disable | provider_id | ProviderConfigSummary |
| provider_remove | provider_id | RemoveResult |
| provider_health | optional provider_id | ProviderHealth[] |
| provider_pool_settings_get | none | ProviderPoolSettings |
| provider_pool_settings_set | validated settings | ProviderPoolSettings |

Secret payload is passed directly to Rust for immediate credential-store write. It must never be returned to the frontend or logged.

## 7. Data commands

| Command | Input | Output |
|---|---|---|
| data_export | export scope and destination chosen by user | ExportResult |
| data_delete_all | explicit confirmation token | DeleteResult |
| data_backup | user-selected destination | BackupResult |
| data_restore | user-selected backup | RestoreResult |

## 8. Shared error envelope

- code: stable machine-readable error code.
- message: concise Indonesian user-facing message.
- retryable: boolean.
- correlation_id: safe diagnostic identifier.

Do not include API key values, raw survey content, profile facts, screenshots, or internal stack traces in user-facing errors.
