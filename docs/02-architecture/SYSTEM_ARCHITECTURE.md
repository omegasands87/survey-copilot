# System Architecture

**Target:** Windows 11 x64  
**Status:** Normative technical architecture

## 1. Architecture overview

Survey Copilot is a local-first desktop app. React renders the UI. Tauri/Rust owns native screen capture, SQLite access, Windows Credential Manager access, filesystem operations, and outbound AI requests. All communication crosses a typed, validated IPC boundary.

## 2. Technology

- Desktop shell: Tauri 2.
- Native backend: Rust.
- UI: React + TypeScript.
- Styling: Tailwind CSS and accessible UI primitives.
- Local persistence: SQLite with immutable migrations.
- Secrets: Windows Credential Manager.
- OCR: local OCR implementation selected after Windows 11 packaging and accuracy evaluation.
- AI: Gemini adapter first; provider adapter abstraction for custom compatible APIs.
- State: lightweight client state; async request state managed explicitly.

## 3. Runtime modules

1. Persona service: profile CRUD, fact status, provenance, persona isolation.
2. Screen intelligence: user-triggered capture, OCR, layout reconstruction, preview and correction.
3. Survey workspace: sessions, question records, options, and review state.
4. Answer copilot: classification, retrieval, consistency, context minimization, generation, validation.
5. Opportunity fit: visible criteria extraction and deterministic persona matching.
6. Provider hub: provider configurations, credential references, pool selection, cooldown, retry, and health.
7. History: recommendation versions and user actions.
8. Data management: export, deletion, migrations, and backup.

## 4. Main request lifecycle

Capture or paste → user verifies question → select active persona → classify question → retrieve eligible facts → check consistency → minimize context → choose compatible provider configuration → dispatch request → validate structured output and source IDs → show recommendation → user edits/confirms/copies/rejects → persist history.

## 5. Provider dispatch lifecycle

User request → operation/model compatibility filter → enabled credential references → health and cooldown filter → pool strategy selects one key → enforce concurrency/rate limit → provider request → normalize response/error → success health update OR cooldown/failover/retry → terminal error if no eligible key remains.

No speculative parallel fan-out. No unbounded retries.

## 6. Trust boundaries

- Survey text, OCR text, screenshots, and model responses are untrusted.
- React is not trusted with API secrets or direct database access.
- Rust IPC commands are privileged and must be allowlisted and validated.
- Provider endpoints are external; custom endpoint configuration is user-controlled but still validated.
- A provider response cannot invoke tools, execute code, or access local files.
- Persona ID must be validated on every query and persistence operation.

## 7. Failure handling

- Capture failure: retain manual text entry path.
- OCR uncertainty: show preview and correction; do not auto-analyze.
- Provider unavailable: retain the question and show retry/manual mode.
- Rate limit: cool down the key and try another eligible key.
- All keys unavailable: show concise status and user-controlled retry.
- Invalid model output: reject and offer retry or abstention.
- Database error: preserve recoverable input; never pretend a write succeeded.
- Migration failure: backup first and provide recovery instructions.

## 8. Native command groups

- persona_*: create, list, activate, update, archive, delete.
- fact_*: create, update, verify, dispute, mark_stale, search, delete.
- capture_*: begin_region_capture, capture_window, preview, discard.
- ocr_*: extract, correct, confirm.
- survey_*: create_session, save_question, list_history.
- answer_*: analyze, cancel, save_review_action.
- provider_*: list, add, update, test, enable, disable, remove, inspect_health.
- data_*: export, delete, backup, restore.

All commands return typed result/error objects. Do not expose generic shell, arbitrary file, or browser-control commands.

## 9. Architectural constraints

- No browser automation, DOM inspection, autofill, auto-click, or auto-submit.
- No earnings/reward tracking module in v1.0.
- No UI-to-SQL direct access.
- No provider-specific logic inside domain services.
- No cross-persona fact retrieval.
- No AI-generated fact can become confirmed without explicit user review.
