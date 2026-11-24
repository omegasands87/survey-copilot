import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const copilotSource = readFileSync(
  new URL("../src/components/views/CopilotView.tsx", import.meta.url),
  "utf8",
);
const darkTheme = readFileSync(
  new URL("../src/styles/dark-bento.css", import.meta.url),
  "utf8",
);

function between(source: string, start: string, end: string): string {
  const from = source.indexOf(start);
  assert.notEqual(from, -1, `missing section: ${start}`);
  const to = source.indexOf(end, from + start.length);
  assert.notEqual(to, -1, `missing section end: ${end}`);
  return source.slice(from, to);
}

test("explicit region capture automatically starts local OCR without a second action", () => {
  const handler = between(
    copilotSource,
    "async function handleStartRegionCapture()",
    "async function handleOpenWindowPicker()",
  );
  assert.match(handler, /await handleExtractOcr\(preview\.captureId\)/);
  assert.match(handler, /setShowCapturePreviewModal\(true\)/);
  assert.match(handler, /sessionId = currentSession\?\.id \|\| selectedSessionId/);
});

test("explicit window capture automatically starts local OCR", () => {
  const handler = between(
    copilotSource,
    "async function handleSelectWindowToCapture",
    "async function handleDiscardCapture",
  );
  assert.match(handler, /await handleExtractOcr\(preview\.captureId\)/);
  assert.match(handler, /setActiveOcrExtraction\(null\)/);
});

test("automatic OCR only opens a review draft and does not start AI analysis", () => {
  const handler = between(
    copilotSource,
    "async function handleExtractOcr",
    "async function handleConfirmOcrUse",
  );
  assert.match(handler, /ocrService\.extractCapture/);
  assert.match(handler, /setShowOcrReviewModal\(true\)/);
  assert.match(handler, /setShowCapturePreviewModal\(false\)/);
  assert.doesNotMatch(handler, /surveyService\.(generate|evaluate|predictNextQuestions)/);
});

test("capture preview offers retry only after OCR error and keeps manual fallback", () => {
  const preview = between(
    copilotSource,
    "{showCapturePreviewModal && activeCapturePreview && (",
    "{showOcrReviewModal && activeOcrExtraction && (",
  );
  assert.doesNotMatch(preview, /Ekstrak Teks & Layout \(OCR\)/);
  assert.match(preview, /ocrError && \(/);
  assert.match(preview, /Coba OCR lokal lagi/);
  assert.match(preview, /Tautkan Manual/);
  assert.match(preview, /Batalkan & Hapus Gambar/);
});

test("local OCR privacy and progress states respect reduced-motion preferences", () => {
  assert.match(copilotSource, /Capture dan OCR lokal/);
  assert.match(copilotSource, /Tidak ada request AI eksternal/);
  assert.match(darkTheme, /\.capture-local-ocr-notice/);
  assert.match(darkTheme, /\.capture-ocr-progress/);
  assert.match(darkTheme, /prefers-reduced-motion:\s*reduce/);
});
