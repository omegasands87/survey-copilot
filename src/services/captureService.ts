import { invokeCommand } from "./ipcClient";
import type {
  CapturePreview,
  CaptureRegionInput,
  CaptureWindowInput,
  CleanupResult,
  DiscardResult,
  WindowInfo,
} from "../types/capture";

export const captureService = {
  /**
   * Retrieves list of visible application windows for window-based screen capture.
   */
  async listWindows(): Promise<WindowInfo[]> {
    return invokeCommand<WindowInfo[]>("capture_list_windows");
  },

  /**
   * Captures a rectangular region of the screen (or full screen if no bounds provided).
   */
  async captureRegion(input: CaptureRegionInput): Promise<CapturePreview> {
    return invokeCommand<CapturePreview>("capture_region", { input });
  },

  /**
   * Captures a specific application window by its window ID.
   */
  async captureWindow(input: CaptureWindowInput): Promise<CapturePreview> {
    return invokeCommand<CapturePreview>("capture_window", { input });
  },

  /**
   * Discards a capture immediately, deleting the temporary image from disk.
   */
  async discardCapture(captureId: string): Promise<DiscardResult> {
    return invokeCommand<DiscardResult>("capture_discard", { captureId });
  },

  /**
   * Cleans up temporary captures older than the specified retention age.
   */
  async cleanupCaptures(maxAgeSeconds?: number): Promise<CleanupResult> {
    return invokeCommand<CleanupResult>("capture_cleanup", { maxAgeSeconds });
  },

  /**
   * Retrieves preview data for an existing capture ID.
   */
  async getPreview(captureId: string): Promise<CapturePreview> {
    return invokeCommand<CapturePreview>("capture_get_preview", { captureId });
  },
};
