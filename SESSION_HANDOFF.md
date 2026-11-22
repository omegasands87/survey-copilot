# Multi-Project Session Handoff: SURVEY COPILOT

- **Waktu Handoff**: 2026-11-23T01:35:37+08:00
- **ID Percakapan Asal**: `dcd86563-640b-448e-9d95-27d4b9de336f`
- **Nama Proyek & Direktori**: `SURVEY COPILOT` (`D:\Antigravity\Projects\SURVEY COPILOT`)
- **Misi / Tujuan Utama**: Membangun aplikasi desktop Windows 11 x64 Survey Copilot v1.0 yang stabil, aman, evidence-grounded, dan Indonesian-first menggunakan Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, dan Gemini AI adapter.

---

## 1. Pekerjaan yang Telah Diselesaikan (SC-001, SC-002, SC-003, SC-010, SC-011 Selesai 100%)

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
  - **Fact History & Audit Trail**:
    - Tabel baru `fact_history` (Migration `0002_typed_facts_and_history.sql`).
    - Mencatat `previous_value_json`, `new_value_json`, `previous_status`, `new_status`, `change_reason`, `source_id`, `changed_at`.
  - **Duplicate & Conflict Detection**:
    - Deteksi otomatis fakta bertentangan pada `fact_key` yang sama dalam profil aktif.
    - Banner peringatan di antarmuka dan modal resolusi konflik untuk menandai fakta lama sebagai outdated atau menghapusnya.
  - **Strict Persona Scoping & AI Fact Eligibility Policy**:
    - Filter SQL dan validasi service berbasis `persona_id`.
    - Mesin seleksi fakta AI memfilter hanya fakta `confirmed`, masa berlaku aktif, tanpa konflik, dan lulus persetujuan sensitivitas.
  - **UI Berbahasa Indonesia Lengkap**:
    - Form pembuatan fakta bertipe data lengkap dengan pemilih tipe data dinamis.
    - Modal edit nilai fakta dengan kewajiban mengisi alasan perubahan.
    - Modal detail fakta yang menampilkan seluruh sumber bukti dan riwayat perubahan (audit trail).
    - Modal penambahan sumber bukti baru ke fakta yang sudah ada.
    - Modal resolusi konflik fakta.

---

## 2. Hasil Verifikasi Aktual

