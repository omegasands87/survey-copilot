# Feedback Instalasi dan Usulan Perbaikan UI/UX — 24 November 2026

**Produk:** Survey Copilot  
**Sumber:** laporan langsung pengguna setelah mencoba instalasi aplikasi  
**Status:** kebutuhan dan masalah yang dilaporkan; belum seluruhnya direproduksi auditor  
**Jenis dokumen:** masukan owner untuk perencanaan desain, usability, dan release QA

## 1. Tujuan

Dokumen ini mencatat masalah pengalaman pengguna yang ditemukan saat aplikasi diinstal dan dicoba, serta kebutuhan UI yang diminta untuk iterasi berikutnya.

Ini bukan bukti bahwa setiap masalah telah direproduksi secara independen. Tim implementasi harus melakukan reproduksi dan membuat acceptance test sebelum menyatakan perbaikan selesai.

## 2. Ringkasan feedback pengguna

| No. | Feedback pengguna | Kategori | Status verifikasi |
|---|---|---|---|
| 1 | Desain tidak responsif | Responsive layout | User reported; reproduce required |
| 2 | Desain UI/UX sangat tidak bagus | Visual hierarchy/usability | User reported; redesign review required |
| 3 | Penambahan manual untuk “profit” dapat diterima, tetapi form saat ini tidak mudah digunakan manusia | Data-entry form/usability; istilah perlu diperjelas | User reported; scope clarification required |
| 4 | Profil seharusnya memiliki profil/template yang tersedia berdasarkan persona yang paling sering dicari dalam survei | Profile presets/templates | Proposed; product/design decision required |
| 5 | Seharusnya ada import profile | Data import | Proposed; format and mapping decision required |
| 6 | Proses pembacaan layar ribet; setelah screenshot pengguna masih perlu klik tombol konfirmasi agar terbaca aplikasi | Capture-to-OCR workflow | User reported; reproduce required |
| 7 | Pengguna ingin mode gelap dengan gaya dark bento | Visual theme | Explicit design preference |
| 8 | Ada halaman yang rusak saat dibuka | Navigation/page runtime defect | User reported; page/repro steps unknown |
| 9 | Diperlukan UI versi web untuk mengecek dan mengoreksi desain | UI-only preview environment | Explicit requirement; scope clarified below |
| 10 | Pengguna tidak menemukan uninstall aplikasi di Control Panel | Windows installation/uninstall discoverability | User reported; installer behavior not verified |

## 3. Detail kebutuhan dan arahan desain

### UX-01 — Responsive desktop layout

**Masalah:** layout tidak responsif menurut pengguna.

**Arah kebutuhan:**
- Layout harus beradaptasi terhadap ukuran window aplikasi desktop, bukan hanya mengandalkan ukuran layar besar.
- Sidebar, topbar, form, dialog, cards, tabel, dan panel Copilot harus memiliki behavior yang masuk akal pada viewport sempit.
- Tidak boleh ada konten terpotong, tombol keluar viewport, overlap, horizontal scrolling yang tidak disengaja, atau dialog yang tidak dapat dijangkau.
- Tetap mendukung Windows display scaling dan ukuran teks yang diperbesar.
- Minimum window size harus konsisten dengan breakpoint responsive.

**Acceptance criteria yang diusulkan:**
- Semua halaman utama dapat dibuka pada ukuran window minimum yang didukung.
- Form, modal, navigasi, dan action button tetap dapat digunakan pada ukuran window sempit.
- Tidak ada overflow/overlap pada viewports pengujian yang disepakati.
- Pengujian dilakukan dengan Windows scaling yang berbeda.

### UX-02 — UI/UX redesign dan dark bento

**Masalah:** pengguna menilai desain dan alur UI/UX sangat tidak baik.

**Preferensi visual:** dark mode dengan pendekatan dark bento.

**Arah kebutuhan:**
- Dark theme menjadi tampilan utama/default sesuai persetujuan desain.
- Gunakan kelompok kartu/bento yang jelas untuk informasi terkait; hindari dashboard yang penuh dekorasi.
- Perkuat hierarchy: judul halaman, konteks persona aktif, konten utama, status, dan tindakan utama.
- Prioritaskan tugas umum: pilih persona, baca pertanyaan, review hasil, dan copy rekomendasi.
- Advanced provider, diagnostics, serta privacy controls tetap tersedia tetapi tidak mengganggu alur utama.
- Form harus memiliki label, contoh input, unit, validasi, helper text, error state, dan pengelompokan yang mudah dipahami.
- Kontras, focus state, keyboard navigation, reduced motion, dan Windows high contrast tetap harus dipertahankan.

