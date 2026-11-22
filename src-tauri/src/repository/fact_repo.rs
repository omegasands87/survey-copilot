use crate::domain::fact::{
    check_validity_period, ConflictWarning, FactHistoryEntry, FactSource, FactSummary,
    ProfileFactDetail, ValidatedFactInput,
};
use rusqlite::{Connection, OptionalExtension};

pub struct FactRepository;

impl FactRepository {
    /// Mengambil daftar ringkasan fakta milik persona tertentu.
    pub fn list_by_persona(
        connection: &Connection,
        persona_id: &str,
        include_archived: bool,
        now_iso: &str,
    ) -> Result<Vec<FactSummary>, rusqlite::Error> {
        let include_archived_int = if include_archived { 1 } else { 0 };

        let mut statement = connection.prepare(
            "SELECT f.id, f.persona_id, f.category, COALESCE(f.fact_key, lower(replace(f.attribute, ' ', '_'))),
                    f.attribute, COALESCE(f.data_type, 'string'), f.value_json,
                    COALESCE(f.normalized_value, ''), f.unit, f.verification_status, f.sensitivity,
                    (SELECT COALESCE(title, description) FROM fact_sources WHERE fact_id = f.id ORDER BY created_at ASC LIMIT 1),
                    f.valid_from, f.valid_until,
                    (SELECT COUNT(1) FROM fact_sources WHERE fact_id = f.id),
                    f.created_at, f.updated_at, f.archived_at
             FROM profile_facts f
             WHERE f.persona_id = ?1 AND (?2 = 1 OR f.archived_at IS NULL)
             ORDER BY f.category COLLATE NOCASE, f.attribute COLLATE NOCASE",
        )?;

        let rows =
            statement.query_map(rusqlite::params![persona_id, include_archived_int], |row| {
                let id: String = row.get(0)?;
                let persona_id: String = row.get(1)?;
                let category: String = row.get(2)?;
                let fact_key: String = row.get(3)?;
                let attribute: String = row.get(4)?;
                let data_type: String = row.get(5)?;
                let value_json: String = row.get(6)?;
                let normalized_value: String = row.get(7)?;
                let unit: Option<String> = row.get(8)?;
                let verification_status: String = row.get(9)?;
                let sensitivity: String = row.get(10)?;
                let source_description: Option<String> = row.get(11)?;
                let valid_from: Option<String> = row.get(12)?;
                let valid_until: Option<String> = row.get(13)?;
                let sources_count: i64 = row.get(14)?;
                let created_at: String = row.get(15)?;
                let updated_at: String = row.get(16)?;
                let archived_at: Option<String> = row.get(17)?;

                let value = serde_json::from_str(&value_json)
                    .unwrap_or(serde_json::Value::String(value_json));
                let is_valid =
                    check_validity_period(valid_from.as_deref(), valid_until.as_deref(), now_iso);

                Ok(FactSummary {
                    id,
                    persona_id,
                    category,
                    fact_key,
                    attribute,
                    data_type,
                    value,
                    normalized_value,
                    unit,
                    verification_status,
                    sensitivity,
                    source_description,
                    valid_from,
                    valid_until,
                    is_valid,
                    has_conflict: false, // akan ditandai jika ada konflik
                    sources_count,
                    created_at,
                    updated_at,
                    archived_at,
                })
            })?;

        let mut facts: Vec<FactSummary> = rows.collect::<Result<_, _>>()?;

        // Periksa konflik nilai antar-fakta pada fact_key yang sama
        let conflicts = Self::find_conflicts(connection, persona_id)?;
        let conflicting_keys: std::collections::HashSet<String> =
            conflicts.into_iter().map(|c| c.fact_key).collect();

        for fact in &mut facts {
            if conflicting_keys.contains(&fact.fact_key) {
                fact.has_conflict = true;
            }
        }

        Ok(facts)
    }

