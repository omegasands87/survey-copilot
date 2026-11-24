import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const tauriConfig = JSON.parse(
  readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
) as {
  bundle?: {
    targets?: string[];
    windows?: {
      nsis?: {
        installMode?: string;
        startMenuFolder?: string;
        installerHooks?: string;
      };
    };
  };
};

const nsisHooks = readFileSync(
  new URL("../src-tauri/windows/nsis-hooks.nsh", import.meta.url),
  "utf8",
);
const packageWorkflow = readFileSync(
  new URL("../.github/workflows/package-windows.yml", import.meta.url),
  "utf8",
);
const validator = readFileSync(
  new URL("../scripts/validate-windows-installer.ps1", import.meta.url),
  "utf8",
);
const installGuide = readFileSync(
  new URL("../docs/06-operations/INSTALLATION.md", import.meta.url),
  "utf8",
);
const backupGuide = readFileSync(
  new URL("../docs/06-operations/BACKUP_AND_RESTORE.md", import.meta.url),
  "utf8",
);
const manualGuide = readFileSync(
  new URL("../docs/05-development/MANUAL_TESTING_GUIDE.md", import.meta.url),
  "utf8",
);

test("Tauri package remains a current-user x64 NSIS installer", () => {
  assert.ok(tauriConfig.bundle?.targets?.includes("nsis"));
  assert.equal(tauriConfig.bundle?.windows?.nsis?.installMode, "currentUser");
  assert.equal(tauriConfig.bundle?.windows?.nsis?.startMenuFolder, "Survey Copilot");
  assert.equal(tauriConfig.bundle?.windows?.nsis?.installerHooks, "windows/nsis-hooks.nsh");
});

test("NSIS hook adds a discoverable uninstall shortcut and removes only that shortcut", () => {
  assert.match(nsisHooks, /NSIS_HOOK_POSTINSTALL/);
  assert.match(nsisHooks, /CreateShortcut .*Uninstall Survey Copilot\.lnk.*uninstall\.exe/);
  assert.match(nsisHooks, /NSIS_HOOK_PREUNINSTALL/);
  assert.match(nsisHooks, /Delete .*Uninstall Survey Copilot\.lnk/);
  assert.doesNotMatch(nsisHooks, /RmDir\s+\/r|DeleteRegKey|cmdkey\s+\/delete/i);
});

test("Windows package workflow runs full gates and uploads a finite-retention CI artifact", () => {
  assert.match(packageWorkflow, /runs-on:\s*windows-latest/);
  assert.match(packageWorkflow, /npm ci/);
  assert.match(packageWorkflow, /npm run build/);
  assert.match(packageWorkflow, /npm test/);
  assert.match(packageWorkflow, /cargo fmt/);
  assert.match(packageWorkflow, /cargo check/);
  assert.match(packageWorkflow, /cargo test/);
  assert.match(packageWorkflow, /tauri .*build.*--ci.*--no-sign/s);
  assert.match(packageWorkflow, /validate-windows-installer\.ps1/);
  assert.match(packageWorkflow, /actions\/upload-artifact@v4/);
  assert.match(packageWorkflow, /retention-days:\s*30/);
  assert.doesNotMatch(packageWorkflow, /softprops\/action-gh-release|gh release create|create-release/i);
});

test("installer validation produces SHA-256 and signer metadata without publishing", () => {
  assert.match(validator, /Get-FileHash .*SHA256/);
  assert.match(validator, /SHA256SUMS\.txt/);
  assert.match(validator, /build-manifest\.json/);
  assert.match(validator, /Get-AuthenticodeSignature/);
  assert.match(validator, /currentUser/);
  assert.match(validator, /x64-setup/);
  assert.doesNotMatch(validator, /gh release create|softprops\/action-gh-release/i);
});

test("install documentation separates AppData deletion from Credential Manager", () => {
  assert.match(installGuide, /Windows Settings.*Apps.*Installed apps/s);
  assert.match(installGuide, /Uninstall Survey Copilot/);
  assert.match(installGuide, /Delete app data/);
  assert.match(installGuide, /Credential Manager/);
  assert.match(backupGuide, /TIDAK|Tidak dicentang|tidak dicentang/);
  assert.match(backupGuide, /Credential Manager/);
});

test("manual guide retains explicit Windows device acceptance gates", () => {
  assert.match(manualGuide, /DEVICE TEST REQUIRED/);
  assert.match(manualGuide, /Delete app data/);
  assert.match(manualGuide, /in-place/i);
  assert.match(manualGuide, /Start Menu/);
  assert.match(manualGuide, /Credential Manager/);
});
