# Documentation Gaps and Clarifications

This file prevents an implementation agent from treating underspecified decisions as permission to invent behavior. Resolve each item with a small ADR or explicit owner decision before the dependent feature is implemented.

## G-001 — Migration strategy
The current startup code executes `0001_initial.sql` with `CREATE TABLE IF NOT EXISTS`; this is not a complete sequential migration runner. Define migration registry, transactional semantics, backup/recovery, and schema version behavior before adding migration 0002.

## G-002 — Fact value types
The starter UI/backend currently store fact values as strings serialized into JSON. Define allowed types (string, number, boolean, date, enum/list, structured object), normalization, locale handling, and comparison semantics before implementing eligibility matching.

## G-003 — Source provenance and deletion
Define source types, user-entered evidence, file/reference provenance, timestamps, and whether source records can be shared by multiple facts. Define how deletion affects recommendation source references and history without retaining deleted personal data unintentionally.

## G-004 — Persona lifecycle
Define rename, archive, restore, delete, active-persona fallback, and child-record deletion behavior. The database currently has an active persona uniqueness index; transitions must be transactional.

## G-005 — Sensitive-data consent
Define a per-request consent UI and exact provider payload preview. Sensitive/highly sensitive facts must be excluded by default. Consent must be scoped to a single request and must not become blanket consent.

## G-006 — Windows capture implementation
Select and document the capture API, multi-monitor coordinate handling, DPI scaling, window selection behavior, cancellation, and temporary file lifecycle. Must work on Windows 11 x64 and must be user-initiated.

## G-007 — OCR choice
Benchmark Windows OCR and Tesseract using representative Indonesian and English survey screenshots. Decide language packs, packaging, offline behavior, accuracy thresholds, and license implications before finalizing the stack.

## G-008 — Provider protocol scope
“Custom compatible provider” is too broad without a precise protocol matrix. Define which adapters are supported (e.g. Gemini native and OpenAI-compatible) and reject unsupported protocols. Define request schema, authentication header behavior, model listing capability, streaming support, and redirects.

## G-009 — Gemini API semantics
Confirm current Gemini API request format, structured output support, model identifiers, error categories, retry headers, quota scopes, and official SDK/HTTP approach against current provider documentation at implementation time. Do not hardcode stale model names.

## G-010 — Provider pooling policy
Clarify adaptive selection formula, per-key versus account/project quota state, maximum concurrency, circuit breaker behavior, and whether a key can belong to multiple billing projects. Never rotate credentials to evade account-level restrictions.

## G-011 — Session and question lifecycle
Define whether a session represents one survey attempt, a browser visit, or a group of questions; define pause/resume/abandon and whether a question may be associated with multiple attempts. Keep persona ownership immutable.

## G-012 — Survey outcome learning
Define what evidence is required to mark an outcome confirmed, minimum sample size, category grouping, calibration, and chronological holdout. Do not display a success probability until statistically validated.

## G-013 — Retention and privacy
Define retention for screenshots, extracted text, questions, history, logs, exports, and backups. Provide user-facing explanation before external AI processing.

## G-014 — Accessibility and UI behavior
Define keyboard shortcuts, focus behavior, minimum supported window dimensions, scaling, screen-reader labels, and error/empty/loading states.

## G-015 — Release signing and updates
Define installer signing, update delivery, versioning, rollback, crash reporting (off by default unless consented), and uninstall data retention.

## G-016 — Legal review
Privacy policy is a draft and is not legal advice or a reviewed legal document. Obtain appropriate legal review before public distribution.
