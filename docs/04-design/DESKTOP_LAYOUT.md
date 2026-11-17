# Desktop Layout

## 1. Window assumptions
Design for a resizable desktop window. Initial target is Windows 10/11; confirm minimum resolution and DPI behavior during implementation.

## 2. Main shell
- Left navigation rail/sidebar.
- Top-level page title and contextual actions.
- Main content region with responsive max width.
- Optional detail panel for evidence when screen width permits.

## 3. Dashboard
Show recent survey sessions, profile completeness as an informational indicator (not a quality score), unresolved fact reviews, and a clear action to start a session.

## 4. Digital Persona
Use searchable categories and editable fact rows/cards. Show verification status, source, last updated, and sensitivity marker. Avoid displaying sensitive values unnecessarily in overview lists.

## 5. Survey Workspace
Use a question input/editor and an analysis/recommendation area. On narrow windows, stack vertically. Preserve the question text while showing provider/network errors.

## 6. History
Filter by session, date, and action. Clearly distinguish AI draft, user-edited text, user-confirmed text, and copied text.

## 7. Settings
Group provider configuration, privacy/network disclosure, data export/delete, appearance, and diagnostics. Never display a full API key after saving.
