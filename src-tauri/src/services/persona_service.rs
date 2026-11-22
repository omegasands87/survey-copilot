use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::persona::{
    CreatePersonaInput, PersonaDependencySummary, PersonaDetail, PersonaSummary, UpdatePersonaInput,
};
use crate::repository::PersonaRepository;
use uuid::Uuid;

pub struct PersonaService;

impl PersonaService {
    /// Mengambil daftar seluruh persona dengan opsi menyertakan arsip.
    pub fn list(
        database: &Database,
        include_archived: bool,
    ) -> Result<Vec<PersonaSummary>, AppError> {
        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        PersonaRepository::list_all(&connection, include_archived).map_err(AppError::from)
    }

    /// Mengambil detail lengkap satu persona.
    pub fn get_detail(database: &Database, persona_id: &str) -> Result<PersonaDetail, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        PersonaRepository::get_detail(&connection, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))
    }

    /// Membuat persona baru. Persona pertama akan otomatis menjadi aktif.
    pub fn create(
        database: &Database,
        input: CreatePersonaInput,
    ) -> Result<PersonaSummary, AppError> {
        let (display_name, description) = input.validate()?;

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        let has_active = PersonaRepository::has_any_active(&transaction).map_err(AppError::from)?;
        let is_active = !has_active;

        let id = Uuid::new_v4().to_string();
        let now = chrono_timestamp();

        PersonaRepository::create(
            &transaction,
            &id,
            &display_name,
            description.as_deref(),
            is_active,
            &now,
        )
        .map_err(AppError::from)?;

        transaction
            .commit()
            .map_err(|e| AppError::database(format!("Gagal menyimpan profil: {e}")))?;

        PersonaRepository::find_by_id(&connection, &id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil yang baru dibuat tidak ditemukan."))
    }

    /// Memperbarui nama dan deskripsi persona.
    pub fn update(
        database: &Database,
        persona_id: &str,
        input: UpdatePersonaInput,
    ) -> Result<PersonaDetail, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let (display_name, description) = input.validate()?;

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        let current = PersonaRepository::find_by_id(&transaction, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))?;

        if current.status != "active" {
            return Err(AppError::validation(
                "status",
                "Profil yang diarsipkan harus dipulihkan terlebih dahulu sebelum diedit.",
            ));
        }

        let now = chrono_timestamp();
        PersonaRepository::update(
            &transaction,
            persona_id,
            &display_name,
            description.as_deref(),
            &now,
        )
        .map_err(AppError::from)?;

        transaction
            .commit()
            .map_err(|e| AppError::database(format!("Gagal memperbarui profil: {e}")))?;

        PersonaRepository::get_detail(&connection, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan setelah pembaruan."))
    }

    /// Menetapkan persona aktif. Menggunakan transaksi atomik untuk memastikan hanya tepat 1 yang aktif.
    pub fn set_active(database: &Database, persona_id: &str) -> Result<PersonaSummary, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|error| AppError::database(format!("Gagal memulai transaksi: {error}")))?;

        let current = PersonaRepository::find_by_id(&transaction, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))?;

        if current.status != "active" {
            return Err(AppError::validation(
                "status",
                "Profil yang diarsipkan tidak dapat diaktifkan. Pulihkan profil terlebih dahulu.",
            ));
        }

        if current.is_active {
            // Sudah aktif, tidak perlu perubahan transaksi
            return Ok(current);
        }

        PersonaRepository::deactivate_all(&transaction).map_err(AppError::from)?;

        let now = chrono_timestamp();
        PersonaRepository::activate(&transaction, persona_id, &now).map_err(AppError::from)?;

        transaction
            .commit()
            .map_err(|error| AppError::database(format!("Gagal menyimpan transaksi: {error}")))?;

        PersonaRepository::find_by_id(&connection, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))
    }

    /// Mengarsipkan persona. Jika yang diarsipkan adalah profil aktif,
    /// sistem akan secara atomik menunjuk profil aktif pengganti.
    pub fn archive(database: &Database, persona_id: &str) -> Result<PersonaSummary, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        let current = PersonaRepository::find_by_id(&transaction, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))?;

        if current.status == "archived" {
            return Err(AppError::validation(
                "status",
                "Profil sudah dalam kondisi diarsipkan.",
            ));
        }

        let now = chrono_timestamp();

        // Jika persona ini aktif, nonaktifkan dan alihkan ke kandidat persona aktif lain jika ada
        if current.is_active {
            PersonaRepository::deactivate_all(&transaction).map_err(AppError::from)?;

            if let Some(replacement_id) =
                PersonaRepository::find_first_active_candidate(&transaction, persona_id)
                    .map_err(AppError::from)?
            {
                PersonaRepository::activate(&transaction, &replacement_id, &now)
                    .map_err(AppError::from)?;
            }
        }

        PersonaRepository::archive(&transaction, persona_id, &now).map_err(AppError::from)?;

        transaction
            .commit()
            .map_err(|e| AppError::database(format!("Gagal mengarsipkan profil: {e}")))?;

        PersonaRepository::find_by_id(&connection, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan setelah pengarsipan."))
    }

    /// Memulihkan persona dari arsip. Jika belum ada profil aktif,
    /// profil yang dipulihkan akan langsung diaktifkan.
    pub fn restore(database: &Database, persona_id: &str) -> Result<PersonaSummary, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        let current = PersonaRepository::find_by_id(&transaction, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))?;

        if current.status != "archived" {
            return Err(AppError::validation(
                "status",
                "Profil tidak sedang dalam kondisi diarsipkan.",
            ));
        }

        let now = chrono_timestamp();
        PersonaRepository::restore(&transaction, persona_id, &now).map_err(AppError::from)?;

        // Periksa apakah ada persona yang sedang aktif. Jika tidak ada, aktifkan profil yang dipulihkan ini
        let has_active = PersonaRepository::has_any_active(&transaction).map_err(AppError::from)?;
        if !has_active {
            PersonaRepository::activate(&transaction, persona_id, &now).map_err(AppError::from)?;
        }

        transaction
            .commit()
            .map_err(|e| AppError::database(format!("Gagal memulihkan profil: {e}")))?;

        PersonaRepository::find_by_id(&connection, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan setelah pemulihan."))
    }

    /// Mendapatkan ringkasan ketergantungan data sebelum konfirmasi penghapusan permanen.
    pub fn get_dependency_summary(
        database: &Database,
        persona_id: &str,
    ) -> Result<PersonaDependencySummary, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        PersonaRepository::get_dependency_summary(&connection, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))
    }

    /// Menghapus persona beserta seluruh datanya secara permanen.
    /// Operasi berjalan dalam transaksi; jika persona aktif dihapus, kandidat lain akan diaktifkan secara atomik.
    pub fn delete(
        database: &Database,
        persona_id: &str,
    ) -> Result<PersonaDependencySummary, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        let summary = PersonaRepository::get_dependency_summary(&transaction, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))?;

        let current = PersonaRepository::find_by_id(&transaction, persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil tidak ditemukan."))?;

        let now = chrono_timestamp();

        // Jika persona ini aktif, alihkan ke kandidat persona aktif lain jika ada
        if current.is_active {
            PersonaRepository::deactivate_all(&transaction).map_err(AppError::from)?;

            if let Some(replacement_id) =
                PersonaRepository::find_first_active_candidate(&transaction, persona_id)
                    .map_err(AppError::from)?
            {
                PersonaRepository::activate(&transaction, &replacement_id, &now)
                    .map_err(AppError::from)?;
            }
        }

        // Hapus persona. Foreign keys dengan ON DELETE CASCADE akan otomatis menghapus record terkait
        let deleted =
            PersonaRepository::delete(&transaction, persona_id).map_err(AppError::from)?;
        if deleted == 0 {
            return Err(AppError::not_found(
                "Gagal menghapus: Profil tidak ditemukan.",
            ));
        }

        transaction
            .commit()
            .map_err(|e| AppError::database(format!("Gagal menghapus profil: {e}")))?;

        Ok(summary)
    }
}

