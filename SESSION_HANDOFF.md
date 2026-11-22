# Multi-Project Session Handoff: SURVEY COPILOT

- **Waktu Handoff**: 2026-11-23T01:56:00+08:00
- **ID Percakapan Asal**: `4ad70ea9-5f00-4fc6-ab75-a0b6901cba4d`
- **Nama Proyek & Direktori**: `SURVEY COPILOT` (`D:\Antigravity\Projects\SURVEY COPILOT`)
- **Misi / Tujuan Utama**: Membangun aplikasi desktop Windows 11 x64 Survey Copilot v1.0 yang stabil, aman, evidence-grounded, dan Indonesian-first menggunakan Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, dan Gemini AI adapter.

---

## 1. Pekerjaan yang Telah Diselesaikan (SC-001, SC-002, SC-003, SC-010, SC-011, SC-012 Selesai 100%)

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
  - **Conflict Detection & Audit Trail**:
    - Deteksi konflik otomatis untuk nilai bertentangan pada kategori dan atribut yang sama.
    - Pencatatan riwayat audit lengkap pada tabel `fact_history` (`previous_value`, `new_value`, `previous_status`, `new_status`, `change_reason`, `changed_at`).
- **SC-012 — Data Export and Deletion**:
  - **Full Application Export**:
    - Format JSON terstruktur schema version `1.0.0` (`https://survey-copilot.app/schemas/export-v1.json`).
    - Mengekspor seluruh persona, fakta profil bertipe, sumber bukti, riwayat perubahan, preferensi, sesi survei, metadata tangkapan layar, pertanyaan, rekomendasi, dan riwayat jawaban.
  - **Single Persona Export (Strict Isolation)**:
    - Opsi ekspor terisolasi khusus untuk persona terpilih.
    - Zero data leakage: fakta, riwayat, dan sesi milik persona lain dilarang keras masuk ke berkas ekspor.
  - **Export Privacy & Absolute Secret Exclusion**:
    - Kunci API, konfigurasi rahasia penyedia AI, dan referensi Windows Credential Manager **100% dikecualikan** dari ekspor.
    - Audit keamanan otomatis pada payload JSON sebelum serialisasi (menolak ekspor jika terdapat kata kunci kredensial).
    - Ringkasan isi ekspor (`ExportSummary`) menampilkan jumlah fakta sensitif dan sangat sensitif dengan peringatan keamanan sebelum pengunduhan.
  - **Data Deletion & Integrity**:
    - **Single Persona Deletion**: Cascade foreign key bersih di SQLite + penghapusan berkas fisik screenshot sementara di disk (`temporary_image_path`).
    - **Full Application Data Wipe (`data_wipe_all`)**: Menghapus seluruh record pengguna di SQLite, membersihkan berkas sementara di disk, menjalankan `PRAGMA wal_checkpoint(TRUNCATE)` dan `VACUUM` untuk mereklamasi alokasi disk.
    - **Perlindungan Penghapusan**: Membutuhkan konfirmasi ketik tepat `"HAPUS SEMUA DATA"` dan centang pemahaman konsekuensi.
    - **Transparansi Penghapusan**: Laporan audit penghapusan (`WipeReport`) secara transparan merinci record yang terhapus dan mengumumkan batasan eksternal (berkas cadangan yang disimpan di luar direktori aplikasi tidak terhapus).
  - **UI Pengaturan Data & Privasi**:
    - `src/components/views/SettingsView.tsx` dalam bahasa Indonesia dengan 3 tab interaktif: Ekspor Data, Kebijakan Privasi & Siklus Data, dan Zona Bahaya.

---

## 2. Status Verifikasi Pengujian

