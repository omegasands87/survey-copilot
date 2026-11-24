# Prosedur Pembaruan Aplikasi (Bug Fix & Update Workflow)

Dokumen ini menjelaskan alur kerja (*workflow*) pembaruan aplikasi **Survey Copilot v1.0** saat ditemukan bug, perbaikan fitur, atau rilis pengujian baru di Windows 11 x64.

---

## 1. Prinsip Pembaruan yang Aman

Pembaruan Survey Copilot dirancang agar:
1. **Tidak Menghapus Data Pengguna**: Seluruh basis data SQLite (`survey-copilot.sqlite3`), persona profil, riwayat jawaban, dan kredensial API di Windows Credential Manager tetap utuh 100%.
2. **Tidak Memerlukan Hak Administrator (UAC)**: Installer beroperasi pada lingkup pengguna saat ini (`currentUser`) di direktori `%LOCALAPPDATA%\Programs\Survey Copilot`.
3. **Pembaruan In-Place**: Menjalankan installer versi baru langsung memperbarui (*overwrite*) biner program tanpa perlu mencopot (*uninstall*) versi lama terlebih dahulu.

---

## 2. Alur Kerja Pembaruan bagi Pengembang / QA

Ketika bug ditemukan dan telah diperbaiki pada kode sumber:

### Langkah 1: Perbaikan Kode Sumber & Pengujian Lokal
1. Lakukan perbaikan bug pada repository (`src/` untuk UI React/TypeScript, atau `src-tauri/src/` untuk Rust backend).
2. Jalankan pengujian frontend:
   ```powershell
   npm.cmd test
   ```
3. Jalankan pengujian backend Rust:
   ```powershell
   $env:PATH = "D:\rust-sysroot\bin;D:\mingw64\bin;" + $env:PATH
   $env:CARGO_TARGET_DIR = "D:\cargo-target-sc"
   cargo test --manifest-path src-tauri/Cargo.toml -j 1
   ```
4. Verifikasi seluruh pengujian lolos tanpa error.

### Langkah 2: Peningkatan Versi (Bila Diperlukan)
Jika pembaruan memerlukan kenaikan versi internal/beta:
- Perbarui bidang `"version"` di file `package.json` dan `src-tauri/tauri.conf.json` (misalnya dari `"0.1.0"` ke `"0.1.1"`).

### Langkah 3: Kompilasi Installer Baru
Jalankan perintah build Tauri NSIS:
```powershell
$env:PATH = "D:\rust-sysroot\bin;D:\mingw64\bin;" + $env:PATH
$env:CARGO_TARGET_DIR = "D:\cargo-target-sc"
npx.cmd tauri build --bundles nsis -- -j 1
```
*Catatan:*
- Gunakan flag `-j 1` agar alokasi memori dan beban CPU tetap stabil.
- Jangan menghapus direktori `D:\cargo-target-sc` agar build inkremental berjalan sangat cepat (hanya mengompilasi ulang berkas yang diubah).

### Langkah 4: Distribusi Installer Pengujian
- Installer hasil kompilasi akan berada di:
  `D:\cargo-target-sc\release\bundle\nsis\Survey Copilot_<versi>_x64-setup.exe`
- Berikan installer ini ke pengguna/tester untuk pengujian internal (jangan publikasikan ke GitHub Releases atau publik).

---

## 3. Langkah Instalasi Pembaruan bagi Pengguna / Tester

Bagi pengguna atau penguji yang sedang menggunakan versi sebelumnya:

1. **Tutup Aplikasi**: Tutup Survey Copilot jika aplikasi sedang terbuka di layar.
2. **(Opsional tapi Disarankan) Buat Salinan Cadangan Database**:
   - Salin file `%APPDATA%\com.surveycopilot.desktop\survey-copilot.sqlite3` ke folder cadangan sementara (lihat panduan `BACKUP_AND_RESTORE.md`).
3. **Jalankan Installer Baru**:
   - Klik dua kali file installer `Survey Copilot_<versi>_x64-setup.exe`.
   - Ikuti wizard instalasi NSIS hingga selesai.
4. **Buka Aplikasi**:
   - Jalankan Survey Copilot melalui Start Menu atau Shortcut Desktop.
   - Periksa bahwa perbaikan bug telah aktif.
   - Pastikan seluruh persona yang telah dibuat sebelumnya, data fakta, riwayat pertanyaan survei, dan koneksi API provider tetap berada di tempatnya dan berfungsi normal.
