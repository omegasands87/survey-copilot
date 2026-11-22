# Implementation Plan

**Target:** Survey Copilot v1.0, Windows 11 x64  
**Current state:** Starter implementation exists; core product capabilities remain incomplete.  
**Execution authority:** `docs/08-agent-handoff/IMPLEMENTATION_BACKLOG.md`

## Implementation sequence

The detailed, testable tasks and acceptance conditions are maintained in the Antigravity backlog. Do not duplicate task state in this file. The required sequence is:

1. **Foundation:** reproducible builds, lockfiles, CI, migration runner, domain/service/repository layers, typed IPC.
2. **Persona data:** persona lifecycle, typed facts, source provenance, verification, sensitivity, strict isolation, export/delete.
3. **Provider infrastructure:** Windows Credential Manager, provider CRUD, Gemini adapter, bounded pool, custom compatible adapters.
4. **Answer workflow:** sessions, manual question input, classification, evidence retrieval, consistency, structured AI answer, user review/history.
5. **Screen Intelligence:** Windows capture, OCR, layout reconstruction, preview, correction, explicit confirmation.
6. **Survey Fit:** visible opportunity criteria, deterministic matching, persona-scoped outcome observations, validated learning.
7. **Release hardening:** privacy, settings, accessibility, diagnostics, installer, clean-machine tests.

## Working rules

- Implement small vertical slices in backlog order.
- A document, placeholder, disabled button, or mock is not a completed feature.
- Every feature needs automated tests and relevant manual Windows acceptance evidence.
- Keep product boundaries and security invariants in `AGENTS.md` and the Antigravity handoff.
- Update the backlog only after acceptance evidence exists.

## Release gates

- No browser automation, autofill, auto-click, navigation, or auto-submit.
- No fabricated facts, fictional personas, or cross-persona leakage.
- No plaintext API keys or silent capture.
- No unbounded retry or quota evasion by key rotation.
- No earnings/reward/balance/income tracking in v1.0.
- No release claim without successful Windows 11 x64 build and install verification.
