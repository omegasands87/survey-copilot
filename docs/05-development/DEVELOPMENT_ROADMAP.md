# Development Roadmap

**Planning estimate:** 10–12 weeks after implementation begins; validate after engineering setup. Not a delivery commitment.

| Stage | Focus | Deliverable |
|---|---|---|
| 1 | Foundation | Tauri 2 shell, CI, SQLite, migrations, Windows credential storage |
| 2 | Personas | Multiple isolated profiles, facts, sources, verification, export/delete |
| 3 | Screen Intelligence | Region/window capture, OCR, correction, manual fallback |
| 4 | Provider Hub | Gemini adapter, API key pool, failover, retry, custom compatible endpoints |
| 5 | Answer Copilot | Classification, retrieval, consistency, structured recommendation, history |
| 6 | Survey Fit | Visible criteria extraction and per-persona matching |
| 7 | Release | Indonesian UI, security, accessibility, installer, clean-machine tests |

## Milestones

- M1: App launches and local data persists.
- M2: Persona isolation tests pass.
- M3: User can capture and verify a question.
- M4: Gemini pool and custom provider configuration work.
- M5: Recommendation is evidence-grounded and reviewable.
- M6: Survey fit results explain each criterion.
- M7: Windows 11 x64 release candidate passes acceptance tests.

## Risks

Windows capture APIs, OCR language quality, Windows Credential Manager integration, provider quota behavior, custom endpoint compatibility, installer signing, and privacy review may affect the schedule. Do not shorten security or data-integrity work to meet an estimated date.
