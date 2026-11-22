import test from "node:test";
import assert from "node:assert";
import { AppError } from "../src/types/error.ts";

test("AppError.from parses raw string", () => {
  const err = AppError.from("Profil tidak ditemukan.");
  assert.strictEqual(err.message, "Profil tidak ditemukan.");
  assert.strictEqual(err.code, "INTERNAL_ERROR");
});

test("AppError.from parses serialized JSON error", () => {
  const serialized = JSON.stringify({
    code: "VALIDATION_ERROR",
    message: "Nama profil harus berisi 1 sampai 80 karakter.",
    field: "displayName",
  });
  const err = AppError.from(serialized);
  assert.strictEqual(err.code, "VALIDATION_ERROR");
  assert.strictEqual(err.message, "Nama profil harus berisi 1 sampai 80 karakter.");
  assert.strictEqual(err.field, "displayName");
});

test("AppError.from parses structured error object directly", () => {
  const err = AppError.from({
    code: "NOT_FOUND",
    message: "Profil tidak ditemukan atau sudah tidak aktif.",
  });
  assert.strictEqual(err.code, "NOT_FOUND");
  assert.strictEqual(err.message, "Profil tidak ditemukan atau sudah tidak aktif.");
  assert.strictEqual(err.field, undefined);
});

test("AppError.from handles standard Error instance", () => {
  const nativeErr = new Error("Koneksi gagal");
  const err = AppError.from(nativeErr);
  assert.strictEqual(err.message, "Koneksi gagal");
  assert.strictEqual(err.code, "INTERNAL_ERROR");
});