**Batasan:** “dark bento” adalah arahan visual, bukan izin untuk menambahkan metrik, earnings dashboard, atau fitur produk baru.

### UX-03 — Form input manual yang mudah digunakan

**Masalah:** pengguna dapat menerima penambahan data secara manual, tetapi form saat ini sulit digunakan.

**Catatan istilah:** pengguna menyebut “profit”. Istilah ini belum ditafsirkan sebagai pendapatan, profit survei, atau profil. Jangan mengasumsikan maknanya.

**Batasan scope saat ini:**
- Product Definition v1.0 secara eksplisit mengecualikan earnings, rewards, points, balance, payout, dan income tracking.
- Fakta finansial yang benar-benar merupakan fakta profil dapat ditangani sebagai profile fact dengan sumber, verifikasi, sensitivitas dan persona ownership, sesuai scope yang berlaku.
- Ledger profit/pendapatan atau pelacakan hasil finansial tidak boleh ditambahkan tanpa keputusan scope eksplisit dari pemilik produk.

**Arah kebutuhan form:**
- Gunakan field sesuai jenis data, bukan satu form generik yang padat.
- Kelompokkan berdasarkan kategori yang dikenali pengguna.
- Beri label dan contoh nilai yang konkret, unit/rentang bila relevan, serta validasi inline.
- Pisahkan informasi wajib dan opsional.
- Sediakan Save/Cancel yang jelas dan pertahankan draft saat validasi gagal.
- Jelaskan sumber dan status verifikasi data.
- Jangan pernah membuat nilai personal otomatis atau menganggap template sebagai fakta.

**Acceptance criteria yang diusulkan:**
- Pengguna dapat menambah/mengubah fakta tanpa menebak arti field.
- Error menjelaskan cara memperbaiki nilai.
- Perubahan tidak mengubah persona atau fakta lain.
- Fakta yang belum dikonfirmasi tetap berstatus belum terkonfirmasi.

### UX-04 — Profile templates/presets

**Masalah/permintaan:** pengguna mengharapkan template profil yang umum dicari dalam survei.

**Interpretasi aman untuk v1.0:**
- Sediakan template struktur/field atau kategori profil sebagai titik awal, bukan persona fiktif yang telah terisi.
- Template dapat mengusulkan kelompok seperti demografi, pekerjaan/pendidikan, rumah tangga, teknologi/perangkat, transportasi, konsumsi, dan finansial bila relevan dengan product scope.
- Pilihan template dapat diprioritaskan berdasarkan pola kategori pertanyaan yang umum ditemui, jika didukung data agregat yang tidak mengungkap personal data.
- Setiap nilai personal harus dimasukkan/dikonfirmasi sendiri oleh pengguna dan memiliki provenance/verifikasi.
- Persona harus tetap merepresentasikan orang nyata yang informasinya boleh dikelola operator.

**Tidak diperbolehkan:**
- Menghasilkan identitas atau profil fiktif.
- Mengisi umur, pekerjaan, pendapatan, pengalaman, penggunaan produk atau opini tanpa fakta pengguna.
- Menjadikan template sebagai cara mengarang jawaban survei.

**Acceptance criteria yang diusulkan:**
- Pengguna dapat memilih template kosong dan melihat daftar field yang akan dibuat.
- Pengguna dapat menghapus field yang tidak relevan sebelum membuat profil.
- Tidak ada fakta personal yang terisi otomatis tanpa input/konfirmasi.
- Template tidak menimpa data persona yang sudah ada.

### UX-05 — Import profile

**Permintaan:** pengguna ingin mengimpor profil.

**Arah kebutuhan:**
- Tentukan format awal yang didukung; rekomendasi awal adalah JSON yang kompatibel dengan skema export aplikasi, lalu pertimbangkan CSV dengan field mapping.
- Sediakan preview sebelum import, validasi schema, ringkasan konflik/duplikasi, pilihan merge atau buat persona baru, dan hasil import yang dapat diaudit.
- Tampilkan field yang tidak dikenal atau tidak valid; jangan membuang data diam-diam.
- Pertahankan persona ownership dan verification status.
- Import tidak boleh memasukkan API key, credential secret, database internal, atau data dari persona lain ke persona aktif tanpa pilihan eksplisit dan validasi.
- Jangan melakukan overwrite profil existing secara diam-diam.

