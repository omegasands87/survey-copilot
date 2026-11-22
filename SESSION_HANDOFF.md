# Multi-Project Session Handoff: SURVEY COPILOT

- **Waktu Handoff**: 2026-11-23T03:51:35+08:00
- **ID Percakapan Asal**: `97451ee8-b661-4395-9537-8e3991be82cf`
- **Nama Proyek & Direktori**: `SURVEY COPILOT` (`D:\Antigravity\Projects\SURVEY COPILOT`)
- **Misi / Tujuan Utama**: Membangun aplikasi desktop Windows 11 x64 Survey Copilot v1.0 yang stabil, aman, evidence-grounded, dan Indonesian-first menggunakan Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, Gemini AI adapter, dan Provider Pool.

---

## 1. Pekerjaan yang Telah Diselesaikan (SC-001 s/d SC-022 Selesai 100%)

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
- **SC-021 — Provider Configuration and Adapter Contracts**:
  - CRUD Konfigurasi Provider: model domain `ProviderConfig`, repository SQLite `provider_configs`, dan service `ProviderService`. Pengambilan dan penyimpanan konfigurasi HANYA menggunakan referensi kredensial buram (`credential_ref`) dari Windows Credential Manager. Zero secret leakage ke IPC atau frontend.
  - Provider-Neutral Adapter Contract: abstraksi `AiProviderAdapter` trait dengan tipe request (`NormalizedRequest`), response (`NormalizedResponse`), dan finish reason (`NormalizedFinishReason`) yang ternormalisasi secara universal.
  - Google Gemini Adapter: endpoint resmi `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`, pemetaan structured JSON output schema via parameter `responseSchema` Gemini, token usage tracking, latency tracking, dan penanganan status error Google API.
  - Endpoint Validation & SSRF Prevention: modul `EndpointValidator` menegakkan default HTTPS untuk seluruh endpoint eksternal. Plain HTTP HANYA diizinkan untuk host loopback localhost (`127.0.0.1`, `::1`, `localhost`).
  - Proteksi Redirect Cross-Host: `DefaultHttpTransport` menolak pengalihan (*redirect*) HTTP lintas origin untuk mencegah kebocoran secret API ke pihak ketiga.
  - OpenAI-Compatible Adapter: dukungan endpoint OpenAI-compatible lokal/kustom.
- **SC-022 — Provider Pool (SELESAI 100%)**:
  - 4 Strategi Seleksi: `Adaptive Availability-Aware` (composite fitness: latensi, beban konkurensi, prioritas, pinalti kegagalan), `Round-Robin` (rotasi deterministik atomic), `Weighted` (distribusi proporsional bobot relatif), dan `Priority` (urut prioritas tertinggi).
  - Provider Health Tracking: 5 status normatif (`ready`, `cooling_down`, `quota_exhausted`, `needs_attention`, `disabled`).
  - Per-Key Cooldown & Retry-After (AT-033): menghormati header `Retry-After`, bounded exponential backoff (`retry_base_delay_ms` s/d `max_retry_delay_ms`), reset cooldown kadaluarsa otomatis, dan failover ke key sehat berikutnya saat terjadi 429 transient.
  - Bounded Concurrency: in-memory concurrency tracker thread-safe dengan RAII `ConcurrencyGuard` yang otomatis mendekremen saat request selesai/batal/panic. Mencegah dispatch jika konkurensi mencapai `max_concurrency`.
  - Timeout & Cancellation (AT-036): `CancellationToken` thread-safe seketika menghentikan queued retries tanpa request tersembunyi.
  - Kebijakan Kuota & Keamanan Mutlak (AT-034 & AT-035): pembatasan kuota tingkat akun/project Google Cloud (`QuotaExhausted` / `RESOURCE_EXHAUSTED` harian) langsung menghentikan retry pada seluruh pool untuk mematuhi ketentuan penyedia; permintaan tidak valid (`InvalidRequest`), penolakan konten (`ContentRefused`), dan auth failure tidak di-retry ke key lain.
  - Preservasi Input Pengguna Mutlak (AT-040 boundary): `PoolExecutionResult` pada semua jalur kegagalan mengembalikan objek `preserved_request` yang menjamin draf pertanyaan responden tidak pernah hilang.
  - Zero Credential Leakage: seluruh log percobaan, ringkasan health, dan payload IPC 0% memuat secret/API token mentah.
  - UI Pengaturan Provider Pool di SettingsView: kontrol strategi, batas percobaan (1-10), toggle auto-retry, delay retry, badge status health live, dan counter konkurensi aktif.