    /// Mengambil detail lengkap fakta beserta seluruh sumber bukti dan riwayat perubahannya.
    pub fn get_detail(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        now_iso: &str,
    ) -> Result<Option<ProfileFactDetail>, rusqlite::Error> {
        let fact_row = connection
            .query_row(
                "SELECT f.id, f.persona_id, f.category, COALESCE(f.fact_key, lower(replace(f.attribute, ' ', '_'))),
                        f.attribute, COALESCE(f.data_type, 'string'), f.value_json,
                        COALESCE(f.normalized_value, ''), f.unit, f.verification_status, f.sensitivity,
                        f.source_date, f.valid_from, f.valid_until, f.notes,
                        f.created_at, f.updated_at, f.archived_at
                 FROM profile_facts f
                 WHERE f.id = ?1 AND f.persona_id = ?2",
                [fact_id, persona_id],
                |row| {
                    let id: String = row.get(0)?;
                    let persona_id: String = row.get(1)?;
                    let category: String = row.get(2)?;
                    let fact_key: String = row.get(3)?;
                    let attribute: String = row.get(4)?;
                    let data_type: String = row.get(5)?;
                    let value_json: String = row.get(6)?;
                    let normalized_value: String = row.get(7)?;
                    let unit: Option<String> = row.get(8)?;
                    let verification_status: String = row.get(9)?;
                    let sensitivity: String = row.get(10)?;
                    let source_date: Option<String> = row.get(11)?;
                    let valid_from: Option<String> = row.get(12)?;
                    let valid_until: Option<String> = row.get(13)?;
                    let notes: Option<String> = row.get(14)?;
                    let created_at: String = row.get(15)?;
                    let updated_at: String = row.get(16)?;
                    let archived_at: Option<String> = row.get(17)?;

                    let value = serde_json::from_str(&value_json).unwrap_or(serde_json::Value::String(value_json));
                    let is_valid = check_validity_period(valid_from.as_deref(), valid_until.as_deref(), now_iso);

                    Ok((
                        id, persona_id, category, fact_key, attribute, data_type, value,
                        normalized_value, unit, verification_status, sensitivity,
                        source_date, valid_from, valid_until, notes, is_valid,
                        created_at, updated_at, archived_at,
                    ))
                },
            )
            .optional()?;

        let (
            id,
            persona_id,
            category,
            fact_key,
            attribute,
            data_type,
            value,
            normalized_value,
            unit,
            verification_status,
            sensitivity,
            source_date,
            valid_from,
            valid_until,
            notes,
            is_valid,
            created_at,
            updated_at,
            archived_at,
        ) = match fact_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // Query seluruh sumber bukti untuk fakta ini
        let mut source_stmt = connection.prepare(
            "SELECT id, persona_id, fact_id, source_type, COALESCE(title, description),
                    description, reference_uri, observed_at, valid_from, valid_until,
                    notes, created_at
             FROM fact_sources
             WHERE fact_id = ?1 AND persona_id = ?2
             ORDER BY created_at ASC",
        )?;

        let source_rows = source_stmt.query_map([fact_id, &persona_id], |row| {
            Ok(FactSource {
                id: row.get(0)?,
                persona_id: row.get(1)?,
                fact_id: row.get(2)?,
                source_type: row.get(3)?,
                title: row.get(4)?,
                description: row.get(5)?,
                reference_uri: row.get(6)?,
                observed_at: row.get(7)?,
                valid_from: row.get(8)?,
                valid_until: row.get(9)?,
                notes: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;

        let sources = source_rows.collect::<Result<Vec<FactSource>, _>>()?;

        // Query riwayat perubahan fakta ini
        let mut hist_stmt = connection.prepare(
            "SELECT id, persona_id, fact_id, previous_value_json, new_value_json,
                    previous_status, new_status, change_reason, source_id, changed_at
             FROM fact_history
             WHERE fact_id = ?1 AND persona_id = ?2
             ORDER BY changed_at DESC",
        )?;

        let history_rows = hist_stmt.query_map([fact_id, &persona_id], |row| {
            Ok(FactHistoryEntry {
                id: row.get(0)?,
                persona_id: row.get(1)?,
                fact_id: row.get(2)?,
                previous_value_json: row.get(3)?,
                new_value_json: row.get(4)?,
                previous_status: row.get(5)?,
                new_status: row.get(6)?,
                change_reason: row.get(7)?,
                source_id: row.get(8)?,
                changed_at: row.get(9)?,
            })
        })?;

        let history = history_rows.collect::<Result<Vec<FactHistoryEntry>, _>>()?;

        Ok(Some(ProfileFactDetail {
            id,
            persona_id,
            category,
            fact_key,
            attribute,
            data_type,
            value,
            normalized_value,
            unit,
            verification_status,
            sensitivity,
            source_date,
            valid_from,
            valid_until,
            notes,
            is_valid,
            created_at,
            updated_at,
            archived_at,
            sources,
            history,
        }))
    }

    /// Menyisipkan fakta baru beserta sumber bukti dan riwayat perubahan awal.
    pub fn insert_fact_and_source(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        source_id: &str,
        history_id: &str,
        input: &ValidatedFactInput,
        now: &str,
    ) -> Result<(), rusqlite::Error> {
        let value_json =
            serde_json::to_string(&input.value).unwrap_or_else(|_| input.normalized_value.clone());

        connection.execute(
            "INSERT INTO profile_facts
             (id, persona_id, category, fact_key, attribute, data_type, value_json, normalized_value,
              unit, verification_status, sensitivity, source_date, valid_from, valid_until,
              notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16)",
            rusqlite::params![
                fact_id,
                persona_id,
                &input.category,
                &input.fact_key,
                &input.attribute,
                input.data_type.as_str(),
                &value_json,
                &input.normalized_value,
                input.unit.as_deref(),
                input.verification_status.as_str(),
                input.sensitivity.as_str(),
                input.observed_at.as_deref(),
                input.valid_from.as_deref(),
                input.valid_until.as_deref(),
                input.notes.as_deref(),
                now,
            ],
        )?;

        connection.execute(
            "INSERT INTO fact_sources
             (id, persona_id, fact_id, source_type, title, description, reference_uri, observed_at,
              valid_from, valid_until, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                source_id,
                persona_id,
                fact_id,
                input.source_type.as_str(),
                &input.source_title,
                &input.source_description,
                input.reference_uri.as_deref(),
                input.observed_at.as_deref(),
                input.valid_from.as_deref(),
                input.valid_until.as_deref(),
                input.notes.as_deref(),
                now,
            ],
        )?;

        // Catat entri riwayat pembuatan fakta
        connection.execute(
            "INSERT INTO fact_history
             (id, persona_id, fact_id, previous_value_json, new_value_json, previous_status,
              new_status, change_reason, source_id, changed_at)
             VALUES (?1, ?2, ?3, NULL, ?4, NULL, ?5, 'Fakta baru dibuat', ?6, ?7)",
            rusqlite::params![
                history_id,
                persona_id,
                fact_id,
                &value_json,
                input.verification_status.as_str(),
                source_id,
                now,
            ],
        )?;

        Ok(())
    }

    /// Memperbarui status verifikasi fakta dan mencatat perubahan pada tabel riwayat.
    pub fn update_status_with_history(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        history_id: &str,
        new_status: &str,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        // Ambil status dan nilai sebelumnya
        let prev: Option<(String, String)> = connection
            .query_row(
                "SELECT verification_status, value_json FROM profile_facts WHERE id = ?1 AND persona_id = ?2",
                [fact_id, persona_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let (prev_status, value_json) = match prev {
            Some(p) => p,
            None => return Ok(0),
        };

        let updated = connection.execute(
            "UPDATE profile_facts SET verification_status = ?1, updated_at = ?2
             WHERE id = ?3 AND persona_id = ?4",
            rusqlite::params![new_status, now, fact_id, persona_id],
        )?;

        if updated > 0 {
            connection.execute(
                "INSERT INTO fact_history
                 (id, persona_id, fact_id, previous_value_json, new_value_json, previous_status,
                  new_status, change_reason, source_id, changed_at)
                 VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6, 'Pembaruan status verifikasi', NULL, ?7)",
                rusqlite::params![
                    history_id,
                    persona_id,
                    fact_id,
                    &value_json,
                    &prev_status,
                    new_status,
                    now,
                ],
            )?;
        }

        Ok(updated)
    }

    /// Memperbarui klasifikasi sensitivitas fakta.
    pub fn update_sensitivity(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        sensitivity: &str,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        connection.execute(
            "UPDATE profile_facts SET sensitivity = ?1, updated_at = ?2
             WHERE id = ?3 AND persona_id = ?4",
            rusqlite::params![sensitivity, now, fact_id, persona_id],
        )
    }

    /// Memperbarui nilai atau atribut fakta beserta pencatatan riwayat perubahan nilai.
    #[allow(clippy::too_many_arguments)]
    pub fn update_fact_value_with_history(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        history_id: &str,
        category: Option<&str>,
        attribute: Option<&str>,
        data_type: Option<&str>,
        new_value_json: Option<&str>,
        new_normalized: Option<&str>,
        unit: Option<&str>,
        valid_from: Option<&str>,
        valid_until: Option<&str>,
        notes: Option<&str>,
        change_reason: Option<&str>,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        let prev: Option<(String, String)> = connection
            .query_row(
                "SELECT value_json, verification_status FROM profile_facts WHERE id = ?1 AND persona_id = ?2",
                [fact_id, persona_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let (prev_value_json, current_status) = match prev {
            Some(p) => p,
            None => return Ok(0),
        };

        let updated = connection.execute(
            "UPDATE profile_facts
             SET category = COALESCE(?1, category),
                 attribute = COALESCE(?2, attribute),
                 data_type = COALESCE(?3, data_type),
                 value_json = COALESCE(?4, value_json),
                 normalized_value = COALESCE(?5, normalized_value),
                 unit = COALESCE(?6, unit),
                 valid_from = COALESCE(?7, valid_from),
                 valid_until = COALESCE(?8, valid_until),
                 notes = COALESCE(?9, notes),
                 updated_at = ?10
             WHERE id = ?11 AND persona_id = ?12",
            rusqlite::params![
                category,
                attribute,
                data_type,
                new_value_json,
                new_normalized,
                unit,
                valid_from,
                valid_until,
                notes,
                now,
                fact_id,
                persona_id,
            ],
        )?;

        if updated > 0 {
            let recorded_new_val = new_value_json.unwrap_or(&prev_value_json);
            let reason = change_reason.unwrap_or("Pembaruan nilai fakta");

            connection.execute(
                "INSERT INTO fact_history
                 (id, persona_id, fact_id, previous_value_json, new_value_json, previous_status,
                  new_status, change_reason, source_id, changed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, NULL, ?8)",
                rusqlite::params![
                    history_id,
                    persona_id,
                    fact_id,
                    &prev_value_json,
                    recorded_new_val,
                    &current_status,
                    reason,
                    now,
                ],
            )?;
        }

        Ok(updated)
    }

    /// Menambahkan sumber bukti baru ke fakta yang sudah ada.
    #[allow(clippy::too_many_arguments)]
    pub fn add_source(
        connection: &Connection,
        source_id: &str,
        persona_id: &str,
        fact_id: &str,
        source_type: &str,
        title: &str,
        description: &str,
        reference_uri: Option<&str>,
        observed_at: Option<&str>,
        valid_from: Option<&str>,
        valid_until: Option<&str>,
        notes: Option<&str>,
        now: &str,
    ) -> Result<(), rusqlite::Error> {
        connection.execute(
            "INSERT INTO fact_sources
             (id, persona_id, fact_id, source_type, title, description, reference_uri, observed_at,
              valid_from, valid_until, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                source_id,
                persona_id,
                fact_id,
                source_type,
                title,
                description,
                reference_uri,
                observed_at,
                valid_from,
                valid_until,
                notes,
                now,
            ],
        )?;

        connection.execute(
            "UPDATE profile_facts SET updated_at = ?1 WHERE id = ?2 AND persona_id = ?3",
            rusqlite::params![now, fact_id, persona_id],
        )?;

        Ok(())
    }

    /// Mengarsipkan fakta (soft-archive).
    pub fn archive(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        connection.execute(
            "UPDATE profile_facts SET archived_at = ?1, updated_at = ?1
             WHERE id = ?2 AND persona_id = ?3 AND archived_at IS NULL",
            rusqlite::params![now, fact_id, persona_id],
        )
    }

    /// Memulihkan fakta dari arsip.
    pub fn restore(
        connection: &Connection,
        persona_id: &str,
        fact_id: &str,
        now: &str,
    ) -> Result<usize, rusqlite::Error> {
        connection.execute(
            "UPDATE profile_facts SET archived_at = NULL, updated_at = ?1
             WHERE id = ?2 AND persona_id = ?3 AND archived_at IS NOT NULL",
            rusqlite::params![now, fact_id, persona_id],
        )
    }

    /// Menghapus fakta secara permanen (cascades via foreign key di SQLite).
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

    /// Mendeteksi konflik dan fakta bertentangan dalam profil persona yang sama.
    /// Contoh: Dua fakta aktif dengan `fact_key` yang sama namun `normalized_value` berbeda.
    pub fn find_conflicts(
        connection: &Connection,
        persona_id: &str,
    ) -> Result<Vec<ConflictWarning>, rusqlite::Error> {
        let mut statement = connection.prepare(
            "SELECT f1.fact_key, f1.category,
                    f1.id, f1.attribute, f1.normalized_value, f1.verification_status,
                    f2.id, f2.attribute, f2.normalized_value, f2.verification_status
             FROM profile_facts f1
             JOIN profile_facts f2
               ON f1.persona_id = f2.persona_id
              AND f1.fact_key = f2.fact_key
              AND f1.id < f2.id
             WHERE f1.persona_id = ?1
               AND f1.archived_at IS NULL
               AND f2.archived_at IS NULL
               AND f1.normalized_value != f2.normalized_value
               AND f1.verification_status != 'outdated'
               AND f2.verification_status != 'outdated'",
        )?;

        let rows = statement.query_map([persona_id], |row| {
            let fact_key: String = row.get(0)?;
            let category: String = row.get(1)?;
            let id_a: String = row.get(2)?;
            let name_a: String = row.get(3)?;
            let val_a: String = row.get(4)?;
            let status_a: String = row.get(5)?;
            let id_b: String = row.get(6)?;
            let name_b: String = row.get(7)?;
            let val_b: String = row.get(8)?;
            let status_b: String = row.get(9)?;

            let message = format!(
                "Konflik informasi terdeteksi pada '{name_a}': Fakta A bernilai '{val_a}' ({status_a}) bertentangan dengan Fakta B bernilai '{val_b}' ({status_b}). Tinjau dan perbarui salah satu menjadi status outdated."
            );

            Ok(ConflictWarning {
                fact_key,
                category,
                existing_fact_id: id_a,
                existing_display_name: name_a,
                existing_value: val_a,
                existing_status: status_a,
                conflicting_fact_id: id_b,
                conflicting_display_name: name_b,
                conflicting_value: val_b,
                conflicting_status: status_b,
                message,
            })
        })?;

        rows.collect()
    }

    /// Mengambil fakta yang memenuhi syarat untuk digunakan oleh mesin AI.
    ///
    /// ATURAN KETAT SC-011 & AI EVIDENCE:
    /// 1. Milik persona aktif.
    /// 2. Berstatus 'confirmed' (unverified, disputed, dan outdated otomatis dilarang).
    /// 3. Masih berlaku (berdasarkan validity period).
    /// 4. Tidak dalam kondisi arsip (`archived_at IS NULL`).
    /// 5. Sensitivitas: jika sensitive/highly_sensitive, harus dengan izin eksplisit.
    /// 6. Tidak sedang dalam status konflik aktif nilai berlawanan.
    pub fn list_eligible_for_ai(
        connection: &Connection,
        persona_id: &str,
        allow_sensitive: bool,
        now_iso: &str,
    ) -> Result<Vec<FactSummary>, rusqlite::Error> {
        let all_facts = Self::list_by_persona(connection, persona_id, false, now_iso)?;

        let eligible = all_facts
            .into_iter()
            .filter(|f| {
                // Syarat 1: Berstatus confirmed
                if f.verification_status != "confirmed" {
                    return false;
                }
                // Syarat 2: Masa berlaku aktif
                if !f.is_valid {
                    return false;
                }
                // Syarat 3: Tidak ada konflik aktif
                if f.has_conflict {
                    return false;
                }
                // Syarat 4: Kebijakan sensitivitas
                if f.sensitivity != "ordinary" && !allow_sensitive {
                    return false;
                }
                true
            })
            .collect();

        Ok(eligible)
    }
}
