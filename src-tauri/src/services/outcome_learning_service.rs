use chrono::Utc;
use rusqlite::Connection;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::attempt::{
    AttemptFilter, CreateAttemptInput, HistoricalLearningSummary, LearningDataStatus,
    OutcomeConfidence, OutcomeLabel, RecordOutcomeEventInput, SurveyAttempt, SurveyAttemptDetail,
    SurveyOutcomeEvent,
};
use crate::domain::error::AppError;
use crate::domain::opportunity::{OpportunityProvenance, OpportunityStatus, SurveyOpportunity};
use crate::repository::attempt_repo::AttemptRepository;
use crate::repository::opportunity_repo::OpportunityRepository;
use crate::repository::persona_repo::PersonaRepository;

/// Ambang batas jumlah sampel minimum untuk mencegah kesimpulan prematur (AT-063)
pub const MIN_SAMPLE_THRESHOLD: usize = 10;

pub struct OutcomeLearningService;

impl OutcomeLearningService {
    /// Mencatat percobaan survei baru beserta peristiwa awal (SC-052)
    pub fn record_attempt(
        conn: &mut Connection,
        input: CreateAttemptInput,
    ) -> Result<SurveyAttemptDetail, AppError> {
        let persona_id_trimmed = input.persona_id.trim();
        if persona_id_trimmed.is_empty() {
            return Err(AppError::validation(
                "personaId",
                "ID persona tidak boleh kosong",
            ));
        }

        // 1. Verifikasi persona
        PersonaRepository::find_by_id(conn, persona_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil data persona: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Persona dengan ID '{persona_id_trimmed}' tidak ditemukan"
                ))
            })?;

        let now = Utc::now().to_rfc3339();
        let started_at = input.started_at.unwrap_or_else(|| now.clone());

        // 2. Verifikasi atau buat peluang survei jika belum ada
        let opportunity_id = match input.opportunity_id {
            Some(opp_id) if !opp_id.trim().is_empty() => {
                let id_trimmed = opp_id.trim().to_string();
                let exists = OpportunityRepository::get_opportunity_by_id(conn, &id_trimmed)
                    .map_err(|e| AppError::database(format!("Gagal memeriksa peluang: {e}")))?
                    .is_some();

                if !exists {
                    return Err(AppError::not_found(format!(
                        "Peluang survei dengan ID '{id_trimmed}' tidak ditemukan"
                    )));
                }
                id_trimmed
            }
            _ => {
                // Buat peluang survei baru secara otomatis untuk menampung riwayat percobaan
                let new_opp_id = Uuid::new_v4().to_string();
                let new_opp = SurveyOpportunity {
                    id: new_opp_id.clone(),
                    title: input
                        .opportunity_title
                        .or_else(|| Some("Survei Mandiri".to_string())),
                    platform_label: input.source_platform.or_else(|| Some("Manual".to_string())),
                    topic: input.category,
                    description: None,
                    raw_text: input.raw_text,
                    provenance: OpportunityProvenance::ManualEntry,
                    status: OpportunityStatus::Confirmed,
                    source_capture_id: None,
                    source_ocr_id: None,
                    captured_at: started_at.clone(),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };

                OpportunityRepository::save_opportunity(conn, &new_opp).map_err(|e| {
                    AppError::database(format!("Gagal membuat peluang survei: {e}"))
                })?;

                new_opp_id
            }
        };

        let initial_outcome = input.initial_outcome.unwrap_or(OutcomeLabel::Pending);
        let ended_at = if initial_outcome.is_terminal() {
            Some(started_at.clone())
        } else {
            None
        };

        let attempt_id = Uuid::new_v4().to_string();
        let attempt = SurveyAttempt {
            id: attempt_id.clone(),
            opportunity_id,
            persona_id: persona_id_trimmed.to_string(),
            session_id: input.session_id,
            started_at: started_at.clone(),
            ended_at,
            current_outcome: initial_outcome,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        let event_id = Uuid::new_v4().to_string();
        let event = SurveyOutcomeEvent {
            id: event_id,
            attempt_id: attempt_id.clone(),
            outcome_label: initial_outcome,
            evidence_source: input
                .evidence_source
                .unwrap_or_else(|| "pencatatan_manual_pengguna".to_string()),
            evidence_note: input.evidence_note,
            confidence: input.confidence.unwrap_or(OutcomeConfidence::Observed),
            observed_at: started_at,
            created_at: now,
        };

        // Simpan attempt dan initial event di dalam transaksi
        let tx = conn
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        AttemptRepository::save_attempt(&tx, &attempt)
            .map_err(|e| AppError::database(format!("Gagal menyimpan percobaan survei: {e}")))?;

        AttemptRepository::save_outcome_event(&tx, &event)
            .map_err(|e| AppError::database(format!("Gagal menyimpan event percobaan: {e}")))?;

        tx.commit()
            .map_err(|e| AppError::database(format!("Gagal commit transaksi: {e}")))?;

        Self::get_attempt_detail(conn, &attempt_id, Some(persona_id_trimmed))
    }

    /// Menambahkan peristiwa hasil observasi baru pada percobaan survei yang sedang berjalan
    pub fn record_outcome_event(
        conn: &mut Connection,
        input: RecordOutcomeEventInput,
    ) -> Result<SurveyAttemptDetail, AppError> {
        let attempt_id_trimmed = input.attempt_id.trim();
        let persona_id_trimmed = input.persona_id.trim();

        if attempt_id_trimmed.is_empty() {
            return Err(AppError::validation(
                "attemptId",
                "ID percobaan survei tidak boleh kosong",
            ));
        }
        if persona_id_trimmed.is_empty() {
            return Err(AppError::validation(
                "personaId",
                "ID persona tidak boleh kosong",
            ));
        }

        // 1. Ambil percobaan yang ada dan verifikasi isolasi kepemilikan persona
        let existing = AttemptRepository::get_attempt_by_id(conn, attempt_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil percobaan: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Percobaan survei dengan ID '{attempt_id_trimmed}' tidak ditemukan"
                ))
            })?;

        if existing.persona_id != persona_id_trimmed {
            return Err(AppError::validation(
                "personaId",
                "Akses ditolak: percobaan survei ini bukan milik persona yang ditentukan",
            ));
        }

        let now = Utc::now().to_rfc3339();
        let observed_at = input.observed_at.unwrap_or_else(|| now.clone());
        let ended_at_str = if input.outcome_label.is_terminal() {
            Some(observed_at.clone())
        } else {
            None
        };

        let event = SurveyOutcomeEvent {
            id: Uuid::new_v4().to_string(),
            attempt_id: attempt_id_trimmed.to_string(),
            outcome_label: input.outcome_label,
            evidence_source: input.evidence_source,
            evidence_note: input.evidence_note,
            confidence: input.confidence.unwrap_or(OutcomeConfidence::Observed),
            observed_at,
            created_at: now.clone(),
        };

        let tx = conn
            .transaction()
            .map_err(|e| AppError::database(format!("Gagal memulai transaksi: {e}")))?;

        AttemptRepository::save_outcome_event(&tx, &event)
            .map_err(|e| AppError::database(format!("Gagal menyimpan event percobaan: {e}")))?;

        AttemptRepository::update_attempt_outcome(
            &tx,
            attempt_id_trimmed,
            input.outcome_label,
            ended_at_str.as_deref(),
            &now,
        )
        .map_err(|e| AppError::database(format!("Gagal memperbarui status percobaan: {e}")))?;

        tx.commit()
            .map_err(|e| AppError::database(format!("Gagal commit transaksi: {e}")))?;

        Self::get_attempt_detail(conn, attempt_id_trimmed, Some(persona_id_trimmed))
    }

    /// Mengambil detail lengkap sebuah percobaan survei
    pub fn get_attempt_detail(
        conn: &Connection,
        id: &str,
        persona_id: Option<&str>,
    ) -> Result<SurveyAttemptDetail, AppError> {
        AttemptRepository::get_attempt_detail(conn, id, persona_id)
            .map_err(|e| AppError::database(format!("Gagal mengambil detail percobaan: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!("Percobaan survei dengan ID '{id}' tidak ditemukan"))
            })
    }

    /// Mengambil daftar percobaan survei dengan filter dinamis
    pub fn list_attempts(
        conn: &Connection,
        filter: AttemptFilter,
    ) -> Result<Vec<SurveyAttemptDetail>, AppError> {
        AttemptRepository::list_attempts(conn, &filter).map_err(|e| {
            AppError::database(format!("Gagal mengambil daftar percobaan survei: {e}"))
        })
    }

    /// Menghapus percobaan survei dengan isolasi persona
    pub fn delete_attempt(conn: &Connection, id: &str, persona_id: &str) -> Result<bool, AppError> {
        let id_trimmed = id.trim();
        let persona_id_trimmed = persona_id.trim();

        if id_trimmed.is_empty() || persona_id_trimmed.is_empty() {
            return Err(AppError::validation(
                "id/personaId",
                "ID percobaan dan ID persona tidak boleh kosong",
            ));
        }

        AttemptRepository::delete_attempt(conn, id_trimmed, persona_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal menghapus percobaan survei: {e}")))
    }

    /// Mengevaluasi ringkasan pembelajaran historis (SC-052 / AT-063)
    ///
    /// Menegakkan aturan:
    /// 1. Jika data sparse / < 10 observasi, tandai `insufficient_data` dan kembalikan None untuk prediksi probabilitas.
    /// 2. Jika data cukup (>= 10), jalankan validasi chronological holdout (70% train, 30% test kronologis)
    ///    dan hitung tingkat konsistensi tanpa data leakage.
    /// 3. Pisahkan fakta observasi dari estimasi, dan sertakan disclaimer ketidakpastian.
    /// 4. Dilarang menghasilkan saldo/reward/poin atau rekomendasi manipulasi jawaban.
    pub fn evaluate_historical_learning(
        conn: &Connection,
        persona_id: &str,
        category: Option<&str>,
        source_platform: Option<&str>,
    ) -> Result<HistoricalLearningSummary, AppError> {
        let persona_id_trimmed = persona_id.trim();
        if persona_id_trimmed.is_empty() {
            return Err(AppError::validation(
                "personaId",
                "ID persona tidak boleh kosong",
            ));
        }

        let persona = PersonaRepository::find_by_id(conn, persona_id_trimmed)
            .map_err(|e| AppError::database(format!("Gagal mengambil data persona: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Persona dengan ID '{persona_id_trimmed}' tidak ditemukan"
                ))
            })?;

        let attempts = AttemptRepository::get_historical_attempts_for_learning(
            conn,
            persona_id_trimmed,
            category,
            source_platform,
        )
        .map_err(|e| AppError::database(format!("Gagal mengambil riwayat historis: {e}")))?;

        let sample_size = attempts.len();

        let mut counts_by_outcome: HashMap<String, usize> = HashMap::new();
        for att in &attempts {
            *counts_by_outcome
                .entry(att.current_outcome.as_str().to_string())
                .or_insert(0) += 1;
        }

        let observation_window_start = attempts.first().map(|a| a.started_at.clone());
        let observation_window_end = attempts.last().map(|a| a.started_at.clone());

        let mut disclaimers = vec![
            "Hasil historis adalah observasi masa lalu dan tidak menjamin kelayakan untuk survei baru."
                .to_string(),
            "Tidak ada fitur pelacakan saldo, reward, uang tunai, atau poin."
                .to_string(),
            "Dilarang memanipulasi profil atau merekayasa jawaban demi lolos kriteria skrining."
                .to_string(),
            "Observasi faktual terpisah secara tegas dari model pembelajaran historis."
                .to_string(),
        ];

        // AT-063: Evaluasi data sparse / insufficient
        if sample_size < MIN_SAMPLE_THRESHOLD {
            return Ok(HistoricalLearningSummary {
                persona_id: persona_id_trimmed.to_string(),
                persona_name: persona.display_name,
                category: category.map(|s| s.to_string()),
                source_platform: source_platform.map(|s| s.to_string()),
                sample_size,
                minimum_required_sample: MIN_SAMPLE_THRESHOLD,
                status: LearningDataStatus::InsufficientData,
                observation_window_start,
                observation_window_end,
                counts_by_outcome,
                completion_rate: None,
                qualification_rate: None,
                validation_method: "chronological_holdout".to_string(),
                holdout_sample_size: 0,
                holdout_consistency_rate: None,
                message: format!(
                    "Data historis belum mencukupi (tercatat {} dari minimal {} observasi terkalibrasi). Persentase probabilitas dan prediksi kelayakan tidak ditampilkan untuk mencegah fabrikasi atau kesimpulan semu.",
                    sample_size, MIN_SAMPLE_THRESHOLD
                ),
                uncertainty_disclosure: "Ketidakpastian sangat tinggi (ukuran sampel di bawah ambang batas minimal). Model dilarang mengekstrapolasi probabilitas kelayakan.".to_string(),
                disclaimers,
            });
        }

        // Ukuran sampel memadai: Jalankan Chronological Holdout Validation (70/30 split kronologis)
        let split_idx = (sample_size * 70) / 100;
        let train_set = &attempts[..split_idx];
        let holdout_set = &attempts[split_idx..];

        let train_qualified_or_completed = train_set
            .iter()
            .filter(|a| {
                matches!(
                    a.current_outcome,
                    OutcomeLabel::Qualified | OutcomeLabel::Completed
                )
            })
            .count();
        let train_rate = if !train_set.is_empty() {
            train_qualified_or_completed as f64 / train_set.len() as f64
        } else {
            0.0
        };

        let holdout_qualified_or_completed = holdout_set
            .iter()
            .filter(|a| {
                matches!(
                    a.current_outcome,
                    OutcomeLabel::Qualified | OutcomeLabel::Completed
                )
            })
            .count();
        let holdout_rate = if !holdout_set.is_empty() {
            holdout_qualified_or_completed as f64 / holdout_set.len() as f64
        } else {
            0.0
        };

        // Konsistensi antara train dan holdout tanpa data leakage
        let difference = (train_rate - holdout_rate).abs();
        let holdout_consistency_rate = (1.0 - difference).clamp(0.0, 1.0);

        // Hitung metrik deskriptif empiris keseluruhan
        let total_completed = *counts_by_outcome.get("completed").unwrap_or(&0);
        let total_qualified = *counts_by_outcome.get("qualified").unwrap_or(&0);

        let completion_rate = total_completed as f64 / sample_size as f64;
        let qualification_rate = (total_qualified + total_completed) as f64 / sample_size as f64;

        // Hitung margin ketidakpastian sederhana (wilson/normal approximation standard error)
        let std_err =
            ((qualification_rate * (1.0 - qualification_rate)) / sample_size as f64).sqrt();
        let margin_percent = (1.96 * std_err * 100.0).clamp(1.0, 50.0);

        disclaimers.push(format!(
            "Validasi kronologis: {} sampel pelatihan awal dievaluasi terhadap {} sampel uji holdout independen.",
            train_set.len(),
            holdout_set.len()
        ));

        Ok(HistoricalLearningSummary {
            persona_id: persona_id_trimmed.to_string(),
            persona_name: persona.display_name,
            category: category.map(|s| s.to_string()),
            source_platform: source_platform.map(|s| s.to_string()),
            sample_size,
            minimum_required_sample: MIN_SAMPLE_THRESHOLD,
            status: LearningDataStatus::SufficientData,
            observation_window_start,
            observation_window_end,
            counts_by_outcome,
            completion_rate: Some(completion_rate),
            qualification_rate: Some(qualification_rate),
            validation_method: "chronological_holdout".to_string(),
            holdout_sample_size: holdout_set.len(),
            holdout_consistency_rate: Some(holdout_consistency_rate),
            message: format!(
                "Tercatat {} observasi historis terkalibrasi. Tingkat kelulusan observasi empiris: {:.1}%, selesai penuh: {:.1}%.",
                sample_size,
                qualification_rate * 100.0,
                completion_rate * 100.0
            ),
            uncertainty_disclosure: format!(
                "Ketidakpastian estimasi: ±{:.1}% (interval kepercayaan 95%). Konsistensi holdout kronologis: {:.1}%.",
                margin_percent,
                holdout_consistency_rate * 100.0
            ),
            disclaimers,
        })
    }
}
