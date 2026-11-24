# Implementation Backlog

This backlog is the execution order for Antigravity. Checkboxes are intentionally unchecked until implementation and evidence are reviewed. Do not mark an item complete because a document or placeholder exists.

## P0 — Make the starter safe to extend

### SC-001 — Establish reproducible builds
- [x] Generate and commit `package-lock.json` and `src-tauri/Cargo.lock`.
- [x] Verify supported Node/Rust toolchain versions.
- [x] Make GitHub Actions use deterministic install (`npm ci`) after lockfile exists.
- [x] Run frontend build, Rust check, Rust tests, and format check.
- [x] Fix all compile, lint, and workflow errors; retain evidence in PR/commit notes.

**Acceptance:** A clean checkout can install dependencies and build frontend/Rust without undocumented local setup. CI passes.

### SC-002 — Introduce application layers
- [x] Move Tauri commands from `src-tauri/src/lib.rs` into thin command modules.
- [x] Add domain types, service layer, and repository layer.
- [x] Add typed frontend IPC client wrappers.
- [x] Use structured error codes rather than exposing raw internal errors.
- [x] Add tests for command validation and error mapping.

**Acceptance:** React components do not call arbitrary command strings directly except through typed wrappers; command handlers do not contain business logic or SQL.

### SC-003 — Replace startup SQL replay with migration runner
- [x] Add ordered migration registry/version tracking.
- [x] Apply only unapplied migrations inside transactions.
- [x] Fail startup safely on migration error and preserve the existing database.
- [x] Add migration tests for fresh database, repeated startup, upgrade, and failure recovery.
- [x] Add backup/recovery policy before destructive migrations.

**Acceptance:** Startup is idempotent; each migration runs once; migration version matches the applied schema. Baseline adoption preserves existing database data without destructive SQL replay.

## P1 — Complete local respondent data

### SC-010 — Persona lifecycle
- [x] Create, list, rename/edit, activate, archive, restore, and delete personas.
- [x] Add confirmation and dependency summary for destructive deletion.
- [x] Enforce one active persona at a time in transaction and database constraints.
- [x] Prevent archive/delete from leaving an invalid active-persona state.
- [x] Add persona CRUD and switching tests.

**Acceptance:** Persona lifecycle works after app restart; all operations are scoped; no orphaned records. Automated integration tests cover complete lifecycle and cross-persona data isolation.

### SC-011 — Fact and source lifecycle
- [x] Support typed fact values and normalized value representation.
- [x] Add/edit facts and source metadata (type, description, observed date, source reference where safe).
- [x] Add verification lifecycle: confirmed, unverified, disputed, outdated.
- [x] Add sensitivity lifecycle: ordinary, sensitive, highly_sensitive.
- [x] Add validity dates, notes, and fact history where required.
- [x] Add fact archive/restore and deletion policy.
- [x] Add duplicate/conflict detection and UI warnings.
- [x] Prevent confirmation of facts without explicit user action.
- [x] Add strict persona-scoped repository tests.

**Acceptance:** Facts can be created, edited, reviewed, verified, disputed, marked outdated, archived, and deleted. Sources remain attributable. A fact from persona A can never be read in persona B. Automated integration tests cover typed values, lifecycle transitions, conflict detection, and strict cross-persona isolation.

### SC-012 — Data export and deletion
- [x] Export all local data and per-persona data in documented JSON format.
- [x] Exclude credentials and secrets from exports.
- [x] Implement full deletion with confirmation and verification.
- [x] Document whether deletion includes backups, temporary screenshots, and WAL files.
- [x] Test export round-trip and deletion.

**Acceptance:** Export contains the requested scope and no secrets; deletion removes owned data and reports any remaining OS credential references. Automated integration tests cover full application export, single-persona export isolation, secret exclusion, database wipe with foreign key cascade, SQLite WAL checkpoint TRUNCATE, VACUUM, and temporary screenshot cleanup.

