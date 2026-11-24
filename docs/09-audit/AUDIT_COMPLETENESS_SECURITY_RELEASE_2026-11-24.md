# Audit Kelengkapan, Keamanan, dan Kesiapan Rilis — 24 November 2026

**Repository:** omegasands87/survey-copilot  
**Branch dasar yang diperiksa:** main  
**Commit snapshot:** 239590dceb726b8619d4ec5ac19e8249f0c73a71  
**Workflow evidence:** Windows build #60  
**Jenis audit:** read-only repository audit; tidak menjalankan binary lokal  
**Status dokumen:** temuan audit; bukan sertifikasi keamanan atau persetujuan rilis

## 1. Ringkasan eksekutif

Repository memiliki implementasi aplikasi yang luas: React/TypeScript, Tauri 2/Rust, SQLite dengan 13 migrasi, provider adapters dan pool, persona lifecycle, answer engine, screen capture/OCR, Survey Memory, Next Question Intelligence, privacy settings, dan test suites.

Windows CI run #60 pada commit snapshot berhasil. Log job memperlihatkan 130 frontend tests dan 221 Rust tests lulus, selain frontend build, Rust formatting, dan Rust compile check.

Namun, keberhasilan CI tidak sama dengan bukti bahwa installer dapat dipasang, aplikasi dapat digunakan pada Windows 11 x64, atau semua release gate terpenuhi. Pada snapshot yang diperiksa:
- GitHub Releases tidak memiliki release asset.
- Direktori release-testing yang dirujuk README tidak ada di tree main.
- SC-062 masih memiliki clean-install, launch, update, uninstall, dan data-retention manual test yang belum dicentang.
- Audit menemukan kekurangan yang berpengaruh pada kebenaran OCR, SSRF protection, credential cleanup, dan integritas data wipe.
- Pengguna kemudian melaporkan masalah UI/UX dan uninstall; laporan pengguna dicatat terpisah dalam INSTALLATION_AND_UI_UX_FEEDBACK_2026-11-24.md.

**Keputusan audit:** NO-GO untuk production release. Persiapan controlled QA baru layak setelah temuan Critical/High yang relevan ditangani dan aplikasi/installer diuji pada Windows 11 x64.

## 2. Metode dan batasan

Pemeriksaan dilakukan terhadap file dan metadata repository GitHub, termasuk:
- README, AGENTS, dokumentasi produk/arsitektur/security/design, handoff, backlog, acceptance matrix, repository map, dan addendum Survey Memory/NQI.
- Source code Rust/TypeScript, migrasi SQL, konfigurasi Tauri, serta workflow CI.
- Metadata GitHub Releases dan workflow run terbaru.
- Log job Windows run #60.

Tidak dilakukan:
- Perubahan source code atau dokumentasi pada snapshot audit ini.
- Instalasi atau eksekusi binary pada komputer Windows milik auditor.
- Pengujian native Credential Manager melalui aplikasi terpasang.
- Live request ke Gemini atau provider eksternal.
- Pengujian screenshot/OCR nyata pada desktop Windows.
- Clean install, upgrade, uninstall, maupun data-preservation test.

Status menggunakan empat kategori:
- **Complete:** implementasi dan bukti otomatis/repository yang relevan tersedia.
- **Partial:** implementasi terlihat tetapi ada gap, temuan, atau acceptance yang belum terpenuhi.
- **Not implemented:** tidak ditemukan implementasi untuk ruang lingkup yang dinilai.
- **Not verified:** bukti runtime/manual yang diperlukan belum tersedia.

## 3. Project completeness — implementation backlog

Backlog memuat 24 milestone bernama SC, bukan setiap angka berurutan dari SC-001 hingga SC-062.

