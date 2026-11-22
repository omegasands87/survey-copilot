use crate::domain::error::AppError;

/// Kontrak penyimpanan kredensial rahasia OS (Windows Credential Manager).
///
/// Memisahkan penyimpanan rahasia mentah dari basis data SQLite.
/// Komponen ini hanya menyimpan secret di memory/OS credential store,
/// sedangkan SQLite hanya menyimpan referensi buram (`credential_ref`).
pub trait CredentialStore: Send + Sync {
    /// Menyimpan atau memperbarui secret rahasia pada target name tertentu.
    fn set_secret(&self, target_name: &str, secret: &str) -> Result<(), AppError>;

    /// Membaca secret rahasia untuk penggunaan internal backend (misal: otentikasi provider).
    /// PERINGATAN: Nilai kembalian TIDAK BOLEH dikirim ke antarmuka pengguna frontend!
    fn get_secret(&self, target_name: &str) -> Result<String, AppError>;

    /// Menghapus secret rahasia dari penyimpanan OS.
    fn delete_secret(&self, target_name: &str) -> Result<(), AppError>;

    /// Memeriksa apakah secret ada dan dapat diakses pada target name tertentu.
    fn exists(&self, target_name: &str) -> Result<bool, AppError>;
}
