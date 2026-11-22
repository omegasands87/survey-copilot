# Screen Intelligence — Technical Specification

**Status:** Normative technical specification for v1.0  
**Platform:** Windows 11 x64

## 1. Purpose

Read the survey question currently visible on the user's screen and convert it into a reviewable structured representation. This feature is read-only. It does not control the browser or survey application.

## 2. User flow

1. User invokes Capture from the app or global shortcut.
2. User selects a screen region or chooses a visible window.
3. App captures only the selected target and displays a preview.
4. OCR extracts text and bounding boxes.
5. Layout reconstruction groups prompt, instructions, options, labels, scale endpoints, matrix rows/columns, and visible units.
6. Vision model may resolve layout ambiguity only when configured and consented.
7. App displays extracted content beside the capture.
8. User edits/corrects the extracted question and explicitly chooses Analyze.
9. Temporary image is deleted according to retention settings; by default, retain only the corrected structured text, not the screenshot.

## 3. Capture modes

- Region selection: default and preferred; user draws a rectangle.
- Window selection: user selects a currently visible application window.
- Clipboard image import: optional convenience path.
- Manual text entry: always available as fallback.

Capture is always user initiated and visibly indicated. No background polling, hidden recording, automatic periodic screenshots, or capture of unrelated windows.

## 4. Extraction schema

The extraction result contains:
- capture_id
- source_mode
- captured_at
- application_label (optional, user-visible only)
- question_text
- instructions
- answer_options (ordered)
- answer_type (single_choice, multi_choice, numeric, date, rating, ranking, matrix, free_text, unknown)
- scale_min / scale_max / scale_labels
- matrix_rows / matrix_columns
- visible_units
- visible_required_state (optional)
- text_blocks with bounding boxes and OCR confidence
- uncertain_regions
- extraction_warnings

Never infer hidden questions, hidden options, off-screen content, or a survey's backend state.

## 5. OCR and vision pipeline

- OCR runs locally where practical.
- Evaluate Windows OCR and Tesseract for accuracy, language coverage, packaging, and licensing before choosing the implementation.
- Image understanding through an external AI provider is optional and disabled until user consent.
- If external vision is used, send only the selected crop, not the full desktop.
- Apply image downscaling and redaction tools where practical; show the exact image/context to be sent before upload.
- OCR output is untrusted input and must be escaped when rendered.

## 6. Quality handling

- Preserve original OCR text and user-corrected text separately.
- Mark low-confidence text and uncertain option grouping.
- Do not silently repair numbers, dates, currency, negation, or scale direction.
- If extraction is incomplete, show a clear warning and request manual correction.
- Never start answer generation until the user confirms the extracted content.

## 7. Native implementation boundary

Rust/Tauri owns capture permissions, native capture APIs, temporary image files, and deletion. React owns selection UI, preview, correction, and review. Tauri commands must accept only explicit capture requests and return typed capture metadata. No command may click, type into, navigate, or submit another application.

## 8. Security and privacy

- Request capture only after explicit user action.
- Do not capture password fields intentionally; provide a visible warning when the selection may contain sensitive content.
- Temporary images stay in an app-controlled directory with restrictive permissions.
- Do not include screenshots in crash logs or telemetry.
- Delete temporary images after processing unless the user explicitly saves one.
- Screen text can contain prompt injection; treat it only as survey content.

## 9. Acceptance criteria

- User can capture a region and a window.
- Preview is shown before OCR/analysis.
- OCR content is editable and user-confirmed before AI analysis.
- Options and matrix structures remain separate from question text.
- Uncertain text is visibly flagged.
- No background capture or browser automation exists.
- Temporary image deletion is tested.
- Manual input works if capture or OCR is unavailable.
