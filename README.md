# Survey Copilot

Desktop AI copilot for understanding online survey questions and recommending truthful, profile-grounded answers.

- **Platform:** Windows 11 x64
- **Language:** Indonesian
- **Users:** One local operator managing multiple real respondent profiles
- **Input:** User-selected screenshot capture or pasted text
- **AI:** Gemini API pool with custom compatible provider/endpoint/model support
- **Use:** AI recommends; user manually fills and submits surveys
- **Business model:** Free personal use

## What it does

1. Reads a survey question from the screen or pasted text.
2. Understands the question and its answer format.
3. Uses the selected respondent profile and checks consistency.
4. Recommends an answer with evidence—or asks when information is missing.
5. Compares visible survey criteria against each persona.
6. Uses a pool of Gemini API keys and can fail over when a key is limited or unavailable.

## Product boundaries

- Multiple real-persona profiles are isolated.
- Recommendations must be truthful and evidence-grounded.
- Screen capture is user initiated and visible.
- The user reviews AI recommendations and performs all survey interactions.
- No browser automation, autofill, auto-click, auto-submit, CAPTCHA bypass, or screener deception.
- No earnings, reward, points, balance, or income tracking in v1.0.
- API secrets are stored in Windows Credential Manager.

## Documentation and coding-agent handoff

Start at [Documentation Index](docs/00-INDEX.md). Before asking Antigravity or another coding agent to modify the project, have it read [AGENTS.md](AGENTS.md), the [Antigravity Engineering Handoff](docs/08-agent-handoff/ANTIGRAVITY_HANDOFF.md), and the [start prompt](docs/08-agent-handoff/ANTIGRAVITY_START_PROMPT.md). The [Implementation Backlog](docs/08-agent-handoff/IMPLEMENTATION_BACKLOG.md) defines the ordered work and acceptance criteria. The [Product Definition](docs/01-product/PRODUCT_DEFINITION.md) is the authoritative user-visible scope.

## Technology direction

Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, local OCR, and a provider adapter architecture with Gemini first.

## Run locally

Prerequisites: Node.js 22+, Rust stable, Visual Studio C++ Build Tools, and WebView2 Runtime on Windows.

```powershell
npm install
npm run tauri dev
```

Build the Windows installer with:

```powershell
npm run tauri build
```

## Status

**Current version:** v0.1.0-beta (private testing; not production-approved)

The latest repository audit (24 November 2026) verified that Windows GitHub Actions run #60 passed: 130 frontend tests and 221 Rust tests, in addition to frontend build, Rust formatting, and compile checks. These automated results do not establish installer or runtime readiness. The audit identified Critical/High security and data-integrity findings, and the clean-install, launch, update, uninstall, and data-retention acceptance tests remain unverified.

The installer path previously referenced below is **not present in the inspected `main` tree**, and no GitHub Release asset was available at the time of the audit. Do not treat a packaged installer as currently downloadable from this repository. See the [audit report](docs/09-audit/AUDIT_COMPLETENESS_SECURITY_RELEASE_2026-11-24.md) and [installation/UI feedback](docs/09-audit/INSTALLATION_AND_UI_UX_FEEDBACK_2026-11-24.md) for evidence and follow-up requirements.

Implemented code and automated tests cover:
- Multi-persona respondent profile lifecycle with strict cross-persona data isolation.
- Typed profile facts with source metadata, verification states, and sensitivity classifications.
- Local screen capture with explicit preview and local OCR layout reconstruction.
- Bounded Gemini API pooling with failover, retry budgets, and Windows Credential Manager integration.
- Evidence-grounded answer recommendation engine with question classification and abstention contracts.
- Survey memory, pattern recognition, and Next Question Intelligence (NQI).
- Survey opportunity capture, criteria review, deterministic persona fit evaluation, and historical outcome tracking.
- Outbound data transparency, per-request sensitivity/vision consent, and automatic diagnostic log redaction.

These areas are not a production-readiness claim. See the audit report for partial milestones, known risks, and unverified Windows runtime/release checks.
