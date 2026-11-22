mod db;

use db::Database;
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::fs;
use tauri::{Manager, State};
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersonaSummary {
    id: String,
    display_name: String,
    is_active: bool,
    status: String,
}

fn read_persona(
    connection: &rusqlite::Connection,
    persona_id: &str,
) -> Result<PersonaSummary, String> {
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
        .map_err(|error| format!("Gagal membaca profil: {error}"))?
        .ok_or_else(|| "Profil tidak ditemukan.".to_string())
}

#[tauri::command]
fn app_status() -> &'static str {
    "ready"
}

#[tauri::command]
fn persona_list(database: State<'_, Database>) -> Result<Vec<PersonaSummary>, String> {
    let connection = database
        .connection
        .lock()
        .map_err(|_| "Database sedang tidak tersedia.".to_string())?;

    let mut statement = connection
        .prepare(
            "SELECT id, display_name, is_active, status
             FROM personas
             WHERE status != 'disabled'
             ORDER BY is_active DESC, display_name COLLATE NOCASE ASC",
        )
        .map_err(|error| format!("Gagal membaca daftar profil: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok(PersonaSummary {
                id: row.get(0)?,
                display_name: row.get(1)?,
                is_active: row.get::<_, i64>(2)? == 1,
                status: row.get(3)?,
            })
        })
        .map_err(|error| format!("Gagal membaca daftar profil: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Gagal memuat profil: {error}"))
}

