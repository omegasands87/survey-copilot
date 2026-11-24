export type CaptureMode = "region" | "window" | "clipboard_image" | "manual";

export interface CaptureRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface CaptureRegionInput {
  sessionId?: string | null;
  rect?: CaptureRect | null;
}

export interface CaptureWindowInput {
  sessionId?: string | null;
  windowId: number;
}

export interface WindowInfo {
  id: number;
  title: string;
  appName?: string | null;
  x: number;
  y: number;
  width: number;
  height: number;
  isMinimized: boolean;
}

export interface CapturePreview {
  captureId: string;
  sessionId?: string | null;
  sourceMode: CaptureMode;
  capturedAt: string;
  filePath: string;
  fileName: string;
  fileSizeBytes: number;
  width: number;
  height: number;
  targetLabel?: string | null;
  previewDataUrl?: string | null;
}

export interface DiscardResult {
  captureId: string;
  deleted: boolean;
  filePath?: string | null;
  message: string;
}

export interface CleanupResult {
  deletedCount: number;
  reclaimedBytes: number;
  message: string;
}