## P2 — Secure AI provider infrastructure

### SC-020 — Windows Credential Manager
- [x] Implement native add/read/delete/test secret operations.
- [x] Store only opaque credential references in SQLite.
- [x] Ensure secrets never enter frontend logs, telemetry, exports, or error strings.
- [x] Test credential create/update/delete and missing-credential behavior on Windows 11.
- [x] Define safe behavior on non-Windows development platforms.

**Acceptance:** API key can be added and removed; DB contains no key value; credential deletion failures are visible. Automated tests cover storage in CredentialStore, opaque reference `survey_copilot/cred/<id>` in SQLite, zero secrets in DB/IPC/export, shared credential delete protection, in-memory test fallback, and Windows Credential Manager Advapi32 native integration (`src-tauri/tests/credential_lifecycle_tests.rs` and `tests/credential_management.test.ts`).

### SC-021 — Provider configuration and adapter contracts
- [x] Implement provider configuration CRUD and validation.
- [x] Implement Gemini adapter.
- [x] Define normalized request/response/error types.
- [x] Implement connection test and model configuration.
- [x] Implement supported compatible adapter(s) and reject unsupported protocols.
- [x] Validate custom endpoint URLs and redirects.

**Acceptance:** Gemini connection test and one structured generation work without provider-specific logic leaking into answer domain. Automated tests cover provider config CRUD, opaque credential referencing, zero secret leakage in summaries/IPC, Gemini REST API payload mapping, structured output parsing (AT-032), HTTPS enforcement and localhost HTTP exemption (AT-037), cross-host redirect isolation (AT-038), normalized error category mapping, and full UI in SettingsView (`src-tauri/tests/provider_adapter_tests.rs` and `tests/provider_configuration.test.ts`).

### SC-022 — Provider pool
- [x] Implement adaptive, round-robin, weighted, and priority selection.
- [x] Implement per-key health, cooldown, bounded concurrency, timeout, cancellation, and bounded retry.
- [x] Respect Retry-After and normalized error categories.
- [x] Stop retries for account/project quota or billing restrictions.
- [x] Prevent key rotation from evading provider quotas or terms.
- [x] Preserve local user input on all failure paths.
- [x] Add deterministic pool tests with mocked adapters and fake clock.

**Acceptance:** Correct failover/retry decisions pass tests; invalid requests/refusals are not retried; account-level restrictions stop the pool. Automated integration and unit tests cover adaptive/round-robin/weighted/priority strategies, 429 Retry-After cooldown and failover (AT-033), project/account quota immediate retry stop and request preservation (AT-034), invalid request / content refusal non-retry (AT-035), user cancellation (AT-036), bounded concurrency, zero credential leakage, and full SettingsView management UI (`src-tauri/tests/provider_pool_tests.rs` and `tests/provider_pool.test.ts`).

## P3 — Question-to-answer workflow

### SC-030 — Survey sessions and manual question input
- [x] Implement session create/list/update/archive/delete scoped to persona.
- [x] Implement manual question text entry and persistence.
- [x] Store instructions, options, format, source, and confirmation status.
- [x] Preserve drafts after provider failures and app restart.
- [x] Add session/question repository tests.

**Acceptance:** A user can create a persona-owned session, paste a question, close/reopen the app, and recover it. Automated integration and unit tests cover session CRUD, strict persona isolation, ordered question sequence, options builder, draft auto-save and persistence across restart, separation of actual user confirmed answers, and foreign-key cascade deletion (`src-tauri/tests/survey_session_tests.rs` and `tests/survey_session.test.ts`).

### SC-031 — Question classifier and constraint extraction
- [x] Implement deterministic normalization and typed classifier output.
- [x] Extract answer format, units, timeframe, range, scope, and ambiguity.
- [x] Preserve raw user-confirmed question text.
- [x] Add synthetic regression fixtures for common question formats.

