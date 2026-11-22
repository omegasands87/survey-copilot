# Implementation Plan

**Target:** Survey Copilot v1.0, Windows 11 x64  
**Current state:** Documentation/specification only until implementation commits exist.

## Phase 1 — App foundation
- Bootstrap Tauri 2, Rust, React, TypeScript.
- Configure CI, lint, formatting, type checks, tests, and lockfiles.
- Implement typed Tauri IPC and capability allowlists.
- Add SQLite migrations, backup, and repository layer.
- Add Windows Credential Manager wrapper.

## Phase 2 — Persona
- Implement multiple real-persona CRUD.
- Implement persona-scoped facts, sources, preferences, verification, and sensitivity.
- Implement active persona selection.
- Enforce isolation in repository queries and service layer.
- Add data export and deletion.

## Phase 3 — Screen intelligence
- Implement region/window capture initiated by the user.
- Implement OCR and question/option layout extraction.
- Add preview, correction, uncertainty markers, and manual paste fallback.
- Add optional external vision with explicit consent.
- Verify temporary screenshot deletion.

## Phase 4 — Provider hub
- Define normalized provider interface and error taxonomy.
- Implement Gemini adapter.
- Implement multiple API keys, adaptive pool, round-robin, weighted, priority.
- Implement rate-limit cooldown, bounded retry, cancellation, and health.
- Implement custom compatible provider and endpoint configuration.
- Test secret storage and redaction.

## Phase 5 — Answer copilot
- Implement classifier and constraint extraction.
- Implement persona-specific fact retrieval.
- Implement consistency checks and abstention.
- Implement strict structured response and local evidence-ID validation.
- Implement edit, confirm, copy, reject, regenerate, and history.

## Phase 6 — Survey fit
- Extract visible opportunity criteria.
- Implement deterministic match/mismatch/unknown/stale/conflict checks.
- Add transparent fit explanation.
- Add historical outcome learning only where sufficient persona-specific evidence and uncertainty are available.

## Phase 7 — Release
- Complete Indonesian-first UI.
- Complete keyboard/accessibility review.
- Run security, privacy, migration, performance, and clean-machine tests.
- Build and test signed Windows x64 installer.
- Complete installation, configuration, and troubleshooting guides.

## Release gates
- No browser automation, autofill, auto-click, or auto-submit.
- No fabricated facts or fictional personas.
- No cross-persona leakage.
- No plaintext API keys.
- No silent capture.
- No unbounded retry.
- No earnings/reward tracking in v1.0.
- No feature may be described as shipped before implementation and verification.
