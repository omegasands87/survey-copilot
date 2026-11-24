# Prosedur Pencadangan dan Pemulihan Data (Backup & Restore)

Dokumen ini menjelaskan prosedur pencadangan (*backup*) dan pemulihan (*restore*) data pengguna untuk aplikasi **Survey Copilot v1.0** di lingkungan Windows 11 x64.

---

## 1. Lokasi Penyimpanan Data Aplikasi

Survey Copilot memisahkan secara ketat antara file instalasi aplikasi (biner program), basis data lokal, dan kredensial rahasia:

| Komponen Data | Lokasi Penyimpanan | Keterangan |
|---|---|---|
| **Biner Aplikasi** | `%LOCALAPPDATA%\Programs\Survey Copilot` | File eksekusi program (`.exe`), pustaka dinamis (`.dll`), dan resource UI. |
| **Database Lokal SQLite** | `%APPDATA%\com.surveycopilot.desktop\survey-copilot.sqlite3` | Berisi seluruh data persona, profil responden, kategori fakta, sumber evidensi, riwayat pertanyaan survei, rekomendasi jawaban, dan pengaturan pool provider. |
| **Database WAL/SHM** | `%APPDATA%\com.surveycopilot.desktop\survey-copilot.sqlite3-wal`<br>`%APPDATA%\com.surveycopilot.desktop\survey-copilot.sqlite3-shm` | File sementara Write-Ahead Log SQLite (hadir saat transaksi aktif atau sebelum checkpoint penuh). |
| **Kredensial API (Secrets)** | **Windows Credential Manager** (Generic Credentials) | Kunci API Gemini dan custom provider disimpan secara aman di OS keyring, terikat pada user account Windows lokal. |

---

## 2. Prosedur Pencadangan (Backup)

Lakukan pencadangan sebelum memperbarui aplikasi, melakukan migrasi sistem, atau secara berkala.

### Langkah-langkah Backup:
1. **Tutup Aplikasi**: Pastikan Survey Copilot telah ditutup sepenuhnya (tidak berjalan di taskbar atau system tray).
2. **Buka Direktori Data**:
   - Tekan `Win + R`, ketik `%APPDATA%\com.surveycopilot.desktop`, lalu tekan `Enter`.
   - Atau via PowerShell:
     ```powershell
     cd "$env:APPDATA\com.surveycopilot.desktop"
     ```
3. **Salin File Database**:
   - Salin seluruh file yang ada di dalam folder tersebut:
     - `survey-copilot.sqlite3`
     - `survey-copilot.sqlite3-wal` (jika ada)
     - `survey-copilot.sqlite3-shm` (jika ada)
   - Simpan ke direktori backup aman di luar direktori aplikasi (misal: `D:\Backups\SurveyCopilot\backup-YYYYMMDD\`).
4. **Catatan Kredensial API**:
   - Kredensial di Windows Credential Manager tetap tersimpan di akun Windows Anda. Namun, demi kehati-hatian saat berpindah PC atau instalasi ulang OS Windows, pastikan Anda memiliki catatan cadangan kunci API Gemini / Custom Provider di pengelola kata sandi (*password manager*) pribadi Anda.

---

## 3. Prosedur Pemulihan (Restore)

Gunakan prosedur ini untuk mengembalikan data persona dan riwayat jawaban setelah perbaikan sistem atau saat memindahkan data ke perangkat baru.

### Langkah-langkah Restore:
1. **Tutup Aplikasi**: Pastikan Survey Copilot tidak sedang berjalan.
2. **Siapkan Direktori Target**:
   - Buka `%APPDATA%\com.surveycopilot.desktop`.
   - Jika aplikasi baru saja diinstal dan belum pernah dibuka, buat folder `com.surveycopilot.desktop` di dalam `%APPDATA%`, atau jalankan Survey Copilot sekali lalu tutup kembali agar direktori terbuat secara otomatis.
3. **Salin File Database Cadangan**:
   - Salin file `survey-copilot.sqlite3` dari folder backup ke dalam `%APPDATA%\com.surveycopilot.desktop\`.
   - Pastikan nama file adalah tepat `survey-copilot.sqlite3`.
   - Jika terdapat file `-wal` atau `-shm` lama yang korup, hapus file `-wal` dan `-shm` tersebut agar SQLite membaca file utama secara bersih.
4. **Jalankan Aplikasi**:
   - Buka kembali Survey Copilot.
   - Rust backend akan otomatis membuka database, memverifikasi integritas, dan menjalankan migrasi skema inkremental jika versi aplikasi lebih baru dari versi backup.
5. **Verifikasi Kredensial (Bila di Komputer Baru)**:
   - Jika data dipulihkan di komputer Windows yang berbeda, buka menu **Settings -> Provider Keys** dan masukkan kembali API Key Anda karena Windows Credential Store terikat dengan SID (*Security Identifier*) user Windows lokal.

---

## 4. Keamanan Saat Uninstall Aplikasi

- Installer NSIS Survey Copilot dikonfigurasi dengan mode pengguna (`installMode: "currentUser"`).
- Jika pengguna melakukan pencopotan (*uninstall*) Survey Copilot melalui Windows Settings (*Apps > Installed apps*):
  - File biner program di `%LOCALAPPDATA%\Programs\Survey Copilot` akan dihapus bersih.
  - **Database SQLite di `%APPDATA%\com.surveycopilot.desktop` TIDAK AKAN DIHAPUS.** Data responden dan riwayat Anda tetap aman dan tidak akan hilang jika Anda menginstal ulang aplikasi di kemudian hari.
  - Kredensial API di Windows Credential Manager juga tidak dihapus secara otomatis demi mencegah kehilangan kunci API.