---

## 2. Bukti Verifikasi dan Pengujian

- **Frontend Build (`npm run build`)**: **PASS** (46 modul ditransformasikan, bundle dist valid, 0 error).
- **Frontend Automated Tests (`npm test`)**: **PASS (39/39 tests passed, 0 failures, 491ms)**:
  - `tests/provider_pool.test.ts` (7 tests).
  - `tests/provider_configuration.test.ts` (7 tests).
  - `tests/credential_management.test.ts` (6 tests).
  - `tests/data_management.test.ts` (6 tests).
  - `tests/error_normalization.test.ts` (4 tests).
  - `tests/fact_lifecycle.test.ts` (6 tests).
  - `tests/persona_lifecycle.test.ts` (3 tests).
- **Rust Code Formatting (`cargo fmt --check`)**: **PASS** (Exit code 0, zero diffs di seluruh kode backend).
- **Rust Type Check & Test Harness Compilation (`cargo check --tests`)**: **PASS** (Exit code 0, 0 errors, 0 warnings).
- **Rust Clippy Strict Linter (`cargo clippy --lib --tests -- -D warnings`)**: **PASS** (Exit code 0, 0 warnings).
- **Rust Integration Tests (`src-tauri/tests/provider_pool_tests.rs` & `src-tauri/tests/provider_adapter_tests.rs`)**:
  - `test_at_032_gemini_structured_request_normalized_and_returned`
  - `test_at_033_pool_failover_on_429_rate_limited_and_respect_retry_after`
  - `test_at_034_account_project_quota_exhausted_stops_pool_retries_and_preserves_request`
  - `test_at_035_invalid_request_and_content_refusal_do_not_rotate_through_keys`
  - `test_at_036_user_cancellation_stops_queued_retries`
  - `test_at_037_http_endpoint_outside_localhost_rejected_default_https_only`
  - `test_at_038_redirect_to_another_host_never_forwards_credentials`
  - `test_selection_strategies_round_robin_weighted_priority_adaptive`
  - `test_zero_credential_leakage_in_pool_results_and_history`
- **Acceptance Criteria Terpenuhi**:
  - AT-001 s/d AT-005, AT-010 s/d AT-014, AT-020 s/d AT-023: **PASS**
  - AT-030 (*Add API key: secret stored in Windows Credential Manager; only opaque ref in DB*): **PASS**
  - AT-031 (*Remove provider / delete credential: secret removed; shared protection*): **PASS**
  - AT-032 (*Gemini structured request: normalized response validated and returned*): **PASS**
  - AT-033 (*Respect Retry-After/cooldown and fail over if permitted*): **PASS**
  - AT-034 (*Account/project quota exhausted: stop retries; preserve question*): **PASS**
  - AT-035 (*Invalid request/content refusal: do not rotate through all keys*): **PASS**
  - AT-036 (*User cancellation: queued retries stop; no hidden request continues*): **PASS**
  - AT-037 (*HTTP endpoint outside localhost: reject unless explicitly allowed; default HTTPS only*): **PASS**
  - AT-038 (*Redirect to another host: never forward API credentials to unapproved origin*): **PASS**
  - AT-070 (*Export all local data in documented JSON; excludes API secrets*): **PASS**
  - AT-071 (*Delete all app data reports database, temp files, and external backup limits*): **PASS**

