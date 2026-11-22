# Multi-Project Session Handoff: SURVEY COPILOT

- **Waktu Handoff**: 2026-11-23T02:25:00+08:00
- **ID Percakapan Asal**: `ba54e966-6afc-4eda-babf-448f4f013c36`
- **Nama Proyek & Direktori**: `SURVEY COPILOT` (`D:\Antigravity\Projects\SURVEY COPILOT`)
- **Misi / Tujuan Utama**: Membangun aplikasi desktop Windows 11 x64 Survey Copilot v1.0 yang stabil, aman, evidence-grounded, dan Indonesian-first menggunakan Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, dan Gemini AI adapter.

---

## 1. Pekerjaan yang Telah Diselesaikan (SC-001, SC-002, SC-003, SC-010, SC-011, SC-012, SC-020 Selesai 100%)

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
  - **CRUD Persona Lengkap**: Create (nama 1-80 karakter, deskripsi opsional hingga 500 karakter), List (filter arsip/aktif), View detail, Edit/rename, Switch active, Archive, Restore, Delete.
  - **Active Persona Management**: Enforce tepat satu active persona di tingkat transaksi dan SQLite UNIQUE partial index (`idx_one_active_persona`). Persisten lintas restart aplikasi.
  - **Safe Replacement**: Jika persona aktif diarsipkan atau dihapus, status aktif secara atomik dialihkan ke kandidat persona aktif berikutnya.
  - **Strict Persona Data Isolation**:
    - Validasi kepemilikan data di tingkat backend repository dan service.
    - Pembuktian tes: Persona B tidak dapat membaca, mengubah, atau menghapus fakta milik Persona A.
    - Isolasi total profil fakta, sumber bukti, sesi survei, dan riwayat.
  - **Safe Deletion dengan Dependency Summary**:
    - Perhitungan menyeluruh record terkait sebelum penghapusan (`profile_facts`, `fact_sources`, `survey_sessions`, `survey_questions`, `answer_recommendations`, `answer_history`, `survey_fit_results`, `survey_attempts`).
    - Modal konfirmasi eksplisit dengan rincian data yang akan terhapus dan checkbox konfirmasi.
    - Transaksi atomik dengan foreign keys `ON DELETE CASCADE` di SQLite tanpa meninggalkan orphan records.
- **SC-011 — Fact and source lifecycle**:
  - **Typed Facts**:
    - Didukung penuh tipe data: `string`, `number` (integer & desimal), `boolean` (ya/tidak/true/false), `date` (ISO YYYY-MM-DD), dan `json` (objek/array terstruktur).
    - Normalisasi otomatis tanpa mengubah esensi nilai (contoh: "15.000.000" -> `15000000`, "ya" -> `true`).
    - Metadata lengkap per fakta: `id`, `persona_id`, `category`, `fact_key`, `attribute` (display name), `data_type`, `value`, `normalized_value`, `unit`, `verification_status`, `sensitivity`, `source_date`, `valid_from`, `valid_until`, `notes`.
  - **Fact Verification Lifecycle**:
    - Siklus status: `unverified` (default otomatis saat pembuatan), `confirmed` (hanya tindakan eksplisit pengguna), `disputed`, dan `outdated`.
    - AI dilarang keras mengonfirmasi fakta secara otomatis.
    - Fakta `disputed` dan `outdated` dicegah secara ketat di backend agar tidak pernah digunakan sebagai bukti definitif.
  - **Sensitivity Classification**:
    - Klasifikasi: `ordinary`, `sensitive`, `highly_sensitive`.
    - Kebijakan bukti: fakta `sensitive` & `highly_sensitive` dilarang dikirim ke AI provider tanpa persetujuan eksplisit per-permintaan.
    - Redaksi nilai sensitif pada logs dan diagnostics.
  - **Fact Sources & Provenance**:
    - Multi-sumber bukti per fakta (`fact_sources`).
    - Jenis sumber: `manual_entry`, `user_declaration`, `uploaded_document`, `observed_action`, `external_record`.
    - Metadata sumber: `source_type`, `title`, `description`, `reference_uri`, `observed_at`, `valid_from`, `valid_until`, `notes`.
- **SC-012 — Data export and deletion**:
  - **Ekspor Data Terstruktur JSON**:
    - Ekspor persona tunggal terisolasi atau seluruh aplikasi.
    - Jaminan privasi: rahasia API key / kredensial OS 100% dikecualikan.
    - Penghitungan dan peringatan privasi otomatis jika ekspor memuat fakta sensitif.
  - **Pembersihan Data Destruktif (Full Application Wipe)**:
    - Penghapusan seluruh data ber-cascade dengan jaminan 0 orphan records.
    - Eksekusi checkpoint `WAL TRUNCATE` dan pemadatan `VACUUM`.
    - Pembersihan folder berkas screenshot sementara (`screen_captures`).
