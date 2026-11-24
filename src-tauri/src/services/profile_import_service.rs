use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::export::ExportPayload;
use crate::domain::fact::{CreateFactInput, DataType, SourceType};
use crate::domain::persona::{CreatePersonaInput, ProfileImportResult};
use crate::repository::{FactRepository, PersonaRepository};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const MAX_PROFILE_IMPORT_BYTES: usize = 5 * 1024 * 1024;
const MAX_IMPORTED_FACTS: usize = 2_000;
const MAX_IMPORTED_SOURCES: usize = 8_000;

pub struct ProfileImportService;

impl ProfileImportService {
    fn value_to_input_string(
        data_type: DataType,
        value: &serde_json::Value,
    ) -> Result<String, AppError> {
        match data_type {
            DataType::String | DataType::Date => value.as_str().map(str::to_string).ok_or_else(|| {
                AppError::validation(
                    "facts.value",
                    "Nilai teks/tanggal dalam berkas profil tidak sesuai dengan tipe datanya.",
                )
            }),
            DataType::Number => value.as_number().map(|number| number.to_string()).ok_or_else(|| {
                AppError::validation("facts.value", "Nilai angka dalam berkas profil bukan angka yang valid.")
            }),
            DataType::Boolean => value.as_bool().map(|value| value.to_string()).ok_or_else(|| {
                AppError::validation("facts.value", "Nilai ya/tidak dalam berkas profil tidak valid.")
            }),
            DataType::Json => serde_json::to_string(value).map_err(|error| {
                AppError::validation("facts.value", format!("Nilai JSON profil tidak dapat dibaca: {error}"))
            }),
        }
    }

    fn validate_source(source: &crate::domain::fact::FactSource) -> Result<SourceType, AppError> {
        if source.title.chars().count() > 120
            || source.description.trim().is_empty()
            || source.description.chars().count() > 500
            || source.reference_uri.as_ref().is_some_and(|value| value.chars().count() > 2_048)
            || source.notes.as_ref().is_some_and(|value| value.chars().count() > 1_000)
        {
            return Err(AppError::validation(
                "factSources",
                "Sumber bukti memiliki judul, deskripsi, URL, atau catatan yang kosong/terlalu panjang.",
            ));
        }

        match source.source_type.as_str() {
            "user_declaration" => Ok(SourceType::UserDeclaration),
            "uploaded_document" => Ok(SourceType::UploadedDocument),
            "manual_entry" => Ok(SourceType::ManualEntry),
            "observed_action" => Ok(SourceType::ObservedAction),
            "external_record" => Ok(SourceType::ExternalRecord),
            _ => Err(AppError::validation(
                "factSources.sourceType",
                format!("Jenis sumber bukti '{}' tidak didukung.", source.source_type),
            )),
        }
    }

