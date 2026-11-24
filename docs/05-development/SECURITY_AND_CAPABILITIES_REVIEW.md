# Tinjauan Keamanan, Kapabilitas Tauri, dan Lisensi Dependensi (Security & Capabilities Review)

**Survey Copilot v1.0 — Milestone SC-062**

---

## 1. Tinjauan Keamanan Aplikasi & Sandboxing

### A. Pembatasan Kapabilitas Tauri (Tauri Capabilities)
- Berkas konfigurasi: `src-tauri/capabilities/default.json`.
- **Izin yang diberikan ke Frontend**: Hanya `core:default`.
- **Prinsip Least Privilege**:
  - Frontend React **tidak memiliki akses langsung** ke sistem berkas (*filesystem*), command shell (*process execution*), clipboard OS tanpa kontrol, maupun raw HTTP networking.
  - Seluruh operasi sensitif (interaksi database SQLite, enkripsi/penyimpanan kredensial Windows Credential Manager, pemanggilan API Gemini/Custom Provider, tangkapan layar, dan OCR) diproses secara eksklusif oleh **Rust Backend** melalui Tauri IPC Commands yang memiliki validasi skema bertipe kuat (*strongly-typed*).
  - Tidak ada perintah SQL bebas (*generic/raw SQL execution*) yang diekspos ke frontend.

### B. Kebijakan Keamanan Konten (Content Security Policy - CSP)
- Berkas konfigurasi: `src-tauri/tauri.conf.json`.
- Kebijakan yang diterapkan:
  ```text
  default-src 'self'; img-src 'self' asset: http://asset.localhost data:; style-src 'self' 'unsafe-inline'; font-src 'self' data:; connect-src ipc: http://ipc.localhost
  ```
- **Analisis CSP**:
  - `default-src 'self'`: Mencegah pemuatan skrip pihak ketiga yang tidak tepercaya.
  - `connect-src ipc: http://ipc.localhost`: Membatasi koneksi jaringan dari lapisan UI secara ketat hanya ke Tauri local IPC. Lapisan webview frontend tidak dapat mengirim data langsung ke server eksternal di luar kontrol Rust backend.
  - `img-src 'self' asset: http://asset.localhost data:`: Mengizinkan rendering pratinjau tangkapan layar lokal secara aman.

### C. Keamanan Kredensial & Basis Data
- **Kunci API**: Disimpan menggunakan `WindowsCredentialStore` di Windows Credential Manager (`Generic Credentials`). Basis data SQLite hanya menyimpan referensi opaque UUID, bukan kunci API mentah (*raw plaintext key*).
- **Isolasi Persona**: Setiap entitas (fakta responden, sumber evidensi, riwayat pertanyaan survei, rekomendasi jawaban, transisi cabang survei) diikat secara ketat dengan `persona_id`. Kueri SQL selalu memvalidasi dan membatasi data berdasarkan persona aktif.
- **Redaksi Log Diagnostik**: Mesin `LogRedactor` secara otomatis membersihkan kunci API, token otorisasi, NIK, kartu kredit, dan PII dari log diagnostik sebelum dicatat.

---

## 2. Batasan Etika & Integritas Produk

| Batasan Non-Negosiasi | Status Implementasi |
|---|---|
| **Otomasi Browser (Auto-fill/Auto-click)** | **TIDAK DIIMPLEMENTASIKAN (Dilarang)**. Pengguna mengoperasikan browser survei secara manual 100%. |
| **Bypass CAPTCHA / Proteksi Platform** | **TIDAK DIIMPLEMENTASIKAN (Dilarang)**. |
| **Fabrikasi Fakta / Manipulasi Profil** | **DICEGAH SECARA SISTEMATIS**. Copilot hanya menyarankan jawaban yang didukung oleh fakta profil responden nyata. Jika fakta tidak ada (*missing fact*), Copilot menyatakan ketidaktersediaan fakta dan menolak berhalusinasi. |
| **Screen Capture Diam-diam (Silent Capture)** | **DICEGAH SECARA SISTEMATIS**. Tangkapan layar hanya dipicu oleh klik eksplisit pengguna, dan selalu menampilkan dialog pratinjau (*preview*) sebelum teks diekstrak atau dianalisis. |

---

## 3. Tinjauan Lisensi Dependensi (Open Source Licenses)

Seluruh dependensi utama yang digunakan dalam Survey Copilot v1.0 menggunakan lisensi open-source komersial-friendly (MIT / Apache 2.0 / BSD / ISC / zlib / Public Domain):

- **Tauri Framework (`tauri`, `tauri-build`, `wry`, `tao`)**: MIT OR Apache-2.0.
- **SQLite Engine (`rusqlite`, `libsqlite3-sys`)**: MIT / Public Domain (SQLite Core).
- **UI Framework (`React`, `React-DOM`, `Lucide React`)**: MIT.
- **Build Tooling (`Vite`, `TypeScript`, `Tailwind CSS`)**: MIT.
- **Serialization & Async (`serde`, `tokio`, `reqwest`)**: MIT OR Apache-2.0.
- **Windows Integration (`windows`, `windows-sys`, `keyring`)**: MIT OR Apache-2.0.

Tidak ada dependensi berlisensi copyleft ketat (seperti AGPL/GPL v3) yang dapat membatasi distribusi internal atau lisensi aplikasi.
