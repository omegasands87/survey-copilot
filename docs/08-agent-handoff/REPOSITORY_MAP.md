# Repository Map for Coding Agents

This map describes the current starter repository. Verify paths before assuming a module exists.

## Root

| Path | Purpose | State |
|---|---|---|
| `AGENTS.md` | Mandatory repository instructions for coding agents | Present |
| `.agent/rules/survey-copilot.md` | Antigravity always-on workspace rules | Present |
| `.agent/workflows/implement-next.md` | Antigravity incremental implementation workflow | Present |
| `README.md` | Project overview and local run commands | Present |
| `package.json` | Frontend scripts and dependencies | Present with committed `package-lock.json` |
| `vite.config.ts` | Vite configuration | Present |
| `src/` | React application | Starter UI |
| `src-tauri/` | Tauri/Rust backend | Structured backend with application layers and migration runner |
| `.github/workflows/windows.yml` | Windows CI build/check/test workflow | Present with deterministic caching and checks |

## Frontend

| Path | Current responsibility |
|---|---|
| `src/main.tsx` | React entry point |
| `src/App.tsx` | App root shell coordinating navigation, state, and typed service delegates |
| `src/types/` | Strongly-typed domain and IPC contracts (`error.ts`, `persona.ts`, `fact.ts`, `survey.ts`, `dataManagement.ts`, `credential.ts`, `provider.ts`, `providerPool.ts`, `capture.ts`, `ocr.ts`, `opportunity.ts`, `fit.ts`, `attempt.ts`, `privacy.ts`) |
| `src/services/` | Typed IPC client and domain service wrappers (`ipcClient.ts`, `personaService.ts`, `factService.ts`, `surveyService.ts`, `dataManagementService.ts`, `credentialService.ts`, `providerService.ts`, `providerPoolService.ts`, `captureService.ts`, `ocrService.ts`, `opportunityService.ts`, `fitService.ts`, `attemptService.ts`, `privacyService.ts`) |
| `src/components/` | Modular layout and view components (`Sidebar.tsx`, `Topbar.tsx`, views: `HomeView`, `CopilotView` with Question Classifier, Constraints, Evidence Retrieval, Structured AI Recommendation with Abstention, Review Actions/History/Revision audit workflow, Screen Capture preview, OCR Review Modal, Survey Memory Banner, Recognition modal, Next Question Intelligence & Branch Prediction panel, Outbound Data Transparency modal, and Opportunity Capture, Criteria Review & Deterministic Persona Fit Evaluation (`OpportunityView.tsx`), `ProfilesView`, `HistoryView` with Historical Outcome Learning (SC-052) & Survey Sessions, `SettingsView` with Windows Credential Manager, Provider Adapters, Provider Pool, Privacy & Diagnostics tab (SC-060) & Export/Wipe UI) with full keyboard accessibility (Escape to dismiss, focus indicators) and complete Indonesian UI states (loading, empty, success, error, retry, offline) |
| `src/styles.css` | Global styles, WCAG 2.1 AA focus rings (`:focus-visible`), Windows 11 responsive breakpoints, high contrast / forced colors, and reduced motion accessibility |
| `tests/` | Unit and integration tests for frontend error handling, types, data management, credential lifecycle, provider adapters, provider pool, survey sessions/questions, question classifier, evidence retrieval, structured AI answer engine, review actions & history audit, screen capture, OCR extraction, opportunity capture/criteria, deterministic persona fit, historical outcome learning, survey memory & recognition, next question intelligence, privacy/diagnostics contracts, and UI quality & accessibility (`error_normalization.test.ts`, `persona_lifecycle.test.ts`, `fact_lifecycle.test.ts`, `data_management.test.ts`, `credential_management.test.ts`, `provider_configuration.test.ts`, `provider_pool.test.ts`, `survey_session.test.ts`, `question_classifier.test.ts`, `evidence_retrieval.test.ts`, `ai_answer_engine.test.ts`, `review_actions_and_history.test.ts`, `capture.test.ts`, `ocr.test.ts`, `opportunity.test.ts`, `survey_fit.test.ts`, `survey_attempt.test.ts`, `survey_memory.test.ts`, `next_question_intelligence.test.ts`, `privacy_and_diagnostics.test.ts`, `ui_quality_and_accessibility.test.ts`) |

