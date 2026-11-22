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
- [ ] Export all local data and per-persona data in documented JSON format.
- [ ] Exclude credentials and secrets from exports.
- [ ] Implement full deletion with confirmation and verification.
- [ ] Document whether deletion includes backups, temporary screenshots, and WAL files.
- [ ] Test export round-trip and deletion.

**Acceptance:** Export contains the requested scope and no secrets; deletion removes owned data and reports any remaining OS credential references.

## P2 — Secure AI provider infrastructure

### SC-020 — Windows Credential Manager
- [ ] Implement native add/read/delete/test secret operations.
- [ ] Store only opaque credential references in SQLite.
- [ ] Ensure secrets never enter frontend logs, telemetry, exports, or error strings.
- [ ] Test credential create/update/delete and missing-credential behavior on Windows 11.
- [ ] Define safe behavior on non-Windows development platforms.

**Acceptance:** API key can be added and removed; DB contains no key value; credential deletion failures are visible.

### SC-021 — Provider configuration and adapter contracts
- [ ] Implement provider configuration CRUD and validation.
- [ ] Implement Gemini adapter.
- [ ] Define normalized request/response/error types.
- [ ] Implement connection test and model configuration.
- [ ] Implement supported compatible adapter(s) and reject unsupported protocols.
- [ ] Validate custom endpoint URLs and redirects.

**Acceptance:** Gemini connection test and one structured generation work without provider-specific logic leaking into answer domain.

### SC-022 — Provider pool
- [ ] Implement adaptive, round-robin, weighted, and priority selection.
- [ ] Implement per-key health, cooldown, bounded concurrency, timeout, cancellation, and bounded retry.
- [ ] Respect Retry-After and normalized error categories.
- [ ] Stop retries for account/project quota or billing restrictions.
- [ ] Prevent key rotation from evading provider quotas or terms.
- [ ] Preserve local user input on all failure paths.
- [ ] Add deterministic pool tests with mocked adapters and fake clock.

**Acceptance:** Correct failover/retry decisions pass tests; invalid requests/refusals are not retried; account-level restrictions stop the pool.

## P3 — Question-to-answer workflow

### SC-030 — Survey sessions and manual question input
- [ ] Implement session create/list/update/archive/delete scoped to persona.
- [ ] Implement manual question text entry and persistence.
- [ ] Store instructions, options, format, source, and confirmation status.
- [ ] Preserve drafts after provider failures and app restart.
- [ ] Add session/question repository tests.

**Acceptance:** A user can create a persona-owned session, paste a question, close/reopen the app, and recover it.

### SC-031 — Question classifier and constraint extraction
- [ ] Implement deterministic normalization and typed classifier output.
- [ ] Extract answer format, units, timeframe, range, scope, and ambiguity.
- [ ] Preserve raw user-confirmed question text.
- [ ] Add synthetic regression fixtures for common question formats.

**Acceptance:** Classifier output is typed, tested, and never changes the original question.

### SC-032 — Evidence retrieval and consistency engine
- [ ] Retrieve only relevant, current, confirmed facts from active persona.
- [ ] Filter disputed/outdated/unverified facts from definitive claims.
- [ ] Apply sensitivity consent per request.
- [ ] Detect conflicting facts and previous-answer contradictions.
- [ ] Validate session/persona ownership at every boundary.

**Acceptance:** Automated tests prove zero cross-persona leakage and correct abstention for missing/conflicting evidence.

### SC-033 — Structured AI answer engine
- [ ] Implement versioned prompts.
- [ ] Implement provider-neutral structured generation.
- [ ] Validate JSON schema, enums, field lengths, and source-ID allowlist locally.
- [ ] Reject unsupported factual drafts.
- [ ] Ask one focused clarification or abstain when needed.
- [ ] Store model/provider/prompt metadata without raw personal prompts by default.
- [ ] Add prompt-injection, malformed-output, and evidence-grounding regression tests.

**Acceptance:** The answer engine returns a supported draft or explicit abstention. No model output can bypass local evidence policy.

### SC-034 — Review actions and history
- [ ] Implement edit, confirm, copy, reject, regenerate.
- [ ] Keep AI draft separate from user-edited answer.
- [ ] Confirmation means user approval of draft only, never survey submission.
- [ ] Implement persona-scoped history and filters.
- [ ] Never auto-promote an answer to a verified fact.

**Acceptance:** User actions persist and are auditable; no external survey action occurs.

## P4 — Screen intelligence

### SC-040 — Windows capture
- [ ] Evaluate Windows capture APIs and choose supported implementation.
- [ ] Implement explicit region capture and selected-window capture.
- [ ] Display visible preview and capture cancellation.
- [ ] Restrict file access to app-managed temporary paths.
- [ ] Delete temporary images according to retention policy.
- [ ] Add Windows-specific integration tests.

