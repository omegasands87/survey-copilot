use crate::domain::persona::PersonaSummary;
use rusqlite::{Connection, OptionalExtension};

pub struct PersonaRepository;

impl PersonaRepository {
    pub fn list_active(connection: &Connection) -> Result<Vec<PersonaSummary>, rusqlite::Error> {
        let mut statement = connection.prepare(
            "SELECT id, display_name, is_active, status
             FROM personas
             WHERE status = 'active'
             ORDER BY is_active DESC, display_name COLLATE NOCASE ASC",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(PersonaSummary {
                id: row.get(0)?,
                display_name: row.get(1)?,
                is_active: row.get::<_, i64>(2)? == 1,
                status: row.get(3)?,
            })
        })?;

        rows.collect()
    }

    pub fn find_by_id(
        connection: &Connection,
        persona_id: &str,
    ) -> Result<Option<PersonaSummary>, rusqlite::Error> {
        connection
            .query_row(
                "SELECT id, display_name, is_active, status FROM personas WHERE id = ?1",
                [persona_id],
                |row| {
                    Ok(PersonaSummary {
                        id: row.get(0)?,
                        display_name: row.get(1)?,
                        is_active: row.get::<_, i64>(2)? == 1,
                        status: row.get(3)?,
                    })
                },
            )
            .optional()
    }

    pub fn has_any_active(connection: &Connection) -> Result<bool, rusqlite::Error> {
        let count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM personas WHERE is_active = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn create(
        connection: &Connection,
        id: &str,
        display_name: &str,
        is_active: bool,
        now: &str,
    ) -> Result<(), rusqlite::Error> {
        let is_active_int = if is_active { 1 } else { 0 };
        connection.execute(
            "INSERT INTO personas (id, display_name, is_active, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?4)",
            (id, display_name, is_active_int, now),
        )?;
        Ok(())
    }

    pub fn is_active_and_exists(
        connection: &Connection,
        persona_id: &str,
    ) -> Result<bool, rusqlite::Error> {
        let exists: Option<String> = connection
            .query_row(
                "SELECT id FROM personas WHERE id = ?1 AND status = 'active'",
                [persona_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(exists.is_some())
    }

    pub fn deactivate_all(connection: &Connection) -> Result<(), rusqlite::Error> {
        connection.execute("UPDATE personas SET is_active = 0 WHERE is_active = 1", [])?;
        Ok(())
    }

    pub fn activate(
        connection: &Connection,
        persona_id: &str,
        now: &str,
    ) -> Result<(), rusqlite::Error> {
        connection.execute(
            "UPDATE personas SET is_active = 1, updated_at = ?1 WHERE id = ?2",
            (now, persona_id),
        )?;
        Ok(())
    }
}