All planned views and controls are fully implemented and wired to backend domain services. In compliance with project boundaries, no browser automation, auto-fill, auto-submit, or points tracking controls exist.

## Native backend

| Path | Current responsibility |
|---|---|
| `src-tauri/src/main.rs` | Windows/Tauri process entry |
| `src-tauri/src/lib.rs` | Tauri setup, state management (`CredentialStoreState`, `HttpTransportState`, `ProviderPoolState`, `CaptureEngineState`, `OcrEngineState`), and command registration |
| `src-tauri/src/commands/` | Thin Tauri command handlers (`app.rs`, `persona.rs`, `fact.rs`, `survey.rs`, `data_management.rs`, `credential.rs`, `provider.rs`, `provider_pool.rs`, `capture.rs`, `ocr.rs`, `opportunity.rs`, `survey_fit.rs`, `survey_attempt.rs`, `privacy.rs`) without business logic or SQL |
| `src-tauri/src/domain/` | Core domain entities, validation, error types, NQI / Survey Memory, Export/Wipe, Credential, Endpoint validation, Provider contracts, Provider Pool, Question Classifier, Evidence Retrieval, Answer Engine, Screen Capture, OCR, Opportunity/Criteria, Deterministic Fit, Historical Outcome Learning, and Privacy/Diagnostics (`error.rs`, `persona.rs`, `fact.rs`, `survey.rs`, `export.rs`, `credential.rs`, `endpoint_validator.rs`, `provider.rs`, `pool.rs`, `classifier.rs`, `evidence.rs`, `answer_engine.rs`, `capture.rs`, `ocr.rs`, `opportunity.rs`, `fit.rs`, `attempt.rs`, `privacy.rs`) |
| `src-tauri/src/services/` | Service layer containing business logic and transactions (`persona_service.rs`, `fact_service.rs`, `survey_service.rs`, `survey_recognition_service.rs`, `next_question_service.rs`, `export_service.rs`, `deletion_service.rs`, `credential_service.rs`, `provider_service.rs`, `provider_pool.rs`, `question_classifier.rs`, `evidence_engine.rs`, `answer_engine/`, `capture_service.rs`, `layout_reconstructor.rs`, `ocr_service.rs`, `criteria_extractor.rs`, `opportunity_service.rs`, `fit_evaluator.rs`, `fit_service.rs`, `outcome_learning_service.rs`, `privacy_service.rs`) |
| `src-tauri/src/infrastructure/` | Native OS infrastructure adapters (`credentials/windows.rs`, `credentials/in_memory.rs`, `adapters/gemini.rs`, `adapters/openai_compatible.rs`, `http/mod.rs`, `capture/windows.rs`, `capture/in_memory.rs`, `capture/codec.rs`, `ocr/windows.rs`, `ocr/fixture.rs`, `ocr/composite.rs`) |
| `src-tauri/src/repository/` | Parameterized SQLite repository layer with strict persona scoping, opaque credential metadata, provider configs, pool settings/health, survey sessions/questions/memory/predictions/branch transitions, answer recommendations, OCR extractions, opportunities/criteria, survey fit results, survey attempts/outcome events, and privacy settings/diagnostic logs (`persona_repo.rs`, `fact_repo.rs`, `survey_repo.rs`, `export_repo.rs`, `deletion_repo.rs`, `credential_repo.rs`, `provider_repo.rs`, `pool_repo.rs`, `answer_repo.rs`, `ocr_repo.rs`, `opportunity_repo.rs`, `fit_repo.rs`, `attempt_repo.rs`, `privacy_repo.rs`) |
| `src-tauri/src/migrations/` | Sequential, versioned, transactional migration runner and integrity verification (`mod.rs`, migrations 0001 through 0013) |
| `src-tauri/tests/` | Automated integration tests for migration runner (`migration_tests.rs`), persona lifecycle (`persona_lifecycle_tests.rs`), fact/source lifecycle (`fact_lifecycle_tests.rs`), data export/deletion (`export_and_deletion_tests.rs`), credential lifecycle (`credential_lifecycle_tests.rs`), provider adapters (`provider_adapter_tests.rs`), provider pool (`provider_pool_tests.rs`), survey sessions/questions (`survey_session_tests.rs`), question classifier (`question_classifier_tests.rs`), evidence retrieval & consistency engine (`evidence_retrieval_tests.rs`), AI answer engine (`ai_answer_engine_tests.rs`), review actions & history audit (`review_actions_and_history_tests.rs`), comprehensive end-to-end P3 workflow (`p3_comprehensive_integration_tests.rs`), Windows screen capture (`capture_tests.rs`), OCR extraction / layout reconstruction (`ocr_tests.rs`), opportunity capture / criteria extraction (`opportunity_tests.rs`), deterministic survey fit (`survey_fit_tests.rs`), historical outcome learning (`survey_attempt_tests.rs`), survey memory & recognition (`survey_memory_tests.rs`), next question intelligence / branch prediction (`next_question_intelligence_tests.rs`), and privacy/diagnostics/redaction/retention (`privacy_and_diagnostics_tests.rs`) |
| `src-tauri/src/db.rs` | SQLite connection initialization, pragmas, and migration execution |
| `src-tauri/migrations/` | Versioned SQL migrations (`0001_initial.sql` through `0013_privacy_settings_and_diagnostics.sql`) |
| `src-tauri/tauri.conf.json` | Tauri app, window, CSP, and bundle configuration |
| `src-tauri/capabilities/default.json` | Minimal Tauri capability allowlist |
| `src-tauri/Cargo.toml` | Rust dependencies |

