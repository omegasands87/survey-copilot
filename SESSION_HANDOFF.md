# Multi-Project Session Handoff: SURVEY COPILOT

- **Waktu Handoff**: 2026-11-23T01:30:00+08:00
- **ID Percakapan Asal**: `dcd86563-640b-448e-9d95-27d4b9de336f`
- **Nama Proyek & Direktori**: `SURVEY COPILOT` (`d:\Antigravity\Projects\SURVEY COPILOT`)
- **Misi / Tujuan Utama**: Mengembangkan aplikasi desktop Windows 11 x64 Survey Copilot v1.0 yang stabil, aman, dan evidence-grounded menggunakan Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, dan Gemini AI adapter.

---

## 1. Pekerjaan yang Telah Diselesaikan (SC-001, SC-002, SC-003, SC-010, SC-011 Complete)

- **SC-001 — Establish reproducible builds, lockfiles, dan CI**:
  - `package-lock.json` dan `src-tauri/Cargo.lock` terkunci dan konsisten.
  - CI Windows workflow `.github/workflows/windows.yml` terkonfigurasi dengan `npm ci`, cache, build, dan checks.
- **SC-002 — Introduce application layers & typed IPC**:
  - Commands tipis bebas SQL di `src-tauri/src/commands/`.
  - Domain validation di `src-tauri/src/domain/`.
  - Scoped repository di `src-tauri/src/repository/`.
  - Business logic & service isolation di `src-tauri/src/services/`.
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
    - Validasi kepemilikan data di tingkat backend repository dan service, bukan sekadar filter UI.
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
    - Modal tambah sumber bukti tambahan.
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

## 3. Kondisi Git

- **Branch Aktif**: `main`
- **Status Kerja**: Bersih (`working tree clean`).
- **Commit Lokal Belum di-Push** (ahead 5 commits dari `origin/main`):
  1. `bfc58d1` feat(infra): establish reproducible builds, lockfiles, and CI (SC-001)
  2. `0282b44` feat(arch): introduce application layers and typed IPC client (SC-002)
  3. `f8a74e6` feat(db): replace startup SQL replay with transactional migration runner (SC-003)
  4. `78f3e81` feat(persona): implement persona lifecycle and cross-persona data isolation (SC-010)
  5. `c0072e4` feat(fact): implement typed facts, provenance, verification lifecycle, and conflict detection (SC-011)
- **Backup Branches**:
  - `backup-head-before-sc011`
  - `backup-sc010-clean`

---

## 4. Langkah Kerja Berikutnya

- **SC-012 — Data export and deletion**:
  - Ekspor seluruh data lokal dan data per-persona ke format JSON yang terstruktur.
  - Memastikan kredensial dan API keys dikecualikan secara mutlak dari berkas ekspor.
  - Implementasi penghapusan total data lokal aplikasi dengan konfirmasi dan laporan audit.
  - Verifikasi round-trip ekspor dan penghapusan database/file cache.
- **P2 — SC-020: Windows Credential Manager** (Integrasi penyimpanan aman API secret di Windows).
