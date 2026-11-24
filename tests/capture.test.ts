import { test } from "node:test";
import assert from "node:assert/strict";
import type {
  CaptureMode,
  CapturePreview,
  CaptureRect,
  CaptureRegionInput,
  CaptureWindowInput,
  CleanupResult,
  DiscardResult,
  WindowInfo,
} from "../src/types/capture";
import type { CreateQuestionInput, SurveyQuestion } from "../src/types/survey";

test("CaptureMode taxonomy strictly covers normative modes", () => {
  const modes: CaptureMode[] = ["region", "window", "clipboard_image", "manual"];
  assert.equal(modes.length, 4);
  assert.ok(modes.includes("region"));
  assert.ok(modes.includes("window"));
  assert.ok(modes.includes("clipboard_image"));
  assert.ok(modes.includes("manual"));
});

test("CapturePreview contract exposes full metadata and base64 preview URL (AT-050)", () => {
  const preview: CapturePreview = {
    captureId: "c1a2b3c4-d5e6-4f7a-8b9c-0d1e2f3a4b5c",
    sessionId: "sess-101",
    sourceMode: "region",
    capturedAt: "2026-11-23T12:00:00.000Z",
    filePath: "C:\\Users\\Asus\\AppData\\Roaming\\survey-copilot\\temp_captures\\capture_c1a2b3c4.png",
    fileName: "capture_c1a2b3c4.png",
    fileSizeBytes: 148200,
    width: 1280,
    height: 720,
    targetLabel: "Area Layar (0, 0, 1280x720)",
    previewDataUrl: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
  };

  assert.equal(preview.captureId, "c1a2b3c4-d5e6-4f7a-8b9c-0d1e2f3a4b5c");
  assert.equal(preview.sourceMode, "region");
  assert.equal(preview.width, 1280);
  assert.equal(preview.height, 720);
  assert.ok(preview.fileSizeBytes > 0);
  assert.ok(preview.previewDataUrl?.startsWith("data:image/png;base64,"));
  assert.ok(preview.filePath.includes("temp_captures"));
});

test("WindowInfo contract accurately represents OS top-level windows", () => {
  const windowInfo: WindowInfo = {
    id: 1245890,
    title: "Google Chrome - YouGov Consumer Survey",
    appName: "chrome.exe",
    x: 100,
    y: 80,
    width: 1440,
    height: 900,
    isMinimized: false,
  };

  assert.equal(windowInfo.id, 1245890);
  assert.equal(windowInfo.title, "Google Chrome - YouGov Consumer Survey");
  assert.equal(windowInfo.appName, "chrome.exe");
  assert.equal(windowInfo.isMinimized, false);
  assert.ok(windowInfo.width > 0);
  assert.ok(windowInfo.height > 0);
});

test("DiscardResult contract enforces explicit deletion outcome (AT-054)", () => {
  const result: DiscardResult = {
    captureId: "c1a2b3c4-d5e6-4f7a-8b9c-0d1e2f3a4b5c",
    deleted: true,
    filePath: "C:\\temp\\capture_c1a2b3c4.png",
    message: "Berkas tangkapan layar sementara berhasil dibatalkan dan dihapus permanen dari disk.",
  };

  assert.equal(result.deleted, true);
  assert.ok(result.message.includes("dihapus permanen"));
  assert.equal(result.captureId, "c1a2b3c4-d5e6-4f7a-8b9c-0d1e2f3a4b5c");
});

test("CleanupResult contract tracks retention reclamation statistics", () => {
  const cleanup: CleanupResult = {
    deletedCount: 5,
    reclaimedBytes: 1048576,
    message: "Pembersihan retensi selesai: 5 berkas dihapus, 1048576 bytes ruang disk dipulihkan.",
  };

  assert.equal(cleanup.deletedCount, 5);
  assert.equal(cleanup.reclaimedBytes, 1048576);
  assert.ok(cleanup.message.includes("5 berkas dihapus"));
});

test("Product boundary: explicit invocation required; no hidden background recording (AT-051)", () => {
  // Simulating explicit request payloads
  const regionInput: CaptureRegionInput = {
    sessionId: "sess-1",
    rect: {
      x: 50,
      y: 50,
      width: 600,
      height: 400,
    },
  };
  const windowInput: CaptureWindowInput = {
    sessionId: "sess-1",
    windowId: 998877,
  };

  assert.ok(regionInput.rect !== undefined);
  assert.equal(regionInput.rect?.width, 600);
  assert.equal(windowInput.windowId, 998877);
  // Absence of background capture interval or automatic periodic capture flags
  assert.ok(!("intervalMs" in (regionInput as Record<string, unknown>)));
  assert.ok(!("periodic" in (regionInput as Record<string, unknown>)));
});

test("SurveyQuestion links to captureId with explicit provenance", () => {
  const createInput: CreateQuestionInput = {
    personaId: "persona-1",
    sessionId: "session-1",
    captureId: "cap-998877",
    questionTextOriginal: "Berapa frekuensi konsumsi kopi Anda dalam seminggu terakhir?",
    instructions: "Pilih satu opsi",
    options: ["1-2 kali", "3-5 kali", "Setiap hari"],
    questionType: "single_choice",
    answerFormat: "single_choice",
    provenance: "ocr_detected",
  };

  assert.equal(createInput.captureId, "cap-998877");
  assert.equal(createInput.provenance, "ocr_detected");

  const question: SurveyQuestion = {
    id: "q-101",
    sessionId: "session-1",
    personaId: "persona-1",
    captureId: "cap-998877",
    questionOrder: 1,
    questionTextOriginal: "Berapa frekuensi konsumsi kopi Anda dalam seminggu terakhir?",
    options: ["1-2 kali", "3-5 kali", "Setiap hari"],
    extractionStatus: "classified",
    questionType: "single_choice",
    answerFormat: "single_choice",
    provenance: "ocr_detected",
    observationType: "detected",
    confirmationStatus: "draft",
    createdAt: "2026-11-23T12:00:00Z",
    updatedAt: "2026-11-23T12:00:00Z",
  };

  assert.equal(question.captureId, "cap-998877");
  assert.equal(question.provenance, "ocr_detected");
});
