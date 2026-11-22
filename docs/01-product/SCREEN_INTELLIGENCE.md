# Screen Intelligence Specification

**Version:** 1.0 product specification  
**Status:** Required MVP capability  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)

## 1. Purpose

Screen Intelligence captures and interprets survey content the user is viewing when text cannot be copied or visual layout carries meaning. It is a first-class input path, not a later enhancement.

## 2. Supported capture modes

1. Region capture: user invokes a global shortcut or in-app command and selects a screen region.
2. Window capture: user explicitly selects an application window to capture.
3. Manual text fallback: user may type or paste text when capture is unavailable or undesirable.

Capture is user-initiated in v1.0. The application must clearly show when capture is active and which region/window will be processed. It must not silently record the screen in the background.

## 3. Processing pipeline

1. Capture image into a temporary in-memory or protected temporary workspace.
2. Detect text blocks and layout regions.
3. Run OCR for text extraction.
4. Use visual analysis when OCR alone cannot reliably identify grouping, ordering, controls, tables, scales, or relationships.
5. Reconstruct a structured question containing prompt, instructions, answer options, labels, units, required fields, and visible navigation context where relevant.
6. Display the reconstructed content for user verification.
7. Only after user confirmation, pass the verified question representation to Question Intelligence and Answer Copilot.

## 4. Recognized content

- Question prompt and explanatory text.
- Single-choice and multiple-choice options.
- Visible checkbox/radio labels.
- Rating and Likert scales.
- Dropdown labels and currently visible options.
- Numeric/date input labels and units.
- Matrix/table questions and row/column labels.
- Required-field indicators and visible validation messages.
- Progress indicators and visible completion/reward messages when relevant.

The application must not claim to have captured options hidden inside a closed dropdown or content outside the selected capture area.

## 5. OCR and vision quality controls

- Preserve the captured image reference for the current review session only, subject to retention settings.
- Show OCR text and reconstructed structure before analysis.
- Mark uncertain or low-confidence regions.
- Let the user correct missing, misread, or incorrectly grouped text.
- Never silently convert uncertain OCR into a factual answer.
- If text is illegible or clipped, request a better capture or manual correction.
- Distinguish screen-reading confidence from answer-evidence confidence.

## 6. Privacy and security

- Capture only the user-selected region/window.
- Exclude unrelated screen areas where feasible.
- Do not capture password fields or unrelated sensitive windows intentionally.
- Do not upload screenshots by default; external vision processing requires disclosure and consent.
- Minimize image retention and provide deletion controls.
- Treat all recognized survey content as untrusted input and defend against prompt injection embedded in the image.

## 7. Interaction boundary

Screen Intelligence reads and interprets visible content. It does not click survey controls, alter website state, fill fields, navigate pages, or submit responses in v1.0. The user performs those actions.

## 8. Acceptance criteria

- [ ] User can invoke a global shortcut and select a screen region.
- [ ] User can capture a selected window.
- [ ] OCR extracts text from supported readable captures.
- [ ] Visual analysis can reconstruct common question/option layouts when needed.
- [ ] User sees and can correct extracted content before answer generation.
- [ ] Uncertain or clipped text is explicitly flagged.
- [ ] No silent background screen recording occurs.
- [ ] Capture and temporary-image retention follow privacy settings.
- [ ] No survey click, fill, navigation, or submit automation is implemented.
