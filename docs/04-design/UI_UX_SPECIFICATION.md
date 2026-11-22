# UI/UX Specification

## 1. Product experience

The application should feel simple: choose a profile, capture a question, get a recommendation. Advanced provider-pool configuration stays in Settings and should not interrupt ordinary use.

## 2. Main navigation

Use four primary destinations:
1. Beranda
2. Profil
3. Copilot
4. Riwayat

Settings is accessed from a persistent gear icon. Do not add an earnings dashboard.

## 3. Beranda

Show:
- Active persona selector.
- Primary action: Baca Pertanyaan.
- Secondary action: Tempel Teks.
- Recent sessions.
- Provider status in a compact indicator.

Avoid analytics dashboards and decorative metrics.

## 4. Profil

- List persona profiles with clear names.
- Create, edit, archive, and delete profile.
- Show facts grouped by category.
- Show verification and source status.
- Keep each persona's facts and history isolated.
- Require clear confirmation before deleting a persona.

## 5. Copilot workspace

Main layout:
- Left/upper area: screenshot preview or pasted question and answer options.
- Right/lower area: question interpretation and recommendation.
- Active persona always visible.
- Extraction errors and uncertain text are highlighted before analysis.
- User must confirm extracted question content before AI analysis.

Recommendation panel order:
1. Recommended answer or request for clarification.
2. Evidence status.
3. Supporting profile facts.
4. Short explanation.
5. Warnings/conflicts.
6. Actions: Edit, Copy, Confirm, Reject, Regenerate.

The app must not present a recommendation as submitted or accepted by the survey website.

## 6. Survey fit

Show each explicit criterion with one of: Cocok, Tidak cocok, Belum diketahui, Perlu diperbarui, Konflik. Each result links to the supporting profile fact or explains what is missing. Never use a green success indicator to imply guaranteed qualification.

## 7. Provider settings

Basic view:
- Gemini key list.
- Add API key.
- Test.
- Enable/disable.
- Health state.
- Add provider.

Advanced view:
- Custom endpoint and model.
- Adapter type.
- Pool strategy.
- Retry and timeout controls.
- Priority/weight and concurrency.

Mask secrets after saving. Keep provider health understandable without exposing raw technical errors by default.

## 8. Language and accessibility

- Indonesian is the default UI language.
- Use simple labels and short explanations.
- Keyboard access and visible focus for all primary actions.
- Do not rely on color alone for state.
- Support scalable text and Windows display scaling.
- Preserve user text when dialogs close or requests fail.

## 9. Error states

Use concise, actionable messages: screenshot gagal, teks kurang jelas, API terkena batas, semua API tidak tersedia, konfigurasi perlu diperiksa, atau jawaban belum didukung data profil.