#[tauri::command]
fn persona_create(
    display_name: String,
    database: State<'_, Database>,
) -> Result<PersonaSummary, String> {
    let display_name = display_name.trim();
    let length = display_name.chars().count();

    if length == 0 || length > 80 {
        return Err("Nama profil harus berisi 1 sampai 80 karakter.".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();
    let connection = database
        .connection
        .lock()
        .map_err(|_| "Database sedang tidak tersedia.".to_string())?;

    connection
        .execute(
            "INSERT INTO personas (id, display_name, is_active, status, created_at, updated_at)
             VALUES (?1, ?2, CASE WHEN EXISTS (SELECT 1 FROM personas WHERE is_active = 1) THEN 0 ELSE 1 END, 'active', ?3, ?3)",
            (&id, display_name, &now),
        )
        .map_err(|error| format!("Gagal menyimpan profil: {error}"))?;

    read_persona(&connection, &id)
}

#[tauri::command]
fn persona_set_active(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaSummary, String> {
    let mut connection = database
        .connection
        .lock()
        .map_err(|_| "Database sedang tidak tersedia.".to_string())?;

    let transaction = connection
        .transaction()
        .map_err(|error| format!("Gagal memulai perubahan profil: {error}"))?;

    let exists: Option<String> = transaction
        .query_row(
            "SELECT id FROM personas WHERE id = ?1 AND status = 'active'",
            [&persona_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Gagal memeriksa profil: {error}"))?;

    if exists.is_none() {
        return Err("Profil tidak ditemukan atau sudah tidak aktif.".to_string());
    }

    transaction
        .execute("UPDATE personas SET is_active = 0 WHERE is_active = 1", [])
        .map_err(|error| format!("Gagal mengganti profil aktif: {error}"))?;

    transaction
        .execute(
            "UPDATE personas SET is_active = 1, updated_at = ?1 WHERE id = ?2",
            (chrono_timestamp(), &persona_id),
        )
        .map_err(|error| format!("Gagal mengaktifkan profil: {error}"))?;

    transaction
        .commit()
        .map_err(|error| format!("Gagal menyimpan profil aktif: {error}"))?;

    read_persona(&connection, &persona_id)
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FactSummary {
    id: String,
    category: String,
    attribute: String,
    value: serde_json::Value,
    verification_status: String,
    sensitivity: String,
    source_description: Option<String>,
}

#[tauri::command]
fn fact_list(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<Vec<FactSummary>, String> {
    let connection = database
        .connection
        .lock()
        .map_err(|_| "Database sedang tidak tersedia.".to_string())?;

    let mut statement = connection
        .prepare(
            "SELECT f.id, f.category, f.attribute, f.value_json,
                    f.verification_status, f.sensitivity,
                    (SELECT description FROM fact_sources WHERE fact_id = f.id ORDER BY created_at LIMIT 1)
             FROM profile_facts f
             WHERE f.persona_id = ?1 AND f.archived_at IS NULL
             ORDER BY f.category COLLATE NOCASE, f.attribute COLLATE NOCASE",
        )
        .map_err(|error| format!("Gagal membaca data profil: {error}"))?;

    let rows = statement
        .query_map([&persona_id], |row| {
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
        })
        .map_err(|error| format!("Gagal membaca data profil: {error}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Gagal memuat data profil: {error}"))
}

#[tauri::command]
fn fact_create(
    persona_id: String,
    category: String,
    attribute: String,
    value: String,
    source_description: String,
    verification_status: String,
    sensitivity: String,
    database: State<'_, Database>,
) -> Result<FactSummary, String> {
    let category = category.trim();
    let attribute = attribute.trim();
    let value = value.trim();
    let source_description = source_description.trim();

    if category.is_empty() || attribute.is_empty() || value.is_empty() {
        return Err("Kategori, informasi, dan nilai wajib diisi.".to_string());
    }
    if source_description.is_empty() {
        return Err("Sumber informasi wajib diisi.".to_string());
    }
    if category.chars().count() > 80
        || attribute.chars().count() > 100
        || value.chars().count() > 4000
        || source_description.chars().count() > 500
    {
        return Err("Ada informasi yang melebihi batas panjang.".to_string());
    }
    if !["confirmed", "unverified", "disputed", "outdated"].contains(&verification_status.as_str()) {
        return Err("Status verifikasi tidak valid.".to_string());
    }
    if !["ordinary", "sensitive", "highly_sensitive"].contains(&sensitivity.as_str()) {
        return Err("Klasifikasi sensitivitas tidak valid.".to_string());
    }

    let fact_id = Uuid::new_v4().to_string();
    let source_id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();
    let value_json = serde_json::to_string(value)
        .map_err(|error| format!("Gagal memproses nilai profil: {error}"))?;

    let mut connection = database
        .connection
        .lock()
        .map_err(|_| "Database sedang tidak tersedia.".to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Gagal memulai penyimpanan: {error}"))?;

    let persona_exists: Option<String> = transaction
        .query_row(
            "SELECT id FROM personas WHERE id = ?1 AND status = 'active'",
            [&persona_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Gagal memeriksa profil: {error}"))?;
    if persona_exists.is_none() {
        return Err("Profil tidak ditemukan atau sudah tidak aktif.".to_string());
    }

    transaction
        .execute(
            "INSERT INTO profile_facts
             (id, persona_id, category, attribute, value_json, normalized_value,
              verification_status, sensitivity, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            (
                &fact_id,
                &persona_id,
                category,
                attribute,
                &value_json,
                value,
                &verification_status,
                &sensitivity,
                &now,
            ),
        )
        .map_err(|error| format!("Gagal menyimpan informasi profil: {error}"))?;

    transaction
        .execute(
            "INSERT INTO fact_sources
             (id, persona_id, fact_id, source_type, description, observed_at, created_at)
             VALUES (?1, ?2, ?3, 'user_input', ?4, ?5, ?5)",
            (&source_id, &persona_id, &fact_id, source_description, &now),
        )
        .map_err(|error| format!("Gagal menyimpan sumber informasi: {error}"))?;

    transaction
        .commit()
        .map_err(|error| format!("Gagal menyelesaikan penyimpanan: {error}"))?;
    drop(connection);

    let connection = database
        .connection
        .lock()
        .map_err(|_| "Database sedang tidak tersedia.".to_string())?;
    connection
        .query_row(
            "SELECT f.id, f.category, f.attribute, f.value_json,
                    f.verification_status, f.sensitivity,
                    (SELECT description FROM fact_sources WHERE fact_id = f.id ORDER BY created_at LIMIT 1)
             FROM profile_facts f
             WHERE f.id = ?1 AND f.persona_id = ?2",
            (&fact_id, &persona_id),
            |row| {
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
            },
        )
        .map_err(|error| format!("Gagal membaca informasi yang disimpan: {error}"))
}

#[tauri::command]
fn fact_set_status(
    persona_id: String,
    fact_id: String,
    status: String,
    database: State<'_, Database>,
) -> Result<(), String> {
    if !["confirmed", "unverified", "disputed", "outdated"].contains(&status.as_str()) {
        return Err("Status verifikasi tidak valid.".to_string());
    }

    let connection = database
        .connection
        .lock()
        .map_err(|_| "Database sedang tidak tersedia.".to_string())?;
    let changed = connection
        .execute(
            "UPDATE profile_facts SET verification_status = ?1, updated_at = ?2
             WHERE id = ?3 AND persona_id = ?4",
            (&status, chrono_timestamp(), &fact_id, &persona_id),
        )
        .map_err(|error| format!("Gagal memperbarui status informasi: {error}"))?;

    if changed == 0 {
        return Err("Informasi profil tidak ditemukan.".to_string());
    }
    Ok(())
}

#[tauri::command]
fn fact_delete(
    persona_id: String,
    fact_id: String,
    database: State<'_, Database>,
) -> Result<(), String> {
    let connection = database
        .connection
        .lock()
        .map_err(|_| "Database sedang tidak tersedia.".to_string())?;
    let changed = connection
        .execute(
            "DELETE FROM profile_facts WHERE id = ?1 AND persona_id = ?2",
            (&fact_id, &persona_id),
        )
        .map_err(|error| format!("Informasi ini masih dipakai oleh riwayat dan tidak dapat dihapus: {error}"))?;

    if changed == 0 {
        return Err("Informasi profil tidak ditemukan.".to_string());
    }
    Ok(())
}

fn chrono_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("survey-copilot.sqlite3");
            let database = Database::initialize(&database_path)
                .map_err(std::io::Error::other)?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_status,
            persona_list,
            persona_create,
            persona_set_active,
            fact_list,
            fact_create,
            fact_set_status,
            fact_delete
        ])
        .run(tauri::generate_context!())
        .expect("error while running Survey Copilot");
}