- `npm.cmd test` (`node --experimental-strip-types --test tests/**/*.test.ts`): **PASS** (13/13 tests pass, exit code 0).
- `npm.cmd run build` (`tsc -b && vite build`): **PASS** (42 modules transformed, exit code 0).
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: **PASS** (exit code 0).
- `cargo +stable-x86_64-pc-windows-gnu check --tests --manifest-path src-tauri/Cargo.toml`: **PASS** (exit code 0).
- `cargo +stable-x86_64-pc-windows-gnu clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: **PASS** (0 warnings, exit code 0).
- Acceptance criteria yang terverifikasi:
  - **AT-001** (*Fresh checkout installs and builds*): **PASS**.
  - **AT-002** (*Fresh DB startup creates tables and _migrations tracking*): **PASS**.
  - **AT-003** (*App restarts with existing DB are idempotent*): **PASS**.
  - **AT-004** (*Migration fails midway triggers rollback, preserving user data*): **PASS**.
  - **AT-005** (*Layered command and IPC decoupling*): **PASS**.
  - **AT-010** (*Create first persona becomes active*): **PASS**.
  - **AT-011** (*Create second persona keeps first active*): **PASS**.
  - **AT-012** (*Switch active persona atomic & strictly exactly one active*): **PASS**.
  - **AT-013** (*Strict persona data isolation: facts of A invisible & inaccessible to B*): **PASS**.
  - **AT-014** (*Persona deletion with dependency summary, cascade, and zero orphan records*): **PASS**.
  - **AT-020** (*Create typed fact and source atomically*): **PASS**.
  - **AT-021** (*Mark fact disputed/outdated excluded from AI evidence*): **PASS**.
  - **AT-022** (*Fact delete cascades sources and history cleanly with 0 orphans*): **PASS**.
  - **AT-023** (*Sensitive facts excluded from AI payload absent consent*): **PASS**.

---

## 3. File-file Kunci

- **Fakta & Sumber Bukti**:
  - Domain Model: `src-tauri/src/domain/fact.rs`
  - Repository: `src-tauri/src/repository/fact_repo.rs`
  - Service: `src-tauri/src/services/fact_service.rs`
  - Commands: `src-tauri/src/commands/fact.rs`, `src-tauri/src/commands/mod.rs`
  - Integration Tests Rust: `src-tauri/tests/fact_lifecycle_tests.rs`
  - Frontend Types: `src/types/fact.ts`
  - Frontend Service: `src/services/factService.ts`
  - Frontend View & UI: `src/components/views/ProfilesView.tsx`, `src/App.tsx`, `src/styles.css`
  - Frontend Tests: `tests/fact_lifecycle.test.ts`
- **Persona Lifecycle**:
  - Domain Model: `src-tauri/src/domain/persona.rs`
  - Repository: `src-tauri/src/repository/persona_repo.rs`
  - Service: `src-tauri/src/services/persona_service.rs`
  - Commands: `src-tauri/src/commands/persona.rs`
  - Integration Tests Rust: `src-tauri/tests/persona_lifecycle_tests.rs`
  - Frontend Types: `src/types/persona.ts`
  - Frontend Service: `src/services/personaService.ts`
  - Frontend Tests: `tests/persona_lifecycle.test.ts`
- **Migrasi Database**:
  - Runner: `src-tauri/src/migrations/mod.rs`
  - Inisialisasi: `src-tauri/src/db.rs`
  - SQL Migrasi v1: `src-tauri/migrations/0001_initial.sql`
  - SQL Migrasi v2: `src-tauri/migrations/0002_typed_facts_and_history.sql`
  - Migration Tests: `src-tauri/tests/migration_tests.rs`
- **Panduan & Dokumentasi**:
  - Backlog: `docs/08-agent-handoff/IMPLEMENTATION_BACKLOG.md`
  - Acceptance Test Matrix: `docs/08-agent-handoff/ACCEPTANCE_TEST_MATRIX.md`
  - Repository Map: `docs/08-agent-handoff/REPOSITORY_MAP.md`
  - Multi-Persona Spec: `docs/01-product/MULTI_PERSONA_SPECIFICATION.md`

---

## 4. Keputusan & Aturan Kritis (Non-Negotiable)

- **Tanpa Automasi Browser**: Dilarang membuat browser automation, auto-click, auto-fill, auto-submit, atau bypass platform survei.
- **Integritas Fakta Asli**: Seluruh fakta mewakili kondisi nyata individu sebenarnya; dilarang mengarang identitas responden atau data penipuan screener.
- **Isolasi Mutlak Antar-Persona**: Data persona A tidak boleh bocor atau diakses oleh persona B.
- **Kredensial Aman**: API secret AI hanya disimpan di Windows Credential Manager; tidak boleh di SQLite atau frontend state.
- **AI Policy**: AI tidak boleh mengonfirmasi fakta secara otomatis, dan dilarang menyajikan jawaban definitif jika fakta disputed, outdated, atau berkonflik.

---

## 5. Status Terkini & Langkah Berikutnya

- **Status Git**: Bersih (`working tree clean`), branch `main` ahead 6 commits dari `origin/main`. Backup branches `backup-head-before-sc011` dan `backup-sc010-clean` tersimpan aman.
- **Langkah Berikutnya (Next Steps)**:
  1. **SC-012 — Data Export and Deletion**:
     - Implementasi ekspor data lokal & per-persona ke format JSON yang terisolasi dan bebas dari API secrets.
     - Implementasi penghapusan total data aplikasi lokal dengan audit laporan pembersihan.
  2. **P2 — SC-020: Windows Credential Manager**:
     - Integrasi native Rust untuk penyimpanan dan pengelolaan rahasia API keys provider AI di Windows Credential Manager.
  3. **P2 — SC-021: Provider Configuration & Pool**:
     - Antarmuka konfigurasi provider AI, Gemini adapter, pooling, retry/cooldown, dan proteksi kuota.
