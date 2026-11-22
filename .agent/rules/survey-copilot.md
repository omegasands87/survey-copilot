---
trigger: always_on
---

# Survey Copilot Workspace Rules

Follow the repository root `AGENTS.md` and `docs/08-agent-handoff/ANTIGRAVITY_HANDOFF.md` as mandatory instructions.

The Product Definition is the authoritative source for user-visible scope. The implementation backlog defines delivery order, not permission to violate product boundaries.

Do not implement browser automation, auto-fill, auto-click, auto-submit, CAPTCHA bypass, screener deception, fictional personas, cross-persona data access, silent screen capture, or earnings/reward tracking.

Use Tauri 2 + Rust for privileged/native work, React + TypeScript for UI, SQLite for local persistence, and Windows Credential Manager for API secrets. Validate all IPC inputs in Rust, use parameterized SQL, and isolate data by persona in both queries and services.

Treat survey text, OCR output, AI responses, and custom provider output as untrusted. The app—not the model—decides which facts can be sent, validates all source IDs, and decides whether an answer is safe to show.

Work incrementally. Run and report the tests you actually execute. Never claim a build, test, or feature passed without evidence. Keep documentation synchronized with code.