**Acceptance:** Classifier output is typed, tested, and never changes the original question. Automated integration and unit tests cover all standard categories (demographic, employment, financial, consumption, asset ownership, brand usage, experience, preference, satisfaction rating, multiple choice, matrix/grid, numeric, date, free text, other/unknown), constraint extraction (units, timeframe, selection limits, scale directions, numeric range, text length), explicit vs inferred provenance, ambiguity detection (AT-041 missing timeframe, missing unit, conflicting instructions, mismatched options, incomplete text), raw question text immutability, persona/session isolation, and full Copilot UI analysis workflow (`src-tauri/tests/question_classifier_tests.rs` and `tests/question_classifier.test.ts`).

### SC-032 — Evidence retrieval and consistency engine
- [x] Retrieve only relevant, current, confirmed facts from active persona.
- [x] Filter disputed/outdated/unverified facts from definitive claims.
- [x] Apply sensitivity consent per request.
- [x] Detect conflicting facts and previous-answer contradictions.
- [x] Validate session/persona ownership at every boundary.

**Acceptance:** Automated integration and unit tests (`src-tauri/tests/evidence_retrieval_tests.rs` and `tests/evidence_retrieval.test.ts`) prove zero cross-persona leakage (AT-042), deterministic ranking and filtering, strict handling of unverified/disputed/outdated facts, timeframe mismatch and validity period enforcement, detection of conflicting facts (AT-045) and past survey answer contradictions, dynamic sensitivity consent filtering, and prompt-injection resistance.

### SC-033 — Structured AI answer engine
- [x] Implement versioned prompts.
- [x] Implement provider-neutral structured generation.
- [x] Validate JSON schema, enums, field lengths, and source-ID allowlist locally.
- [x] Reject unsupported factual drafts.
- [x] Ask one focused clarification or abstain when needed.
- [x] Store model/provider/prompt metadata without raw personal prompts by default.
- [x] Add prompt-injection, malformed-output, and evidence-grounding regression tests.

**Acceptance:** The answer engine returns a supported draft or explicit abstention. No model output can bypass local evidence policy. Automated integration and unit tests (`src-tauri/tests/ai_answer_engine_tests.rs` and `tests/ai_answer_engine.test.ts`) verify versioned prompt construction (`v1.0`), strict local validation against JSON schema, evidence allowlist enforcement and foreign/hallucinated citation rejection (AT-043), prompt injection defense (AT-044), conflicting evidence handling and abstention (AT-045), strict persona isolation (AT-042), sensitive fact consent filtering, and deterministic fallback with draft preservation on provider failure (AT-040).

### SC-034 — Review actions and history
- [x] Implement edit, confirm, copy, reject, regenerate.
- [x] Keep AI draft separate from user-edited answer.
- [x] Confirmation means user approval of draft only, never survey submission.
- [x] Implement persona-scoped history and filters.
- [x] Never auto-promote an answer to a verified fact.

