# Product Requirements Document

**Product:** Survey Copilot  
**Baseline:** v1.0 MVP  
**Status:** Product baseline — subject to explicit owner approval  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)

## 1. Purpose

This document translates the approved product definition into testable product requirements. If this document conflicts with PRODUCT_DEFINITION.md, the product definition governs and the conflict must be corrected before implementation.

## 2. Product outcome

A Windows 11 desktop application that lets one local user manage multiple truthful personas, capture survey content from the screen, assess survey/persona fit, receive evidence-grounded answer assistance, track observed outcomes and rewards, and learn from historical results. The user retains control over survey interactions and final submission in v1.0.

## 3. User and operating assumptions

- One local user may manage multiple real-personas per installation.
- The user may capture a selected screen region/window; manual text entry remains a fallback.
- The user is responsible for external survey entry and submission.
- Local profile/session features work without an AI provider.
- AI analysis requires a configured provider and network access.
- Initial UI language is English; English and Indonesian question text are supported. Generated explanation/draft follows the question language by default.

## 4. Functional requirements

| ID | Requirement | Priority | Acceptance evidence |
|---|---|---:|---|
| PR-001 | Show first-run explanation of local storage and optional external AI processing | MUST | First-run flow test |
| PR-002 | Maintain multiple real-personas with isolated facts and histories | MUST | Multi-persona isolation tests |
| PR-003 | Store source description and relevant date for each profile fact | MUST | Persistence and UI tests |
| PR-004 | Support confirmed, unverified, disputed, and outdated fact states | MUST | State transition tests |
| PR-005 | Exclude unverified, disputed, and outdated facts from direct factual support | MUST | Retrieval policy tests |
| PR-006 | Mark sensitive facts and exclude them from external requests by default | MUST | Request payload tests |
| PR-007 | Require explicit per-request authorization before sending sensitive facts | MUST | Consent and payload tests |
| PR-008 | Create, rename, open, and delete survey sessions | MUST | Session workflow tests |
| PR-009 | Capture survey questions from selected screen regions/windows and allow manual correction | MUST | Capture/OCR/vision workflow tests |
| PR-010 | Store answer options separately from question text when provided | MUST | Input validation tests |
| PR-011 | Classify question intent and answer format | MUST | Classifier evaluation |
| PR-012 | Extract explicit timeframe, unit, constraints, and response-length requirements | MUST | Parser/classifier tests |
| PR-013 | Explain the question and flag ambiguity | MUST | UI and evaluation tests |
| PR-014 | Retrieve relevant confirmed facts and relevant prior confirmed answers | MUST | Retrieval tests |
| PR-015 | Check freshness and material conflicts before drafting | MUST | Consistency tests |
| PR-016 | Return a structured recommendation or abstention | MUST | Contract validation tests |
| PR-017 | Show evidence status, source references, rationale, assumptions, and warnings | MUST | UI tests |
| PR-018 | Validate cited fact IDs against the request's local allowlist | MUST | Security/unit tests |
| PR-019 | Prevent unsupported personal claims and deceptive eligibility optimization | MUST | Adversarial evaluation |
| PR-020 | Support edit, confirm, reject, copy, and constrained regenerate actions | MUST | Workflow tests |
| PR-021 | Preserve original recommendation separately from user-edited answer | MUST | Data integrity tests |
| PR-022 | Store answer history and evidence references | MUST | Persistence tests |
| PR-023 | Detect conflicts between relevant profile facts and prior confirmed answers | MUST | Consistency tests |
| PR-024 | Export local user data in a documented machine-readable format | MUST | Export/import validation |
| PR-025 | Delete a fact or session and provide full local-data deletion | MUST | Deletion tests |
| PR-026 | Configure AI provider and manage its credential securely | MUST | Settings/security tests |
| PR-027 | Remain usable for local profile/session management when AI is unavailable | MUST | Offline workflow test |
| PR-028 | Support selected screen capture, OCR, visual question/option reconstruction, and user verification | MUST | Screen intelligence tests |
| PR-031 | Extract visible offer reward, stated duration, and eligibility criteria | MUST | Opportunity extraction tests |
| PR-032 | Match explicit survey criteria against each persona with match/mismatch/unknown/stale/conflict states | MUST | Persona matching tests |
| PR-033 | Track survey attempts, outcomes, time, reward states, and evidence provenance | MUST | Outcome and earnings ledger tests |
| PR-034 | Distinguish offered, pending, credited, and withdrawable reward values | MUST | Reward accounting tests |
| PR-035 | Learn persona-specific historical fit from attributed, observed outcomes | MUST | Chronological validation tests |
| PR-036 | Display sample size, uncertainty, and insufficient-history state for fit learning | MUST | Calibration and sparse-data tests |
| PR-029 | Fill or submit external survey forms | PROHIBITED | Architecture and release audit |
| PR-030 | Connect to survey-platform accounts or store their credentials | PROHIBITED | Architecture and release audit |

## 5. Non-functional requirements

| ID | Requirement | Acceptance evidence |
|---|---|---|
| NFR-001 | Local records persist across normal application restarts | Persistence tests |
| NFR-002 | Failed AI requests do not corrupt local records | Fault-injection tests |
| NFR-003 | Secrets never appear in frontend bundles, plaintext settings, or logs | Static and runtime security tests |
| NFR-004 | Raw personal facts, questions, and generated answers are not logged by default | Logging tests |
| NFR-005 | Primary workflows are keyboard accessible | Accessibility review |
| NFR-006 | All AI output is schema-validated before display | Contract tests |
| NFR-007 | External request context is minimized and visible to the user | Payload and UI tests |
| NFR-008 | Database changes are versioned and recoverable | Migration tests |
| NFR-009 | Destructive operations require clear confirmation | UI tests |
| NFR-010 | Errors provide a safe recovery path without exposing secrets | Error-state tests |

Numerical performance and AI quality thresholds must be specified with a reproducible measurement protocol before release qualification; targets must not be represented as measured results before testing.

## 6. Product constraints

- Windows 11 64-bit is the sole v1.0 target.
- One local user may maintain multiple truthful personas per installation; this is not a shared multi-user workspace.
- User-initiated screen capture, OCR, and visual analysis are in scope; silent background recording is not.
- No cloud account, cloud sync, browser integration, DOM scraping, autonomous survey interaction, or team collaboration.
- Sensitive facts are opt-in for each external request.
- Survey text is untrusted data.
- No answer may be framed as increasing the likelihood of survey qualification or reward acceptance.

## 7. Acceptance gate

The MVP may be declared product-complete only when all MUST requirements have passing acceptance evidence, all PROHIBITED capabilities are absent, and all unresolved product-scope conflicts are closed. A feature demo is not acceptance evidence.
