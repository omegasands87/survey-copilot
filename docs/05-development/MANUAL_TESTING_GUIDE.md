# Panduan Pengujian Manual (Manual Testing Checklist & Verification Guide)

**Survey Copilot v0.1.0-beta — Windows 11 x64 · private beta**

Checklist ini digunakan setelah semua lima tahap implementasi repository selesai. Checklist bukan bukti bahwa item telah diuji. Jangan memakai data responden nyata selain milik tester yang memberi persetujuan; gunakan profil sintetis untuk pemeriksaan awal dan jangan pernah mengirim screenshot/credential ke issue.

## Ketentuan pengujian

- Tidak ada browser automation, auto-fill, auto-click, auto-submit, CAPTCHA bypass, atau screener deception.
- Persona harus merepresentasikan orang nyata; template profil hanya berisi field kosong.
- Rekomendasi AI harus berbasis fakta terverifikasi dan dapat abstain bila informasi kurang.
- Capture hanya setelah tindakan eksplisit pengguna. Local OCR tidak memberikan persetujuan untuk transmisi AI eksternal.
- Bedakan pengujian browser UI preview dengan aplikasi Windows terinstal.

## Checklist manual — DEVICE TEST REQUIRED

### 1. Installer, clean install, dan uninstall discovery

- [ ] Unduh artifact `Survey-Copilot-NSIS-<commit-sha>` dari GitHub Actions workflow **Windows NSIS installer package**.
- [ ] Verifikasi SHA-256 installer dengan `SHA256SUMS.txt`.
- [ ] Catat status Authenticode; build unsigned hanya untuk pengujian internal.
- [ ] Jalankan clean install sebagai user biasa (tanpa Administrator/UAC) pada Windows 11 x64.
- [ ] Pastikan program berada di bawah `%LOCALAPPDATA%\Programs\Survey Copilot`.
- [ ] Pastikan shortcut Start Menu **Survey Copilot** dan **Uninstall Survey Copilot** muncul dan berfungsi.
- [ ] Verifikasi aplikasi muncul di Windows Settings → Apps → Installed apps.
- [ ] Periksa Control Panel → Programs and Features bila Windows menampilkan current-user package di sana; jangan menjadikan Control Panel satu-satunya jalur uninstall.
- [ ] Jalankan shortcut uninstall, batalkan wizard, dan pastikan aplikasi tetap terpasang.
- [ ] Jalankan uninstall kembali. Biarkan **Delete app data** tidak dicentang; periksa folder AppData dan database tetap ada.
- [ ] Install ulang; pastikan data yang dipertahankan dapat dibaca.
- [ ] Ulangi pada instalasi disposable, centang **Delete app data**, lalu verifikasi folder data aplikasi memang dihapus.
- [ ] Pastikan credential Windows Credential Manager diperlakukan terpisah dari penghapusan folder AppData.

### 2. Upgrade in-place dan retensi

- [ ] Buat dua persona uji, fakta, sumber bukti, satu sesi, dan catatan riwayat sintetis.
- [ ] Buat backup database sesuai panduan.
- [ ] Jalankan installer versi berikutnya di atas instalasi yang sama (same Windows account/currentUser).
- [ ] Pastikan installer tidak meminta Administrator.
- [ ] Buka aplikasi dan verifikasi semua data uji tetap tersedia setelah migrasi.
- [ ] Pastikan provider config tetap ada, tetapi API secret tetap dibaca dari Windows Credential Manager.
- [ ] Catat versi aplikasi dan schema migration sebelum/sesudah upgrade.

### 3. Tema Dark Bento, responsivitas, dan navigasi

- [ ] Buka setiap halaman utama pada aplikasi terinstal.
- [ ] Uji ukuran window minimum yang diizinkan dan ukuran umum 1280×820.
- [ ] Uji Windows display scaling (100%, 125%, 150%) dan ukuran teks yang diperbesar.
- [ ] Pastikan tidak ada tombol terpotong, overlap, horizontal scroll tak disengaja, atau modal yang tidak dapat dijangkau.
- [ ] Uji sidebar, active persona selector, Settings, Copilot, History dan Profile navigation.
- [ ] Uji keyboard-only: Tab, Shift+Tab, Enter, Space, Escape; cek focus-visible dan urutan fokus.
- [ ] Periksa contrast, forced/high contrast, dan reduced-motion states.

### 4. Profile form, templates, dan JSON import

