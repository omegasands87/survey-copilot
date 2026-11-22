# Multi-Project Session Handoff: SURVEY COPILOT

- **Waktu Handoff**: 2026-11-23T02:42:00+08:00
- **ID Percakapan Asal**: `ba54e966-6afc-4eda-babf-448f4f013c36`
- **Nama Proyek & Direktori**: `SURVEY COPILOT` (`D:\Antigravity\Projects\SURVEY COPILOT`)
- **Misi / Tujuan Utama**: Membangun aplikasi desktop Windows 11 x64 Survey Copilot v1.0 yang stabil, aman, evidence-grounded, dan Indonesian-first menggunakan Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, dan Gemini AI adapter.

---

## 1. Pekerjaan yang Telah Diselesaikan (SC-001 s/d SC-020 Selesai 100%)

- **SC-001 — Establish reproducible builds, lockfiles, dan CI**:
  - `package-lock.json` dan `src-tauri/Cargo.lock` terkunci dan konsisten.
  - Windows CI workflow `.github/workflows/windows.yml` terkonfigurasi dengan `npm ci`, cache, build, dan checks.
- **SC-002 — Introduce application layers & typed IPC**:
  - Commands tipis bebas SQL di `src-tauri/src/commands/`.
  - Domain validation di `src-tauri/src/domain/`.
  - Parameterized repository ber-scope persona di `src-tauri/src/repository/`.
  - Service layer & transaction isolation di `src-tauri/src/services/`.
  - Frontend typed IPC client di `src/services/ipcClient.ts` dengan error normalization.
- **SC-003 — Replace startup SQL replay with migration runner**:
  - Sequential, versioned, transactional migration runner di `src-tauri/src/migrations/mod.rs`.
  - Tracking tabel `_migrations` menyimpan: `version`, `name`, `checksum`, `applied_at`, `execution_duration_ms`.
  - Integritas checksum SHA-256 CRLF/LF agnostic.
  - Transaksi `IMMEDIATE` per migrasi dengan rollback otomatis jika terjadi kegagalan.
  - Baseline adoption non-destruktif untuk database yang telah ada sebelumnya.
- **SC-010 — Persona lifecycle**:
  - CRUD Persona Lengkap (create, list, view, edit/rename, switch active, archive, restore, delete).
  - Active persona atomic single-active constraint di transaksi dan SQLite UNIQUE partial index (`idx_one_active_persona`).
  - Strict persona data isolation di repository dan service layer.
  - Safe deletion dengan dependency summary dan cascade bersih 0 orphan records.
- **SC-011 — Fact and source lifecycle**:
  - Typed facts (`string`, `number`, `boolean`, `date`, `json`) dan normalisasi otomatis.
  - Siklus verifikasi fakta (`unverified`, `confirmed`, `disputed`, `outdated`). AI dilarang konfirmasi fakta otomatis.
  - Klasifikasi sensitivitas (`ordinary`, `sensitive`, `highly_sensitive`).
  - Provenance multi-sumber bukti (`fact_sources`) dan deteksi konflik fakta kontradiktif.
- **SC-012 — Data export and deletion**:
  - Ekspor data JSON ber-scope persona tunggal atau full application.
  - Jaminan keamanan mutlak: rahasia kunci API dan kredensial OS 100% dikecualikan.
  - Pembersihan total (wipe): pemadatan SQLite VACUUM, checkpoint WAL TRUNCATE, pembersihan tangkapan layar sementara.
- **SC-020 — Windows Credential Manager**:
  - Native Windows Credential Manager API via `Advapi32.dll` (`CredWriteW`, `CredReadW`, `CredDeleteW`, `CredFree`) dengan tipe `CRED_TYPE_GENERIC` dan persistensi `CRED_PERSIST_LOCAL_MACHINE`.
  - Penanganan error Win32 akurat: `1168` (`ERROR_NOT_FOUND`), `5` (`ERROR_ACCESS_DENIED`) dipetakan ke structured `AppError`.
  - Abstraksi `CredentialStore` trait (`WindowsCredentialStore` di produksi, `InMemoryCredentialStore` untuk automated test harness dan non-Windows development).
  - Skema Migrasi 0003: Tabel `app_credentials` SQLite HANYA memuat referensi buram (`survey_copilot/cred/<id>`) dan metadata non-sensitif. Kunci API asli 0% di SQLite.
  - Proteksi kredensial bersama: jika kredensial masih direferensikan oleh konfigurasi provider (`usage_count > 0`), penghapusan dicegah kecuali dikonfirmasi eksplisit (`force = true`).
  - UI Pengaturan bahasa Indonesia: tab `🔑 Kredensial & Provider AI`, input password yang seketika dibersihkan dari memori peramban, kartu status OS, uji akses, ganti kunci, dan modal hapus aman.

---

## 2. Bukti Verifikasi dan Pengujian

- **Frontend Build (`npm run build`)**: **PASS** (44 modul ditransformasikan, bundle dist valid).
- **Frontend Automated Tests (`npm test`)**: **PASS (25/25 tests passed, 0 failures, 240ms)**:
  - `tests/credential_management.test.ts` (6 tests).
  - `tests/data_management.test.ts` (6 tests).
  - `tests/error_normalization.test.ts` (4 tests).
  - `tests/fact_lifecycle.test.ts` (6 tests).
  - `tests/persona_lifecycle.test.ts` (3 tests).
