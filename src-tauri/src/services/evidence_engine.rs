use std::collections::{HashMap, HashSet};

use chrono::Utc;
use rusqlite::Connection;

use crate::domain::classifier::{QuestionCategory, QuestionClassificationResult, TimeframeType};
use crate::domain::error::AppError;
use crate::domain::evidence::{
    ConflictSeverity, ConflictType, EvidenceConfidence, EvidenceConflictIndicator, EvidencePackage,
    EvidenceRecommendation, EvidenceRetrievalInput, RetrievedEvidenceItem, ValidityStatus,
};
use crate::domain::fact::{
    check_validity_period, ProfileFactDetail, Sensitivity, VerificationStatus,
};
use crate::domain::survey::SurveyQuestion;
use crate::repository::fact_repo::FactRepository;
use crate::repository::persona_repo::PersonaRepository;
use crate::repository::survey_repo::SurveyRepository;
use crate::services::question_classifier::QuestionClassifierService;

/// Layanan Mesin Pencarian Bukti Profil & Konsistensi (SC-032)
///
/// Bertanggung jawab untuk:
/// 1. Mengambil fakta profil yang relevan berdasarkan pertanyaan, kategori SC-031, dan batasan.
/// 2. Melakukan ranking multi-dimensi (relevansi, validitas waktu, status verifikasi, provenance).
/// 3. Menegakkan isolasi data persona secara mutlak (zero cross-persona leakage).
/// 4. Mendeteksi konflik antar-fakta, kontradiksi jawaban historis, dan ketidaksesuaian timeframe.
/// 5. Menghasilkan kontrak paket bukti terstruktur untuk SC-033 tanpa mengirim data ke AI eksternal.
pub struct EvidenceEngineService;

