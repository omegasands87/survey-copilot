use crate::domain::fact::{FactSummary, ValidatedFactInput};
use rusqlite::{Connection, OptionalExtension};

pub struct FactRepository;

impl FactRepository {
    pub fn list_by_persona(
        connection: &Connection,
        persona_id: &str,
    ) -> Result<Vec<FactSummary>, rusqlite::Error> {
        let mut statement = connection.prepare(
            "SELECT f.id, f.category, f.attribute, f.value_json,
                    f.verification_status, f.sensitivity,
                    (SELECT description FROM fact_sources WHERE fact_id = f.id ORDER BY created_at LIMIT 1)
             FROM profile_facts f
             WHERE f.persona_id = ?1 AND f.archived_at IS NULL
             ORDER BY f.category COLLATE NOCASE, f.attribute COLLATE NOCASE",
        )?;

        let rows = statement.query_map([persona_id], |row| {
            let value_json: String = row.get(3)?;
            let value = serde_json::from_str(&value_json).unwrap_or(serde_json::Value::Null);
            Ok(FactSummary {
                id: row.get(0)?,
                category: row.get(1)?,
                attribute: row.get(2)?,
                value,
                verification_status: row.get(4)?,
                sensitivity: row.get(5)?,
                source_description: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    pub fn find_by_id(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
    ) -> Result<Option<FactSummary>, rusqlite::Error> {
        connection
            .query_row(
                "SELECT f.id, f.category, f.attribute, f.value_json,
                        f.verification_status, f.sensitivity,
                        (SELECT description FROM fact_sources WHERE fact_id = f.id ORDER BY created_at LIMIT 1)
                 FROM profile_facts f
                 WHERE f.id = ?1 AND f.persona_id = ?2 AND f.archived_at IS NULL",
                [fact_id, persona_id],
                |row| {
                    let value_json: String = row.get(3)?;
                    let value =
                        serde_json::from_str(&value_json).unwrap_or(serde_json::Value::Null);
                    Ok(FactSummary {
                        id: row.get(0)?,
                        category: row.get(1)?,
                        attribute: row.get(2)?,
                        value,
                        verification_status: row.get(4)?,
                        sensitivity: row.get(5)?,
                        source_description: row.get(6)?,
                    })
                },
            )
            .optional()
    }

    pub fn insert_fact_and_source(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        source_id: &str,
        input: &ValidatedFactInput,
        now: &str,
    ) -> Result<(), rusqlite::Error> {
        let value_json =
            serde_json::to_string(&input.value).unwrap_or_else(|_| format!("\"{}\"", input.value));

        connection.execute(
            "INSERT INTO profile_facts
             (id, persona_id, category, attribute, value_json, normalized_value,
              verification_status, sensitivity, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            (
                fact_id,
                persona_id,
                &input.category,
                &input.attribute,
                &value_json,
                &input.value,
                input.verification_status.as_str(),
                input.sensitivity.as_str(),
                now,
            ),
        )?;

        connection.execute(
            "INSERT INTO fact_sources
             (id, persona_id, fact_id, source_type, description, observed_at, created_at)
             VALUES (?1, ?2, ?3, 'user_input', ?4, ?5, ?5)",
            (
                source_id,
                persona_id,
                fact_id,
                &input.source_description,
                now,
            ),
        )?;

        Ok(())
    }

    pub fn update_status(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        status: &str,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        connection.execute(
            "UPDATE profile_facts SET verification_status = ?1, updated_at = ?2
             WHERE id = ?3 AND persona_id = ?4",
            (status, now, fact_id, persona_id),
        )
    }

    pub fn delete(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
    ) -> Result<usize, rusqlite::Error> {
        connection.execute(
            "DELETE FROM profile_facts WHERE id = ?1 AND persona_id = ?2",
            [fact_id, persona_id],
        )
    }
}
