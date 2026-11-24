# Stage 3 — Dark Bento UI, Profile Forms, Templates, and JSON Import

**Scope:** UI/UX desktop and shared browser preview presentation, respondent profile form usability, blank field templates, and local single-profile JSON import.

## 1. Visual system

The shared design tokens and component overrides are in `src/styles/dark-bento.css`. It is imported by the normal desktop entry and the browser preview so both surfaces use the same visual language.

The theme covers:
- Dark surfaces, borders, typography, semantic accents, focus states, and card elevation.
- Sidebar, active persona selector, page headers, bento/card layouts, form controls, dialogs, status badges, and settings panels.
- Responsive behavior for desktop, tablet, and narrow/mobile widths, including visibility of the Settings navigation action.

## 2. Manual profile data entry

The profile fact form is grouped into responsive field rows with shorter, concrete labels and field guidance. New facts default to **unverified** and require a real value and a source description. The template chooser never fills personal values.

The form no longer promotes earnings/income as an example field. Financial/reward/income tracking remains out of the v1.0 product scope unless the owner explicitly changes that scope.

## 3. Blank profile field templates

The available blank templates are:
- Demografi dasar
- Pekerjaan & pendidikan
- Rumah tangga
- Teknologi & perangkat
- Konsumsi & kebiasaan
- Transportasi & mobilitas

A template field only sets the category, attribute name, data type, and optional unit in the manual-entry form. It does not create a fact, supply a personal value, or change the default unverified status. The user must enter and confirm every actual value.

## 4. JSON profile import

### Supported source

The first supported format is a Survey Copilot **single-persona JSON export**, schema version `1.0.0`, generated from the existing profile export feature.

### User flow

1. Select a JSON file.
2. The UI checks the single-persona envelope and shows a preview of the source profile, facts, and evidence-source count.
3. The user may edit the new profile's display name.
4. The user confirms **Import as new profile**.
5. The desktop command validates the JSON again and imports the profile, current facts, and evidence sources in one SQLite transaction.

### Safety and integrity rules

- A profile import always creates a **new inactive persona** with newly generated IDs. It does not overwrite or merge an existing profile.
- Duplicate display names are warned about in the UI; the user can rename the new profile before import.
- The JSON envelope must declare `single_persona`, schema version `1.0.0`, and consistent target/persona IDs.
- The file must be at most 5 MB; imports are bounded to 2,000 facts and 8,000 evidence sources.
- Every fact and evidence source must belong to the one exported persona. Unsupported fact types, malformed values, invalid statuses/sensitivities, and broken source references reject the import.
- Existing profile, fact, and source IDs are never reused.
- If a fact has no evidence source in the export, it is imported as **unverified** with an explicit “source not included” note.
- Any validation or database error aborts the whole transaction; no partial persona/fact import remains.
- The import path does not import survey sessions, answers, history, screenshots, raw capture files, provider settings, API keys, Windows Credential Manager secrets, or diagnostics.
- No network request or external AI provider is used during import.

### Browser preview behavior

The browser preview keeps the same import UI surface for design review, but the import is a mock: data is added to React memory only and disappears on refresh. It does not call the desktop command or persist a real profile.

## 5. Repository tests

The Stage 3 repository validation includes:
- TypeScript/Vite build and frontend regression tests.
- Rust format, compile check, and all Rust tests.
- Dedicated import tests for valid single-persona import, full-application rejection, invalid-value rollback, and the 5 MB limit.
- UI tests for dark theme inclusion, blank template semantics, and import scope.

## 6. Device verification — deferred until all five stages are complete

The following are **DEVICE TEST REQUIRED** and must be run on the supported Windows 11 x64 device after repository work for all five locked stages has finished:
- Visual review on the actual installed desktop application at supported window sizes and Windows scaling.
- Keyboard navigation, dialog behavior, and usability of the new fact form.
- Import a real single-persona JSON export into a test profile and verify facts/sources; verify no overwrite/merge and that unrelated sessions/credentials remain unchanged.
- Compare the installed desktop theme with the browser preview.

These device checks do not block the repository implementation work in the remaining stages.