---

## 3. File-file Kunci Proyek

- **Backend Rust**:
  - Model Domain: `src-tauri/src/domain/pool.rs`, `src-tauri/src/domain/provider.rs`, `src-tauri/src/domain/endpoint_validator.rs`
  - Service: `src-tauri/src/services/provider_pool.rs`, `src-tauri/src/services/provider_service.rs`
  - Repository SQLite: `src-tauri/src/repository/pool_repo.rs`, `src-tauri/src/repository/provider_repo.rs`
  - Tauri Commands: `src-tauri/src/commands/provider_pool.rs`, `src-tauri/src/commands/provider.rs`
  - Transport & Adapters: `src-tauri/src/infrastructure/http/mod.rs`, `src-tauri/src/infrastructure/adapters/gemini.rs`
  - Integration Tests: `src-tauri/tests/provider_pool_tests.rs`, `src-tauri/tests/provider_adapter_tests.rs`
- **Frontend React & TypeScript**:
  - Definisi Tipe: `src/types/providerPool.ts`, `src/types/provider.ts`
  - Services: `src/services/providerPoolService.ts`, `src/services/providerService.ts`
  - UI Views: `src/components/views/SettingsView.tsx`
  - Unit Tests: `tests/provider_pool.test.ts`, `tests/provider_configuration.test.ts`
- **Dokumentasi & Backlog**:
  - `docs/08-agent-handoff/IMPLEMENTATION_BACKLOG.md`
  - `docs/08-agent-handoff/ACCEPTANCE_TEST_MATRIX.md`
  - `docs/08-agent-handoff/REPOSITORY_MAP.md`
  - `docs/02-architecture/API_PROVIDER_POOLING.md`

---

## 4. Keputusan & Aturan Kritis (Non-Negotiable)

- **Tanpa Automasi Browser**: Dilarang membuat browser automation, auto-click, auto-fill, auto-submit, atau bypass platform survei.
- **Integritas Fakta Asli**: Seluruh fakta mewakili kondisi nyata responden sebenarnya; dilarang mengarang identitas responden atau data penipuan screener.
- **Isolasi Mutlak Antar-Persona**: Data persona A tidak boleh bocor atau diakses oleh persona B baik saat interaksi maupun ekspor file.
- **Kredensial Aman**: API secret AI hanya disimpan di Windows Credential Manager; tidak boleh masuk ke SQLite, file ekspor, logs, atau frontend state.
- **Kepatuhan Kuota Provider**: Dilarang menggunakan pool untuk menghindari batas free-tier, billing controls, atau project/account quotas. Hentikan retry seketika jika kuota akun habis.
- **Preservasi Input**: Input dan draf pengguna wajib selalu dipertahankan utuh pada semua skenario kegagalan provider.

---

## 5. Status Terkini & Langkah Berikutnya

- **Status Git**: Bersih (`working tree clean`), commit `b6a88aa` pada branch `main`.
- **Langkah Berikutnya (Next Steps)**:
  1. **P3 — SC-030: Survey sessions and manual question input**:
     - Sesi survei ber-scope persona aktif (`create`, `list`, `update`, `archive`, `delete`).
     - Persistensi entri teks pertanyaan manual, opsi jawaban, instruksi, format, dan status konfirmasi.
     - Preservasi draf saat provider gagal dan restart aplikasi.
  2. **SC-031 — Question classifier and constraint extraction**:
     - Normalisasi deterministik dan output classifier bertipe kuat.
     - Ekstraksi format jawaban, unit, timeframe, range, dan ambiguitas.
  3. **SC-032 — Evidence retrieval and consistency engine**:
     - Pengambilan fakta relevan, terverifikasi, dan non-kadaluarsa ber-scope persona aktif.
     - Deteksi kontradiksi profil dan fakta sensitif.
