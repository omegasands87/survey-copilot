use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::fact::{CreateFactInput, FactSummary, VerificationStatus};
use crate::repository::{FactRepository, PersonaRepository};
use uuid::Uuid;

pub struct FactService;

impl FactService {
    pub fn list(database: &Database, persona_id: &str) -> Result<Vec<FactSummary>, AppError> {
        let persona_id = persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        FactRepository::list_by_persona(&connection, persona_id).map_err(AppError::from)
    }

    pub fn create(database: &Database, input: CreateFactInput) -> Result<FactSummary, AppError> {
        let validated = input.validate()?;

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let transaction = connection
            .transaction()
            .map_err(|error| AppError::database(format!("Gagal memulai transaksi: {error}")))?;

        let persona_exists =
            PersonaRepository::is_active_and_exists(&transaction, &validated.persona_id)
                .map_err(AppError::from)?;
        if !persona_exists {
            return Err(AppError::not_found(
                "Profil tidak ditemukan atau sudah tidak aktif.",
            ));
        }

        let fact_id = Uuid::new_v4().to_string();
        let source_id = Uuid::new_v4().to_string();
        let now = chrono_timestamp();

        FactRepository::insert_fact_and_source(
            &transaction,
            &validated.persona_id,
            &fact_id,
            &source_id,
            &validated,
            &now,
        )
        .map_err(AppError::from)?;

        transaction
            .commit()
            .map_err(|error| AppError::database(format!("Gagal menyimpan informasi: {error}")))?;

        FactRepository::find_by_id(&connection, &validated.persona_id, &fact_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Informasi yang disimpan tidak ditemukan."))
    }

    pub fn set_status(
        database: &Database,
        persona_id: &str,
        fact_id: &str,
        status: &str,
    ) -> Result<(), AppError> {
        let persona_id = persona_id.trim();
        let fact_id = fact_id.trim();

        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan ID informasi wajib diisi.",
            ));
        }

        let verification_status = VerificationStatus::parse(status)?;
        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let now = chrono_timestamp();
        let changed = FactRepository::update_status(
            &connection,
            persona_id,
            fact_id,
            verification_status.as_str(),
            &now,
        )
        .map_err(AppError::from)?;

        if changed == 0 {
            return Err(AppError::not_found("Informasi profil tidak ditemukan."));
        }

        Ok(())
    }

    pub fn delete(database: &Database, persona_id: &str, fact_id: &str) -> Result<(), AppError> {
        let persona_id = persona_id.trim();
        let fact_id = fact_id.trim();

        if persona_id.is_empty() || fact_id.is_empty() {
            return Err(AppError::validation(
                "id",
                "ID profil dan ID informasi wajib diisi.",
            ));
        }

        let connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;

        let changed =
            FactRepository::delete(&connection, persona_id, fact_id).map_err(|error| {
                AppError::conflict(format!(
                    "Informasi ini masih dipakai oleh riwayat dan tidak dapat dihapus: {error}"
                ))
            })?;

        if changed == 0 {
            return Err(AppError::not_found("Informasi profil tidak ditemukan."));
        }

        Ok(())
    }
}

fn chrono_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::persona::CreatePersonaInput;
    use crate::services::PersonaService;
    use std::path::Path;

    fn setup_test_db() -> Database {
        Database::initialize(Path::new(":memory:")).expect("memory database should initialize")
    }

    #[test]
    fn strict_persona_isolation_enforced() {
        let db = setup_test_db();

        let persona_a = PersonaService::create(&db, CreatePersonaInput::new("Persona A"))
            .expect("create persona A");
        let persona_b = PersonaService::create(&db, CreatePersonaInput::new("Persona B"))
            .expect("create persona B");

        // Add fact to Persona A
        let fact_a = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona_a.id.clone(),
                category: "Pekerjaan".to_string(),
                attribute: "Profesi".to_string(),
                value: "Dokter".to_string(),
                source_description: "Kartu ID".to_string(),
                verification_status: "confirmed".to_string(),
                sensitivity: "ordinary".to_string(),
            },
        )
        .expect("create fact for persona A");

        // Verify Persona A sees the fact
        let facts_a = FactService::list(&db, &persona_a.id).expect("list facts for A");
        assert_eq!(facts_a.len(), 1);
        assert_eq!(facts_a[0].id, fact_a.id);

        // Verify Persona B NEVER sees Persona A's fact
        let facts_b = FactService::list(&db, &persona_b.id).expect("list facts for B");
        assert_eq!(facts_b.len(), 0);

        // Persona B cannot modify or delete Persona A's fact
        let update_err =
            FactService::set_status(&db, &persona_b.id, &fact_a.id, "disputed").unwrap_err();
        assert_eq!(update_err.code, "NOT_FOUND");

        let delete_err = FactService::delete(&db, &persona_b.id, &fact_a.id).unwrap_err();
        assert_eq!(delete_err.code, "NOT_FOUND");
    }

    #[test]
    fn fact_lifecycle_create_update_delete() {
        let db = setup_test_db();
        let persona =
            PersonaService::create(&db, CreatePersonaInput::new("Budi")).expect("create persona");

        let fact = FactService::create(
            &db,
            CreateFactInput {
                persona_id: persona.id.clone(),
                category: "Demografi".to_string(),
                attribute: "Kota".to_string(),
                value: "Surabaya".to_string(),
                source_description: "KTP".to_string(),
                verification_status: "unverified".to_string(),
                sensitivity: "ordinary".to_string(),
            },
        )
        .expect("create fact");

        assert_eq!(fact.verification_status, "unverified");

        FactService::set_status(&db, &persona.id, &fact.id, "confirmed")
            .expect("update status to confirmed");

        let updated_facts = FactService::list(&db, &persona.id).expect("list facts");
        assert_eq!(updated_facts[0].verification_status, "confirmed");

        FactService::delete(&db, &persona.id, &fact.id).expect("delete fact");
        let remaining_facts = FactService::list(&db, &persona.id).expect("list facts after delete");
        assert_eq!(remaining_facts.len(), 0);
    }
}
