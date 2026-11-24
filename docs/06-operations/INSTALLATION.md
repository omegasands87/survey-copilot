# Instalasi Survey Copilot — Windows 11 x64

**Status:** paket internal/private beta. Paket CI tidak otomatis menjadi rilis publik atau build bertanda tangan.

## Sumber installer

Installer dibuat oleh workflow GitHub Actions **Windows NSIS installer package**. Setelah workflow selesai, buka tab **Actions**, pilih run yang sesuai dengan commit, lalu unduh artifact bernama `Survey-Copilot-NSIS-<commit-sha>`.

Artifact berisi:
- `Survey Copilot_<version>_x64-setup.exe`
- `SHA256SUMS.txt` dan file `.sha256` pendamping
- `build-manifest.json` berisi versi, ukuran, hash, mode instalasi, serta status Authenticode

Artifact CI memiliki masa retensi terbatas (30 hari). Ini bukan tautan GitHub Releases permanen. Karena repository bersifat public, perlakukan artifact sebagai dapat diakses pembaca repository—bukan distribusi privat. Paket hanya boleh berisi installer dan manifest tanpa data pengguna, database, screenshot, API key, atau secret. Jangan mengunduh installer yang dikirim ulang dari sumber tidak resmi.

## Verifikasi integritas

Di PowerShell, dari folder hasil ekstraksi artifact:

```powershell
Get-FileHash ".\Survey Copilot_0.1.0_x64-setup.exe" -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

Bandingkan hash hasil perintah pertama dengan nilai pada `SHA256SUMS.txt`. Jika berbeda, jangan jalankan file tersebut.

Build internal saat ini dapat berstatus **NotSigned**. SHA-256 memverifikasi integritas file terhadap manifest, tetapi bukan identitas penerbit. Jangan mendistribusikan build unsigned sebagai rilis publik.

## Instalasi

1. Pastikan Windows 11 x64, WebView2 Runtime, dan hak akses user lokal tersedia.
2. Verifikasi SHA-256 sebelum menjalankan file.
3. Jalankan `Survey Copilot_<version>_x64-setup.exe` sebagai user Windows yang akan memakai aplikasi. Konfigurasi NSIS menggunakan `currentUser`; elevasi Administrator tidak seharusnya diperlukan.
4. Ikuti wizard instalasi sampai selesai.
5. Buka Survey Copilot dari Start Menu: **Survey Copilot → Survey Copilot**.
6. Shortcut **Uninstall Survey Copilot** tersedia pada folder Start Menu yang sama.

Lokasi instalasi default current-user berada di bawah `%LOCALAPPDATA%\Programs\Survey Copilot`. Metadata uninstall untuk mode current-user dicatat pada konteks user saat ini, bukan sebagai aplikasi machine-wide.

## Uninstall

Uninstall dapat dimulai dari salah satu jalur berikut:
- **Windows Settings → Apps → Installed apps → Survey Copilot → Uninstall**
- **Start Menu → Survey Copilot → Uninstall Survey Copilot**

Jika Windows menampilkan aplikasi di Control Panel, jalur klasiknya adalah **Control Panel → Programs → Programs and Features**. Start Menu menyediakan jalan langsung ke uninstaller apabila pengguna tidak menemukannya di daftar Windows.

### Pilihan retensi data

Uninstaller NSIS menyediakan pilihan **Delete app data**. Periksa pilihan tersebut sebelum melanjutkan:
- Biarkan tidak dicentang untuk mempertahankan folder data aplikasi, termasuk database lokal, sehingga data dapat dipakai lagi setelah instalasi ulang.
- Centang hanya bila memang ingin menghapus folder data lokal aplikasi. Tindakan ini destruktif dan tidak dapat dibatalkan dari aplikasi.
- Penghapusan folder data aplikasi tidak identik dengan penghapusan rahasia pada Windows Credential Manager. Kelola credential melalui aplikasi/Windows Credential Manager secara terpisah.

Buat backup sebelum uninstall atau update yang berisiko. Ikuti [Backup & Restore](BACKUP_AND_RESTORE.md) untuk langkah pencadangan yang aman.

## Catatan rilis

- Installer dari workflow ini adalah paket uji internal.
- Belum ada klaim bahwa clean install, upgrade in-place, uninstall, retensi database, shortcut, Authenticode trust, atau Windows scaling telah diverifikasi pada perangkat pengguna.
- Pengujian tersebut dicatat pada [Manual Testing Guide](../05-development/MANUAL_TESTING_GUIDE.md) dan harus diselesaikan sebelum persetujuan rilis.