**Acceptance criteria yang diusulkan:**
- File invalid ditolak dengan error yang dapat dimengerti.
- Pengguna dapat melihat data yang akan dibuat/diubah sebelum commit.
- Import yang gagal tidak meninggalkan data setengah jadi.
- Credentials/secrets tidak pernah diimpor ke profile data.
- Semua record tetap terikat ke persona yang tepat.

### UX-06 — Screenshot langsung memulai OCR lokal

**Masalah:** pengguna menganggap proses screenshot terlalu banyak langkah karena perlu klik konfirmasi lagi sebelum aplikasi membaca gambar.

**Arah perubahan alur yang diusulkan:**
1. Pengguna secara eksplisit menekan Capture dan memilih area/window.
2. Aplikasi mengambil gambar dan segera menampilkan preview.
3. OCR lokal otomatis dimulai tanpa tombol konfirmasi kedua hanya untuk memulai pembacaan.
4. Hasil ekstraksi muncul dengan penanda teks tidak jelas/uncertain dan dapat diedit pengguna.
5. Pengguna tetap meninjau/mengoreksi teks sebelum analisis AI.

**Pemisahan consent yang wajib dipertahankan:**
- Aksi Capture eksplisit adalah pemicu capture dan local OCR.
- Auto-start OCR lokal tidak berarti persetujuan untuk mengirim screenshot/teks ke provider AI eksternal.
- Pengiriman data pribadi/sensitif atau external vision tetap mengikuti disclosure dan per-request consent.
- Tidak boleh ada silent/background capture, polling layar, browser automation, atau auto-submit.
- Manual paste/type tetap tersedia saat capture/OCR gagal.

**Acceptance criteria yang diusulkan:**
- Satu aksi capture eksplisit memulai preview dan local OCR.
- Pengguna tidak perlu menekan konfirmasi tambahan hanya untuk memulai local OCR.
- Gagal OCR menghasilkan error/insufficient result, bukan synthetic question.
- AI analysis tidak otomatis dimulai sebelum pengguna meninjau hasil sesuai flow yang disetujui.
- Tidak ada external transmission tanpa consent yang relevan.

### UX-07 — Halaman rusak saat dibuka

**Masalah:** pengguna melaporkan ada halaman yang rusak ketika dibuka.

**Status:** belum diketahui nama halaman, langkah reproduksi, pesan error, atau apakah kerusakan terjadi setiap kali.

**Langkah verifikasi yang diperlukan:**
- Catat nama halaman dan navigasi yang dilakukan.
- Rekam screenshot atau pesan error tanpa data pribadi.
- Periksa console/runtime error dan kegagalan IPC di halaman tersebut.
- Reproduksi pada installed build serta UI preview.
- Tambahkan regression test untuk alur yang diperbaiki.

Jangan menebak halaman atau penyebab sebelum ada bukti reproduksi.

### UX-08 — UI-only web preview

**Maksud pengguna:** versi web hanya untuk memeriksa, menguji, dan mengoreksi desain UI/UX. Fungsi asli Survey Copilot tetap dijalankan oleh aplikasi Windows yang terinstal.

**Ini BUKAN:**
- Port penuh aplikasi menjadi web app.
- Pengganti Tauri desktop.
- Web backend yang mengelola persona/data nyata.
- Jalur untuk mengirim screenshot, fakta profil, atau jawaban nyata ke server.
- Integrasi AI/provider, Windows Credential Manager, SQLite desktop, capture, atau OCR produksi.

**Ini ADALAH:**
- Visual preview environment yang dapat dibuka melalui browser.
- Representasi tampilan dan navigasi UI utama untuk mengevaluasi hierarchy, spacing, cards, forms, responsive behavior, dark theme, dialog, empty/loading/error states.
- Menggunakan synthetic/demo data dan mock service behavior.
- Diberi label jelas seperti “UI Preview — demo data only”.
- Tidak menyimpan data pribadi, secret, atau API key.
- Dapat digunakan untuk mengoreksi UI sebelum perubahan diterapkan pada desktop app.

**Arah teknis yang perlu diputuskan:**
- Pilihan sederhana: web preview build dari React dengan mock IPC/service adapter.
- Tauri IPC yang tidak tersedia pada browser harus diganti oleh mock yang jelas terisolasi.
- Mock tidak boleh diperlakukan sebagai implementasi backend atau menampilkan aksi yang seolah-olah benar-benar menyimpan data.
- Pertahankan satu source of truth untuk komponen presentasi agar koreksi visual dapat diterapkan ke desktop UI tanpa menduplikasi seluruh desain.
- Tidak perlu menambahkan akun web, cloud database, atau sinkronisasi.

