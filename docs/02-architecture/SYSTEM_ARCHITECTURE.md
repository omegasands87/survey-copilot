# System Architecture

**Status:** Proposed architecture for the MVP.

## 1. Architectural style
Local-first desktop application with a narrow native boundary and a modular TypeScript frontend. The desktop process owns filesystem/database access and secrets; the UI communicates through typed commands. AI calls are explicit network operations.

## 2. Technology baseline
- Desktop runtime: Tauri 2.
- Native layer: Rust.
- UI: React + TypeScript.
- Styling/components: Tailwind CSS and shadcn/ui, subject to accessibility review.
- Persistence: SQLite with migrations.
- Validation: Zod in the UI/service boundary and Rust-side validation for native commands.
- State: Zustand for local UI state; TanStack Query for asynchronous operations where useful.
- Search: SQLite FTS5 if local search requirements justify it.

See [technology stack](TECH_STACK.md).

## 3. Logical components
1. **Presentation:** Dashboard, Persona, Workspace, Recommendation Review, History, Settings.
2. **Application services:** session orchestration, profile management, answer workflow, export/delete.
3. **Domain:** fact, question, recommendation, evidence, and consistency models.
4. **Infrastructure:** SQLite repositories, migration runner, credential-store adapter, AI provider adapter, logging.
5. **Native boundary:** allowlisted Tauri commands with typed inputs and outputs.

## 4. AI request lifecycle
Question input → normalize and classify → extract constraints → retrieve relevant facts → run consistency checks → build minimized context → provider request → parse and validate structured output → validate source IDs locally → present evidence → user action → persist outcome.

## 5. Trust boundaries
- Survey text is untrusted external content.
- Model output is untrusted until validated.
- Provider network is outside the local trust boundary.
- API keys are secrets and never enter frontend logs or persisted plain-text settings.
- Tauri commands are privileged and must be allowlisted and input-validated.

## 6. Failure behavior
- Database unavailable: do not silently fall back to volatile writes; explain and preserve recoverable input.
- Provider unavailable: retain question locally and show retry/manual mode.
- Invalid model output: reject it, log a redacted diagnostic, and offer retry or abstention.
- Migration failure: back up database before migration and provide a recovery path.
- Missing credential: prompt user to configure it; never ask them to paste secrets into survey content.

## 7. Suggested repository layout
```
src/
  app/
  components/
  features/
    persona/
    survey-workspace/
    answer-copilot/
    history/
    settings/
  domain/
  services/
  lib/
src-tauri/
  src/
  capabilities/
  migrations/
docs/
```

## 8. Architectural constraints
- No browser automation or hidden form interaction.
- UI must not access SQLite directly.
- Provider-specific code stays behind an adapter interface.
- Repositories own SQL; domain services own business rules.
- No AI output may create or update a confirmed profile fact without user review.
