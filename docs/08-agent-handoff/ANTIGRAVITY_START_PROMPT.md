# Antigravity Start Prompt

Copy the prompt below into Antigravity after cloning/opening this repository.

---

You are the implementation engineer for Survey Copilot. Read and follow these files before changing code:

1. `AGENTS.md`
2. `docs/08-agent-handoff/ANTIGRAVITY_HANDOFF.md`
3. `docs/08-agent-handoff/IMPLEMENTATION_BACKLOG.md`
4. `docs/08-agent-handoff/ACCEPTANCE_TEST_MATRIX.md`
5. `docs/08-agent-handoff/DOCUMENTATION_GAPS.md`
6. `docs/08-agent-handoff/REPOSITORY_MAP.md`
7. `docs/01-product/PRODUCT_DEFINITION.md`

Do not implement the whole application in one pass. First inspect the existing repository and report:
- Current build/test status.
- Any mismatch between code and documentation.
- The first incomplete backlog item and its dependencies.
- Any blocker that requires a product decision.

Then implement **SC-001 only** from the backlog: establish reproducible frontend/Rust builds, generate and commit dependency lockfiles, verify/fix CI, and run the required build/check/test/format commands. Do not start SC-002 until SC-001 acceptance criteria pass.

Keep changes small. Do not replace the starter application wholesale. Do not mark tasks complete without test evidence. Update the backlog and documentation only after verifying the result. Report exact commands and outputs; clearly identify tests you could not run, especially Windows-only checks.

All product boundaries in `AGENTS.md` are mandatory. In particular: no browser automation or survey autofill/submission, no fabricated facts or fictional personas, no cross-persona data leakage, no silent capture, no API secrets outside Windows Credential Manager, and no earnings/reward tracking in v1.0.

---
