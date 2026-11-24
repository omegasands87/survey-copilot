# Prosedur Pembaruan Survey Copilot — Windows 11 x64

Dokumen ini menjelaskan pembaruan installer internal NSIS. Build CI hanya untuk pengujian private beta; jangan mendistribusikannya sebagai rilis publik.

## Prinsip keamanan pembaruan

1. Tutup Survey Copilot sebelum menjalankan installer baru.
2. Buat backup database sebelum update yang membawa migrasi atau perubahan besar.
3. Pertahankan identifier aplikasi `com.surveycopilot.desktop` dan `installMode: currentUser` selama update biasa.
4. Jangan menghapus folder data aplikasi atau credential untuk melakukan update in-place.
5. Jalankan migrasi database saat aplikasi baru dibuka dan verifikasi bahwa data lama masih terbaca.
6. Jangan menaikkan versi produk hanya untuk setiap commit; gunakan versi rilis yang konsisten dan terdokumentasi saat menyiapkan kandidat rilis.

## Alur build untuk developer/QA

1. Buat perubahan di branch fitur dan jalankan:
   ```powershell
   npm ci
   npm run build
   npm test
   cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
   cargo check --manifest-path src-tauri/Cargo.toml
   cargo test --manifest-path src-tauri/Cargo.toml
   ```
2. Buka Pull Request ke `main`. Workflow **Windows build** memeriksa frontend dan backend. Workflow **Windows NSIS installer package** menjalankan gate yang sama lalu membangun installer x64.
3. Unduh artifact `Survey-Copilot-NSIS-<commit-sha>` dari halaman Actions. Artifact retensi 30 hari dan bukan GitHub Release.
4. Periksa `build-manifest.json`, `SHA256SUMS.txt`, ukuran file dan status Authenticode sebelum membagikan paket ke tester internal.
5. Bila build berstatus `NotSigned`, beri label unsigned/internal test only. Jangan menerbitkan ke publik.

## Prosedur update bagi tester

1. Tutup aplikasi sepenuhnya.
2. Backup folder database `%APPDATA%\com.surveycopilot.desktop` mengikuti [Backup & Restore](BACKUP_AND_RESTORE.md).
3. Verifikasi SHA-256 installer baru.
4. Jalankan installer versi baru pada user Windows yang sama. Gunakan current-user install yang sudah ada; tidak perlu uninstall versi sebelumnya untuk update normal.
5. Selesaikan wizard, lalu buka aplikasi.
6. Verifikasi persona, fakta, sumber, riwayat, dan konfigurasi lokal yang memang harus dipertahankan. Periksa diagnostik dan status provider tanpa menampilkan API key.
7. Jika aplikasi gagal dibuka atau migrasi gagal, hentikan penggunaan dan pulihkan backup mengikuti prosedur restore.

## Retensi data saat uninstall

Update in-place berbeda dari uninstall. Jangan centang **Delete app data** bila tujuan hanya memperbarui aplikasi. Pada uninstall yang disengaja, pilihan tersebut menentukan apakah folder data aplikasi lokal akan dihapus; credential Windows Credential Manager dikelola terpisah. Lihat [Installation](INSTALLATION.md) dan [Backup & Restore](BACKUP_AND_RESTORE.md).

## Rollback

Jika kandidat update merusak startup atau migrasi:
- Hentikan percobaan update berikutnya.
- Simpan log yang sudah disanitasi; jangan kirim database, screenshot pribadi, atau API key.
- Gunakan installer internal sebelumnya hanya setelah pemeriksaan kompatibilitas.
- Restore database hanya dari backup yang dibuat sebelum update dan ketika aplikasi tertutup.
- Catat versi installer, commit SHA, versi skema database, dan langkah reproduksi.
