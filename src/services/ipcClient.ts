import { invoke } from "@tauri-apps/api/core";
import { AppError } from "../types/error";

/**
 * Typed client wrapper for Tauri IPC commands.
 * Normalizes internal errors into structured AppError instances.
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (rawError) {
    throw AppError.from(rawError);
  }
}