    /// Import dibatasi pada ekspor satu persona. Sesi, jawaban, screenshot, provider,
    /// dan credential tidak pernah diimpor melalui jalur ini.
    pub fn import_single_persona_json(
        database: &Database,
        json_content: &str,
        display_name_override: Option<&str>,
    ) -> Result<ProfileImportResult, AppError> {
        if json_content.trim().is_empty() {
            return Err(AppError::validation("file", "Berkas JSON profil kosong."));
        }
        if json_content.len() > MAX_PROFILE_IMPORT_BYTES {
            return Err(AppError::validation("file", "Ukuran berkas profil melebihi batas 5 MB."));
        }

        let payload: ExportPayload = serde_json::from_str(json_content).map_err(|error| {
            AppError::validation(
                "file",
                format!("Format JSON bukan ekspor Survey Copilot yang valid: {error}"),
            )
        })?;

        if payload.schema_version != "1.0.0"
            || payload.schema_url != "https://survey-copilot.app/schemas/export-v1.json"
        {
            return Err(AppError::validation(
                "schemaVersion",
                "Versi skema ekspor tidak didukung. Gunakan file JSON ekspor profil Survey Copilot versi 1.0.0.",
            ));
        }
        if payload.export_scope != "single_persona"
            || payload.summary.export_scope != "single_persona"
        {
            return Err(AppError::validation(
                "exportScope",
                "Import hanya menerima ekspor satu profil. Buat ekspor JSON dari satu profil, bukan seluruh aplikasi.",
            ));
        }
        if payload.data.personas.len() != 1 {
            return Err(AppError::validation(
                "personas",
                "Berkas import harus berisi tepat satu profil.",
            ));
        }
        if payload.data.facts.len() > MAX_IMPORTED_FACTS
            || payload.data.fact_sources.len() > MAX_IMPORTED_SOURCES
        {
            return Err(AppError::validation(
                "data",
                "Jumlah fakta atau sumber bukti dalam file melebihi batas import.",
            ));
        }

        let source_persona = &payload.data.personas[0];
        if source_persona.id.trim().is_empty()
            || payload.target_persona_id.as_deref() != Some(source_persona.id.as_str())
            || payload.summary.target_persona_id.as_deref() != Some(source_persona.id.as_str())
        {
            return Err(AppError::validation(
                "targetPersonaId",
                "Identitas profil target pada metadata ekspor tidak konsisten.",
            ));
        }

        let requested_name = display_name_override
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or(source_persona.display_name.trim());
        let (display_name, description) = CreatePersonaInput::with_description(
            requested_name.to_string(),
            source_persona.description.clone(),
        )
        .validate()?;

        let mut fact_ids = HashSet::new();
        for fact in &payload.data.facts {
            if fact.persona_id != source_persona.id
                || fact.id.trim().is_empty()
                || !fact_ids.insert(fact.id.clone())
            {
                return Err(AppError::validation(
                    "facts",
                    "Daftar fakta berisi persona yang tidak sesuai atau ID fakta duplikat.",
                ));
            }
        }

        let mut sources_by_fact: HashMap<String, Vec<&crate::domain::fact::FactSource>> =
            HashMap::new();
        let mut source_ids = HashSet::new();
        for source in &payload.data.fact_sources {
            if source.persona_id != source_persona.id
                || !fact_ids.contains(&source.fact_id)
                || source.id.trim().is_empty()
                || !source_ids.insert(source.id.clone())
            {
                return Err(AppError::validation(
                    "factSources",
                    "Sumber bukti mengacu ke profil/fakta yang tidak sesuai atau memiliki ID duplikat.",
                ));
            }
            Self::validate_source(source)?;
            sources_by_fact.entry(source.fact_id.clone()).or_default().push(source);
        }

        let mut connection = database
            .connection
            .lock()
            .map_err(|_| AppError::database("Database sedang tidak tersedia."))?;
        let transaction = connection
            .transaction()
            .map_err(|error| AppError::database(format!("Gagal memulai transaksi import profil: {error}")))?;

        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let new_persona_id = Uuid::new_v4().to_string();
        PersonaRepository::create(
            &transaction,
            &new_persona_id,
            &display_name,
            description.as_deref(),
            false,
            &now,
        )
        .map_err(AppError::from)?;

        let mut imported_facts = 0usize;
        let mut imported_sources = 0usize;

        for exported_fact in &payload.data.facts {
            let data_type = DataType::parse(&exported_fact.data_type)?;
            let raw_value = Self::value_to_input_string(data_type, &exported_fact.value)?;
            let fact_sources = sources_by_fact
                .get(&exported_fact.id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let first_source = fact_sources.first().copied();
            let original_source_count = fact_sources.len();

            // Jika file tidak membawa sumber asli, fakta diimpor sebagai belum terverifikasi.
            let fallback_description =
                "Sumber asli tidak tersedia pada file ekspor; pengguna perlu memverifikasi kembali.".to_string();
            let (source_type, source_title, source_description, reference_uri, observed_at,
                verification_status) = if let Some(source) = first_source {
                    let source_type = Self::validate_source(source)?;
                    (
                        source_type.as_str().to_string(),
                        Some(source.title.clone()),
                        source.description.clone(),
                        source.reference_uri.clone(),
                        source.observed_at.clone().or_else(|| exported_fact.source_date.clone()),
                        exported_fact.verification_status.clone(),
                    )
                } else {
                    (
                        "manual_entry".to_string(),
                        Some("Impor profil — sumber tidak disertakan".to_string()),
                        fallback_description,
                        None,
                        exported_fact.source_date.clone(),
                        "unverified".to_string(),
                    )
                };

            let input = CreateFactInput {
                persona_id: new_persona_id.clone(),
                category: exported_fact.category.clone(),
                fact_key: Some(exported_fact.fact_key.clone()),
                attribute: exported_fact.attribute.clone(),
                data_type: Some(data_type.as_str().to_string()),
                value: raw_value,
                unit: exported_fact.unit.clone(),
                source_type: Some(source_type),
                source_title,
                source_description,
                reference_uri,
                observed_at,
                valid_from: exported_fact.valid_from.clone(),
                valid_until: exported_fact.valid_until.clone(),
                notes: exported_fact.notes.clone(),
                verification_status: Some(verification_status),
                sensitivity: Some(exported_fact.sensitivity.clone()),
            };

            // Gunakan kontrak validasi yang sama dengan input fakta manual.
            let validated = input.validate()?;
            let new_fact_id = Uuid::new_v4().to_string();
            let first_source_id = Uuid::new_v4().to_string();
            let history_id = Uuid::new_v4().to_string();
            FactRepository::insert_fact_and_source(
                &transaction,
                &new_persona_id,
                &new_fact_id,
                &first_source_id,
                &history_id,
                &validated,
                &now,
            )
            .map_err(AppError::from)?;
            // This initial row is a disclosed placeholder when no source exists; only count
            // actual source records from the source export in the import result.
            imported_sources += original_source_count.min(1);

            if let Some(archived_at) = exported_fact.archived_at.as_deref() {
                transaction
                    .execute(
                        "UPDATE profile_facts SET archived_at = ?1 WHERE id = ?2 AND persona_id = ?3",
                        rusqlite::params![archived_at, new_fact_id, new_persona_id],
                    )
                    .map_err(AppError::from)?;
            }

            for source in fact_sources.iter().skip(1) {
                let source_type = Self::validate_source(source)?;
                FactRepository::add_source(
                    &transaction,
                    &Uuid::new_v4().to_string(),
                    &new_persona_id,
                    &new_fact_id,
                    source_type.as_str(),
                    &source.title,
                    &source.description,
                    source.reference_uri.as_deref(),
                    source.observed_at.as_deref(),
                    source.valid_from.as_deref(),
                    source.valid_until.as_deref(),
                    source.notes.as_deref(),
                    &now,
                )
                .map_err(AppError::from)?;
                imported_sources += 1;
            }

            imported_facts += 1;
        }

        transaction.commit().map_err(|error| {
            AppError::database(format!("Import profil dibatalkan karena transaksi gagal: {error}"))
        })?;

        let persona = PersonaRepository::find_by_id(&connection, &new_persona_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("Profil hasil import tidak ditemukan."))?;

        Ok(ProfileImportResult {
            persona,
            imported_facts,
            imported_sources,
        })
    }
}
