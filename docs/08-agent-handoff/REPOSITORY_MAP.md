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
| `src/types/` | Strongly-typed domain and IPC contracts (`error.ts`, `persona.ts`, `fact.ts`, `survey.ts`) |
| `src/services/` | Typed IPC client and domain service wrappers (`ipcClient.ts`, `personaService.ts`, `factService.ts`) |
| `src/components/` | Modular layout and view components (`Sidebar.tsx`, `Topbar.tsx`, views: `HomeView`, `CopilotView`, `ProfilesView`, `HistoryView`, `SettingsView`) |
| `src/styles.css` | Current global styles |
| `tests/` | Unit and integration tests for frontend error handling and types |

The current UI still contains placeholder screens and disabled screenshot/analysis actions. Replace placeholders only as their corresponding backlog items are implemented. Do not present a disabled control as working functionality.

## Native backend

| Path | Current responsibility |
|---|---|
| `src-tauri/src/main.rs` | Windows/Tauri process entry |
| `src-tauri/src/lib.rs` | Tauri setup, state management, and command registration |
| `src-tauri/src/commands/` | Thin Tauri command handlers (`app.rs`, `persona.rs`, `fact.rs`) without business logic or SQL |
| `src-tauri/src/domain/` | Core domain entities, validation, error types, and NQI / Survey Memory contracts (`error.rs`, `persona.rs`, `fact.rs`, `survey.rs`) |
| `src-tauri/src/services/` | Service layer containing business logic and transactions (`persona_service.rs`, `fact_service.rs`) |
| `src-tauri/src/repository/` | Parameterized SQLite repository layer with strict persona scoping (`persona_repo.rs`, `fact_repo.rs`) |
| `src-tauri/src/migrations/` | Sequential, versioned, transactional migration runner and integrity verification (`mod.rs`) |
| `src-tauri/tests/` | Automated integration tests for migration runner (`migration_tests.rs`), persona lifecycle (`persona_lifecycle_tests.rs`), and fact/source lifecycle (`fact_lifecycle_tests.rs`) |
| `src-tauri/src/db.rs` | SQLite connection initialization, pragmas, and migration execution |
| `src-tauri/migrations/` | Versioned SQL migrations (`0001_initial.sql`, `0002_typed_facts_and_history.sql`) |
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
