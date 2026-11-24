# Stage 4 — Capture → Automatic Local OCR → Human Review

**Scope:** simplify the screen-reading workflow after an explicit user capture. Capture and OCR stay inside the Windows desktop application. This stage does not add background capture or automatic AI transmission.

## Updated workflow

1. The user chooses **Capture area** or opens the Windows window picker and selects a visible application window.
2. A selected active survey session and respondent persona are required so the capture is correctly associated with local records.
3. The image is saved to the app's temporary local capture directory and preview metadata is displayed.
4. Local Windows OCR starts automatically from the same capture action; there is no separate “start OCR” click.
5. While OCR is running, the preview shows a clear progress and privacy state. Actions that could race the OCR operation (close, discard, and manual-link) are disabled.
6. On success, the OCR review dialog opens with extracted text, options, layout, and uncertainty indicators. The user can correct the extraction.
7. The user explicitly chooses to apply the result to the editor or confirm/record the question in the selected session.
8. AI analysis remains a separate, later user action and continues to use the existing outbound-data disclosure and per-request consent controls.

## OCR failure and manual fallback

- The capture preview remains available when local OCR fails.
- The error is shown next to the preview.
- The user can retry local OCR or use **Tautkan Manual** and paste/type the text.
- Synthetic fixture OCR is not used in production to fill a failed result.
- OCR persistence failures are surfaced as errors instead of presenting an unsaved result as successful.

## Privacy boundary

- Screen capture only occurs after the user explicitly chooses an area/window capture action.
- OCR runs locally and does not contact a remote model/provider.
- A capture action is not consent for external vision or AI processing.
- No AI request starts automatically after OCR completes.
- External AI analysis remains gated behind the existing disclosure and per-request consent flow.
- No background screen recording, silent capture, browser automation, autofill, auto-click, or auto-submit is added.

## Implementation

- `CopilotView.tsx`: region and window capture handlers call the existing OCR command immediately after successful capture.
- OCR extraction helper opens the review dialog only after a successful local extraction and leaves the preview visible on failure.
- Capture preview shows local-only OCR progress and privacy messaging.
- Retry is available only when OCR returned an error; the separate first-time “start OCR” button is removed.
- Window selection uses an explicit button rather than a clickable non-semantic container.
- Dark Bento styles define progress and local-privacy states and honor `prefers-reduced-motion`.
- `tests/capture_auto_ocr_flow.test.ts` guards automatic local OCR, review gating, manual fallback, and absence of automatic AI-analysis calls.

## Repository validation

Run the full TypeScript/Vite build, frontend tests, Rust format, Rust compile, and Rust tests in GitHub Actions for the cumulative branch.

## DEVICE TEST REQUIRED — after all five stages

On a supported Windows 11 x64 device, verify:
- Capture region and application window both trigger local OCR automatically.
- Progress, OCR errors, retry, review, correction, cancel/delete, and manual fallback work.
- No external AI request occurs until the user deliberately starts analysis.
- OCR failure never displays synthetic survey text.
- The local capture file lifecycle follows the configured retention and deletion policy.