| Fase | ID | Hasil audit | Catatan utama |
|---|---|---|---|
| P0 | SC-001 | Complete | Lockfiles, build, test dan CI tersedia |
| P0 | SC-002 | Complete | Domain/service/repository/typed IPC tersedia |
| P0 | SC-003 | Complete | Migration runner transaksional dan checksum |
| P1 | SC-010 | Complete | Persona lifecycle dan isolation tests |
| P1 | SC-011 | Complete | Fact/source lifecycle, verification dan sensitivity |
| P1 | SC-012 | Partial | Export/delete ada; wipe tidak membersihkan seluruh credential/record |
| P2 | SC-020 | Partial | Windows Credential Manager FFI ada; native installed-app round-trip belum diverifikasi |
| P2 | SC-021 | Partial | Provider adapters tersedia; SSRF/endpoint validation belum cukup kuat |
| P2 | SC-022 | Complete | Provider pool, cooldown, retry, cancellation dan mock tests |
| P3 | SC-030 | Complete | Session/question persistence |
| P3 | SC-031 | Complete | Classifier dan constraint extraction |
| P3 | SC-032 | Complete | Evidence retrieval, persona scoping dan conflict handling |
| P3 | SC-033 | Complete | Structured answer validation dan abstention |
| P3 | SC-034 | Complete | Review actions dan history |
| P4 | SC-040 | Partial | Windows capture implementation ada; native interactive test belum diverifikasi |
| P4 | SC-041 | Partial | OCR/layout code ada; fixture fallback dapat memberi teks sintetis pada runtime |
| P5 | SC-050 | Complete | Opportunity capture dan criteria extraction |
| P5 | SC-051 | Partial | Fit evaluator ada; umur dari tanggal lahir memakai tahun tetap 2026 |
| P5 | SC-052 | Complete | Historical outcome learning dan sparse-data guards |
| P5 | SC-053 | Complete | Survey Memory persistence, recognition dan answer separation tests |
| P5 | SC-054 | Complete | Branch transitions dan next-question prediction tests |
| P6 | SC-060 | Partial | Privacy/consent/redaction ada; disclosure dan deletion memiliki gap |
| P6 | SC-061 | Partial | UI accessibility contracts ada; end-user usability belum tervalidasi |
| P6 | SC-062 | Partial | NSIS config/package diklaim ada; artifact tidak tersedia untuk inspeksi dan manual install gates terbuka |

### Acceptance matrix dan addendum

Acceptance matrix berisi 53 ID pengujian (AT-001 hingga AT-076 dengan rentang nomor tidak berurutan). Test otomatis luas untuk migration, persona isolation, provider, answer engine, capture/OCR contracts, fit, learning, Survey Memory, NQI, privacy, dan UI accessibility.

Beberapa acceptance yang tetap **Not verified**:
- AT-072: clean Windows 11 x64 install dan primary workflows.
- AT-073: uninstall dan perilaku preservasi/penghapusan data.
- Runtime native capture dan OCR pada screenshot sebenarnya.
- Native Credential Manager round-trip dari installed app.
- Perilaku keyboard/screen reader dan seluruh UI di Windows desktop.
- Live Gemini provider connection.

Addendum Survey Memory dan NQI telah dihubungkan ke backlog, acceptance matrix, migrasi, repository/service, dan UI. Automated tests memverifikasi ordered questions, recognition evidence, answer separation, persona isolation, prediction labeling, dan sparse-history behavior. Ini bukti automated contract/domain behavior, bukan pengukuran akurasi prediksi pada populasi survei nyata.

## 4. Temuan audit teknis

### R-01 — Critical: fallback OCR produksi dapat mengeluarkan pertanyaan sintetis

**Lokasi:**
- src-tauri/src/lib.rs
- src-tauri/src/infrastructure/ocr/composite.rs
- src-tauri/src/infrastructure/ocr/fixture.rs
- src-tauri/src/services/ocr_service.rs

WindowsMediaOcrEngine dipasang sebagai engine primer, sedangkan FixtureOcrEngine dipasang sebagai fallback. FixtureOcrEngine memiliki default synthetic response yang berisi pertanyaan contoh tentang usia dan beberapa pilihan jawaban. CompositeOcrEngine memanggil fallback jika engine primer tidak tersedia atau gagal.

**Dampak:** OCR dapat menampilkan pertanyaan yang bukan berasal dari screenshot pengguna. Pengguna dapat keliru menganggap konten sintetis sebagai hasil pembacaan layar.

**Rekomendasi:** Fixture engine hanya boleh digunakan oleh unit/integration tests. Di production, OCR unavailable/failure harus menghasilkan error atau insufficient extraction, tidak pernah synthetic content. Tambahkan regression test untuk memastikan kondisi ini.

### R-02 — High: endpoint validator tidak cukup untuk proteksi SSRF yang kuat

**Lokasi:**
- src-tauri/src/domain/endpoint_validator.rs
- src-tauri/src/infrastructure/http/mod.rs

Validator mem-parsing host menggunakan pemisahan string dan memblokir beberapa metadata host/link-local tertentu. Ia tidak secara menyeluruh menolak semua alamat private/reserved IPv4/IPv6 atau memastikan hasil DNS tetap publik. HTTPS menuju host private seperti alamat RFC1918 tidak otomatis ditolak oleh pemeriksaan saat ini.

