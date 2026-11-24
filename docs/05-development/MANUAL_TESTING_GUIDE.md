# Panduan Pengujian Manual (Manual Testing Checklist & Verification Guide)

**Survey Copilot v1.0 — Testing / Private Beta Release**

Dokumen ini berisi panduan pengujian manual langkah-demi-langkah bagi tester/pengguna untuk memverifikasi fungsionalitas, keamanan data, dan batasan produk Survey Copilot di Windows 11 x64 sebelum rilis final.

---

## Ringkasan Ketentuan & Batasan Pengujian

- **Status Rilis**: Testing / Private Beta Release (bukan rilis produksi final).
- **Etika Survei**:
  - Aplikasi **tidak boleh** melakukan otomasi browser (tanpa auto-fill, tanpa auto-click, tanpa auto-submit, tanpa bypass CAPTCHA).
  - Jawaban yang direkomendasikan harus berbasis evidensi profil nyata responden.
  - Persona yang berbeda adalah representasi manusia nyata yang berbeda (isolasi ketat antar-persona).
  - Pengambilan tangkapan layar (*screen capture*) **hanya** dilakukan atas persetujuan eksplisit pengguna dengan tampilan pratinjau (*preview*) sebelum dianalisis.

---

## Checklist Pengujian Manual

### 1. Instalasi & First Run
- [ ] **Pemasangan Installer**:
  - Jalankan file installer `Survey Copilot_0.1.0_x64-setup.exe`.
  - Pastikan instalasi selesai tanpa meminta hak Administrator (UAC) dan terpasang di `%LOCALAPPDATA%\Programs\Survey Copilot`.
  - Pastikan shortcut aplikasi muncul di Start Menu dan Desktop.
- [ ] **Peluncuran Aplikasi**:
  - Luncurkan Survey Copilot.
  - Periksa apakah jendela utama aplikasi terbuka dengan benar, responsif, dan menampilkan tema UI yang bersih.
  - Verifikasi bahwa direktori database `%APPDATA%\com.surveycopilot.desktop` telah dibuat secara otomatis dan memuat file `survey-copilot.sqlite3`.

### 2. Manajemen Persona & Fakta Responden
- [ ] **Pembuatan Persona Baru**:
  - Buka tab **Persona**.
  - Buat persona responden nyata pertama (misal: "Budi Santoso", usia, pekerjaan, domisili).
  - Simpan dan pastikan persona muncul di daftar persona aktif.
- [ ] **Penambahan Kategori & Fakta Evidensi**:
  - Tambahkan kategori fakta (misal: "Demografi", "Pekerjaan", "Kepemilikan Gadget").
  - Tambahkan fakta spesifik (misal: "Pekerjaan: Software Engineer", "Smartphone: Samsung Galaxy S23").
  - Pastikan setiap fakta dapat disimpan dengan sumber evidensi dan status konfirmasi.
- [ ] **Isolasi Multi-Persona**:
  - Buat persona kedua (misal: "Siti Rahma").
  - Beralih ke persona kedua dan pastikan daftar fakta persona pertama **tidak muncul** pada persona kedua (isolasi data ketat).

### 3. Konfigurasi AI Provider (Gemini / Custom Provider)
- [ ] **Penyimpanan Kunci API**:
  - Buka menu **Settings -> Provider Keys**.
  - Masukkan Gemini API Key.
  - Pilih model (misal: `gemini-1.5-flash` atau model yang didukung).
  - Lakukan **Test Connection**. Pastikan status tes koneksi berhasil (*Success*).
  - Simpan kunci.
- [ ] **Verifikasi Keamanan Kredensial**:
  - Pastikan kunci API **tidak pernah** disimpan dalam teks biasa di database SQLite.
  - Buka Windows Credential Manager (`cmdkey /list` di terminal) dan pastikan kredensial tersimpan di Windows Credential Store lokal.

### 4. Pengujian Screen Intelligence (Capture Preview & OCR)
- [ ] **Tangkapan Layar Eksplisit**:
  - Buka salah satu halaman survei online di browser Anda (atau gambar pertanyaan survei).
  - Di Survey Copilot, klik tombol **Capture Screen** / **Area Capture**.
  - Verifikasi bahwa aplikasi menampilkan jendela **Pratinjau Gambar (Preview)** dan **tidak langsung mengirim gambar** ke AI tanpa izin Anda.
- [ ] **Ekstraksi Teks (OCR / Untrusted Input Handling)**:
  - Konfirmasi pengiriman gambar pada layar pratinjau.
  - Pastikan teks pertanyaan survei dan opsi jawaban berhasil diekstrak dan ditampilkan di area input pertanyaan.
  - Coba juga fitur **Paste Text** manual untuk memastikan alur tanpa kamera/layar bekerja mulus.

### 5. Pengujian Rekomendasi Jawaban & Survey Memory
- [ ] **Analisis Pertanyaan Berbasis Fakta**:
  - Kirim pertanyaan survei yang cocok dengan fakta persona (misal: "Apa profesi Anda saat ini?").
  - Pastikan rekomendasi jawaban yang dihasilkan AI:
    - Sesuai dengan fakta yang tersimpan di persona aktif.
    - Menampilkan sitasi evidensi / fakta pendukung.
    - Menampilkan tingkat keyakinan (*confidence level*).
- [ ] **Penanganan Pertanyaan Tanpa Fakta (Missing Facts)**:
  - Ajukan pertanyaan tentang topik yang belum ada datanya di persona (misal: "Apakah Anda memiliki mobil listrik?").
  - Pastikan Copilot **tidak mengarang jawaban (tidak berhalusinasi)**, melainkan memberi tahu bahwa fakta belum tersedia dan meminta klarifikasi atau menyarankan jawaban netral/jujur.
- [ ] **Konsistensi Pertanyaan Berikutnya (Survey Memory)**:
  - Ajukan pertanyaan turunan dalam sesi yang sama.
  - Pastikan riwayat pertanyaan dan jawaban sebelumnya dalam sesi survei tersebut diingat untuk menjaga konsistensi jawaban.

### 6. Ekspor Data & Penghapusan
- [ ] **Ekspor Data**:
  - Buka menu **Data Management / Export**.
  - Lakukan ekspor data persona dan riwayat jawaban ke format JSON/CSV.
  - Buka file hasil ekspor dan pastikan datanya lengkap dan valid.
- [ ] **Penghapusan Data Persona**:
  - Buat persona uji coba dummy, lalu lakukan delete persona.
  - Pastikan seluruh fakta dan riwayat terkait persona dummy tersebut terhapus dari SQLite secara bersih.

### 7. Pengujian Persistensi (Restart & Update In-Place)
- [ ] **Uji Restart Aplikasi**:
  - Tutup Survey Copilot sepenuhnya.
  - Buka kembali aplikasi.
  - Pastikan seluruh persona, fakta, riwayat jawaban, dan konfigurasi API provider tetap utuh dan aktif.
- [ ] **Uji Pembaruan Installer (Overwriting Update)**:
  - Tutup aplikasi.
  - Jalankan kembali installer `Survey Copilot_0.1.0_x64-setup.exe` di atas instalasi yang sudah ada.
  - Buka aplikasi dan pastikan tidak ada data persona atau riwayat yang hilang atau tereset.