- [ ] Buat persona dengan nama responden uji; jangan gunakan identitas fiktif sebagai data produksi.
- [ ] Pilih satu blank field template dan pastikan hanya nama/kategori/tipe field yang diisi; nilai personal tetap kosong.
- [ ] Tambahkan fakta sintetis. Pastikan status awal **unverified**.
- [ ] Konfirmasi fakta setelah memeriksa sumber dan pastikan riwayat perubahan tercatat.
- [ ] Ekspor satu persona ke JSON, lalu impor sebagai profil baru dengan nama berbeda.
- [ ] Pastikan ID profil/fakta baru tidak sama dengan ID sumber.
- [ ] Pastikan fakta, provenance, sensitivitas dan status verifikasi sesuai aturan import.
- [ ] Pastikan sesi, jawaban, screenshots, provider settings, API secrets, dan diagnostics tidak ikut diimpor.
- [ ] Coba file rusak, full-application export, file lebih dari 5 MB, sumber fakta tidak cocok, dan nilai tipe data invalid; semua harus ditolak tanpa data parsial.

### 5. Capture → Local OCR → review

- [ ] Dengan sesi/persona aktif, capture area layar; pastikan local OCR mulai tanpa tombol kedua untuk memulai ekstraksi.
- [ ] Ulangi dengan memilih jendela aplikasi.
- [ ] Verifikasi progress, preview gambar, hasil OCR, koreksi manual, dan status ketidakpastian.
- [ ] Pastikan AI analysis tidak berjalan otomatis sesudah OCR; lakukan tindakan AI hanya secara terpisah.
- [ ] Matikan/gagalkan OCR secara terkontrol; pastikan preview tetap ada, pesan error jelas, retry dan manual text tersedia.
- [ ] Pastikan OCR failure tidak pernah menampilkan synthetic fixture question.
- [ ] Uji close/discard/manual-link saat OCR sedang berjalan; tindakan yang berisiko race harus dinonaktifkan sampai operasi selesai.
- [ ] Periksa kebijakan retensi dan penghapusan temporary capture files.

### 6. Privacy, credential, data deletion, dan backup

- [ ] Verifikasi disclosure menunjukkan provider/destination yang benar sebelum request eksternal.
- [ ] Pastikan sensitive facts dan external vision ditolak tanpa consent per-request yang relevan.
- [ ] Pastikan capture/local OCR tidak membuat request provider.
- [ ] Periksa log diagnostik hanya berisi data yang sudah disanitasi.
- [ ] Uji hapus satu credential dan verifikasi Windows Credential Manager serta metadata database konsisten.
- [ ] Pada profil sintetis, uji data wipe dengan dan tanpa secret; cocokkan ringkasan jumlah record dengan hasil aktual.
- [ ] Backup dan restore database dalam instalasi disposable; verifikasi migrasi saat restore.

### 7. Browser UI-only preview (bukan pengujian fungsi desktop)

- [ ] Jalankan `npm ci` dan `npm run preview:ui`, buka `http://127.0.0.1:4173/?preview=1`.
- [ ] Pastikan banner **UI PREVIEW** selalu terlihat dan semua persona/fakta merupakan data sintetis.
- [ ] Uji seluruh halaman, ukuran viewport, template, import mock, dialog, empty/loading/error states.
- [ ] Pastikan browser preview tidak meminta screen-capture permission, tidak memanggil Tauri, AI, jaringan, atau penyimpanan permanen.
- [ ] Refresh halaman dan pastikan perubahan mock hilang.
- [ ] Jangan memasukkan data pribadi, screenshot asli atau API key ke preview.

### 8. Hasil dan bukti

- [ ] Catat Windows edition/build, architecture, scaling, application version, installer SHA-256, commit SHA, dan workflow run ID.
- [ ] Untuk setiap test catat Pass/Fail/Blocked, langkah reproduksi, dan bukti yang sudah disamarkan.
- [ ] Jangan melampirkan database SQLite, screenshot pribadi, bearer token, API key, NIK, alamat, atau data responden.
- [ ] Buat defect/regression test untuk setiap kegagalan sebelum menyatakan release gate lulus.

**Penting:** sampai checklist ini benar-benar dijalankan pada perangkat Windows 11 x64, clean install/upgrade/uninstall/data-retention/credential behavior tetap berstatus **DEVICE TEST REQUIRED**. CI yang lulus tidak menggantikan pengujian device.
