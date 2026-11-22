use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::persona::{CreatePersonaInput, PersonaSummary};
use crate::repository::PersonaRepository;
use uuid::Uuid;

pub struct PersonaService;

impl PersonaService {
    pub fn list(database: &Database) -> Result<Vec<PersonaSummary>, AppError> {
        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        PersonaRepository::list_active(&connection).map_err(AppError::from)
    }

    pub fn create(
        database: &Database,
        input: CreatePersonaInput,
    ) -> Result<PersonaSummary, AppError> {
        let display_name = input.validate()?;

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let has_active = PersonaRepository::has_any_active(&connection).map_err(AppError::from)?;
        let is_active = !has_active;

        let id = Uuid::new_v4().to_string();
        let now = chrono_timestamp();

        PersonaRepository::create(&connection, &id, &display_name, is_active, &now)
            .map_err(AppError::from)?;

        PersonaRepository::find_by_id(&connection, &id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil yang baru dibuat tidak ditemukan."))
    }

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

        let exists = PersonaRepository::is_active_and_exists(&transaction, persona_id)
            .map_err(AppError::from)?;
        if !exists {
            return Err(AppError::not_found(
                "Profil tidak ditemukan atau sudah tidak aktif.",
            ));
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
    fn persona_lifecycle_first_is_active_second_is_not() {
        let db = setup_test_db();

        let first = PersonaService::create(&db, CreatePersonaInput::new("Budi"))
            .expect("create first persona");
        assert!(first.is_active);

        let second = PersonaService::create(&db, CreatePersonaInput::new("Siti"))
            .expect("create second persona");
        assert!(!second.is_active);

        let personas = PersonaService::list(&db).expect("list personas");
        assert_eq!(personas.len(), 2);
        assert_eq!(personas[0].display_name, "Budi");
        assert!(personas[0].is_active);

        let activated = PersonaService::set_active(&db, &second.id).expect("activate second");
        assert!(activated.is_active);

        let personas_after = PersonaService::list(&db).expect("list personas after switch");
        let active_count = personas_after.iter().filter(|p| p.is_active).count();
        assert_eq!(active_count, 1);
        assert_eq!(personas_after[0].id, second.id);
    }

    #[test]
    fn set_active_non_existent_fails() {
        let db = setup_test_db();
        let err = PersonaService::set_active(&db, "non-existent-id").unwrap_err();
        assert_eq!(err.code, "NOT_FOUND");
    }
}
