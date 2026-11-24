import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const profileView = readFileSync(
  new URL("../src/components/views/ProfilesView.tsx", import.meta.url),
  "utf8",
);
const theme = readFileSync(
  new URL("../src/styles/dark-bento.css", import.meta.url),
  "utf8",
);
const mainEntry = readFileSync(
  new URL("../src/main.tsx", import.meta.url),
  "utf8",
);
const importService = readFileSync(
  new URL("../src-tauri/src/services/profile_import_service.rs", import.meta.url),
  "utf8",
);

test("profile templates prefill only field metadata and never respondent values", () => {
  assert.match(profileView, /PROFILE_FIELD_TEMPLATES/);
  assert.match(profileView, /setFactValue\(""/);
  assert.match(profileView, /Tanpa Nilai Demo|TANPA NILAI DEMO/);
  assert.doesNotMatch(profileView, /Penghasilan Bulanan/);
  assert.doesNotMatch(profileView, /option value="Finansial"/);
});

test("profile import requires a single-persona export and confirms a new profile", () => {
  assert.match(profileView, /exportScope !== "single_persona"/);
  assert.match(profileView, /Import sebagai profil baru/);
  assert.match(profileView, /data\.factSources/);
  assert.match(importService, /payload\.export_scope != "single_persona"/);
  assert.match(importService, /transaction\.commit\(\)/);
  assert.match(importService, /MAX_PROFILE_IMPORT_BYTES/);
});

test("profile import explicitly excludes unrelated account and survey data", () => {
  assert.match(profileView, /Sesi survei, jawaban, screenshot, riwayat, pengaturan provider, serta credential tidak diimpor/);
  assert.match(importService, /credentials are never imported|credential.*tidak pernah diimpor/i);
});

test("dark bento theme is shared by desktop and browser entry points", () => {
  assert.match(theme, /color-scheme:\s*dark/);
  assert.match(theme, /\.sidebar/);
  assert.match(theme, /\.profile-form/);
  assert.match(theme, /\.modal-dialog/);
  assert.match(theme, /@media \(max-width:\s*620px\)/);
  assert.match(mainEntry, /\.\/styles\/dark-bento\.css/);
});
