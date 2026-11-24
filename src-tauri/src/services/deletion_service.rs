use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::export::{RetentionPolicyInfo, WipeReport};
use crate::repository::{AllRecordCounts, DeletionRepository};
use std::path::Path;

pub struct DeletionService;

impl DeletionService {
    /// Mendapatkan rincian jumlah record yang akan dihapus sebelum konfirmasi reset aplikasi.
    pub fn get_wipe_summary(database: &Database) -> Result<AllRecordCounts, AppError> {
        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        DeletionRepository::count_all_records(&connection).map_err(AppError::from)
    }

    /// Menghapus seluruh data pengguna dan pengaturan aplikasi secara menyeluruh dan permanen.
    /// Membutuhkan teks konfirmasi eksplisit 'HAPUS SEMUA DATA'.
    pub fn wipe_all_data(
        database: &Database,
        app_data_dir: Option<&Path>,
        confirmation_text: &str,
    ) -> Result<WipeReport, AppError> {
        let clean_confirmation = confirmation_text.trim();
        if clean_confirmation != "HAPUS SEMUA DATA" {
            return Err(AppError::validation(
                "confirmationText",
                "Teks konfirmasi tidak sesuai. Harap ketik tepat: HAPUS SEMUA DATA",
            ));
        }

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        // 1. Kumpulkan seluruh path file tangkapan layar sebelum penghapusan data relasional
        let temp_file_paths =
            DeletionRepository::get_all_capture_image_paths(&connection).unwrap_or_default();

        // 2. Lakukan penghapusan database di dalam transaksi eksplisit
        let transaction = connection.transaction().map_err(|e| {
            AppError::database(format!("Gagal memulai transaksi pembersihan data: {e}"))
        })?;

        let table_counts =
            DeletionRepository::wipe_all_data(&transaction).map_err(AppError::from)?;

        transaction
            .commit()
            .map_err(|e| AppError::database(format!("Gagal mengeksekusi pembersihan data: {e}")))?;

        // 3. Bersihkan file fisik tangkapan layar dan file sementara di disk
        let mut deleted_files_count = 0i64;
        for path_str in temp_file_paths {
            let path = Path::new(&path_str);
            if path.exists() && std::fs::remove_file(path).is_ok() {
                deleted_files_count += 1;
            }
        }

        // Jika direktori aplikasi diberikan, bersihkan subdirektori temporary / captures jika ada
        if let Some(data_dir) = app_data_dir {
            for sub in &["captures", "temp_captures"] {
                let dir = data_dir.join(sub);
                if dir.exists() && dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() && std::fs::remove_file(path).is_ok() {
                                deleted_files_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // 4. Lakukan WAL checkpoint TRUNCATE untuk mengosongkan berkas log WAL
        let wal_status = DeletionRepository::wal_checkpoint_truncate(&connection)
            .unwrap_or_else(|e| format!("WAL checkpoint gagal: {e}"));

        // 5. Lakukan VACUUM untuk mereklamasi alokasi disk dan membersihkan page sisa
        let vacuum_status = DeletionRepository::vacuum(&connection)
            .unwrap_or_else(|e| format!("VACUUM gagal: {e}"));

        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        // 6. Catat dan umumkan batasan penghapusan secara transparan
        let unaffected_external_items = vec![
            "Berkas ekspor JSON yang telah disimpan di luar direktori aplikasi oleh pengguna tidak terhapus otomatis.".to_string(),
            "Cadangan sistem eksternal atau backup drive di luar kendali aplikasi tidak terpengaruh.".to_string(),
            "Sisa alokasi blok fisik media penyimpanan (disk block allocation / unallocated space) berada di bawah kendali OS dan tidak dijamin terhapus secara cryptographic secure erase.".to_string(),
        ];

        Ok(WipeReport {
            deleted_personas_count: table_counts.personas,
            deleted_facts_count: table_counts.facts,
            deleted_sources_count: table_counts.sources,
            deleted_history_count: table_counts.history,
            deleted_sessions_count: table_counts.sessions,
            deleted_questions_count: table_counts.questions,
            deleted_recommendations_count: table_counts.recommendations,
            deleted_answers_count: table_counts.answers,
            deleted_preferences_count: table_counts.preferences,
            deleted_fit_results_count: table_counts.fit_results,
            deleted_attempts_count: table_counts.attempts,
            deleted_provider_configs_count: table_counts.provider_configs,
            deleted_temp_files_count: deleted_files_count,
            wal_checkpoint_status: wal_status,
            vacuum_status,
            wiped_at: now,
            unaffected_external_items,
        })
    }

    /// Menampilkan informasi kebijakan retensi data dan transparansi privasi.
    pub fn get_retention_policy_info() -> RetentionPolicyInfo {
        RetentionPolicyInfo {
            active_facts_policy: "Disimpan secara lokal di SQLite; hanya fakta relevan, terverifikasi, dan bukan fakta sensitif yang dapat dikirimkan ke model AI atas persetujuan pengguna.".into(),
            archived_facts_policy: "Disimpan dalam basis data lokal namun tidak aktif; tidak pernah dikirim ke model AI atau digunakan sebagai bukti jawaban rekomendasi.".into(),
            screen_capture_policy: "Tangkapan layar diproses di memori/file sementara dan otomatis dihapus segera setelah OCR selesai, kecuali pengguna secara eksplisit menyimpannya.".into(),
            ocr_data_policy: "Hasil teks OCR disimpan sebagai pertanyaan survei lokal yang dapat diedit oleh pengguna. Gambar mentah tidak diunggah ke penyedia eksternal tanpa izin eksplisit.".into(),
            survey_history_policy: "Riwayat sesi dan pertanyaan survei disimpan secara terisolasi per persona untuk audit dan pengenalan survei (Survey Memory).".into(),
            export_file_policy: "Berkas ekspor JSON dibuat atas permintaan eksplisit pengguna; sama sekali tidak pernah menyertakan kunci API atau kredensial rahasia apa pun.".into(),
            logs_and_diagnostics_policy: "Seluruh log diagnostik lokal otomatis meredaksi nilai sensitif, nomor identitas, dan kunci rahasia.".into(),
            deletion_scope_disclaimer: "Penghapusan aplikasi menghapus seluruh record lokal di SQLite, file sementara, dan memadatkan disk via VACUUM & WAL checkpoint. Berkas cadangan yang disimpan pengguna di luar aplikasi tidak terhapus.".into(),
        }
    }
}
