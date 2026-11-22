use serde::{Deserialize, Serialize};

/// Status kesiapan kredensial di Windows Credential Manager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStatus {
    /// Kredensial tersimpan dan berhasil diverifikasi di Credential Manager
    Configured,
    /// Kredensial belum dikonfigurasi atau tidak ditemukan di penyimpanan OS
    Unconfigured,
    /// Kredensial perlu diperbarui (misal: kunci kadaluwarsa atau diubah)
    NeedsUpdate,
    /// Kredensial tidak dapat diakses (akses ditolak atau error sistem)
    Inaccessible,
}

impl CredentialStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Configured => "configured",
            Self::Unconfigured => "unconfigured",
            Self::NeedsUpdate => "needs_update",
            Self::Inaccessible => "inaccessible",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "configured" => Self::Configured,
            "unconfigured" => Self::Unconfigured,
            "needs_update" => Self::NeedsUpdate,
            "inaccessible" => Self::Inaccessible,
            _ => Self::Unconfigured,
        }
    }
}

/// Metadata kredensial tersimpan di basis data lokal SQLite.
/// CATATAN KEAMANAN: Struct ini HANYA memuat referensi buram (opaque reference)
/// dan metadata non-sensitif. Kunci rahasia asli TIDAK PERNAH disimpan di sini.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCredential {
    pub id: String,
    pub provider_type: String,
    pub label: String,
    pub credential_ref: String,
    pub status: CredentialStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// Ringkasan publik kredensial yang aman dikembalikan melalui Tauri IPC ke frontend.
/// TIDAK MEMUAT rahasia, token, atau kunci API asli dalam bentuk apa pun.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialSummary {
    pub id: String,
    pub provider_type: String,
    pub label: String,
    pub credential_ref: String,
    pub status: CredentialStatus,
    pub usage_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Input penyimpanan kredensial baru dari antarmuka pengguna.
/// Nilai `secret` hanya diproses di backend untuk disimpan ke Windows Credential Manager
/// dan segera dibersihkan dari memori tanpa pernah disimpan ke basis data lokal.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCredentialInput {
    pub id: Option<String>,
    pub provider_type: String,
    pub label: String,
    pub secret: String,
}

/// Input pembaruan kredensial yang telah ada.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCredentialInput {
    pub id: String,
    pub label: Option<String>,
    pub secret: Option<String>,
}

/// Hasil operasi penghapusan kredensial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCredentialResult {
    pub id: String,
    pub credential_ref: String,
    pub deleted_from_store: bool,
    pub deleted_from_database: bool,
    pub message: String,
}

/// Hasil pengujian keberadaan dan akses kredensial di OS Credential Store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCredentialResult {
    pub id: String,
    pub credential_ref: String,
    pub is_valid: bool,
    pub status: CredentialStatus,
    pub message: String,
    pub tested_at: String,
}