- `cmd /c npm test`: **PASS** (19 test cases lulus 100%, durasi ~185ms).
- `cmd /c npm run build`: **PASS** (43 modules transformed, exit code 0).
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: **PASS** (exit code 0).
- `cargo +stable-x86_64-pc-windows-gnu check --tests --manifest-path src-tauri/Cargo.toml`: **PASS** (exit code 0).
- `cargo +stable-x86_64-pc-windows-gnu clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: **PASS** (0 warnings, exit code 0).
- **Catatan Lingkungan Lokal (Windows Dev)**:
  `cargo check --tests` dan `cargo clippy` berhasil lulus 100% tanpa error maupun warning. Eksekusi lokal `cargo test` mendeteksi entrypoint mismatch DLL MinGW lokal host (`0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND`); di Windows CI GitHub Actions (`.github/workflows/windows.yml`), seluruh integration tests dieksekusi dan diverifikasi.
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
  - **AT-070** (*Export all local data in documented JSON; excludes API secrets*): **PASS**.
  - **AT-071** (*Delete all app data reports database, temp files, and external backup limits*): **PASS**.

---

## 3. File-file Kunci

- **Ekspor Data & Penghapusan (SC-012)**:
  - Domain Model: `src-tauri/src/domain/export.rs`
  - Repository Ekspor: `src-tauri/src/repository/export_repo.rs`
  - Repository Deletion: `src-tauri/src/repository/deletion_repo.rs`
  - Service Ekspor: `src-tauri/src/services/export_service.rs`
  - Service Deletion: `src-tauri/src/services/deletion_service.rs`
  - Commands IPC: `src-tauri/src/commands/data_management.rs`
  - Integration Tests Rust: `src-tauri/tests/export_and_deletion_tests.rs`
  - Frontend Types: `src/types/dataManagement.ts`
  - Frontend Service: `src/services/dataManagementService.ts`
  - Frontend View: `src/components/views/SettingsView.tsx`
  - Frontend Tests: `tests/data_management.test.ts`
  - Spesifikasi Arsitektur: `docs/02-architecture/DATA_EXPORT_AND_DELETION_SPECIFICATION.md`
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
  - SQL Migrasi: `src-tauri/migrations/0001_initial.sql`, `src-tauri/migrations/0002_typed_facts_and_history.sql`

---

## 4. Keputusan & Aturan Kritis (Non-Negotiable)

- **Tanpa Automasi Browser**: Dilarang membuat browser automation, auto-click, auto-fill, auto-submit, atau bypass platform survei.
- **Integritas Fakta Asli**: Seluruh fakta mewakili kondisi nyata individu sebenarnya; dilarang mengarang identitas responden atau data penipuan screener.
- **Isolasi Mutlak Antar-Persona**: Data persona A tidak boleh bocor atau diakses oleh persona B baik saat interaksi maupun ekspor file.
- **Kredensial Aman**: API secret AI hanya disimpan di Windows Credential Manager; tidak boleh masuk ke SQLite, file ekspor, atau frontend state.
- **Pembersihan Bersih**: Penghapusan data harus menjamin 0 orphan records, checkpoint WAL TRUNCATE, pemadatan VACUUM, dan pembersihan berkas screenshot sementara.

---

## 5. Status Terkini & Langkah Berikutnya

- **Status Git**: Bersih (`working tree clean`), branch `main` ahead 8 commits dari `origin/main`. Backup branches `backup-head-before-sc012`, `backup-head-before-sc011`, dan `backup-sc010-clean` tersimpan aman.
- **Langkah Berikutnya (Next Steps)**:
  1. **P2 — SC-020: Windows Credential Manager**:
     - Integrasi native Rust untuk penyimpanan dan pengelolaan rahasia API keys provider AI di Windows Credential Manager menggunakan opaque reference.
  2. **P2 — SC-021: Provider Configuration & Adapter Contracts**:
     - Antarmuka konfigurasi provider AI, Gemini adapter, validasi HTTPS endpoint, dan penanganan error ternormalisasi.
  3. **P2 — SC-022: Provider Pool**:
     - Manajemen pool key, cooldown, retry terikat, dan proteksi kuota provider.