**Acceptance:** User can select a region/window and preview the image; no capture occurs without explicit action.

### SC-041 — OCR and layout reconstruction
- [ ] Evaluate Windows OCR and Tesseract for Indonesian/English survey content.
- [ ] Implement OCR text and bounding boxes.
- [ ] Reconstruct prompt, instruction, options, scales, units, and matrix rows/columns.
- [ ] Surface low-confidence, clipped, and ambiguous regions.
- [ ] Preserve capture-to-extraction provenance.
- [ ] Add OCR fixture tests with representative synthetic screenshots.

**Acceptance:** User can review/edit extracted content and must confirm before analysis. Manual input remains available if OCR fails.

## P5 — Survey fit

### SC-050 — Opportunity capture and criteria extraction
- [ ] Store visible opportunity title, description, platform label, and capture provenance.
- [ ] Extract explicit criteria and preserve original text.
- [ ] Do not infer hidden platform rules or hidden screener state.

**Acceptance:** Criteria shown to the user are traceable to visible captured text.

### SC-051 — Deterministic persona fit
- [ ] Compare explicit criteria to current persona facts.
- [ ] Compute match/mismatch/unknown/stale/conflict locally.
- [ ] Show supporting fact IDs and explanation.
- [ ] Unknown is never treated as match.
- [ ] Compare across personas only for truthful respondent selection, never identity substitution.

**Acceptance:** Each criterion has an auditable deterministic result; model output cannot set final eligibility state.

### SC-052 — Historical outcome learning
- [ ] Record survey attempt outcomes with persona, time, category, criteria, evidence.
- [ ] Keep outcome labels distinct: qualified, screened_out, quota_full, completed, quality_rejected, technical_failure, abandoned_by_user, pending, unknown.
- [ ] Do not create reward or income tracking.
- [ ] Do not show predictive percentages until minimum sample/calibration requirements are specified and validated.
- [ ] Use chronological holdout and prevent data leakage.

**Acceptance:** Sparse history is labeled insufficient; predictions disclose sample size and uncertainty; no answer manipulation is suggested.

### SC-053 — Survey Memory persistence and recognition
- [ ] Record survey session metadata: title, source platform, URL/domain, timestamps, persona ownership, and structural fingerprint.
- [ ] Store ordered question sequence with exact prompt text, instructions, options, and provenance (user-entered, OCR-detected, manually confirmed).
- [ ] Store actual user-confirmed answers strictly separately from AI recommendations.
- [ ] Implement survey recognition comparing structure, title, and source against Survey Memory with confidence scoring.
- [ ] Enforce strict persona isolation: historical personal answers from persona A are inaccessible to persona B.
- [ ] Provide Survey History UI to browse, inspect, and manage historical survey sessions and questions.

**Acceptance:** Survey structure and question history are persisted in order; actual answers are kept distinct from AI drafts; persona isolation is enforced in repository and service layers.

### SC-054 — Next Question Intelligence and branch prediction
- [ ] Model observed branch transitions and question sequences from historical survey sessions.
- [ ] Predict likely next question(s) with confidence, source evidence, and explicit "Predicted — not yet detected" labeling.
- [ ] Update branch models when actual questions are observed after manual screen capture / user advancement.
- [ ] Return insufficient evidence when history is sparse or conflicting; never invent certain predictions.
- [ ] Integrate Next Question Intelligence panel into the active Copilot workspace view.

**Acceptance:** Predicted questions are clearly distinguished from detected questions; prediction quality improves with verified observations; screener deception or fabricated answers are prohibited.

## P6 — Release hardening

### SC-060 — Privacy, settings, and diagnostics
- [ ] Add provider, privacy, data, and diagnostics settings.
- [ ] Show what data leaves the device before external analysis.
- [ ] Obtain explicit per-request consent for sensitive data and external vision.
- [ ] Redact logs by default.
- [ ] Add data retention and temporary image deletion controls.

### SC-061 — UI quality and accessibility
- [ ] Replace placeholder/disabled controls with implemented actions or remove them.
- [ ] Add loading, empty, success, error, retry, and offline states.
- [ ] Keyboard navigation and accessible names.
- [ ] Validate Indonesian copy, scaling, and minimum window behavior.

### SC-062 — Windows release
- [ ] Build NSIS/MSI installer.
- [ ] Test clean Windows 11 x64 install, launch, update, uninstall, and data retention behavior.
- [ ] Test app data backup/restore and migration.
- [ ] Review Tauri capabilities, CSP, dependency licenses, and security.
- [ ] Complete release notes and troubleshooting.
- [ ] Do not publish until all MUST requirements and release gates pass.