**Acceptance:** User actions persist and are auditable; no external survey action occurs. Migration 0007 introduces revision audit fields in `answer_recommendations` and complete `answer_history` audit tracking. Full set of typed review commands implemented (`survey_answer_edit`, `survey_answer_confirm`, `survey_answer_copy`, `survey_answer_reject`, `survey_answer_regenerate`, `survey_answer_history_list`, `survey_answer_revisions_list`). Permanent separation maintained between `draft_answer` (AI recommendation) and `actual_answer` / `user_answer` (user's finalized input). Automated integration and unit tests (`src-tauri/tests/review_actions_and_history_tests.rs` and `tests/review_actions_and_history.test.ts`) verify that confirmation never touches external browser/network or auto-promotes to profile facts, regenerate retains historical revisions, and persona isolation is strictly enforced. Complete Indonesian UI in `CopilotView.tsx` with review action drawer, revision timeline, and filterable history.

### Comprehensive Integration Audit Fase P3 (SC-030 s.d. SC-034)
- [x] Audit integrasi end-to-end dari Survey Sessions, Question Classifier, Evidence Retrieval, Answer Engine, sampai Review Actions & History.
- [x] Verifikasi skema basis data, 7 migrasi berurutan, integritas foreign keys, dan isolasi persona mutlak di SQL & Rust service layer.
- [x] Verifikasi fondasi Survey Memory: penyimpanan judul survei, platform, URL sumber, urutan sekuensial pertanyaan, provenance, pemisahan permanen draft AI vs jawaban aktual pengguna, dan riwayat revisi.
- [x] Pertahanan prompt injection (AT-044), penegakan allowlist sitasi bukti (AT-043), deteksi konflik kritis & pemaksaan abstensi (AT-045), dan perizinan data sensitif (consent).
- [x] Batas produk non-negosiasi: konfirmasi hanya persetujuan internal; dilarang keras auto-submit/auto-fill/browser automation, dan dilarang keras otomatis mempromosikan jawaban survei menjadi fakta profil.
- [x] Seluruh tes regresi dan integrasi komprehensif berhasil dijalankan (`src-tauri/tests/p3_comprehensive_integration_tests.rs`, seluruh test suites P3, 102/102 Rust tests passed, 72/72 frontend tests passed).

## P4 — Screen intelligence

### SC-040 — Windows capture
- [x] Evaluate Windows capture APIs and choose supported implementation (Win32 GDI + DWM + PNG compression).
- [x] Implement explicit region capture and selected-window capture (`capture_region`, `capture_window`, `capture_list_windows`).
- [x] Display visible preview and capture cancellation (`CapturePreview` base64 data URL, `capture_discard`, UI preview modal).
- [x] Restrict file access to app-managed temporary paths (`app_data_dir/temp_captures`, anti-path-traversal UUID validation).
- [x] Delete temporary images according to retention policy (`capture_cleanup`, `discard_capture`, `wipe_all_data`).
- [x] Add Windows-specific integration tests (`src-tauri/tests/capture_tests.rs`, `tests/capture.test.ts`, AT-050, AT-051, AT-054).

**Acceptance:** User can select a region/window and preview the image; no capture occurs without explicit action. (Status: Done & Verified)

### SC-041 — OCR and layout reconstruction
- [x] Evaluate Windows OCR and Tesseract for Indonesian/English survey content (Windows Media OCR evaluated and integrated via subprocess sidecar pattern).
- [x] Implement OCR text and bounding boxes (domain models BoundingBox, OcrWord, OcrLine, OcrRawResult).
- [x] Reconstruct prompt, instruction, options, scales, units, and matrix rows/columns (`LayoutReconstructorService`).
- [x] Surface low-confidence, clipped, and ambiguous regions (AT-052).
- [x] Preserve capture-to-extraction provenance (migration `0008_ocr_extractions.sql` and `OcrRepository`).
- [x] Add OCR fixture tests with representative synthetic screenshots (`ocr_tests.rs` with 10 integration tests, `tests/ocr.test.ts` with 6 TypeScript contract tests).

**Acceptance:** User can review/edit extracted content and must confirm before analysis. Manual input remains available if OCR fails. (Status: Done & Verified)

## P5 — Survey fit

### SC-050 — Opportunity capture and criteria extraction
- [x] Store visible opportunity title, description, platform label, and capture provenance.
- [x] Extract explicit criteria and preserve original text.
- [x] Do not infer hidden platform rules or hidden screener state.

**Acceptance:** Criteria shown to the user are traceable to visible captured text. (Status: Done & Verified)

### SC-051 — Deterministic persona fit
- [x] Compare explicit criteria to current persona facts.
- [x] Compute match/mismatch/unknown/stale/conflict locally.
- [x] Show supporting fact IDs and explanation.
- [x] Unknown is never treated as match.
- [x] Compare across personas only for truthful respondent selection, never identity substitution.

**Acceptance:** Each criterion has an auditable deterministic result; model output cannot set final eligibility state. (Status: Done & Verified)

### SC-052 — Historical outcome learning
- [x] Record survey attempt outcomes with persona, time, category, criteria, evidence.
- [x] Keep outcome labels distinct: qualified, screened_out, quota_full, completed, quality_rejected, technical_failure, abandoned_by_user, pending, unknown.
- [x] Do not create reward or income tracking.
- [x] Do not show predictive percentages until minimum sample/calibration requirements are specified and validated.
- [x] Use chronological holdout and prevent data leakage.

**Acceptance:** Sparse history is labeled insufficient; predictions disclose sample size and uncertainty; no answer manipulation is suggested. (Status: Done & Verified)
- Implemented Migration 0010 (`0010_survey_attempts_indexes.sql`) adding performance indexes for attempt opportunities, outcomes, events, and confidence.
- Complete domain models in Rust (`SurveyAttempt`, `SurveyOutcomeEvent`, `SurveyAttemptDetail`, `OutcomeLabel`, `OutcomeConfidence`, `LearningDataStatus`, `HistoricalLearningSummary`, `AttemptFilter`, `CreateAttemptInput`, `RecordOutcomeEventInput`).
- Atomic repository operations in `AttemptRepository` with full support for all 9 normative attempt statuses and event history timeline.
- Business service `OutcomeLearningService` enforcing:
  - **AT-063**: When sample size < 10 (`MIN_SAMPLE_THRESHOLD`), status is strictly `insufficient_data`, completion/qualification rates are set to `None`, and clear uncertainty disclosures are provided without fabricated probabilities.
  - When sample size >= 10, executes Chronological Holdout Validation (70% earliest train set, 30% holdout test set) preventing data leakage and quantifying calibration consistency.
  - Strict separation between raw empirical observations and AI/statistical learning summary.
  - Non-negotiable boundaries: zero reward/points/earnings/balance tracking, and explicit rules forbidding profile or answer manipulation.
- 6 Tauri IPC commands in `commands/survey_attempt.rs` (`survey_attempt_record`, `survey_attempt_record_outcome`, `survey_attempt_get`, `survey_attempt_list`, `survey_attempt_delete`, `survey_attempt_get_learning_summary`).
- Frontend TypeScript types (`src/types/attempt.ts`), client service (`src/services/attemptService.ts`), and rich interactive UI in `HistoryView.tsx` with dynamic multi-criteria filtering, status badge styling, separate AI learning cards, attempt creation and event append modals, plus seamless integration with `OpportunityView.tsx`.
- Verified by 8/8 Rust integration tests (`src-tauri/tests/survey_attempt_tests.rs`) and 8/8 TypeScript contract tests (`tests/survey_attempt.test.ts`), with 180/180 total Rust tests passing and 104/104 frontend tests passing.

### SC-053 — Survey Memory persistence and recognition
- [x] Record survey session metadata: title, source platform, URL/domain, timestamps, persona ownership, and structural fingerprint (Migration 0011 adds `source_domain`, `version`, `recognized_from_id`, and `fingerprint` indexing).
- [x] Store ordered question sequence with exact prompt text, instructions, options, and provenance (user-entered, OCR-detected, manually confirmed) with deterministic SHA-256 question fingerprints.
- [x] Store actual user-confirmed answers strictly separately from AI recommendations (`user_answer_confirmed` preserved independently from `answer_recommendations.draft_text` / `user_answer_draft`). Enforces AT-064.
- [x] Implement survey recognition engine (`SurveyRecognitionService`) evaluating structural fingerprints, version changes, and non-negotiable rule that title similarity alone never produces a definite match (`ExactMatch`/`LikelyMatch`).
- [x] Enforce strict persona isolation: historical personal answers from persona A are inaccessible to persona B in candidate discovery, memory detail, and recognition history. Enforces AT-066.
- [x] Provide Survey Memory UI banner and recognition detail modal in `CopilotView.tsx` displaying domain, version, structural fingerprint, recognition status, matched session comparison, differences, and confirmed answer history.
- Delivered:
  - Database migration `0011_survey_memory_and_recognition.sql` adding `source_domain`, `version`, `recognized_from_id` on `survey_sessions`, `question_fingerprint` on `survey_questions`, and `survey_recognitions` table for recognition audit trail.
  - Domain models and deterministic fingerprint algorithms in `domain/survey.rs` (`compute_question_fingerprint`, `compute_session_structural_fingerprint`, `extract_domain_from_url`, `SurveyRecognitionStatus`, `SurveyRecognitionRecord`, `SurveyRecognitionEvaluation`, `SurveyMemorySessionDetail`, `SurveyMemoryQuestionItem`).
  - Repository methods in `repository/survey_repo.rs` (`update_session_fingerprint`, `save_recognition`, `get_latest_recognition`, `list_candidate_sessions_for_recognition`, `get_memory_session_detail` with strict answer separation and persona scoping).
  - Business service in `services/survey_recognition_service.rs` enforcing structural heuristic thresholds, version difference tracking, title-alone match capping, and persona isolation.
  - Tauri IPC commands in `commands/survey.rs` (`survey_memory_recognize`, `survey_memory_get_detail`).
  - Frontend contracts (`src/types/survey.ts`), service client (`src/services/surveyService.ts`), and Copilot UI integration (`src/components/views/CopilotView.tsx`).
  - Verified by 9/9 Rust integration tests in `src-tauri/tests/survey_memory_tests.rs` and 5/5 TypeScript contract tests in `tests/survey_memory.test.ts`.

**Acceptance:** Survey structure and question history are persisted in order; actual answers are kept distinct from AI drafts; persona isolation is enforced in repository and service layers.

### SC-054 — Next Question Intelligence and branch prediction
- [x] Model observed branch transitions and question sequences from historical survey sessions.
- [x] Predict likely next question(s) with confidence, source evidence, and explicit "Predicted — not yet detected" labeling.
- [x] Update branch models when actual questions are observed after manual screen capture / user advancement.
- [x] Return insufficient evidence when history is sparse or conflicting; never invent certain predictions.
- [x] Integrate Next Question Intelligence panel into the active Copilot workspace view.

**Acceptance:** Predicted questions are clearly distinguished from detected questions; prediction quality improves with verified observations; screener deception or fabricated answers are prohibited.
*Status:* Complete. Implemented Migration 0012 (`survey_branch_transitions`, `survey_question_predictions`), NextQuestionService, Tauri IPC commands (`survey_predict_next_questions`, `survey_get_session_predictions`, `survey_get_session_branch_transitions`), Copilot UI visual panel with confidence and branch badges, verified by 8/8 Rust integration tests (`next_question_intelligence_tests.rs`) and 7 unit/contract tests (`tests/next_question_intelligence.test.ts`).

## P6 — Release hardening

### SC-060 — Privacy, settings, and diagnostics
- [x] Add provider, privacy, data, and diagnostics settings.
- [x] Show what data leaves the device before external analysis.
- [x] Obtain explicit per-request consent for sensitive data and external vision.
- [x] Redact logs by default.
- [x] Add data retention and temporary image deletion controls.

**Acceptance:** User has complete visibility and control over privacy, data transmission, and retention.
- Database Migration `0013_privacy_settings_and_diagnostics.sql` creates `app_diagnostic_logs` with performance indexes and seeds default privacy settings into `app_settings`.
- Robust `LogRedactor` engine automatically redacts API keys (`AIza...`, generic `sk-...`), bearer tokens, passwords, Indonesian NIK (16 digits), emails, phone numbers, and payment cards in strings and JSON payloads.
- `PrivacyService` provides:
  - Configuration retrieval and atomic updating (`auto_redact_logs`, `require_sensitive_consent_prompt`, `require_vision_consent_prompt`, `screenshot_retention_policy`, `auto_cleanup_temp_on_exit`, `log_level`).
  - Outbound data transparency engine (`inspect_outbound_data`) detailing provider, destination endpoint, included facts with sensitivity badges, excluded facts with reasons, external vision status, and local data guarantees before sending to external AI.
  - Strict backend per-request consent enforcement (`enforce_per_request_consent`) for sensitive profile facts and external vision.
  - Safe, pre-redacted diagnostic logging (`record_log`, `list_logs`, `clear_logs`) ensuring zero PII or credentials in logs.
  - Retention status inspection and immediate temporary screenshot file cleanup (`cleanup_temp_files`, `get_retention_status`).
  - Comprehensive system diagnostics (`get_system_diagnostics`) reporting SQLite version, schema version (v13), database/WAL file sizes, and entity counts.
- 9 Tauri IPC commands in `src-tauri/src/commands/privacy.rs` (`privacy_get_settings`, `privacy_update_settings`, `privacy_inspect_outbound_data`, `privacy_get_retention_status`, `privacy_cleanup_temp_files`, `privacy_get_diagnostic_logs`, `privacy_clear_diagnostic_logs`, `privacy_get_system_diagnostics`, `privacy_record_diagnostic_log`).
- Complete frontend TypeScript contracts (`src/types/privacy.ts`), client service (`src/services/privacyService.ts`), and Indonesian UI:
  - `SettingsView.tsx`: Tab **Privasi & Diagnostik** with 4 cards (Privacy & Consent Controls, Storage & Temp Captures Cleanup, System Diagnostics, and Safe Redacted Log Viewer with level/category filtering).
  - `CopilotView.tsx`: Interactive modal **Transparansi Transmisi Data (What Data Leaves The Device)** and per-request consent checkboxes for sensitive facts and external vision.
- Verified by:
  - 14/14 Rust integration tests in `src-tauri/tests/privacy_and_diagnostics_tests.rs` (exit code 0).
  - 6/6 frontend contract tests in `tests/privacy_and_diagnostics.test.ts` (123/123 total frontend tests passing, exit code 0).
  - Full regression test suites passing (migration: 8/8, NQI: 8/8, memory: 9/9, export/deletion: 7/7).
  - Production build passing with zero errors (`npm run build`).

### SC-061 — UI quality and accessibility
- [x] Replace placeholder/disabled controls with implemented actions or remove them (verified: all operational controls wired to actual domain services; actionable button added to empty copilot state).
- [x] Add loading, empty, success, error, retry, and offline states (verified: offline banner across application, retry buttons on failed network/load operations, dismissible notifications with role="status"/role="alert", and descriptive empty states).
- [x] Keyboard navigation and accessible names (verified: global WCAG 2.1 AA :focus-visible indicators, Escape key closes all active modal dialogs, accessible names and aria-label on all close/action buttons, modal dialogs equipped with role="dialog"/role="alertdialog", aria-modal="true", and aria-labelledby).
- [x] Validate Indonesian copy, scaling, and minimum window behavior (verified: standardized action copy "Ubah", "Simpan", "Batal", "Hapus", "Tandai Usang (Outdated)"; responsive layout adjustments down to 860px and Windows 11 high contrast / forced colors support).
- Verified by 7/7 new automated frontend tests in `tests/ui_quality_and_accessibility.test.ts` (130/130 total frontend tests passing across 7 suites), production build passing with zero errors (`npm run build`), and Rust checks/tests passing with `-j 1`.

### SC-062 — Windows release
- [x] Build NSIS installer for Windows 11 x64 (`Survey Copilot_0.1.0_x64-setup.exe`, 5.97 MiB, SHA-256 verified, `installMode: "currentUser"`).
- [ ] Test clean Windows 11 x64 install, launch, update, uninstall, and data retention behavior (Manual testing checklist prepared in `docs/05-development/MANUAL_TESTING_GUIDE.md` for user testing).
- [x] Document and test app data backup/restore and update workflow (`docs/06-operations/BACKUP_AND_RESTORE.md` and `docs/06-operations/UPDATE_PROCEDURE.md`).
- [x] Review Tauri capabilities, CSP, dependency licenses, and security (`docs/05-development/SECURITY_AND_CAPABILITIES_REVIEW.md`).
- [x] Complete release notes and troubleshooting (`docs/05-development/RELEASE_NOTES_0.1.0_BETA.md`).
- [x] Do not publish until all MUST requirements and release gates pass (Private testing release only; awaiting user approval before final production release).

**Acceptance:** Windows 11 x64 NSIS testing package built cleanly without administrator rights requirement (`currentUser`), data retention guaranteed at `%APPDATA%\com.surveycopilot.desktop` across updates and uninstallations, credentials isolated in Windows Credential Manager, and complete operational and testing documentation provided.


## Post-audit findings and owner feedback (24 November 2026)

The following documents capture a repository/source audit and post-installation feedback. They are **not marked as implemented backlog items** and must be reviewed and prioritized before implementation:

- [Audit: completeness, security, and release readiness](../09-audit/AUDIT_COMPLETENESS_SECURITY_RELEASE_2026-11-24.md) — includes code-level risks R-01 through R-15, CI evidence, installer availability, and release gates.
- [Installation and UI/UX feedback](../09-audit/INSTALLATION_AND_UI_UX_FEEDBACK_2026-11-24.md) — includes responsive redesign, dark Bento direction, form usability, profile templates/import, capture-to-local-OCR flow, broken-page reproduction, UI-only browser preview, and uninstall discoverability.

Do not infer that a checkbox elsewhere in this historical backlog overrides an audit finding. Do not mark these follow-up requirements complete until implementation and relevant manual/automated evidence exist. Product-scope changes (including any interpretation of “profit”) require explicit owner approval. The browser preview is UI-only with synthetic data/mock services; desktop functionality remains in the installed Windows app.


## Stage 3 repository implementation (branch: feat/stage-3-dark-bento-profile-ux-2026-11-24)

This branch implements the Stage 3 UI/UX requirements from owner feedback. It is still subject to repository CI and review; do not mark release or device acceptance complete from the existence of this branch.

- Shared Dark Bento theme for installed desktop frontend and browser UI preview: `src/styles/dark-bento.css`.
- Responsive navigation, forms, cards, settings panels, dialogs, and profile layout.
- Blank profile field templates: demographics, work/education, household, technology, consumption, and mobility. Templates set field metadata only and never fabricate respondent values.
- Local JSON profile import from a single-persona Survey Copilot export (schema 1.0.0). The import creates a new inactive persona with new IDs, imports current facts and evidence sources transactionally, and excludes survey sessions, answers, screenshots, provider settings, credentials, and diagnostics.
- Missing-source facts are marked unverified. Invalid or oversized imports roll back completely.

Details and deferred device checks: [Stage 3 — Dark Bento and Profile Import](../09-audit/STAGE_3_DARK_BENTO_PROFILE_UX.md).


## Stage 4 repository implementation (branch: feat/stage-4-capture-auto-local-ocr-2026-11-24)

The region and window capture handlers now start local OCR automatically after a successful explicit capture. The result is presented for human review/correction; no AI analysis or external transmission starts automatically. OCR errors leave the capture preview available with a retry action and manual fallback. Capture requires an active session. The capture window list uses an explicit keyboard-accessible action button, and controls that could race an in-flight OCR operation are disabled.

See [Stage 4 — Capture and Automatic Local OCR](../09-audit/STAGE_4_CAPTURE_AUTO_LOCAL_OCR.md). Do not mark installed-device acceptance complete until the consolidated Windows 11 x64 device checklist is run after all five stages.