Migration framework is implemented via `src-tauri/src/migrations/` and invoked sequentially by `Database::initialize` with baseline adoption and transactional rollback.

## Product documentation

- `docs/01-product/PRODUCT_DEFINITION.md`: authoritative product scope.
- `docs/01-product/PRODUCT_REQUIREMENTS.md`: MUST/MUST NOT requirements.
- `docs/01-product/FEATURE_SPECIFICATION.md`: feature behavior.
- `docs/01-product/USER_FLOWS.md`: user journeys.
- `docs/01-product/SCREEN_INTELLIGENCE.md`: capture/OCR user behavior.
- `docs/01-product/MULTI_PERSONA_SPECIFICATION.md`: persona isolation and truthfulness.
- `docs/01-product/SURVEY_FIT_LEARNING.md`: deterministic fit and outcome learning.
- `docs/01-product/EARNINGS_INTELLIGENCE.md`: explicitly out of scope for v1.0.

## Technical documentation

- `docs/02-architecture/`: system, schema, security, provider, capture, IPC, and adapter contracts.
- `docs/03-ai-engine/`: prompt, classifier, answer, memory, consistency, evaluation, and structured output.
- `docs/04-design/`: UX, design system, desktop layout.
- `docs/05-development/`: roadmap, implementation plan, coding and testing standards.
- `docs/06-operations/`: installation, configuration, privacy draft, troubleshooting.
- `docs/07-decisions/`: architecture decision records.
- `docs/08-agent-handoff/`: implementation handoff, backlog, acceptance tests, and unresolved decisions.

## Key implementation entry points

1. Start at `docs/08-agent-handoff/IMPLEMENTATION_BACKLOG.md`.
2. For P0, inspect `package.json`, `src-tauri/Cargo.toml`, `.github/workflows/windows.yml`, `src-tauri/src/db.rs`, and `src-tauri/src/lib.rs`.
3. For persona work, inspect the SQL tables `personas`, `profile_facts`, `fact_sources`, and current Tauri commands.
4. For provider work, follow `PROVIDER_ADAPTER_CONTRACT.md` and `API_PROVIDER_POOLING.md`.
5. For capture work, follow `SCREEN_INTELLIGENCE_TECHNICAL_SPEC.md`.
6. For answer generation, follow `STRUCTURED_OUTPUT_SCHEMA.md`, `AI_ARCHITECTURE.md`, and `ANSWER_ENGINE.md`.

Do not assume a document implies its corresponding feature exists in code.
