---
description: Implement the next Survey Copilot backlog item safely and verify it.
---

1. Read root `AGENTS.md`.
2. Read `docs/08-agent-handoff/ANTIGRAVITY_HANDOFF.md` and identify the first incomplete, unblocked item in `IMPLEMENTATION_BACKLOG.md`.
3. Inspect all relevant source files, tests, schema, and contracts before editing.
4. State the narrow scope and acceptance criteria in the working notes.
5. Implement the smallest coherent vertical slice. Do not create mock functionality that appears production-ready.
6. Add unit, repository, integration, or UI tests appropriate to the slice.
7. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
   - `cargo test --manifest-path src-tauri/Cargo.toml`
   - Any focused test command introduced by the project.
8. Fix failures caused by the change. Do not suppress errors or weaken tests to get a green result.
9. Update relevant documentation and mark backlog items only when acceptance evidence exists.
10. Summarize files changed, exact commands/results, limitations, and the next unblocked item.