- **Rust Code Formatting (`cargo fmt --check`)**: **PASS** (zero diffs).
- **Rust Integration Tests (`src-tauri/tests/credential_lifecycle_tests.rs`)**:
  - `test_at_030_save_credential_stores_secret_in_manager_and_opaque_ref_in_db`
  - `test_at_031_delete_credential_removes_from_store_and_db`
  - `test_read_secret_internal_returns_original_value_and_validates_format`
  - `test_update_credential_preserves_old_on_failure`
  - `test_delete_shared_credential_requires_force_when_in_use`
  - `test_test_credential_differentiates_status`
  - `test_ipc_summary_and_export_contain_zero_secrets`
- **Acceptance Criteria Terpenuhi**:
  - AT-001, AT-002, AT-003, AT-004, AT-005: **PASS**
  - AT-010, AT-011, AT-012, AT-013, AT-014: **PASS**
  - AT-020, AT-021, AT-022, AT-023: **PASS**
  - AT-030 (*Add API key: secret stored in Windows Credential Manager; only opaque ref in DB*): **PASS**
  - AT-031 (*Remove provider / delete credential: secret removed; shared protection*): **PASS**
  - AT-070 (*Export all local data in documented JSON; excludes API secrets*): **PASS**
  - AT-071 (*Delete all app data reports database, temp files, and external backup limits*): **PASS**

---

## 3. File-file Kunci

- **Windows Credential Manager (SC-020)**:
  - Store Trait: `src-tauri/src/infrastructure/credentials/store.rs`
  - Native Win32 Advapi32: `src-tauri/src/infrastructure/credentials/windows.rs`
  - In-Memory Mock Store: `src-tauri/src/infrastructure/credentials/in_memory.rs`
  - Domain Model: `src-tauri/src/domain/credential.rs`
  - Repository: `src-tauri/src/repository/credential_repo.rs`
  - Service: `src-tauri/src/services/credential_service.rs`
  - Commands IPC: `src-tauri/src/commands/credential.rs`
  - SQL Migrasi 0003: `src-tauri/migrations/0003_app_credentials.sql`
  - Integration Tests Rust: `src-tauri/tests/credential_lifecycle_tests.rs`
  - Frontend Types: `src/types/credential.ts`
  - Frontend Service: `src/services/credentialService.ts`
  - Frontend View: `src/components/views/SettingsView.tsx`
  - Frontend Tests: `tests/credential_management.test.ts`
  - Styling: `src/styles.css`
- **Data Export & Deletion (SC-012)**:
  - `src-tauri/src/domain/export.rs`, `src-tauri/src/services/export_service.rs`, `src-tauri/src/services/deletion_service.rs`
- **Fakta & Persona (SC-010 & SC-011)**:
  - `src-tauri/src/domain/persona.rs`, `src-tauri/src/domain/fact.rs`, `src-tauri/src/services/persona_service.rs`, `src-tauri/src/services/fact_service.rs`
- **Migrasi Runner (SC-003)**:
  - `src-tauri/src/migrations/mod.rs`, `src-tauri/src/db.rs`

---

## 4. Keputusan & Aturan Kritis (Non-Negotiable)

- **Tanpa Automasi Browser**: Dilarang membuat browser automation, auto-click, auto-fill, auto-submit, atau bypass platform survei.
- **Integritas Fakta Asli**: Seluruh fakta mewakili kondisi nyata individu sebenarnya; dilarang mengarang identitas responden atau data penipuan screener.
- **Isolasi Mutlak Antar-Persona**: Data persona A tidak boleh bocor atau diakses oleh persona B baik saat interaksi maupun ekspor file.
- **Kredensial Aman**: API secret AI hanya disimpan di Windows Credential Manager; tidak boleh masuk ke SQLite, file ekspor, logs, atau frontend state.
- **Pembersihan Bersih**: Penghapusan data harus menjamin 0 orphan records, checkpoint WAL TRUNCATE, pemadatan VACUUM, dan pembersihan berkas screenshot sementara.

---

## 5. Status Terkini & Langkah Berikutnya

- **Status Git**: Bersih (`working tree clean`), commit `6d95dea` pada branch `main`.
- **Langkah Berikutnya (Next Steps)**:
  1. **P2 — SC-021: Provider Configuration and Adapter Contracts**:
     - Implementasi CRUD konfigurasi provider di SQLite (`provider_configs`).
     - Gemini provider adapter implementation.
     - Validasi endpoint HTTPS (izin HTTP khusus localhost setelah konfirmasi).
     - Standardisasi kontrak request, structured output, dan normalisasi error provider.
     - Connection test menggunakan opaque credential reference dari Windows Credential Manager.
  2. **P2 — SC-022: Provider Pool**:
     - Rotasi availability-aware, cooldown, bounded concurrency, quota detection.
  3. **P3 — Question-to-Answer Workflow**:
     - Pertanyaan teks manual / tangkapan layar eksplisit ke rekomendasi jawaban berbasis fakta.
