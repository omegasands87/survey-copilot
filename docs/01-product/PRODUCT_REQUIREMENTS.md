# Product Requirements — Survey Copilot v1.0

**Platform:** Windows 11 x64  
**Language:** Indonesian  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)

## 1. Outcome

A simple local desktop copilot that reads survey questions from screenshots or pasted text, uses the selected respondent persona to recommend truthful answers, evaluates explicit survey fit, and supports configurable pooled AI providers. The user manually completes every survey interaction.

## 2. Functional requirements

| ID | Requirement | Priority |
|---|---|---|
| PR-001 | Create and manage multiple real-persona profiles | MUST |
| PR-002 | Isolate facts, sources, preferences, and history by persona | MUST |
| PR-003 | Store source, date, verification, and sensitivity metadata for facts | MUST |
| PR-004 | Support confirmed, unverified, disputed, and outdated facts | MUST |
| PR-005 | Select and visibly identify the active persona for every analysis | MUST |
| PR-006 | Capture a user-selected screen region or application window | MUST |
| PR-007 | Extract visible prompt, instructions, options, scales, units, and matrix layout | MUST |
| PR-008 | Allow OCR correction and confirmation before analysis | MUST |
| PR-009 | Provide manual text entry as capture fallback | MUST |
| PR-010 | Explain and classify survey questions and answer formats | MUST |
| PR-011 | Retrieve only relevant eligible facts from the active persona | MUST |
| PR-012 | Detect stale, missing, and conflicting information | MUST |
| PR-013 | Return a structured recommendation or abstention | MUST |
| PR-014 | Display evidence, rationale, assumptions, and uncertainty | MUST |
| PR-015 | Validate every cited fact ID against the active request allowlist | MUST |
| PR-016 | Support edit, confirm, reject, copy, and regenerate actions | MUST |
| PR-017 | Preserve original AI output and user-edited answer history separately | MUST |
| PR-018 | Never silently promote an answer into a verified profile fact | MUST |
| PR-019 | Extract visible survey opportunity criteria from user-selected capture | MUST |
| PR-020 | Compare explicit criteria against each persona | MUST |
| PR-021 | Show match, mismatch, unknown, stale, and conflict with evidence | MUST |
| PR-022 | Separate deterministic eligibility matching from historical outcome learning | MUST |
| PR-023 | Configure Gemini as the default AI provider | MUST |
| PR-024 | Add and manage multiple API keys for the same provider | MUST |
| PR-025 | Automatically rotate/fail over on rate limits and retryable errors | MUST |
| PR-026 | Support adaptive, round-robin, weighted, and priority pool strategies | MUST |
| PR-027 | Configure retry count, timeout, cooldown, and cancellation | MUST |
| PR-028 | Add custom provider configurations, compatible endpoints, and model IDs | MUST |
| PR-029 | Store API secrets in Windows Credential Manager | MUST |
| PR-030 | Test, enable, disable, edit, and remove provider configurations | MUST |
| PR-031 | Export and fully delete local profile and history data | MUST |
| PR-032 | Remain usable for local data management without AI connectivity | MUST |
| PR-033 | Provide an Indonesian-first interface | MUST |
| PR-034 | Never automate survey clicks, filling, navigation, or submission | MUST NOT |
| PR-035 | Never fabricate personal facts or recommend deceptive screener answers | MUST NOT |
| PR-036 | Never capture the screen silently or in the background | MUST NOT |
| PR-037 | Do not implement earnings/reward/balance tracking in v1.0 | MUST NOT |

## 3. Non-functional requirements

| ID | Requirement |
|---|---|
| NFR-001 | SQLite changes use versioned, tested migrations |
| NFR-002 | Failed provider requests do not lose local question content |
| NFR-003 | API secrets never appear in logs, exports, frontend bundles, or plaintext settings |
| NFR-004 | Raw personal prompts and screenshots are not logged by default |
| NFR-005 | External AI context is minimized and disclosed before transmission |
| NFR-006 | Tauri commands use typed inputs, validation, and least privilege |
| NFR-007 | Primary workflows are keyboard accessible |
| NFR-008 | AI output is schema-validated before display |
| NFR-009 | Persona isolation is enforced in database queries and domain services |
| NFR-010 | App installs and runs on a clean Windows 11 x64 environment |

## 4. Release blockers

- Cross-persona data leakage.
- Any plaintext API key storage.
- Silent screenshot capture.
- AI-generated unsupported personal claims presented as facts.
- Automated survey interaction.
- Unbounded retry loops.
- Failure to preserve local input when providers are unavailable.
