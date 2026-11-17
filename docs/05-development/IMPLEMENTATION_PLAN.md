# Implementation Plan

## Phase 0 — Repository foundation
Add app scaffold and lockfile; configure formatting, linting, type checking, unit tests, and CI; establish Tauri capability policy and Rust error conventions; add SQLite migration runner and repository interfaces.

## Phase 1 — Domain and persistence
Implement user, profile fact, source, session, question, recommendation, and history models. Add repository and migration tests. Implement export/delete with referential integrity.

## Phase 2 — Core UX
Build navigation, Digital Persona, Survey Workspace, History, and Settings. Add keyboard/accessibility checks. Keep workflows local until provider configuration is complete.

## Phase 3 — AI pipeline
Implement provider interface and evaluated adapter. Implement classifier/constraint extraction and strict output schema. Implement retrieval and consistency checks before model invocation. Validate every source reference after response.

## Phase 4 — Security and release
Integrate OS credential store. Add data-minimization disclosure and redacted diagnostics. Conduct prompt-injection/privacy tests. Build release artifacts and verify clean-machine installation.

## Definition of done
A feature is complete when requirements are documented, implementation is typed and tested, errors are handled, privacy implications reviewed, accessibility considered, and user-facing documentation updated.
