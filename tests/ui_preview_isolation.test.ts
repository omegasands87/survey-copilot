import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const previewSource = readFileSync(
  new URL("../src/preview/PreviewApp.tsx", import.meta.url),
  "utf8",
);
const bootstrapSource = readFileSync(
  new URL("../src/main.tsx", import.meta.url),
  "utf8",
);
const sharedViewSources = [
  "../src/components/Sidebar.tsx",
  "../src/components/Topbar.tsx",
  "../src/components/views/HomeView.tsx",
  "../src/components/views/ProfilesView.tsx",
].map((path) => readFileSync(new URL(path, import.meta.url), "utf8"));

test("UI preview module does not import desktop services or Tauri IPC", () => {
  assert.doesNotMatch(previewSource, /@tauri-apps/);
  assert.doesNotMatch(previewSource, /from\s+["'][^"']*(?:services|ipcClient|repository|infrastructure)[^"']*["']/);
  for (const source of sharedViewSources) {
    assert.doesNotMatch(source, /@tauri-apps/);
    assert.doesNotMatch(source, /from\s+["'][^"']*(?:services|ipcClient|repository|infrastructure)[^"']*["']/);
  }
});

test("UI preview has no network or persistent browser storage APIs", () => {
  assert.doesNotMatch(
    previewSource,
    /\b(?:fetch|XMLHttpRequest|WebSocket|localStorage|sessionStorage|indexedDB)\b/,
  );
});

test("browser preview requires an explicit preview=1 query parameter", () => {
  assert.match(
    bootstrapSource,
    /new URLSearchParams\(window\.location\.search\)\.get\("preview"\) === "1"/,
  );
  assert.match(bootstrapSource, /await import\("\.\/preview\/PreviewApp"\)/);
  assert.match(bootstrapSource, /Survey Copilot — UI Preview/);
});

test("preview visibly marks demo-only behavior and lack of desktop connectivity", () => {
  assert.match(previewSource, /UI PREVIEW/);
  assert.match(previewSource, /tanpa backend desktop/);
  assert.match(previewSource, /jangan masukkan data pribadi\/kredensial/);
  assert.match(previewSource, /perubahan hilang saat refresh/);
});
