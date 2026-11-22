# Multi-Project Session Handoff: SURVEY COPILOT

- **Waktu Handoff**: 2026-11-23T03:17:30+08:00
- **ID Percakapan Asal**: `fa131863-0d89-4e71-8652-74f80b4fafa0`
- **Nama Proyek & Direktori**: `SURVEY COPILOT` (`D:\Antigravity\Projects\SURVEY COPILOT`)
- **Misi / Tujuan Utama**: Membangun aplikasi desktop Windows 11 x64 Survey Copilot v1.0 yang stabil, aman, evidence-grounded, dan Indonesian-first menggunakan Tauri 2, Rust, React, TypeScript, SQLite, Windows Credential Manager, dan Gemini AI adapter.

---

## 1. Pekerjaan yang Telah Diselesaikan (SC-001 s/d SC-021 Selesai 100%)

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
- **SC-021 — Provider Configuration and Adapter Contracts**:
  - **CRUD Konfigurasi Provider**: Model domain `ProviderConfig`, repository SQLite `provider_configs`, dan service `ProviderService`. Pengambilan dan penyimpanan konfigurasi HANYA menggunakan referensi kredensial buram (`credential_ref`) dari Windows Credential Manager. Zero secret leakage ke IPC atau frontend.
  - **Provider-Neutral Adapter Contract**: Abstraksi `AiProviderAdapter` trait dengan tipe request (`NormalizedRequest`), response (`NormalizedResponse`), dan finish reason (`NormalizedFinishReason`) yang ternormalisasi secara universal.
  - **Gemini Adapter (Official Google API)**: Endpoint resmi `https://generativelanguage.googleapis.com/v1beta/models/{model_id}:generateContent`, pengiriman API key via header aman `x-goog-api-key`, penanganan `generationConfig` (responseMimeType application/json & responseSchema untuk structured output), dan ekstraksi penggunaan token (`promptTokenCount`, `candidatesTokenCount`).
  - **OpenAI-Compatible Adapter**: Mendukung endpoint eksternal/lokal (Ollama, vLLM, OpenAI, Groq, OpenRouter) via standar `/v1/chat/completions` dengan header `Authorization: Bearer <API_KEY>`.
  - **Validasi Custom Endpoint & SSRF Protection (AT-037)**: `EndpointValidator` menegakkan HTTPS wajib untuk endpoint internet. HTTP diizinkan secara ketat HANYA untuk `localhost`, `127.0.0.1`, atau `[::1]`. Pemblokiran ketat terhadap skema berbahaya (`file:`, `ftp:`, `javascript:`, `data:`) dan IP metadata cloud internal (`169.254.169.254`, `metadata.google.internal`).
  - **Perlindungan Pengalihan Kredensial (AT-038)**: Kebijakan redirect ketat pada transport HTTP mencegah pengalihan lintas origin host berbeda sehingga rahasia otentikasi API tidak pernah tercuri atau terkirim ke host lain.
  - **Normalisasi Error**: Seluruh error provider dipetakan ke kategori standar: `rate_limited`, `quota_exhausted`, `authentication_failed`, `permission_denied`, `model_unavailable`, `invalid_request`, `content_refused`, `timeout`, `network_error`, `provider_server_error`, `response_parse_error`, `cancelled`, `unknown`.
  - **Connection Testing & Model Configuration**: Fitur uji koneksi real-time (`provider_config_test_connection`) yang membaca secret langsung dari OS Credential Manager di backend Rust tanpa membeberkan secret, mencatat latensi (ms), memperbarui tabel `provider_health`, serta menyediakan daftar model rekomendasi (`gemini-2.5-flash`, `gemini-2.5-pro`, `gemini-1.5-flash`, dll.).
  - **UI Pengaturan Provider Berbahasa Indonesia**: Kartu provider interaktif di `SettingsView.tsx`, tombol uji koneksi langsung, pemilihan model dropdown/kustom, toggle aktif/nonaktif, dan dialog ubah/hapus aman.

---

## 2. Bukti Verifikasi dan Pengujian

- **Frontend Build (`npm run build`)**: **PASS** (45 modul ditransformasikan, bundle dist valid, 0 error).
- **Frontend Automated Tests (`npm test`)**: **PASS (32/32 tests passed, 0 failures, 228ms)**:
  - `tests/provider_configuration.test.ts` (7 tests).
  - `tests/credential_management.test.ts` (6 tests).
  - `tests/data_management.test.ts` (6 tests).
  - `tests/error_normalization.test.ts` (4 tests).
  - `tests/fact_lifecycle.test.ts` (6 tests).
  - `tests/persona_lifecycle.test.ts` (3 tests).