- **SC-020 — Windows Credential Manager**:
  - **Native Windows Credential Storage**:
    - Implementasi backend native Rust via `Advapi32.dll` (`CredWriteW`, `CredReadW`, `CredDeleteW`, `CredFree`) dengan `CRED_TYPE_GENERIC` dan `CRED_PERSIST_LOCAL_MACHINE`.
    - Penanganan kode error Win32 akurat: `1168` (`ERROR_NOT_FOUND`), `5` (`ERROR_ACCESS_DENIED`), dan mapping ke structured `AppError`.
    - Abstraction trait `CredentialStore` dengan `WindowsCredentialStore` di produksi dan `InMemoryCredentialStore` untuk automated test harness dan non-Windows developer environments.
  - **Arsitektur Referensi Buram (Opaque Credential Reference)**:
    - Migration 0003: Tabel `app_credentials` di SQLite HANYA memuat: `id`, `provider_type`, `label`, `credential_ref`, `status`, `created_at`, `updated_at`.
    - Format referensi buram: `survey_copilot/cred/<id>`.
    - Kunci API asli **TIDAK PERNAH** disimpan di SQLite, file JSON, registry plaintext, environment variable permanen, ataupun frontend state.
  - **Perlindungan Kredensial Bersama**:
    - Pengecekan referensi `provider_configs` sebelum penghapusan (`count_references_in_provider_configs`).
    - Jika `usage_count > 0`, penghapusan ditolak dengan `CONFLICT_ERROR` kecuali disertai konfirmasi eksplisit (`force = true`).
  - **UI Pengelolaan Kredensial & Kunci API**:
    - Halaman Pengaturan dengan tab utama `🔑 Kredensial & Provider AI` dalam bahasa Indonesia.
    - Input password (`type="password"`) yang seketika dibersihkan dari memori peramban setelah pengiriman.
    - Status badge: `✓ Tersimpan di OS`, `✕ Belum di OS`, `⚠ Perlu Update`, `⚠ Akses Ditolak`.
    - Aksi lengkap: Tambah kunci baru, Uji Akses, Ganti Kunci, Hapus dengan modal proteksi kredensial bersama.

---

## 2. Hasil Verifikasi dan Bukti Pengujian

- **Frontend Build (`npm run build`)**: **PASS** (44 modules, built cleanly with Vite + TypeScript compiler `tsc -b`).
- **Frontend Automated Tests (`npm test`)**: **PASS (25/25 tests passed, 0 failures, 268ms)**:
  - `tests/credential_management.test.ts` (6 tests).
  - `tests/data_management.test.ts` (6 tests).
  - `tests/error_normalization.test.ts` (4 tests).
  - `tests/fact_lifecycle.test.ts` (6 tests).
  - `tests/persona_lifecycle.test.ts` (3 tests).
- **Rust Code Formatting (`cargo fmt --check`)**: **PASS** (clean, zero formatting diffs).
- **Rust Integration Tests (`src-tauri/tests/credential_lifecycle_tests.rs`)**:
  - `test_at_030_save_credential_stores_secret_in_manager_and_opaque_ref_in_db`
  - `test_at_031_delete_credential_removes_from_store_and_db`
  - `test_read_secret_internal_returns_original_value_and_validates_format`
  - `test_update_credential_preserves_old_on_failure`
  - `test_delete_shared_credential_requires_force_when_in_use`
  - `test_test_credential_differentiates_status`
  - `test_ipc_summary_and_export_contain_zero_secrets`
- **Acceptance Criteria Terverifikasi**:
  - **AT-001** (*Fresh checkout installs and builds*): **PASS**.
  - **AT-002** (*Fresh DB startup creates tables and _migrations tracking*): **PASS**.
  - **AT-003** (*App restarts with existing DB are idempotent*): **PASS**.
  - **AT-004** (*Migration fails midway triggers rollback*): **PASS**.
  - **AT-005** (*Layered command and IPC decoupling*): **PASS**.
  - **AT-010** (*Create first persona becomes active*): **PASS**.
  - **AT-011** (*Create second persona keeps first active*): **PASS**.
  - **AT-012** (*Switch active persona atomic & strictly exactly one active*): **PASS**.
  - **AT-013** (*Strict persona data isolation*): **PASS**.
  - **AT-014** (*Persona deletion with dependency summary and cascade*): **PASS**.
  - **AT-020** (*Create typed fact and source atomically*): **PASS**.
  - **AT-021** (*Mark fact disputed/outdated excluded from AI evidence*): **PASS**.
  - **AT-022** (*Fact delete cascades sources and history cleanly with 0 orphans*): **PASS**.
  - **AT-023** (*Sensitive facts excluded from AI payload absent consent*): **PASS**.
  - **AT-030** (*Add API key: secret stored in Windows Credential Manager; only opaque ref in DB*): **PASS**.
  - **AT-031** (*Remove provider / delete credential: secret removed; failure surfaced; shared protection*): **PASS**.
  - **AT-070** (*Export all local data in documented JSON; excludes API secrets*): **PASS**.
  - **AT-071** (*Delete all app data reports database, temp files, and external backup limits*): **PASS**.

