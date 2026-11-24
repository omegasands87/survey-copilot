# Stage 5 — Windows NSIS Installer, Uninstall Discovery, and Release Artifact

**Scope:** make the Windows x64 installer reproducible in CI, provide an obvious uninstall entry point, define data-retention behavior, and make the package verifiable. This stage does not publish a public release or assert clean-device acceptance.

## 1. Installer configuration

`src-tauri/tauri.conf.json` keeps the product identifier `com.surveycopilot.desktop`, NSIS target, x64 Windows bundle, and `installMode: currentUser`.

The NSIS configuration sets:
- Start Menu folder: `Survey Copilot`
- Installer hooks: `windows/nsis-hooks.nsh`
- Installer language: English

The NSIS post-install hook adds **Uninstall Survey Copilot** next to the app shortcut in the Start Menu. The post-uninstall hook removes that extra shortcut only after the uninstaller has completed; if the user cancels uninstall because the app is still running, the shortcut remains available. The hooks do not remove AppData, LocalAppData, or Windows Credential Manager entries themselves.

## 2. Reproducible package workflow

Workflow: `.github/workflows/package-windows.yml`

Triggers:
- Pull requests targeting `main`
- Manual workflow dispatch

The job runs on Windows x64 and:
1. Installs Node 22 and the stable Rust toolchain.
2. Installs frontend dependencies using `npm ci`.
3. Runs the production frontend build and frontend tests.
4. Runs Rust formatting, `cargo check`, and `cargo test`.
5. Builds the NSIS package with the Tauri CLI in CI mode and without code signing.
6. Requires exactly one x64 `*-setup.exe` package.
7. Validates the package filename against the configured version, package size, NSIS/current-user configuration, and Authenticode state.
8. Generates a SHA-256 manifest and build manifest.
9. Uploads the installer and manifests as a CI artifact with a 30-day retention period.

Artifact name: `Survey-Copilot-NSIS-<commit-sha>`

The workflow intentionally does **not** create a GitHub Release or tag a release. **The repository is public**, and GitHub documents that anyone with repository read access can list/get workflow artifacts; therefore treat this CI artifact as accessible to repository readers, not as a private distribution channel. The artifact must contain only the unsigned installer and non-sensitive verification manifests—never user data, databases, screenshots, API keys, or other secrets.

## 3. Integrity and publisher identity

`scripts/validate-windows-installer.ps1`:
- Requires the configured Tauri x64 NSIS filename and current-user mode.
- Requires a plausible installer size (greater than 1 MiB).
- Copies the generated installer to `release-artifacts/`.
- Produces a per-installer `.sha256`, `SHA256SUMS.txt`, and `build-manifest.json`.
- Records Authenticode status and signer where present.
- Rejects unexpected Authenticode states.

Current CI packaging uses `--no-sign`, so an unsigned internal artifact is expected. SHA-256 detects accidental/tampered file changes relative to the manifest; it does not prove publisher identity. A public release requires a trusted code-signing identity and independent verification.

## 4. Install and uninstall discovery

The current-user NSIS package is intended to install without Administrator elevation and to expose both Windows Installed Apps metadata and a Start Menu folder.

Expected uninstall entry points:
- Windows Settings → Apps → Installed apps → Survey Copilot → Uninstall
- Start Menu → Survey Copilot → Uninstall Survey Copilot

The Start Menu shortcut points directly to the generated NSIS `uninstall.exe`. The classic Control Panel listing is a secondary path and must be checked on the actual Windows versions being supported.

## 5. Data retention and credential separation

Tauri's NSIS uninstaller provides a **Delete app data** choice. When selected for a normal uninstall, Tauri's NSIS script removes the app's AppData and LocalAppData folders under the bundle identifier. Do not select it when the intention is only to remove program files while retaining local data.

The Credential Manager secret store is outside those app-data directories. Deleting the AppData folders must not be described as deleting Windows Credential Manager secrets. Credential cleanup must be performed through the app's credential-management flow or Windows Credential Manager separately.

Do not advertise “data guaranteed preserved” until clean install, upgrade, uninstall with/without the checkbox, and restore have been tested on a device.

## 6. Required device acceptance

All following checks remain **DEVICE TEST REQUIRED** until performed on a supported Windows 11 x64 device:
- Clean current-user install without Administrator/UAC.
- App startup and Start Menu shortcut verification.
- Presence in Windows Installed Apps and, where applicable, Control Panel Programs and Features.
- Uninstall shortcut opens the uninstaller.
- Cancel uninstall leaves the app installed.
- Uninstall without Delete app data preserves SQLite and allows a reinstall.
- Uninstall with Delete app data removes the app data folder as described.
- Credential Manager behavior is verified separately from AppData removal.
- In-place update preserves the intended local database and provider setup.
- Authenticode signature and publisher identity are checked before public release.

See [Manual Testing Guide](../05-development/MANUAL_TESTING_GUIDE.md) for the consolidated five-stage device checklist.