**Acceptance criteria yang diusulkan:**
- Preview dapat dibuka pada browser tanpa menjalankan Tauri.
- Navigasi dan state UI yang relevan dapat dievaluasi dengan demo data.
- Viewport/responsive breakpoints dapat diuji.
- Seluruh mock/demo data diberi label dan tidak pernah tercampur dengan data aplikasi terinstal.
- Tidak ada API calls atau transmisi data personal yang nyata dari preview.
- Tidak ada klaim bahwa fitur desktop bekerja pada versi web.

### UX-09 — Uninstall tidak ditemukan di Control Panel

**Masalah:** pengguna tidak menemukan opsi uninstall pada Control Panel.

**Status:** user reported; belum diuji ulang oleh auditor pada Windows.

**Kemungkinan pemeriksaan (bukan kesimpulan):**
- Apakah installer mendaftarkan aplikasi pada Windows Installed Apps/Apps & Features dan Control Panel Programs and Features.
- Apakah uninstaller executable dibuat di lokasi install.
- Apakah shortcut Start Menu menyediakan entry uninstall.
- Apakah currentUser installation scope menyebabkan aplikasi tampil pada lokasi Windows yang berbeda dari yang dicari pengguna.
- Apakah installer yang dipakai pengguna sama dengan artefak/versi yang dijelaskan repository.

**Acceptance criteria yang diusulkan:**
- Installed app muncul di lokasi uninstall Windows yang sesuai dengan mode instalasi.
- Uninstaller dapat dijalankan tanpa hak Administrator untuk currentUser install.
- Uninstall menghapus binary/shortcut sesuai kebijakan.
- Data SQLite dan credentials mengikuti kebijakan retensi yang jelas dan diberitahukan sebelum uninstall.
- Pengujian dilakukan pada install baru dan upgrade.

## 4. Usulan prioritas

| Prioritas | Pekerjaan |
|---|---|
| P0 | Reproduksi halaman rusak dan uninstall discoverability; verifikasi perilaku install/uninstall |
| P0 | Perbaiki security/data-integrity findings Critical/High pada laporan audit teknis |
| P1 | Redesain dark bento dan responsive layout dengan task-first information architecture |
| P1 | Perbaiki ergonomi seluruh form manual dan profile data-entry |
| P1 | Sederhanakan capture → preview + auto local OCR → review; pisahkan consent external AI |
| P1 | Tambahkan profile templates yang hanya membuat struktur/field kosong |
| P1 | Rancang import profile dengan preview, validation, merge/dedupe, dan rollback |
| P1 | Siapkan browser UI-only preview memakai mock IPC/data |
| P2 | Validasi accessibility, keyboard, display scaling, empty/error states, dan UI regression |

Prioritas ini merupakan usulan untuk perencanaan berikutnya. Tidak ada item yang dianggap sudah diimplementasikan atau disetujui untuk mengubah product scope.

## 5. Product decisions yang harus dipertahankan

Sampai ada keputusan baru:
- Survey Copilot v1.0 tetap aplikasi desktop Windows 11 x64.
- Web preview hanya untuk UI/UX design review.
- Persona tetap orang nyata; template adalah struktur kosong, bukan identitas sintetis.
- Tidak ada browser automation, auto-fill, auto-click, auto-submit, atau CAPTCHA bypass.
- Tidak ada silent/background capture.
- Local OCR auto-start setelah capture eksplisit tidak memberikan izin otomatis untuk external AI transmission.
- Tidak ada earnings/reward/points/balance/payout/income tracking tanpa perubahan scope yang disetujui.
- Facts, sources, history, sessions dan answer records tetap terisolasi per persona.

## 6. Hubungan dengan dokumen lain

- [Audit Kelengkapan, Keamanan, dan Kesiapan Rilis](AUDIT_COMPLETENESS_SECURITY_RELEASE_2026-11-24.md)
- [Product Definition](../01-product/PRODUCT_DEFINITION.md)
- [UI/UX Specification](../04-design/UI_UX_SPECIFICATION.md)
- [Design System](../04-design/DESIGN_SYSTEM.md)
- [Implementation Backlog](../08-agent-handoff/IMPLEMENTATION_BACKLOG.md)

---
**Status:** documented for owner review; implementation pending explicit prioritization and approval.
