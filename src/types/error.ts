export interface StructuredErrorPayload {
  code: string;
  message: string;
  field?: string;
}

export class AppError extends Error {
  readonly code: string;
  readonly field?: string;

  constructor(payload: StructuredErrorPayload | string) {
    if (typeof payload === "string") {
      super(payload);
      this.code = "INTERNAL_ERROR";
    } else {
      super(payload.message);
      this.code = payload.code;
      this.field = payload.field;
    }
    this.name = "AppError";
  }

  static from(error: unknown): AppError {
    if (error instanceof AppError) {
      return error;
    }

    if (error && typeof error === "object") {
      const candidate = error as Partial<StructuredErrorPayload>;
      if (typeof candidate.code === "string" && typeof candidate.message === "string") {
        return new AppError({
          code: candidate.code,
          message: candidate.message,
          field: typeof candidate.field === "string" ? candidate.field : undefined,
        });
      }
    }

    if (typeof error === "string") {
      // Check if Tauri returned serialized JSON
      try {
        const parsed = JSON.parse(error) as unknown;
        if (parsed && typeof parsed === "object") {
          const candidate = parsed as Partial<StructuredErrorPayload>;
          if (typeof candidate.code === "string" && typeof candidate.message === "string") {
            return new AppError({
              code: candidate.code,
              message: candidate.message,
              field: typeof candidate.field === "string" ? candidate.field : undefined,
            });
          }
        }
      } catch {
        // Not JSON, use raw string
      }
      return new AppError(error);
    }

    if (error instanceof Error) {
      return new AppError(error.message);
    }

    return new AppError("Terjadi kesalahan yang tidak terduga.");
  }
}