impl EvidenceEngineService {
    /// Mengevaluasi dan mengambil paket bukti lengkap untuk suatu pertanyaan survei.
    pub fn evaluate_evidence(
        connection: &Connection,
        input: &EvidenceRetrievalInput,
    ) -> Result<EvidencePackage, AppError> {
        let now_iso = Utc::now().to_rfc3339();

        // 1. Verifikasi Kepemilikan & Eksistensi Persona Aktif
        let persona = PersonaRepository::find_by_id(connection, &input.persona_id)
            .map_err(|e| AppError::database(format!("Gagal memverifikasi persona: {e}")))?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "Persona dengan ID '{}' tidak ditemukan atau tidak aktif",
                    input.persona_id
                ))
            })?;

        // 2. Dapatkan Hasil Klasifikasi Pertanyaan (gunakan klasifikasi yang disediakan atau jalankan secara deterministik)
        let classification = match &input.classification {
            Some(c) => c.clone(),
            None => QuestionClassifierService::classify_raw(
                &input.question_text,
                input.instructions.as_deref(),
                input.options.as_deref(),
            ),
        };

        // 3. Ambil Seluruh Fakta Milik Persona Ini (Strictly Isolated by Persona ID)
        let all_facts = FactRepository::list_facts_with_sources_by_persona(
            connection,
            &input.persona_id,
            false, // exclude archived
            &now_iso,
        )
        .map_err(|e| AppError::database(format!("Gagal mengambil fakta persona: {e}")))?;

        // 4. Ambil Riwayat Jawaban Terkonfirmasi Sebelumnya Milik Persona Ini
        let historical_questions = SurveyRepository::list_confirmed_answers_by_persona(
            connection,
            &input.persona_id,
            Some(100),
        )?;

        // 5. Filter Sensitivitas & Persetujuan (Sensitivity Consent Policy)
        let consent = input.consent.unwrap_or_default();
        let mut sensitive_filtered_count = 0usize;
        let mut accessible_facts = Vec::new();

        for fact in all_facts {
            let sensitivity =
                Sensitivity::parse(&fact.sensitivity).unwrap_or(Sensitivity::Ordinary);
            match sensitivity {
                Sensitivity::Ordinary => accessible_facts.push(fact),
                Sensitivity::Sensitive => {
                    if consent.allow_sensitive {
                        accessible_facts.push(fact);
                    } else {
                        sensitive_filtered_count += 1;
                    }
                }
                Sensitivity::HighlySensitive => {
                    if consent.allow_highly_sensitive {
                        accessible_facts.push(fact);
                    } else {
                        sensitive_filtered_count += 1;
                    }
                }
            }
        }

        // 6. Evaluasi dan Ranking Relevansi Setiap Fakta
        let mut candidate_evidence: Vec<RetrievedEvidenceItem> = Vec::new();
        let max_items = input.max_evidence_items.unwrap_or(10);

        for fact in &accessible_facts {
            let (score, reason) = Self::compute_relevance(
                fact,
                &input.question_text,
                input.instructions.as_deref(),
                input.options.as_deref(),
                &classification,
                &now_iso,
            );

            // Hanya sertakan fakta yang memiliki keterkaitan minimal
            if score > 0.15 {
                let validity_status = Self::evaluate_validity(
                    fact.valid_from.as_deref(),
                    fact.valid_until.as_deref(),
                    &now_iso,
                );
                let verification_status = VerificationStatus::parse(&fact.verification_status)
                    .unwrap_or(VerificationStatus::Unverified);
                let sensitivity =
                    Sensitivity::parse(&fact.sensitivity).unwrap_or(Sensitivity::Ordinary);

                let primary_source = fact.sources.first();

                candidate_evidence.push(RetrievedEvidenceItem {
                    fact_id: fact.id.clone(),
                    source_id: primary_source.map(|s| s.id.clone()),
                    fact_key: fact.fact_key.clone(),
                    attribute: fact.attribute.clone(),
                    category: fact.category.clone(),
                    value: fact.value.clone(),
                    normalized_value: fact.normalized_value.clone(),
                    unit: fact.unit.clone(),
                    verification_status,
                    validity_status,
                    sensitivity,
                    relevance_score: score,
                    selection_reason: reason,
                    source_type: primary_source.map(|s| s.source_type.clone()),
                    source_title: primary_source.map(|s| s.title.clone()),
                    source_description: primary_source.map(|s| s.description.clone()),
                    source_date: fact.source_date.clone(),
                    valid_from: fact.valid_from.clone(),
                    valid_until: fact.valid_until.clone(),
                });
            }
        }

        // Urutkan kandidat bukti berdasarkan skor relevansi tertinggi
        candidate_evidence.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if candidate_evidence.len() > max_items {
            candidate_evidence.truncate(max_items);
        }

        // 7. Konsistensi & Deteksi Konflik (Consistency Engine)
        let mut conflict_indicators = Vec::new();
        Self::detect_fact_conflicts(
            &candidate_evidence,
            &accessible_facts,
            &mut conflict_indicators,
        );
        Self::detect_timeframe_mismatches(
            &candidate_evidence,
            &classification,
            &mut conflict_indicators,
        );
        Self::detect_historical_contradictions(
            &candidate_evidence,
            &historical_questions,
            &mut conflict_indicators,
        );
        Self::detect_value_changes_over_time(
            &candidate_evidence,
            &accessible_facts,
            &mut conflict_indicators,
        );

        // 8. Evaluasi Informasi yang Kurang (Missing Information)
        let mut missing_information = Vec::new();

        if candidate_evidence.is_empty() {
            missing_information.push(format!(
                "Tidak ditemukan fakta relevan dalam profil persona '{}' untuk kategori {}.",
                persona.display_name,
                classification.primary_category.as_str()
            ));
        }

        if sensitive_filtered_count > 0 && candidate_evidence.is_empty() {
            missing_information.push(format!(
                "Terdapat {sensitive_filtered_count} fakta berlabel sensitif yang tidak dapat diakses karena belum mendapat persetujuan izin (consent)."
            ));
        }

        if classification.ambiguity.is_ambiguous {
            for reason in &classification.ambiguity.reasons {
                missing_information.push(format!(
                    "Ambiguitas pertanyaan: {} ({})",
                    reason.explanation,
                    match reason.ambiguity_type {
                        crate::domain::classifier::AmbiguityType::MissingTimeframe =>
                            "AT-041: Batasan waktu tidak disebutkan",
                        crate::domain::classifier::AmbiguityType::MissingUnit =>
                            "AT-041: Satuan pengukuran tidak disebutkan",
                        _ => "Instruksi/opsi kurang jelas",
                    }
                ));
            }
        }

        // 9. Sintesis Keyakinan dan Rekomendasi (Confidence & Recommendation)
        let (recommendation, confidence, summary) = Self::synthesize_recommendation_and_confidence(
            &candidate_evidence,
            &conflict_indicators,
            &missing_information,
            sensitive_filtered_count,
        );

        Ok(EvidencePackage {
            question_id: input.question_id.clone(),
            active_persona_id: input.persona_id.clone(),
            retrieved_evidence: candidate_evidence,
            conflict_indicators,
            missing_information,
            evidence_confidence: confidence,
            recommendation,
            evaluation_summary: summary,
            evaluated_at: now_iso,
        })
    }

    // =========================================================================
    // RELEVANCE SCORING & RANKING
    // =========================================================================

    /// Menghitung skor relevansi (0.0 ..= 1.0) dan alasan pemilihan fakta.
    fn compute_relevance(
        fact: &ProfileFactDetail,
        question_text: &str,
        instructions: Option<&str>,
        options: Option<&[String]>,
        classification: &QuestionClassificationResult,
        now_iso: &str,
    ) -> (f64, String) {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        let norm_q = question_text.to_lowercase();
        let norm_attr = fact.attribute.to_lowercase();
        let norm_key = fact.fact_key.to_lowercase();
        let norm_val = fact.normalized_value.to_lowercase();

        let q_tokens: HashSet<String> = norm_q
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 2)
            .map(|s| s.to_string())
            .collect();

        let mut relevance_match = false;

        // 1. Kesesuaian Kategori (bobot maks: 0.25)
        let cat_match = classification
            .categories
            .iter()
            .any(|c| Self::category_matches_fact(c, &fact.category));

        if cat_match {
            score += 0.25;
            relevance_match = true;
            reasons.push(format!(
                "Kategori '{}' cocok dengan pertanyaan",
                fact.category
            ));
        }

        // 2. Pencocokan Atribut dan Kata Kunci Kunci (bobot maks: 0.35)
        if norm_q.contains(&norm_key) || norm_q.contains(&norm_attr) {
            score += 0.35;
            relevance_match = true;
            reasons.push(format!(
                "Nama atribut '{}' tertulis dalam pertanyaan",
                fact.attribute
            ));
        } else {
            // Cek token overlap antara atribut dan pertanyaan
            let attr_tokens: HashSet<String> = norm_attr
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() >= 2)
                .map(|s| s.to_string())
                .collect();

            let overlap_count = q_tokens.intersection(&attr_tokens).count();
            if overlap_count > 0 {
                let token_boost = (overlap_count as f64 * 0.15).min(0.25);
                score += token_boost;
                relevance_match = true;
                reasons.push(format!(
                    "{overlap_count} kata kunci atribut cocok dengan pertanyaan"
                ));
            }
        }

        // 3. Pencocokan Nilai Fakta Terhadap Pilihan Jawaban (bobot: 0.20)
        if let Some(opts) = options {
            let matched_opt = opts.iter().find(|opt| {
                let norm_opt = opt.to_lowercase();
                norm_opt.contains(&norm_val) || norm_val.contains(&norm_opt)
            });

            if let Some(opt_text) = matched_opt {
                score += 0.20;
                relevance_match = true;
                reasons.push(format!(
                    "Nilai fakta ('{}') cocok dengan opsi survei ('{}')",
                    fact.normalized_value, opt_text
                ));
            }
        }

        // Penyesuaian instruksi bila ada
        if let Some(inst) = instructions {
            let norm_inst = inst.to_lowercase();
            if norm_inst.contains(&norm_key) || norm_inst.contains(&norm_attr) {
                score = (score + 0.10).min(1.0);
                relevance_match = true;
                reasons.push("Atribut disebut dalam instruksi pertanyaan".to_string());
            }
        }

        // Jika sama sekali tidak ada kecocokan relevansi (kategori, nama atribut, kata kunci,
        // nilai opsi jawaban, atau instruksi), fakta ini tidak relevan dengan pertanyaan survei.
        // Jangan berikan skor hanya karena status verifikasi atau validitas masa aktif.
        if !relevance_match {
            return (
                0.0,
                "Tidak ada relevansi dengan pertanyaan survei".to_string(),
            );
        }

        // 4. Bobot Kualitas Status Verifikasi dan Masa Berlaku (bobot maks: 0.20)
        match fact.verification_status.as_str() {
            "confirmed" => {
                score += 0.15;
            }
            "unverified" => {
                score += 0.05;
                reasons.push("Status verifikasi: belum diverifikasi".to_string());
            }
            "outdated" => {
                score += 0.00;
                reasons.push("Status verifikasi: kadaluarsa".to_string());
            }
            "disputed" => {
                score -= 0.15;
                reasons.push("Status verifikasi: sedang dalam sengketa/dispute".to_string());
            }
            _ => {}
        }

        let is_valid = check_validity_period(
            fact.valid_from.as_deref(),
            fact.valid_until.as_deref(),
            now_iso,
        );
        if is_valid {
            score += 0.05;
        } else {
            score -= 0.10;
            reasons.push("Masa berlaku fakta telah lewat".to_string());
        }

        let clamped_score = score.clamp(0.0, 1.0);
        let final_reason = if reasons.is_empty() {
            "Kesesuaian umum profil".to_string()
        } else {
            reasons.join("; ")
        };

        (clamped_score, final_reason)
    }

    /// Memeriksa kesesuaian kategori pertanyaan terhadap kategori fakta profil.
    fn category_matches_fact(q_cat: &QuestionCategory, fact_cat: &str) -> bool {
        let norm_fact = fact_cat.trim().to_lowercase();
        match q_cat {
            QuestionCategory::Demographic => {
                matches!(
                    norm_fact.as_str(),
                    "demographic" | "demografi" | "biodata" | "profil"
                )
            }
            QuestionCategory::EmploymentEducation => {
                matches!(
                    norm_fact.as_str(),
                    "employment_education"
                        | "employment"
                        | "education"
                        | "pekerjaan"
                        | "pendidikan"
                        | "karir"
                )
            }
            QuestionCategory::FinancialEconomic => {
                matches!(
                    norm_fact.as_str(),
                    "financial_economic"
                        | "financial"
                        | "economic"
                        | "keuangan"
                        | "pendapatan"
                        | "ekonomi"
                        | "gaji"
                )
            }
            QuestionCategory::ConsumptionHabits => {
                matches!(
                    norm_fact.as_str(),
                    "consumption_habits"
                        | "consumption"
                        | "konsumsi"
                        | "kebiasaan"
                        | "belanja"
                        | "lifestyle"
                )
            }
            QuestionCategory::AssetOwnership => {
                matches!(
                    norm_fact.as_str(),
                    "asset_ownership"
                        | "assets"
                        | "aset"
                        | "kepemilikan"
                        | "kendaraan"
                        | "properti"
                )
            }
            QuestionCategory::BrandProductUsage => {
                matches!(
                    norm_fact.as_str(),
                    "brand_product_usage"
                        | "brand_usage"
                        | "brand"
                        | "merek"
                        | "produk"
                        | "penggunaan_produk"
                )
            }
            QuestionCategory::ExperienceBehavior => {
                matches!(
                    norm_fact.as_str(),
                    "experience_behavior"
                        | "experience"
                        | "behavior"
                        | "pengalaman"
                        | "perilaku"
                        | "aktivitas"
                )
            }
            QuestionCategory::PreferenceOpinion => {
                matches!(
                    norm_fact.as_str(),
                    "preference_opinion"
                        | "preference"
                        | "opinion"
                        | "preferensi"
                        | "opini"
                        | "minat"
                )
            }
            QuestionCategory::SatisfactionRating => {
                matches!(
                    norm_fact.as_str(),
                    "satisfaction_rating" | "satisfaction" | "rating" | "kepuasan" | "penilaian"
                )
            }
            _ => false,
        }
    }

    /// Evaluasi status masa berlaku fakta (Valid, Expired, Future, Unspecified).
    fn evaluate_validity(
        valid_from: Option<&str>,
        valid_until: Option<&str>,
        now_iso: &str,
    ) -> ValidityStatus {
        let today = if now_iso.len() >= 10 {
            &now_iso[..10]
        } else {
            return ValidityStatus::Unspecified;
        };

        if let Some(until) = valid_until {
            let until_part = if until.len() >= 10 {
                &until[..10]
            } else {
                until
            };
            if until_part < today {
                return ValidityStatus::Expired;
            }
        }

        if let Some(from) = valid_from {
            let from_part = if from.len() >= 10 { &from[..10] } else { from };
            if from_part > today {
                return ValidityStatus::Future;
            }
        }

        if valid_from.is_some() || valid_until.is_some() {
            ValidityStatus::Valid
        } else {
            ValidityStatus::Unspecified
        }
    }

    // =========================================================================
    // CONSISTENCY ENGINE & CONFLICT DETECTION
    // =========================================================================

    /// Deteksi konflik antar-fakta dalam profil (Fact vs Fact).
    fn detect_fact_conflicts(
        evidence_items: &[RetrievedEvidenceItem],
        all_persona_facts: &[ProfileFactDetail],
        conflicts: &mut Vec<EvidenceConflictIndicator>,
    ) {
        // Peta fact_key ke daftar fakta
        let mut facts_by_key: HashMap<String, Vec<&ProfileFactDetail>> = HashMap::new();
        for fact in all_persona_facts {
            facts_by_key
                .entry(fact.fact_key.clone())
                .or_default()
                .push(fact);
        }

        let evaluated_keys: HashSet<&str> = evidence_items
            .iter()
            .map(|item| item.fact_key.as_str())
            .collect();

        for key in evaluated_keys {
            if let Some(matching_facts) = facts_by_key.get(key) {
                // Filter fakta yang aktif (bukan outdated)
                let active_matching: Vec<&&ProfileFactDetail> = matching_facts
                    .iter()
                    .filter(|f| f.verification_status != "outdated" && f.archived_at.is_none())
                    .collect();

                if active_matching.len() > 1 {
                    for i in 0..active_matching.len() {
                        for j in (i + 1)..active_matching.len() {
                            let f1 = active_matching[i];
                            let f2 = active_matching[j];

                            if f1.normalized_value != f2.normalized_value {
                                conflicts.push(EvidenceConflictIndicator {
                                    conflict_type: ConflictType::FactConflict,
                                    severity: ConflictSeverity::Critical,
                                    description: format!(
                                        "Konflik fakta profil pada atribut '{}': Fakta ID '{}' bernilai '{}' ({}) bertentangan dengan Fakta ID '{}' bernilai '{}' ({}).",
                                        f1.attribute,
                                        f1.id,
                                        f1.normalized_value,
                                        f1.verification_status,
                                        f2.id,
                                        f2.normalized_value,
                                        f2.verification_status
                                    ),
                                    primary_fact_id: Some(f1.id.clone()),
                                    primary_source_id: f1.sources.first().map(|s| s.id.clone()),
                                    conflicting_fact_id: Some(f2.id.clone()),
                                    conflicting_source_id: f2.sources.first().map(|s| s.id.clone()),
                                    historical_question_id: None,
                                    historical_question_text: None,
                                    historical_answer: None,
                                    resolution_prompt: Some(format!(
                                        "Pilih nilai yang benar untuk '{}' atau tandai salah satu sebagai usang (outdated).",
                                        f1.attribute
                                    )),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Deteksi fakta yang berstatus Disputed
        for item in evidence_items {
            if item.verification_status == VerificationStatus::Disputed {
                conflicts.push(EvidenceConflictIndicator {
                    conflict_type: ConflictType::FactConflict,
                    severity: ConflictSeverity::Critical,
                    description: format!(
                        "Fakta '{}' bernilai '{}' saat ini berstatus 'disputed' (dalam sengketa kebenaran).",
                        item.attribute, item.normalized_value
                    ),
                    primary_fact_id: Some(item.fact_id.clone()),
                    primary_source_id: item.source_id.clone(),
                    conflicting_fact_id: None,
                    conflicting_source_id: None,
                    historical_question_id: None,
                    historical_question_text: None,
                    historical_answer: None,
                    resolution_prompt: Some(format!(
                        "Verifikasi dan selesaikan sengketa pada fakta '{}' sebelum digunakan sebagai jawaban.",
                        item.attribute
                    )),
                });
            }
        }
    }

    /// Deteksi ketidaksesuaian timeframe antara batasan pertanyaan dan fakta (Timeframe Mismatch).
    fn detect_timeframe_mismatches(
        evidence_items: &[RetrievedEvidenceItem],
        classification: &QuestionClassificationResult,
        conflicts: &mut Vec<EvidenceConflictIndicator>,
    ) {
        if let Some(tf) = &classification.constraints.timeframe {
            for item in evidence_items {
                // Periksa jika pertanyaan meminta periode spesifik yang sempit (misal 30 hari atau 3 bulan)
                let is_recent_request = matches!(
                    tf.timeframe_type,
                    TimeframeType::PastNDays
                        | TimeframeType::PastNWeeks
                        | TimeframeType::PastNMonths
                );

                if is_recent_request {
                    // Cek jika fakta kadaluarsa atau memiliki tanggal observasi yang sangat lama
                    if item.validity_status == ValidityStatus::Expired {
                        conflicts.push(EvidenceConflictIndicator {
                            conflict_type: ConflictType::TimeframeMismatch,
                            severity: ConflictSeverity::Critical,
                            description: format!(
                                "Pertanyaan menanyakan periode '{}', namun fakta '{}' telah kadaluarsa (valid until: {}).",
                                tf.raw_text,
                                item.attribute,
                                item.valid_until.as_deref().unwrap_or("-")
                            ),
                            primary_fact_id: Some(item.fact_id.clone()),
                            primary_source_id: item.source_id.clone(),
                            conflicting_fact_id: None,
                            conflicting_source_id: None,
                            historical_question_id: None,
                            historical_question_text: None,
                            historical_answer: None,
                            resolution_prompt: Some(format!(
                                "Periksa apakah fakta '{}' masih berlaku untuk periode '{}'.",
                                item.attribute, tf.raw_text
                            )),
                        });
                    }
                }
            }
        }
    }

    /// Deteksi kontradiksi terhadap riwayat jawaban survei masa lalu persona yang sama.
    fn detect_historical_contradictions(
        evidence_items: &[RetrievedEvidenceItem],
        historical_questions: &[SurveyQuestion],
        conflicts: &mut Vec<EvidenceConflictIndicator>,
    ) {
        for hist in historical_questions {
            let hist_ans = match &hist.user_answer_confirmed {
                Some(a) => a.trim().to_lowercase(),
                None => continue,
            };
            let hist_q_lower = hist.question_text_original.to_lowercase();

            for item in evidence_items {
                let fact_key_lower = item.fact_key.to_lowercase();
                let fact_attr_lower = item.attribute.to_lowercase();
                let fact_val_lower = item.normalized_value.to_lowercase();

                // Cek apakah pertanyaan historis membahas atribut yang sama
                let topic_matches = hist_q_lower.contains(&fact_key_lower)
                    || hist_q_lower.contains(&fact_attr_lower);

                if topic_matches {
                    // Cek kontradiksi nilai boolean/status
                    let is_opposite = (fact_val_lower == "true"
                        && (hist_ans == "tidak"
                            || hist_ans == "false"
                            || hist_ans.contains("tidak pernah")))
                        || (fact_val_lower == "false"
                            && (hist_ans == "ya"
                                || hist_ans == "true"
                                || hist_ans.contains("pernah")))
                        || (fact_val_lower == "ya" && (hist_ans == "tidak" || hist_ans == "bukan"))
                        || (fact_val_lower == "tidak" && (hist_ans == "ya" || hist_ans == "benar"));

                    // Cek kontradiksi nilai eksplisit (misal merokok vs tidak merokok, atau nama merek berbeda)
                    let explicit_contradiction = is_opposite
                        || (!hist_ans.contains(&fact_val_lower)
                            && !fact_val_lower.contains(&hist_ans)
                            && (fact_val_lower.len() > 3 && hist_ans.len() > 3));

                    if is_opposite {
                        conflicts.push(EvidenceConflictIndicator {
                            conflict_type: ConflictType::HistoricalContradiction,
                            severity: ConflictSeverity::Critical,
                            description: format!(
                                "Kontradiksi historis terdeteksi: Pada survei terdahulu untuk pertanyaan '{}', jawaban terkonfirmasi Anda adalah '{}', bertentangan dengan fakta profil '{}' saat ini bernilai '{}'.",
                                hist.question_text_original,
                                hist.user_answer_confirmed.as_deref().unwrap_or(""),
                                item.attribute,
                                item.normalized_value
                            ),
                            primary_fact_id: Some(item.fact_id.clone()),
                            primary_source_id: item.source_id.clone(),
                            conflicting_fact_id: None,
                            conflicting_source_id: None,
                            historical_question_id: Some(hist.id.clone()),
                            historical_question_text: Some(hist.question_text_original.clone()),
                            historical_answer: hist.user_answer_confirmed.clone(),
                            resolution_prompt: Some(format!(
                                "Konfirmasikan apakah kondisi telah berubah dari '{}' menjadi '{}'.",
                                hist.user_answer_confirmed.as_deref().unwrap_or(""),
                                item.normalized_value
                            )),
                        });
                    } else if explicit_contradiction
                        && (fact_key_lower.contains("status")
                            || fact_key_lower.contains("pekerjaan")
                            || fact_key_lower.contains("pendidikan"))
                    {
                        conflicts.push(EvidenceConflictIndicator {
                            conflict_type: ConflictType::HistoricalContradiction,
                            severity: ConflictSeverity::Warning,
                            description: format!(
                                "Perbedaan nilai jawaban historis: Pertanyaan terdahulu dijawab '{}', sedangkan fakta profil saat ini adalah '{}'.",
                                hist.user_answer_confirmed.as_deref().unwrap_or(""),
                                item.normalized_value
                            ),
                            primary_fact_id: Some(item.fact_id.clone()),
                            primary_source_id: item.source_id.clone(),
                            conflicting_fact_id: None,
                            conflicting_source_id: None,
                            historical_question_id: Some(hist.id.clone()),
                            historical_question_text: Some(hist.question_text_original.clone()),
                            historical_answer: hist.user_answer_confirmed.clone(),
                            resolution_prompt: Some("Periksa apakah terdapat pembaruan data terkini.".to_string()),
                        });
                    }
                }
            }
        }
    }

    /// Deteksi perubahan nilai fakta seiring waktu dari tabel `fact_history`.
    fn detect_value_changes_over_time(
        evidence_items: &[RetrievedEvidenceItem],
        all_persona_facts: &[ProfileFactDetail],
        conflicts: &mut Vec<EvidenceConflictIndicator>,
    ) {
        let fact_map: HashMap<&str, &ProfileFactDetail> = all_persona_facts
            .iter()
            .map(|f| (f.id.as_str(), f))
            .collect();

        for item in evidence_items {
            if let Some(fact_detail) = fact_map.get(item.fact_id.as_str()) {
                if let Some(latest_history) = fact_detail.history.first() {
                    if let Some(prev_val) = &latest_history.previous_value_json {
                        let clean_prev = prev_val.trim_matches('"');
                        let clean_new = fact_detail.normalized_value.as_str();

                        if clean_prev != clean_new && !clean_prev.is_empty() {
                            conflicts.push(EvidenceConflictIndicator {
                                conflict_type: ConflictType::ValueChangeOverTime,
                                severity: ConflictSeverity::Note,
                                description: format!(
                                    "Fakta '{}' telah diperbarui dari '{}' menjadi '{}' pada {}.",
                                    item.attribute,
                                    clean_prev,
                                    clean_new,
                                    latest_history.changed_at
                                ),
                                primary_fact_id: Some(item.fact_id.clone()),
                                primary_source_id: item.source_id.clone(),
                                conflicting_fact_id: None,
                                conflicting_source_id: None,
                                historical_question_id: None,
                                historical_question_text: None,
                                historical_answer: None,
                                resolution_prompt: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // =========================================================================
    // RECOMMENDATION & CONFIDENCE SYNTHESIS
    // =========================================================================

    /// Mensintesis rekomendasi akhir, tingkat keyakinan bukti, dan ringkasan audit.
    fn synthesize_recommendation_and_confidence(
        evidence_items: &[RetrievedEvidenceItem],
        conflict_indicators: &[EvidenceConflictIndicator],
        missing_information: &[String],
        sensitive_filtered_count: usize,
    ) -> (EvidenceRecommendation, EvidenceConfidence, String) {
        let has_critical_conflict = conflict_indicators
            .iter()
            .any(|c| c.severity == ConflictSeverity::Critical);

        let has_confirmed_evidence = evidence_items.iter().any(|e| {
            e.verification_status == VerificationStatus::Confirmed && e.relevance_score >= 0.35
        });

        let top_score = evidence_items
            .first()
            .map(|e| e.relevance_score)
            .unwrap_or(0.0);

        if has_critical_conflict {
            let critical_count = conflict_indicators
                .iter()
                .filter(|c| c.severity == ConflictSeverity::Critical)
                .count();

            (
                EvidenceRecommendation::NeedsClarification,
                EvidenceConfidence::Uncertain,
                format!(
                    "Terdeteksi {critical_count} konflik kritis antara fakta profil atau jawaban historis. Diperlukan klarifikasi sebelum menjawab."
                ),
            )
        } else if evidence_items.is_empty() || top_score < 0.20 {
            let reason = if sensitive_filtered_count > 0 {
                format!(
                    "Bukti tidak mencukupi (terdapat {sensitive_filtered_count} fakta sensitif yang memerlukan persetujuan eksplisit). Mesin wajib abstain."
                )
            } else {
                "Tidak ditemukan fakta yang relevan dalam profil responden aktif. Mesin wajib abstain (tidak boleh mengarang).".to_string()
            };

            (
                EvidenceRecommendation::InsufficientEvidence,
                EvidenceConfidence::Uncertain,
                reason,
            )
        } else if !has_confirmed_evidence {
            (
                EvidenceRecommendation::NeedsClarification,
                EvidenceConfidence::Low,
                "Fakta relevan ditemukan namun berstatus belum diverifikasi (unverified) atau kadaluarsa. Konfirmasi pengguna diperlukan.".to_string(),
            )
        } else if top_score >= 0.70 {
            let count = evidence_items.len();
            let missing_note = if !missing_information.is_empty() {
                format!(
                    " (Perhatian: {} catatan informasi tambahan)",
                    missing_information.len()
                )
            } else {
                String::new()
            };
            (
                EvidenceRecommendation::Supported,
                EvidenceConfidence::High,
                format!(
                    "Ditemukan {count} bukti relevan terkonfirmasi dengan tingkat keyakinan tinggi.{missing_note} Siap untuk generasi jawaban."
                ),
            )
        } else if top_score >= 0.45 {
            let count = evidence_items.len();
            let missing_note = if !missing_information.is_empty() {
                format!(
                    " (Perhatian: {} catatan informasi tambahan)",
                    missing_information.len()
                )
            } else {
                String::new()
            };
            (
                EvidenceRecommendation::Supported,
                EvidenceConfidence::Medium,
                format!(
                    "Ditemukan {count} bukti relevan terkonfirmasi dengan tingkat keyakinan sedang.{missing_note}"
                ),
            )
        } else {
            (
                EvidenceRecommendation::Supported,
                EvidenceConfidence::Low,
                "Bukti relevan ditemukan dengan tingkat keyakinan rendah.".to_string(),
            )
        }
    }
}