**Dampak:** Custom endpoint dapat diarahkan ke service internal atau tujuan yang tidak semestinya. Kebijakan ini belum layak disebut perlindungan SSRF yang lengkap.

**Rekomendasi:** Gunakan URL parser standar; tolak loopback, private, link-local, multicast, reserved, IPv6 ULA dan metadata ranges; evaluasi IP hasil DNS; tangani DNS rebinding; dan pastikan redirect tidak mengirim credential ke origin yang tidak disetujui.

### R-03 — High: full data wipe tidak membersihkan credential OS maupun seluruh metadata credential

**Lokasi:**
- src-tauri/src/commands/data_management.rs
- src-tauri/src/services/deletion_service.rs
- src-tauri/src/repository/deletion_repo.rs
- src-tauri/src/services/credential_service.rs

Command data_wipe_all tidak menerima CredentialStoreState dan DeletionRepository tidak menghapus record app_credentials. Penghapusan provider_configs/app_settings tidak menghapus secret yang ada di Windows Credential Manager.

**Dampak:** Sesudah pengguna memilih full wipe, credential metadata dan/atau secret dapat tertinggal. Pengguna dapat salah memahami arti “hapus semua data”.

**Rekomendasi:** Definisikan scope wipe secara eksplisit; enumerasi credential yang dikelola aplikasi; hapus secret OS dan metadata SQLite; laporkan setiap kegagalan; lakukan verifikasi pasca-wipe. Bila ada pilihan untuk mempertahankan credential, persetujuan harus eksplisit dan hasilnya harus terlihat.

### R-04 — High: sebagian error penghapusan tabel diabaikan

**Lokasi:** src-tauri/src/repository/deletion_repo.rs

Operasi delete pada survey_question_predictions, survey_branch_transitions, survey_recognitions dan ocr_extractions menggunakan hasil yang diabaikan.

**Dampak:** Kegagalan penghapusan dapat tidak dilaporkan. Ringkasan wipe juga belum menghitung semua tabel: count_all_records tidak memasukkan seluruh tabel Survey Memory/NQI/OCR, app_credentials, dan diagnostic logs.

**Rekomendasi:** Propagasikan semua error di dalam transaksi, hitung seluruh tabel yang relevan, lalu verifikasi tidak ada orphan/record tersisa setelah wipe.

### R-05 — Medium: confidence OCR diisi dengan angka tetap

**Lokasi:** src-tauri/src/infrastructure/ocr/windows.rs

PowerShell output berisi teks, angle, ukuran gambar, lines dan words, tetapi tidak mengisi per-word confidence. Rust kemudian memakai nilai default 0.95 untuk word confidence dan mengisi line confidence 0.95.

**Dampak:** UI/reconstructor dapat menyampaikan keyakinan 95% yang tidak berasal dari confidence terukur engine.

**Rekomendasi:** Gunakan unavailable/unknown bila engine tidak menyediakan confidence; jangan mengarang nilai confidence. Jika diperlukan, buat quality heuristic terpisah dengan nama yang tidak menyamarkannya sebagai OCR confidence.

### R-06 — Medium: outbound disclosure dapat berbeda dari provider yang benar-benar dipakai

**Lokasi:**
- src-tauri/src/services/privacy_service.rs
- src-tauri/src/repository/provider_repo.rs
- src-tauri/src/services/provider_pool.rs

inspect_outbound_data mengambil provider pertama dari daftar enabled yang diurutkan priority, sementara provider pool dapat memilih konfigurasi provider lain.

**Dampak:** Provider/endpoint/model yang ditampilkan dalam disclosure berpotensi tidak sama dengan tujuan request aktual.

**Rekomendasi:** Disclosure harus menggunakan request plan/provider selection yang sama dengan eksekusi, atau tampilkan kandidat pool dan tujuan final sebelum mengirim data.

### R-07 — Medium: perhitungan umur menggunakan tahun tetap

**Lokasi:** src-tauri/src/services/fit_evaluator.rs

extract_age_value memakai current_year = 2026 untuk mengonversi tahun lahir.

**Dampak:** Hasil menjadi salah saat tahun berganti dan dapat selisih satu tahun karena bulan/tanggal ulang tahun tidak diperhitungkan.