---

## 3. File-file Kunci

- **Windows Credential Manager (SC-020)**:
  - Infrastructure Store Trait: `src-tauri/src/infrastructure/credentials/store.rs`
  - Native Windows Implementation: `src-tauri/src/infrastructure/credentials/windows.rs`
  - In-Memory Mock Store (Tests/Non-Windows): `src-tauri/src/infrastructure/credentials/in_memory.rs`
  - Domain Model: `src-tauri/src/domain/credential.rs`
  - Repository: `src-tauri/src/repository/credential_repo.rs`
  - Service: `src-tauri/src/services/credential_service.rs`
  - Commands IPC: `src-tauri/src/commands/credential.rs`
  - SQL Migrasi 0003: `src-tauri/migrations/0003_app_credentials.sql`
  - Integration Tests Rust: `src-tauri/tests/credential_lifecycle_tests.rs`
  - Frontend Types: `src/types/credential.ts`
  - Frontend Service: `src/services/credentialService.ts`
  - Frontend View: `src/components/views/SettingsView.tsx` (Tab `🔑 Kredensial & Provider AI`)
  - Frontend Tests: `tests/credential_management.test.ts`
  - CSS Styling: `src/styles.css`
- **Ekspor Data & Penghapusan (SC-012)**:
  - Domain Model: `src-tauri/src/domain/export.rs`
  - Repository: `src-tauri/src/repository/export_repo.rs`, `src-tauri/src/repository/deletion_repo.rs`
  - Service: `src-tauri/src/services/export_service.rs`, `src-tauri/src/services/deletion_service.rs`
  - Commands: `src-tauri/src/commands/data_management.rs`
  - Integration Tests Rust: `src-tauri/tests/export_and_deletion_tests.rs`
  - Frontend View: `src/components/views/SettingsView.tsx` (Tab `⬇ Ekspor & Cadangan Data` dan `⚠ Zona Bahaya`)
  - Frontend Tests: `tests/data_management.test.ts`
- **Fakta & Sumber Bukti (SC-011)**:
  - Domain Model: `src-tauri/src/domain/fact.rs`
  - Repository: `src-tauri/src/repository/fact_repo.rs`
  - Service: `src-tauri/src/services/fact_service.rs`
  - Commands: `src-tauri/src/commands/fact.rs`
  - Integration Tests Rust: `src-tauri/tests/fact_lifecycle_tests.rs`
  - Frontend View: `src/components/views/ProfilesView.tsx`
- **Persona Lifecycle (SC-010)**:
  - Domain Model: `src-tauri/src/domain/persona.rs`
  - Repository: `src-tauri/src/repository/persona_repo.rs`
  - Service: `src-tauri/src/services/persona_service.rs`
  - Commands: `src-tauri/src/commands/persona.rs`
  - Integration Tests Rust: `src-tauri/tests/persona_lifecycle_tests.rs`
- **Migrasi Database (SC-003)**:
  - Runner: `src-tauri/src/migrations/mod.rs`
  - Inisialisasi: `src-tauri/src/db.rs`
  - SQL Migrasi: `src-tauri/migrations/0001_initial.sql`, `src-tauri/migrations/0002_typed_facts_and_history.sql`, `src-tauri/migrations/0003_app_credentials.sql`

---

## 4. Keputusan & Aturan Kritis (Non-Negotiable)

- **Tanpa Automasi Browser**: Dilarang membuat browser automation, auto-click, auto-fill, auto-submit, atau bypass platform survei.
- **Integritas Fakta Asli**: Seluruh fakta mewakili kondisi nyata individu sebenarnya; dilarang mengarang identitas responden atau data penipuan screener.
- **Isolasi Mutlak Antar-Persona**: Data persona A tidak boleh bocor atau diakses oleh persona B baik saat interaksi maupun ekspor file.
- **Kredensial Aman**: API secret AI hanya disimpan di Windows Credential Manager; tidak boleh masuk ke SQLite, file ekspor, logs, atau frontend state.
- **Pembersihan Bersih**: Penghapusan data harus menjamin 0 orphan records, checkpoint WAL TRUNCATE, pemadatan VACUUM, dan pembersihan berkas screenshot sementara.

---

## 5. Status Terkini & Langkah Berikutnya

- **Status Git**: Backup branch `backup-head-before-sc020` tersimpan aman.
- **Langkah Berikutnya (Next Steps)**:
  1. **P2 — SC-021: Provider Configuration & Adapter Contracts**:
     - Antarmuka konfigurasi provider AI, Gemini adapter, validasi HTTPS endpoint, connection test, dan penanganan error ternormalisasi.
  2. **P2 — SC-022: Provider Pool**:
     - Manajemen pool key, cooldown, retry terikat, dan proteksi kuota provider.
  3. **P3 — Question-to-Answer Workflow**:
     - Pertanyaan teks manual / tangkapan layar eksplisit ke rekomendasi jawaban berbasis fakta.
