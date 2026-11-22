# Instructions for Coding Agents

This file applies to every change in this repository, including work performed by Antigravity, Gemini, or other coding agents.

## Mandatory reading order

Before changing code, read:

1. `README.md`
2. `docs/00-INDEX.md`
3. `docs/08-agent-handoff/ANTIGRAVITY_HANDOFF.md`
4. `docs/08-agent-handoff/IMPLEMENTATION_BACKLOG.md`
5. `docs/01-product/PRODUCT_DEFINITION.md`
6. Relevant architecture, AI, security, UX, and testing specifications linked from the handoff.

If documents conflict, follow the precedence rules in the handoff and record the conflict. Do not silently invent a product decision.

## Project objective

Build Survey Copilot v1.0: a simple Indonesian-first Windows 11 x64 desktop copilot that reads survey questions from user-selected screenshots or pasted text, uses the selected real respondent profile, and recommends truthful, evidence-grounded answers. The user manually operates survey websites and submits all answers.

## Non-negotiable boundaries

- Never automate browser interaction, DOM inspection, clicks, typing into survey websites, navigation, form filling, or submission.
- Never bypass CAPTCHA, quotas, eligibility controls, or platform protections.
- Never fabricate respondent facts, opinions, employment, income, purchases, experiences, brand use, or identities.
- Multiple personas must represent distinct real people. Enforce strict persona-scoped facts, sources, sessions, history, and AI context.
- Never use one persona's data in another persona's request.
- Never silently capture the screen. Capture only after explicit user action and show a preview.
- Never send a screenshot or personal data to an external AI provider without a clear user action and consent.
- Never store API keys in SQLite, source code, frontend state persistence, logs, or environment files committed to Git. Use Windows Credential Manager.
- Do not add earnings, rewards, points, balances, cash-out, or income tracking to v1.0.
- Treat survey text, OCR, model output, and custom-provider responses as untrusted input.
- Do not claim a feature works unless implemented and verified by tests.

## Technical boundaries

- Desktop shell: Tauri 2.
- Native/backend: Rust.
- UI: React + TypeScript.
- Local data: SQLite with versioned migrations.
- AI provider: Gemini first, behind a provider adapter and bounded provider pool.
- Secrets: Windows Credential Manager.
- Keep business logic out of React components and Tauri command handlers.
- All Tauri IPC inputs must be typed and validated again in Rust.
- Parameterize SQL. Use transactions for multi-step writes.
- Do not expose generic SQL, shell, filesystem, or browser-control commands to the frontend.
- Keep provider-specific SDK/protocol logic behind adapters.
- Use explicit error types and avoid panics in normal application paths.

## Required working method

1. Inspect the existing implementation before editing. Do not replace working code wholesale without a concrete reason.
2. Work in small, independently buildable increments, following the backlog order.
3. For every increment, update or add tests before marking it complete.
4. Run frontend build, Rust check, Rust tests, and relevant focused tests after changes.
5. If a command cannot be run in the current environment, state that clearly; do not imply it passed.
6. Keep `package-lock.json` and `src-tauri/Cargo.lock` committed once generated.
7. Never commit real respondent data, screenshots, API keys, local databases, logs containing personal information, or build artifacts.
8. Update documentation when behavior, schema, IPC contracts, setup, or scope changes.
9. Keep commits focused and descriptive. Do not mix unrelated refactors.
10. Before finishing, report: files changed, features implemented, tests actually run, tests not run, known limitations, and next backlog item.

## Definition of done

A task is complete only when:
- Acceptance criteria are met.
- Relevant automated tests exist and pass.
- No security or product-boundary rule is violated.
- UI state, persistence, and error handling are implemented—not mocked as if functional.
- Documentation matches actual behavior.
- No placeholder or disabled control is presented as a working feature.