**Rekomendasi:** Hitung umur berdasarkan tanggal saat ini dan tanggal lahir penuh. Bila hanya tahun lahir yang tersedia, hasil harus ditandai estimasi/rentang.

### R-08 — Medium: path OCR dimasukkan sebagai string literal PowerShell

**Lokasi:** src-tauri/src/infrastructure/ocr/windows.rs

Path file dimasukkan langsung ke script melalui interpolasi string PowerShell.

**Dampak:** Karakter khusus pada path Windows dapat merusak script parsing; boundary ini tidak meng-escape input secara independen.

**Rekomendasi:** Gunakan argument/parameter proses yang aman, jangan interpolasi path ke script; validasi canonical path berada di temporary directory aplikasi.

### R-09 — Medium: penghapusan credential dapat meninggalkan status OS dan DB tidak sinkron

**Lokasi:** src-tauri/src/services/credential_service.rs

delete_credential mencoba menghapus secret OS lebih dahulu, lalu menghapus metadata SQLite. Jika operasi metadata gagal setelah secret OS berhasil dihapus, status kedua tempat dapat berbeda.

**Dampak:** Pengguna dapat melihat credential record tanpa secret yang dapat dibaca.

**Rekomendasi:** Gunakan state reconciliation/kompensasi yang terdokumentasi dan tampilkan hasil penghapusan OS/DB secara eksplisit.

### R-10 — Medium: jalur error decoding credential berpotensi tidak memanggil CredFree

**Lokasi:** src-tauri/src/infrastructure/credentials/windows.rs

Setelah CredReadW berhasil, konversi UTF-8 menggunakan early return operator sebelum CredFree. Bila decoding gagal, pointer yang dialokasikan Windows berpotensi tidak dibebaskan.

**Dampak:** Memory leak pada error case.

**Rekomendasi:** Bungkus pointer dalam RAII guard yang selalu memanggil CredFree, termasuk saat decoding gagal.

### R-11 — Medium: beberapa persistence error OCR ditangani best-effort

**Lokasi:** src-tauri/src/services/ocr_service.rs

Beberapa operasi save/update extraction mengabaikan error repository.

**Dampak:** UI dapat melanjutkan seolah ekstraksi tersimpan meski persistence gagal.

**Rekomendasi:** Propagasikan kegagalan penyimpanan dan tampilkan status yang berbeda untuk hasil sementara versus hasil tersimpan.

### R-12 — Medium: view frontend besar dan banyak tanggung jawab

**Lokasi:** src/components/views/CopilotView.tsx dan src/components/views/SettingsView.tsx

View besar memuat banyak state, modal, dan alur fitur.

**Dampak:** Perubahan desain dan alur berisiko menimbulkan regresi lintas fitur serta menyulitkan pengujian komponen.

**Rekomendasi:** Refactor bertahap berdasarkan feature boundary; jangan melakukan rewrite tanpa pengujian interaksi dan parity checklist.

### R-13 — Medium: dokumentasi status proyek tidak sinkron

**Lokasi:**
- README.md
- docs/BLUEPRINT.md
- docs/08-agent-handoff/ANTIGRAVITY_HANDOFF.md
- docs/08-agent-handoff/REPOSITORY_MAP.md
- docs/08-agent-handoff/IMPLEMENTATION_BACKLOG.md

README/handoff mengklaim SC-001–SC-061 selesai dan menyebut installer release-testing tersedia. Blueprint masih menyebut starter shell. Repository map juga masih mengidentifikasi current starter repository dan Starter UI. Tree main tidak memuat folder release-testing.

**Dampak:** Pengguna dan coding agent dapat salah memahami status implementasi dan ketersediaan installer.

**Rekomendasi:** Sinkronkan semua status dokumen dengan source code, artifact yang benar-benar tersedia, serta manual release evidence.

### R-14 — Low: responsive breakpoint tidak selaras dengan minimum window

**Lokasi:** src-tauri/tauri.conf.json dan src/styles.css

Minimum window width ditetapkan 900 px, sedangkan dokumen UI menyebut responsive adjustments sampai 860 px.

**Dampak:** Beberapa breakpoint mungkin tidak dapat dicapai dalam window biasa sesuai minimum size.

**Rekomendasi:** Validasi target viewport dan sesuaikan minimum window/breakpoint pada pekerjaan UX.

### R-15 — Low: workflow bernama Windows build tidak membuat paket installer

**Lokasi:** .github/workflows/windows.yml

