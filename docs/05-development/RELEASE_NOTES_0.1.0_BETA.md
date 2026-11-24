# Catatan Rilis (Release Notes) — v0.1.0-beta

**Survey Copilot v1.0 — Pengujian Pribadi (Private Beta / Testing Release)**  
*Tanggal: 24 November 2026*  
*Target Platform: Windows 11 x64*  
*Status: Testing Release (Bukan Rilis Produksi Final - Memerlukan Verifikasi & Persetujuan Pengguna)*

---

## 1. Ikhtisar Rilis

Survey Copilot v1.0-beta adalah desktop AI copilot berbasis Windows 11 x64 yang dirancang khusus untuk membantu responden survei daring di Indonesia memberikan jawaban yang konsisten, faktual, dan berbasis evidensi profil nyata.

Aplikasi ini beroperasi dengan filosofi bahwa **pengguna mengoperasikan peramban secara manual 100%**. Survey Copilot tidak melakukan klik otomatis, tidak mengisi formulir web secara otomatis, dan tidak melewati proteksi platform atau CAPTCHA.

---

## 2. Fitur Utama yang Disertakan

### A. Persona & Multi-Profile Intelligence
- Manajemen persona profil responden nyata dengan isolasi data yang ketat.
- Pengelompokan fakta profil ke dalam kategori terstruktur (Demografi, Pekerjaan, Finansial, Gaya Hidup, Kepemilikan Barang, dll.).
- Pencatatan sumber evidensi (*evidence source*), tingkat keyakinan (*confidence score*), dan penandaan fakta usang (*outdated facts*).

### B. Screen Intelligence (Pratinjau Eksplisit & OCR)
- Pengambilan tangkapan layar area pertanyaan survei dengan persetujuan eksplisit pengguna.
- Dialog pratinjau gambar (*preview dialog*) sebelum data gambar diproses atau dianalisis.
- Ekstraksi teks pertanyaan dan opsi jawaban melalui mesin OCR lokal.
- Dukungan input manual teks pertanyaan (*paste text*) bagi pengguna yang memilih alur kerja tanpa tangkapan layar.

### C. Answer Recommendation Engine & Survey Memory
- Rekomendasi jawaban berbasis fakta responden nyata (tanpa fabrikasi atau halusinasi).
- Pelacakan memori sesi survei (*Survey Memory & Branch Transitions*) untuk menjaga konsistensi jawaban antar-pertanyaan turunan/cabang.
- Penanganan cerdas terhadap data yang belum ada (*missing facts*): Copilot memberikan peringatan bahwa data belum tercatat dan menyarankan jawaban jujur/netral.

### D. Provider Pooling & Keamanan Kredensial
- Dukungan Gemini API sebagai penyedia kecerdasan utama dengan strategi adaptif (*failover, rate-limit cooldown, bounded retries*).
- Dukungan custom HTTPS AI provider yang kompatibel.
- Penyimpanan kunci API secara aman menggunakan **Windows Credential Manager** (tidak ada kunci rahasia yang disimpan dalam bentuk teks biasa di database maupun repository).

### E. Transparansi Privasi & Diagnostik Aman
- Jendela transparansi data keluar (*Outbound Data Transparency*): Menampilkan rincian fakta yang dikirim dan fakta sensitif yang dikecualikan sebelum request dikirim ke AI.
- Redaksi log otomatis (*LogRedactor*) yang membersihkan NIK, nomor telepon, kartu kredit, dan kunci API dari berkas log.
- Pengaturan retensi dan pembersihan instan berkas tangkapan layar sementara.

### F. Ekspor & Penghapusan Data (Data Sovereignty)
- Ekspor seluruh data persona, fakta, dan riwayat jawaban ke format JSON/CSV.
- Penghapusan aman data persona beserta riwayat terkait dari basis data SQLite lokal.

---

## 3. Detail Paket Installer & Distribusi

- **Jenis Installer**: NSIS Setup (`Survey Copilot_0.1.0_x64-setup.exe`).
- **Mode Instalasi**: `currentUser` (terpasang di `%LOCALAPPDATA%\Programs\Survey Copilot` tanpa memerlukan hak Administrator / UAC prompt).
- **Lokasi Basis Data**: `%APPDATA%\com.surveycopilot.desktop\survey-copilot.sqlite3`.
- **Ketahanan Data**:
  - Melakukan instalasi ulang atau pembaruan (*in-place update*) tidak akan menghapus basis data SQLite maupun kredensial Windows Credential Manager.
  - Mencopot pemasangan (*uninstall*) aplikasi juga tetap mempertahankan berkas basis data di `%APPDATA%` demi mencegah kehilangan data persona secara tidak sengaja.

---

## 4. Batasan yang Diketahui (Known Limitations pada Versi Beta)

1. **Lingkungan Pengujian**: Aplikasi dirancang dan dioptimalkan secara spesifik untuk Windows 11 x64 dengan arsitektur 64-bit.
2. **Konektivitas Internet**: Analisis rekomendasi jawaban memerlukan koneksi internet aktif untuk berkomunikasi dengan Gemini API (atau custom provider endpoint yang dipilih).
3. **Bahasa Utama**: Antarmuka pengguna dan lokalisasi petunjuk diprioritaskan untuk Bahasa Indonesia.
4. **Distribusi Privat**: Rilis ini ditujukan khusus untuk pengujian pribadi dan internal; tidak dipublikasikan ke publik atau GitHub Releases.
