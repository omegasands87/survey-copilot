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

**Current version:** v0.1.0-beta (Private Testing Release)

Milestones SC-001 through SC-061 are fully implemented and verified with automated test suites (130/130 frontend tests passing, 23/23 CI pipeline checks green on Windows). Capabilities include:
- Multi-persona respondent profile lifecycle with strict cross-persona data isolation.
- Typed profile facts with source metadata, verification states, and sensitivity classifications.
- Local screen capture with explicit preview and local OCR layout reconstruction.
- Bounded Gemini API pooling with failover, retry budgets, and Windows Credential Manager integration.
- Evidence-grounded answer recommendation engine with question classification and abstention contracts.
- Survey memory, pattern recognition, and Next Question Intelligence (NQI).
- Survey opportunity capture, criteria review, deterministic persona fit evaluation, and historical outcome tracking.
- Outbound data transparency, per-request sensitivity/vision consent, and automatic diagnostic log redaction.
- Packaged Windows 11 x64 NSIS installer (`release-testing/Survey Copilot_0.1.0_x64-setup.exe`) for private manual verification. See [Release Notes](docs/05-development/RELEASE_NOTES_0.1.0_BETA.md) and [Manual Testing Guide](docs/05-development/MANUAL_TESTING_GUIDE.md).
