# Prosedur Pencadangan dan Pemulihan Data (Backup & Restore)

Gunakan panduan ini sebelum update besar, migrasi sistem, pemindahan komputer, atau uninstall. Jangan pernah mengirim file database, screenshot, atau credential ke issue publik.

## 1. Lokasi data

| Komponen | Lokasi | Isi / catatan |
|---|---|---|
| Program | `%LOCALAPPDATA%\Programs\Survey Copilot` | Biner aplikasi dan resource instalasi current-user |
| Data lokal | `%APPDATA%\com.surveycopilot.desktop\survey-copilot.sqlite3` | Persona, fakta, sumber, sesi, pertanyaan, riwayat, provider config non-secret dan settings |
| SQLite WAL/SHM | Folder data yang sama | File pendamping SQLite; dapat ada ketika WAL aktif |
| API secrets | Windows Credential Manager | Terikat ke akun Windows; bukan bagian dari backup SQLite |

## 2. Backup

1. Tutup Survey Copilot sepenuhnya.
2. Buka PowerShell dan jalankan:

   ```powershell
   $source = Join-Path $env:APPDATA "com.surveycopilot.desktop"
   $destination = Join-Path "$env:USERPROFILE\Documents" ("SurveyCopilot-Backup-" + (Get-Date -Format "yyyyMMdd-HHmmss"))
   New-Item -ItemType Directory -Path $destination -Force | Out-Null
   Copy-Item -Path (Join-Path $source "*") -Destination $destination -Recurse -Force
   Get-ChildItem -Path $destination -File | Get-FileHash -Algorithm SHA256
   ```

3. Simpan backup di lokasi aman yang hanya dapat diakses pemilik data.
4. Jangan menganggap backup database sebagai backup API key. Credential berada di Windows Credential Manager dan mengikuti akun Windows yang menyimpannya. Jangan mengekspor atau menempel API key ke file backup teks.

Untuk konsistensi maksimum, pastikan aplikasi benar-benar tertutup sebelum menyalin file database dan WAL/SHM.

## 3. Restore

1. Tutup aplikasi.
2. Simpan salinan folder data yang ada sebelum menimpa apa pun.
3. Pulihkan isi folder backup ke `%APPDATA%\com.surveycopilot.desktop`.
4. Jika backup dibuat saat aplikasi tertutup, pulihkan database dan file pendampingnya sebagaimana adanya. Jangan menghapus WAL/SHM secara otomatis kecuali Anda yakin file tersebut bukan bagian dari backup yang konsisten.
5. Buka aplikasi dan verifikasi persona/fakta/riwayat. Database akan melewati pemeriksaan dan migrasi yang diperlukan.
6. Di komputer Windows baru, credential yang terikat ke akun/perangkat lama mungkin tidak tersedia. Masukkan ulang API key melalui Settings → Provider Keys; jangan menaruh key di file profil atau JSON export.

## 4. Retensi saat uninstall NSIS

Installer menggunakan `installMode: "currentUser"`. Uninstaller NSIS menyediakan pilihan **Delete app data**:
- **Tidak dicentang:** folder data aplikasi di AppData dipertahankan agar dapat digunakan setelah instalasi ulang.
- **Dicentang:** folder data aplikasi lokal dihapus. Backup terlebih dahulu; tindakan ini destruktif.

Menghapus folder data aplikasi **tidak otomatis menghapus secret di Windows Credential Manager**. Kelola credential melalui menu credential aplikasi atau Windows Credential Manager secara terpisah. Jangan mengandalkan penghapusan direktori AppData sebagai bukti bahwa semua rahasia OS sudah dihapus.

## 5. Cakupan pengujian

Perilaku clean install, repair/update, uninstall dengan dan tanpa pilihan Delete app data, retensi SQLite, dan retensi Credential Manager harus diverifikasi pada Windows 11 x64 sebelum rilis publik. Sampai pengujian tersebut selesai, jangan mengklaim retensi atau penghapusan telah teruji di perangkat.
