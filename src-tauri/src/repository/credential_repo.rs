use crate::domain::credential::{AppCredential, CredentialStatus};
use rusqlite::{params, Connection, OptionalExtension, Row};

pub struct CredentialRepository;

impl CredentialRepository {
    fn map_row(row: &Row) -> Result<AppCredential, rusqlite::Error> {
        let status_str: String = row.get(4)?;
        Ok(AppCredential {
            id: row.get(0)?,
            provider_type: row.get(1)?,
            label: row.get(2)?,
            credential_ref: row.get(3)?,
            status: CredentialStatus::from_str(&status_str),
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }

    /// Menyimpan metadata kredensial baru ke basis data.
    /// CATATAN: Kunci API asli TIDAK PERNAH disimpan di basis data lokal.
    pub fn create(conn: &Connection, cred: &AppCredential) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO app_credentials
                (id, provider_type, label, credential_ref, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                cred.id,
                cred.provider_type,
                cred.label,
                cred.credential_ref,
                cred.status.as_str(),
                cred.created_at,
                cred.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Mencari metadata kredensial berdasarkan ID.
    pub fn find_by_id(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<AppCredential>, rusqlite::Error> {
        conn.query_row(
            "SELECT id, provider_type, label, credential_ref, status, created_at, updated_at
             FROM app_credentials
             WHERE id = ?1",
            params![id],
            Self::map_row,
        )
        .optional()
    }

    /// Mencari metadata kredensial berdasarkan referensi buram (`credential_ref`).
    pub fn find_by_ref(
        conn: &Connection,
        credential_ref: &str,
    ) -> Result<Option<AppCredential>, rusqlite::Error> {
        conn.query_row(
            "SELECT id, provider_type, label, credential_ref, status, created_at, updated_at
             FROM app_credentials
             WHERE credential_ref = ?1",
            params![credential_ref],
            Self::map_row,
        )
        .optional()
    }

    /// Mengambil seluruh daftar metadata kredensial berurutan dari yang terbaru.
    pub fn list_all(conn: &Connection) -> Result<Vec<AppCredential>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, provider_type, label, credential_ref, status, created_at, updated_at
             FROM app_credentials
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], Self::map_row)?;
        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        Ok(list)
    }

    /// Memperbarui label atau status metadata kredensial.
    pub fn update(
        conn: &Connection,
        id: &str,
        label: Option<&str>,
        status: Option<&str>,
        updated_at: &str,
    ) -> Result<bool, rusqlite::Error> {
        let affected = conn.execute(
            "UPDATE app_credentials
             SET label = COALESCE(?2, label),
                 status = COALESCE(?3, status),
                 updated_at = ?4
             WHERE id = ?1",
            params![id, label, status, updated_at],
        )?;
        Ok(affected > 0)
    }

    /// Menghapus rekaman metadata kredensial dari basis data.
    pub fn delete_by_id(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
        let affected = conn.execute("DELETE FROM app_credentials WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// Menghitung berapa banyak konfigurasi provider (`provider_configs`) yang sedang
    /// mereferensikan kredensial ini. Digunakan untuk mencegah penghapusan kredensial bersama.
    pub fn count_references_in_provider_configs(
        conn: &Connection,
        credential_ref: &str,
    ) -> Result<i64, rusqlite::Error> {
        conn.query_row(
            "SELECT count(*) FROM provider_configs WHERE credential_ref = ?1",
            params![credential_ref],
            |row| row.get(0),
        )
    }
}
