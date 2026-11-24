# Stage 2 — Browser UI-only Preview

**Scope:** visual and interaction preview only. The Windows-installed Tauri application remains the only runtime for real product functionality.

## Run locally

1. Install project dependencies with `npm ci`.
2. Start the UI preview with `npm run preview:ui`.
3. Open `http://127.0.0.1:4173/?preview=1`.

The query parameter `preview=1` selects the isolated preview entry. Without it, the normal Tauri desktop frontend entry is loaded.

## Safety and isolation contract

- The preview imports no Tauri IPC client, domain service, repository, database adapter, credential service, capture engine, OCR service, or AI provider.
- It uses synthetic example personas, synthetic facts, and synthetic session data only.
- Profile interactions modify React component state only. Refreshing the browser clears those changes.
- The Copilot, History, and Settings preview pages demonstrate visual states and local interactions only.
- Screenshot/capture buttons are simulated controls. They never request browser screen capture or desktop capture permissions.
- Provider controls never contain an API key and never send a request.
- No localStorage, IndexedDB, cookies, or remote persistence are used by the preview.
- No user profile, actual screenshot, credential, or real survey answer may be entered into this preview.
- The visible UI PREVIEW banner must remain present so screenshots cannot be mistaken for the installed application's real state.

## Included surfaces

- Shared desktop navigation shell, page header, and active persona selector.
- Existing HomeView and ProfilesView components with in-memory demo data.
- Copilot question-entry, options, profile context, evidence and sample-answer states.
- History summary, filter controls and session list.
- Settings category tabs and demonstrative controls.

## Out of scope for Stage 2

Dark Bento redesign, full responsive redesign of desktop workflows, profile templates/import, real capture/OCR, live AI, real settings, native database operations, and installer/uninstaller work remain in their separately locked stages.

## Repository validation

The preview is part of the normal TypeScript/Vite build. It must not weaken the Tauri CSP, add remote APIs, or add runtime dependencies.