fn chrono_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn setup_test_db() -> Database {
        Database::initialize(Path::new(":memory:")).expect("memory database should initialize")
    }

    #[test]
    fn persona_lifecycle_full_crud_and_status() {
        let db = setup_test_db();

        // 1. Create first persona -> should be active
        let p1 = PersonaService::create(
            &db,
            CreatePersonaInput::with_description("Budi Santoso", Some("Responden 1".to_string())),
        )
        .expect("create first persona");
        assert!(p1.is_active);
        assert_eq!(p1.display_name, "Budi Santoso");
        assert_eq!(p1.description, Some("Responden 1".to_string()));

        // 2. Create second persona -> should NOT be active
        let p2 = PersonaService::create(
            &db,
            CreatePersonaInput::with_description("Siti Aminah", None),
        )
        .expect("create second persona");
        assert!(!p2.is_active);

        // 3. Update second persona
        let updated_p2 = PersonaService::update(
            &db,
            &p2.id,
            UpdatePersonaInput::new("Siti Rahmawati", Some("Profil Kedua".to_string())),
        )
        .expect("update second persona");
        assert_eq!(updated_p2.display_name, "Siti Rahmawati");
        assert_eq!(updated_p2.description, Some("Profil Kedua".to_string()));

        // 4. Switch active to p2
        let activated_p2 = PersonaService::set_active(&db, &p2.id).expect("activate second");
        assert!(activated_p2.is_active);

        let list_after_switch = PersonaService::list(&db, false).expect("list personas");
        assert_eq!(list_after_switch.len(), 2);
        assert_eq!(list_after_switch[0].id, p2.id);
        assert!(list_after_switch[0].is_active);
        assert!(!list_after_switch[1].is_active);

        // 5. Archive active persona (p2) -> p1 should become active automatically
        let archived_p2 = PersonaService::archive(&db, &p2.id).expect("archive p2");
        assert_eq!(archived_p2.status, "archived");
        assert!(!archived_p2.is_active);

        let p1_after_archive = PersonaService::get_detail(&db, &p1.id).expect("get p1");
        assert!(p1_after_archive.is_active);

        // 6. Restore p2
        let restored_p2 = PersonaService::restore(&db, &p2.id).expect("restore p2");
        assert_eq!(restored_p2.status, "active");
        assert!(!restored_p2.is_active); // p1 is already active

        // 7. Get dependency summary
        let summary =
            PersonaService::get_dependency_summary(&db, &p2.id).expect("dependency summary");
        assert_eq!(summary.persona_id, p2.id);
        assert_eq!(summary.display_name, "Siti Rahmawati");

        // 8. Delete p2
        let deleted_summary = PersonaService::delete(&db, &p2.id).expect("delete p2");
        assert_eq!(deleted_summary.persona_id, p2.id);

        let list_final = PersonaService::list(&db, true).expect("list all");
        assert_eq!(list_final.len(), 1);
        assert_eq!(list_final[0].id, p1.id);
    }

    #[test]
    fn cannot_activate_archived_persona() {
        let db = setup_test_db();
        let p = PersonaService::create(&db, CreatePersonaInput::new("Agus")).expect("create");
        PersonaService::archive(&db, &p.id).expect("archive");

        let err = PersonaService::set_active(&db, &p.id).unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
    }
}