- **Rust Code Formatting (`cargo fmt --check`)**: **PASS** (zero diffs di seluruh kode backend).
- **Rust Integration Tests (`src-tauri/tests/provider_adapter_tests.rs`)**:
  - `test_at_032_gemini_structured_request_normalized_and_returned`
  - `test_at_037_http_endpoint_outside_localhost_rejected_default_https_only`
  - `test_at_038_redirect_to_another_host_never_forwards_credentials`
  - `test_gemini_error_normalization`
  - `test_provider_config_crud_and_credential_integration`
- **Acceptance Criteria Terpenuhi**:
  - AT-001 s/d AT-005, AT-010 s/d AT-014, AT-020 s/d AT-023: **PASS**
  - AT-030 (*Add API key: secret stored in Windows Credential Manager; only opaque ref in DB*): **PASS**
  - AT-031 (*Remove provider / delete credential: secret removed; shared protection*): **PASS**
  - AT-032 (*Gemini structured request: normalized response validated and returned*): **PASS**
  - AT-037 (*HTTP endpoint outside localhost: reject unless explicitly allowed; default HTTPS only*): **PASS**
  - AT-038 (*Redirect to another host: never forward API credentials to unapproved origin*): **PASS**
  - AT-070 (*Export all local data in documented JSON; excludes API secrets*): **PASS**
  - AT-071 (*Delete all app data reports database, temp files, and external backup limits*): **PASS**

---

## 3. File-file Kunci SC-021

- **Backend Rust**:
  - Validasi Endpoint & SSRF: `src-tauri/src/domain/endpoint_validator.rs`
  - Model Domain Provider: `src-tauri/src/domain/provider.rs`
  - HTTP Transport Abstraction & Protection: `src-tauri/src/infrastructure/http/mod.rs`
  - Trait & Factory Adapter: `src-tauri/src/infrastructure/adapters/mod.rs`
  - Google Gemini Adapter: `src-tauri/src/infrastructure/adapters/gemini.rs`
  - OpenAI-Compatible Adapter: `src-tauri/src/infrastructure/adapters/openai_compatible.rs`
  - SQLite Repository Provider & Health: `src-tauri/src/repository/provider_repo.rs`
  - Service Provider: `src-tauri/src/services/provider_service.rs`
  - IPC Commands: `src-tauri/src/commands/provider.rs`
  - Migrasi Skema 0004: `src-tauri/migrations/0004_provider_config_support.sql`
  - Rust Integration Tests: `src-tauri/tests/provider_adapter_tests.rs`
  - Registrasi Lib & State: `src-tauri/src/lib.rs`
- **Frontend React & TypeScript**:
  - Definisi Tipe: `src/types/provider.ts`
  - IPC Service: `src/services/providerService.ts`
  - UI Pengaturan: `src/components/views/SettingsView.tsx`
  - Styling: `src/styles.css`
  - Automated Tests: `tests/provider_configuration.test.ts`

---

## 4. Keputusan & Aturan Kritis (Non-Negotiable)

- **Tanpa Automasi Browser**: Dilarang membuat browser automation, auto-click, auto-fill, auto-submit, atau bypass platform survei.
- **Integritas Fakta Asli**: Seluruh fakta mewakili kondisi nyata individu sebenarnya; dilarang mengarang identitas responden atau data penipuan screener.
- **Isolasi Mutlak Antar-Persona**: Data persona A tidak boleh bocor atau diakses oleh persona B baik saat interaksi maupun ekspor file.
- **Kredensial Aman**: API secret AI hanya disimpan di Windows Credential Manager; tidak boleh masuk ke SQLite, file ekspor, logs, atau frontend state.
- **SSRF & Redirect Protection**: Custom endpoint luar wajib HTTPS; redirect lintas origin ditolak untuk mencegah kebocoran kunci API.

---

## 5. Status Terkini & Langkah Berikutnya

- **Status Git**: Bersih (`working tree clean`), commit `c794951` pada branch `main`.
- **Langkah Berikutnya (Next Steps)**:
  1. **P2 — SC-022: Provider Pool**:
     - Implementasi adaptive, round-robin, weighted, dan priority selection strategy.
     - Pengelolaan cooldown, batas konkurensi (bounded concurrency), timeout, dan pembatalan (cancellation).
     - Deteksi batas kuota akun/project Google Cloud (berhenti retry jika kuota akun habis, hindari rotasi kunci yang melanggar ketentuan penyedia).
     - Penanganan header `Retry-After`.
     - Preservasi input pengguna jika seluruh provider mengalami kegagalan.
  2. **P3 — Question-to-Answer Workflow (SC-030 s/d SC-034)**:
     - Survey sessions & manual question text entry.
     - Question classifier & constraint extraction.
     - Evidence retrieval & consistency engine ber-scope persona.
     - Structured AI answer recommendation.
