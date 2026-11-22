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
            persona_set_active
        ])
        .run(tauri::generate_context!())
        .expect("error while running Survey Copilot");
}
