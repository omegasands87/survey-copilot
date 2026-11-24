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

### Browser UI-only preview

The browser preview is for visual and interaction review only. It uses synthetic data and mock state; it does not connect to Tauri IPC, SQLite, Windows Credential Manager, screen capture, OCR, or AI providers.

```powershell
npm ci
npm run preview:ui
```

Open `http://127.0.0.1:4173/?preview=1`. Changes made in the preview are held in memory and disappear when the page is refreshed. Do not enter real profile data, screenshots, credentials, or survey answers.

See [Stage 2 UI-only Preview](docs/09-audit/STAGE_2_UI_ONLY_PREVIEW.md) for the isolation contract.

### Dark Bento and profile management

The active desktop UI and browser preview share the Dark Bento theme. Profile management includes blank field templates and a local JSON import flow for a single-persona Survey Copilot export. Templates never prefill personal values. JSON import creates a new inactive profile and imports its facts and sources only; sessions, answers, screenshots, provider settings, and credentials are excluded.

See [Stage 3 — Dark Bento and Profile Import](docs/09-audit/STAGE_3_DARK_BENTO_PROFILE_UX.md) for supported format, limits, safety rules, and repository/device validation scope.

### Capture-to-OCR workflow

A deliberate region/window capture now starts local Windows OCR automatically. The captured image is previewed, the extraction is presented for human review/correction, and no external AI analysis starts automatically. Retry and manual text entry remain available when local OCR fails.

See [Stage 4 — Capture and Automatic Local OCR](docs/09-audit/STAGE_4_CAPTURE_AUTO_LOCAL_OCR.md) for the workflow and privacy boundary.

### Windows installer and uninstall

The Windows x64 NSIS package is built by the **Windows NSIS installer package** GitHub Actions workflow. Download the `Survey-Copilot-NSIS-<commit-sha>` artifact from the relevant Actions run. Verify the installer against `SHA256SUMS.txt` before running it.

The current-user installer adds both the app and **Uninstall Survey Copilot** shortcuts to the Start Menu. Windows Settings → Apps → Installed apps is also an uninstall entry point. The NSIS uninstaller offers a **Delete app data** choice; leave it unchecked to retain the local database. Windows Credential Manager secrets are separate and are not removed by deleting the app data folder.

The CI package is a 30-day workflow artifact, not a GitHub Release. Because this repository is public, treat the artifact as accessible to repository readers rather than as private distribution. It contains only an unsigned installer and verification manifests; do not include user data, databases, screenshots, API keys or secrets. Do not distribute the unsigned package as an official public release. Clean install, upgrade, uninstall, database retention, credential behavior and publisher trust remain DEVICE TEST REQUIRED until verified on Windows 11 x64.

See [Stage 5 — Windows Installer and Uninstall](docs/09-audit/STAGE_5_WINDOWS_PACKAGING_UNINSTALL.md), [Installation](docs/06-operations/INSTALLATION.md), and [Backup & Restore](docs/06-operations/BACKUP_AND_RESTORE.md).

Build locally (developer environment):

```powershell
npm run tauri build
```

## Status

**Current version:** v0.1.0-beta (private testing; not production-approved)

The audit snapshot dated 24 November 2026 verified Windows GitHub Actions run #60: 130 frontend tests and 221 Rust tests, in addition to frontend build, Rust formatting, and compile checks. These are historical automated results, not proof that the subsequent Stage 1–5 branch changes have passed CI or that installer/device behavior is accepted. The audit identified Critical/High security and data-integrity findings; use the current PR checks and the manual device checklist for later verification.

At the time of the 24 November audit, no installer was present on `main` and no GitHub Release asset was available. Stage 5 adds a Windows NSIS CI packaging workflow and Start Menu uninstall shortcut on the implementation branch; that work is still subject to CI and Windows device acceptance. No public installer or release is claimed. See the [audit report](docs/09-audit/AUDIT_COMPLETENESS_SECURITY_RELEASE_2026-11-24.md) and [installation/UI feedback](docs/09-audit/INSTALLATION_AND_UI_UX_FEEDBACK_2026-11-24.md).

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
