use crate::domain::persona::{PersonaDependencySummary, PersonaDetail, PersonaSummary};
use rusqlite::{Connection, OptionalExtension};

pub struct PersonaRepository;

impl PersonaRepository {
    /// Mengambil daftar seluruh persona dengan opsi menyertakan persona yang diarsipkan.
    pub fn list_all(
        connection: &Connection,
        include_archived: bool,
    ) -> Result<Vec<PersonaSummary>, rusqlite::Error> {
        let include_archived_int = if include_archived { 1 } else { 0 };

        let mut statement = connection.prepare(
            "SELECT p.id, p.display_name, p.description, p.is_active, p.status, p.created_at, p.updated_at, p.archived_at,
                    (SELECT COUNT(1) FROM profile_facts f WHERE f.persona_id = p.id) AS facts_count,
                    (SELECT COUNT(1) FROM survey_sessions s WHERE s.persona_id = p.id) AS sessions_count
             FROM personas p
             WHERE (?1 = 1 OR p.status = 'active')
             ORDER BY p.is_active DESC, p.status ASC, p.display_name COLLATE NOCASE ASC",
        )?;

        let rows = statement.query_map([include_archived_int], |row| {
            Ok(PersonaSummary {
                id: row.get(0)?,
                display_name: row.get(1)?,
                description: row.get(2)?,
                is_active: row.get::<_, i64>(3)? == 1,
                status: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                archived_at: row.get(7)?,
                facts_count: row.get(8)?,
                sessions_count: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    /// Mengambil daftar persona yang berstatus aktif saja (kompatibilitas).
    pub fn list_active(connection: &Connection) -> Result<Vec<PersonaSummary>, rusqlite::Error> {
        Self::list_all(connection, false)
    }

    /// Mencari persona berdasarkan ID.
    pub fn find_by_id(
        connection: &Connection,
        persona_id: &str,
    ) -> Result<Option<PersonaSummary>, rusqlite::Error> {
        connection
            .query_row(
                "SELECT p.id, p.display_name, p.description, p.is_active, p.status, p.created_at, p.updated_at, p.archived_at,
                        (SELECT COUNT(1) FROM profile_facts f WHERE f.persona_id = p.id) AS facts_count,
                        (SELECT COUNT(1) FROM survey_sessions s WHERE s.persona_id = p.id) AS sessions_count
                 FROM personas p
                 WHERE p.id = ?1",
                [persona_id],
                |row| {
                    Ok(PersonaSummary {
                        id: row.get(0)?,
                        display_name: row.get(1)?,
                        description: row.get(2)?,
                        is_active: row.get::<_, i64>(3)? == 1,
                        status: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                        archived_at: row.get(7)?,
                        facts_count: row.get(8)?,
                        sessions_count: row.get(9)?,
                    })
                },
            )
            .optional()
    }

    /// Mengambil detail lengkap persona.
    pub fn get_detail(
        connection: &Connection,
        persona_id: &str,
    ) -> Result<Option<PersonaDetail>, rusqlite::Error> {
        connection
            .query_row(
                "SELECT p.id, p.display_name, p.description, p.is_active, p.status, p.created_at, p.updated_at, p.archived_at,
                        (SELECT COUNT(1) FROM profile_facts f WHERE f.persona_id = p.id) AS facts_count,
                        (SELECT COUNT(1) FROM survey_sessions s WHERE s.persona_id = p.id) AS sessions_count
                 FROM personas p
                 WHERE p.id = ?1",
                [persona_id],
                |row| {
                    Ok(PersonaDetail {
                        id: row.get(0)?,
                        display_name: row.get(1)?,
                        description: row.get(2)?,
                        is_active: row.get::<_, i64>(3)? == 1,
                        status: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                        archived_at: row.get(7)?,
                        facts_count: row.get(8)?,
                        sessions_count: row.get(9)?,
                    })
                },
            )
            .optional()
    }

    /// Menentukan apakah ada persona yang sedang aktif (is_active = 1).
    pub fn has_any_active(connection: &Connection) -> Result<bool, rusqlite::Error> {
        let count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM personas WHERE is_active = 1 AND status = 'active'",
            [],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Menyisipkan persona baru.
    pub fn create(
        connection: &Connection,
        id: &str,
        display_name: &str,
        description: Option<&str>,
        is_active: bool,
        now: &str,
    ) -> Result<(), rusqlite::Error> {
        let is_active_int = if is_active { 1 } else { 0 };
        connection.execute(
            "INSERT INTO personas (id, display_name, description, is_active, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5)",
            (id, display_name, description, is_active_int, now),
        )?;
        Ok(())
    }

    /// Memperbarui nama dan deskripsi persona.
    pub fn update(
        connection: &Connection,
        persona_id: &str,
        display_name: &str,
        description: Option<&str>,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        connection.execute(
            "UPDATE personas
             SET display_name = ?1, description = ?2, updated_at = ?3
             WHERE id = ?4",
            (display_name, description, now, persona_id),
        )
    }

    /// Mengarsipkan persona dan mencatat waktu arsip.
    pub fn archive(
        connection: &Connection,
        persona_id: &str,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        connection.execute(
            "UPDATE personas
             SET status = 'archived', is_active = 0, archived_at = ?1, updated_at = ?1
             WHERE id = ?2",
            (now, persona_id),
        )
    }

    /// Memulihkan persona dari arsip ke status aktif.
    pub fn restore(
        connection: &Connection,
        persona_id: &str,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        connection.execute(
            "UPDATE personas
             SET status = 'active', archived_at = NULL, updated_at = ?1
             WHERE id = ?2 AND status = 'archived'",
            (now, persona_id),
        )
    }

    /// Menghapus baris persona (cascades via foreign keys di SQLite).
    pub fn delete(connection: &Connection, persona_id: &str) -> Result<usize, rusqlite::Error> {
        connection.execute("DELETE FROM personas WHERE id = ?1", [persona_id])
    }

    /// Mencari kandidat persona aktif pertama yang bukan excluded_id.
    pub fn find_first_active_candidate(
        connection: &Connection,
        exclude_persona_id: &str,
    ) -> Result<Option<String>, rusqlite::Error> {
        connection
            .query_row(
                "SELECT id FROM personas
                 WHERE status = 'active' AND id != ?1
                 ORDER BY created_at ASC
                 LIMIT 1",
                [exclude_persona_id],
                |row| row.get(0),
            )
            .optional()
    }

    /// Memeriksa apakah persona ada dan statusnya adalah 'active'.
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

    /// Menonaktifkan semua persona (`is_active = 0`).
    pub fn deactivate_all(connection: &Connection) -> Result<(), rusqlite::Error> {
        connection.execute("UPDATE personas SET is_active = 0 WHERE is_active = 1", [])?;
        Ok(())
    }

    /// Mengaktifkan persona tertentu (`is_active = 1`).
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

    /// Menghitung seluruh ketergantungan data milik persona sebelum penghapusan permanen.
    pub fn get_dependency_summary(
        connection: &Connection,
        persona_id: &str,
    ) -> Result<Option<PersonaDependencySummary>, rusqlite::Error> {
        let persona = match Self::find_by_id(connection, persona_id)? {
            Some(p) => p,
            None => return Ok(None),
        };

        let facts_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM profile_facts WHERE persona_id = ?1",
            [persona_id],
            |row| row.get(0),
        )?;

        let sources_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM fact_sources WHERE persona_id = ?1",
            [persona_id],
            |row| row.get(0),
        )?;

        let sessions_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM survey_sessions WHERE persona_id = ?1",
            [persona_id],
            |row| row.get(0),
        )?;

        let questions_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM survey_questions q
             JOIN survey_sessions s ON q.session_id = s.id
             WHERE s.persona_id = ?1",
            [persona_id],
            |row| row.get(0),
        )?;

        let recommendations_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM answer_recommendations r
             JOIN survey_questions q ON r.question_id = q.id
             JOIN survey_sessions s ON q.session_id = s.id
             WHERE s.persona_id = ?1",
            [persona_id],
            |row| row.get(0),
        )?;

        let answers_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM answer_history a
             JOIN survey_questions q ON a.question_id = q.id
             JOIN survey_sessions s ON q.session_id = s.id
             WHERE s.persona_id = ?1",
            [persona_id],
            |row| row.get(0),
        )?;

        let fit_results_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM survey_fit_results WHERE persona_id = ?1",
            [persona_id],
            |row| row.get(0),
        )?;

        let attempts_count: i64 = connection.query_row(
            "SELECT COUNT(1) FROM survey_attempts WHERE persona_id = ?1",
            [persona_id],
            |row| row.get(0),
        )?;

        let total = facts_count
            + sources_count
            + sessions_count
            + questions_count
            + recommendations_count
            + answers_count
            + fit_results_count
            + attempts_count;

        Ok(Some(PersonaDependencySummary {
            persona_id: persona.id,
            display_name: persona.display_name,
            facts_count,
            sources_count,
            sessions_count,
            questions_count,
            recommendations_count,
            answers_count,
            fit_results_count,
            attempts_count,
            total_associated_records: total,
        }))
    }
}