Workflow menjalankan npm ci/build/test, cargo fmt/check/test. Workflow tersebut tidak menjalankan Tauri NSIS bundle build dan tidak mengunggah artifact installer.

**Dampak:** Nama workflow dapat disalahartikan sebagai bukti installer berhasil dibuat.

**Rekomendasi:** Pisahkan build/test validation dan packaging/release workflow, dengan artifact/hash yang dapat diverifikasi.

## 5. Installer dan distribusi

Backlog menyebut:
- File: Survey Copilot_0.1.0_x64-setup.exe
- Version: 0.1.0
- Format: NSIS EXE
- Architecture: x64
- Install mode: currentUser
- Size: 5.97 MiB
- SHA-256: disebut sudah diverifikasi, tetapi nilai hash tidak terlihat pada bukti yang diperiksa.

Status audit:
- Repo main: file installer tidak ditemukan.
- GitHub Releases: tidak ada release asset saat diperiksa.
- Workflow run #60: tidak menghasilkan build artifact.
- Binary signature, hash aktual, dan isi installer: Not verified.
- Clean install, launch, update, uninstall, dan preservasi data: Not verified.

Catatan: pengguna melaporkan telah mencoba instalasi. Laporan tersebut dicatat dalam dokumen feedback, tetapi installer yang digunakan pengguna belum diserahkan/diinspeksi oleh auditor. Ketiadaan artifact di repository tidak membuktikan file yang dipakai pengguna tidak valid; hal ini hanya berarti distribusi publik/resmi tidak dapat diverifikasi dari snapshot repository.

## 6. Testing evidence

Sumber: [Windows build GitHub Actions #60](https://github.com/omegasands87/survey-copilot/actions/runs/45398424793)

Commit: 239590dceb726b8619d4ec5ac19e8249f0c73a71

| Langkah CI | Hasil yang terlihat di log |
|---|---:|
| npm ci | Success |
| npm run build | Success |
| npm test | 130 frontend tests passed |
| cargo fmt --all -- --check | Success |
| cargo check | Success |
| cargo test | 221 Rust tests passed |
| Workflow job | 23/23 steps completed, success |

Hasil di atas berasal dari GitHub Actions Windows runner. Auditor tidak menjalankan ulang perintah tersebut secara lokal pada audit ini.

Belum diverifikasi: native app launch, NSIS build/installation, native Credential Manager, screenshot capture, OCR accuracy, live Gemini, upgrade, uninstall, data preservation, keyboard/screen-reader usability.

## 7. Final readiness assessment

| Area | Status | Alasan |
|---|---|---|
| Source code readiness | Good, not release-approved | Banyak layer/fitur tersedia; ada temuan code-level |
| Feature completeness | Partial | Mayoritas fitur ada; OCR dan SC-062 memiliki gap |
| Security readiness | Not ready | SSRF dan credential wipe belum memadai |
| Database integrity | Partial | Migration kuat; wipe/error reporting belum lengkap |
| Windows installation readiness | Not verified | Binary installer tidak tersedia untuk inspeksi dan manual install belum dibuktikan |
| User testing readiness | Conditional | Controlled QA setelah Critical/High remediation |
| Production release readiness | No-Go | Release/security/manual acceptance belum lulus |

## 8. Prioritas tindak lanjut

1. Perbaiki R-01: hilangkan synthetic OCR fallback dari production.
2. Perbaiki R-02: kuatkan URL/IP/DNS validation dan SSRF boundaries.
3. Perbaiki R-03/R-04: full wipe harus membersihkan metadata/secret dan semua data dengan error propagation.
4. Perbaiki R-05/R-06: confidence OCR dan outbound disclosure akurat.
5. Perbaiki R-07 sampai R-11: umur, path PowerShell, credential consistency, native FFI memory management, OCR persistence.
6. Sediakan artifact installer yang dapat diunduh dan diverifikasi.
7. Lakukan clean install, launch, upgrade, uninstall, credential, capture/OCR dan data-preservation testing pada Windows 11 x64.
8. Sinkronkan semua dokumentasi status release.

## 9. Hubungan dengan feedback instalasi

Temuan usability langsung dari pengguna didokumentasikan terpisah pada [Installation & UI/UX Feedback — 24 November 2026](INSTALLATION_AND_UI_UX_FEEDBACK_2026-11-24.md). Feedback tersebut merupakan laporan pengguna; belum semuanya direproduksi oleh auditor.

---
**Audit status:** completed for repository/source review; runtime and release verification remain outstanding.
