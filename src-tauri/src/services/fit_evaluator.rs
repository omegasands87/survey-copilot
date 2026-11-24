use crate::domain::fact::FactSummary;
use crate::domain::fit::{FitCriterionResult, MatchState, OverallFitState};
use crate::domain::opportunity::{CriterionOperator, CriterionType, OpportunityCriterion};

pub struct FitEvaluator;

impl FitEvaluator {
    /// Mengevaluasi seluruh kriteria peluang survei terhadap sekumpulan fakta profil persona
    pub fn evaluate_criteria(
        criteria: &[OpportunityCriterion],
        facts: &[FactSummary],
    ) -> Vec<FitCriterionResult> {
        criteria
            .iter()
            .map(|crit| Self::evaluate_single_criterion(crit, facts))
            .collect()
    }

    /// Menghitung kesimpulan kesesuaian keseluruhan dari kumpulan hasil evaluasi kriteria
    pub fn compute_overall_fit(results: &[FitCriterionResult]) -> OverallFitState {
        if results.is_empty() {
            return OverallFitState::NoCriteria;
        }

        // 1. Jika ada kriteria yang tidak cocok (mismatch), persona tidak memenuhi syarat
        if results
            .iter()
            .any(|r| r.match_state == MatchState::Mismatch)
        {
            return OverallFitState::Ineligible;
        }

        // 2. Jika ada konflik data profil pada kriteria
        if results
            .iter()
            .any(|r| r.match_state == MatchState::Conflict)
        {
            return OverallFitState::Conflict;
        }

        // 3. Jika ada fakta pendukung yang sudah kadaluarsa (stale)
        if results.iter().any(|r| r.match_state == MatchState::Stale) {
            return OverallFitState::Stale;
        }

        // 4. Jika ada kriteria yang belum diketahui (unknown) -> Incomplete (Unknown NEVER treated as match)
        if results.iter().any(|r| r.match_state == MatchState::Unknown) {
            return OverallFitState::Incomplete;
        }

        // 5. Seluruh kriteria cocok (match)
        OverallFitState::Eligible
    }

    /// Evaluasi satu kriteria eksplisit terhadap sekumpulan fakta profil
    pub fn evaluate_single_criterion(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        match criterion.criterion_type {
            CriterionType::Age => Self::evaluate_age(criterion, facts),
            CriterionType::Gender => Self::evaluate_gender(criterion, facts),
            CriterionType::Location => Self::evaluate_location(criterion, facts),
            CriterionType::Occupation => Self::evaluate_occupation(criterion, facts),
            CriterionType::Education => Self::evaluate_education(criterion, facts),
            CriterionType::Income => Self::evaluate_income(criterion, facts),
            CriterionType::MaritalStatus => Self::evaluate_marital_status(criterion, facts),
            CriterionType::ProductUsage => Self::evaluate_product_usage(criterion, facts),
            CriterionType::General => Self::evaluate_general(criterion, facts),
        }
    }

    // ==========================================
    // 1. USIA (Age)
    // ==========================================
    fn evaluate_age(criterion: &OpportunityCriterion, facts: &[FactSummary]) -> FitCriterionResult {
        let matching_facts: Vec<&FactSummary> = facts
            .iter()
            .filter(|f| {
                let k = f.fact_key.to_lowercase();
                let a = f.attribute.to_lowercase();
                k == "age"
                    || k == "usia"
                    || k == "umur"
                    || k == "birth_date"
                    || k == "tanggal_lahir"
                    || a.contains("usia")
                    || a.contains("umur")
                    || a.contains("tanggal lahir")
            })
            .collect();

        if matching_facts.is_empty() {
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![],
                supporting_facts_summary: vec![],
                explanation: "Tidak ada data usia atau tanggal lahir pada profil persona ini."
                    .to_string(),
            };
        }

        // Periksa status kadaluarsa atau konflik (AT-062)
        if let Some(err_result) = Self::check_fact_validity(&matching_facts, criterion, "usia") {
            return err_result;
        }

        // Ambil fakta terkonfirmasi yang valid
        let confirmed_facts: Vec<&&FactSummary> = matching_facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed_facts.is_empty() {
            let fact_ids = matching_facts.iter().map(|f| f.id.clone()).collect();
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Fakta usia berstatus '{}' (belum terkonfirmasi)",
                    matching_facts[0].verification_status
                )],
                explanation: "Data usia tersedia namun belum terkonfirmasi (unverified)."
                    .to_string(),
            };
        }

        // Ekstraksi nilai usia
        let fact = confirmed_facts[0];
        let age_opt = Self::extract_age_value(fact);

        if let Some(age) = age_opt {
            let (is_match, reason) = match criterion.operator {
                CriterionOperator::Range => {
                    let min = criterion
                        .expected_value_json
                        .as_ref()
                        .and_then(|v| v.get("min"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let max = criterion
                        .expected_value_json
                        .as_ref()
                        .and_then(|v| v.get("max"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(120);

                    if age >= min && age <= max {
                        (
                            true,
                            format!(
                                "Usia persona ({age} tahun) berada dalam rentang kriteria ({min}-{max} tahun)."
                            ),
                        )
                    } else {
                        (
                            false,
                            format!(
                                "Usia persona ({age} tahun) berada di luar rentang kriteria ({min}-{max} tahun)."
                            ),
                        )
                    }
                }
                CriterionOperator::GreaterThanOrEqual => {
                    let min = criterion
                        .expected_value_json
                        .as_ref()
                        .and_then(|v| v.get("min"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    if age >= min {
                        (
                            true,
                            format!(
                                "Usia persona ({age} tahun) memenuhi syarat minimal ({min} tahun)."
                            ),
                        )
                    } else {
                        (
                            false,
                            format!(
                                "Usia persona ({age} tahun) kurang dari syarat minimal ({min} tahun)."
                            ),
                        )
                    }
                }
                CriterionOperator::LessThanOrEqual => {
                    let max = criterion
                        .expected_value_json
                        .as_ref()
                        .and_then(|v| v.get("max"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(120);

                    if age <= max {
                        (
                            true,
                            format!(
                                "Usia persona ({age} tahun) memenuhi batas maksimal ({max} tahun)."
                            ),
                        )
                    } else {
                        (
                            false,
                            format!(
                                "Usia persona ({age} tahun) melebihi batas maksimal ({max} tahun)."
                            ),
                        )
                    }
                }
                CriterionOperator::Equals => {
                    let target = criterion
                        .expected_value_json
                        .as_ref()
                        .and_then(|v| v.get("value"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    if age == target {
                        (
                            true,
                            format!("Usia persona ({age} tahun) tepat sesuai kriteria ({target} tahun)."),
                        )
                    } else {
                        (
                            false,
                            format!("Usia persona ({age} tahun) tidak sama dengan kriteria ({target} tahun)."),
                        )
                    }
                }
                _ => (true, format!("Usia persona ({age} tahun) terverifikasi.")),
            };

            FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: if is_match {
                    MatchState::Match
                } else {
                    MatchState::Mismatch
                },
                supporting_fact_ids: vec![fact.id.clone()],
                supporting_facts_summary: vec![format!("Usia: {age} tahun (confirmed)")],
                explanation: reason,
            }
        } else {
            FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![fact.id.clone()],
                supporting_facts_summary: vec![format!("Format nilai usia: {:?}", fact.value)],
                explanation: "Format nilai usia pada profil tidak dapat diparsing sebagai angka."
                    .to_string(),
            }
        }
    }

    // ==========================================
    // 2. GENDER (Jenis Kelamin)
    // ==========================================
    fn evaluate_gender(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        let matching_facts: Vec<&FactSummary> = facts
            .iter()
            .filter(|f| {
                let k = f.fact_key.to_lowercase();
                let a = f.attribute.to_lowercase();
                k == "gender"
                    || k == "jenis_kelamin"
                    || k == "kelamin"
                    || k == "sex"
                    || a.contains("gender")
                    || a.contains("jenis kelamin")
            })
            .collect();

        if matching_facts.is_empty() {
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![],
                supporting_facts_summary: vec![],
                explanation: "Tidak ada data jenis kelamin pada profil persona ini.".to_string(),
            };
        }

        if let Some(err_result) =
            Self::check_fact_validity(&matching_facts, criterion, "jenis kelamin")
        {
            return err_result;
        }

        let confirmed_facts: Vec<&&FactSummary> = matching_facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed_facts.is_empty() {
            let fact_ids = matching_facts.iter().map(|f| f.id.clone()).collect();
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Jenis kelamin berstatus '{}' (belum terkonfirmasi)",
                    matching_facts[0].verification_status
                )],
                explanation: "Data jenis kelamin ada namun belum terkonfirmasi.".to_string(),
            };
        }

        let fact = confirmed_facts[0];
        let persona_gender_norm = Self::normalize_gender_str(&fact.normalized_value);

        // Ambil expected gender dari kriteria
        let mut expected_genders = Vec::new();
        if let Some(json) = &criterion.expected_value_json {
            if let Some(arr) = json.as_array() {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        expected_genders.push(Self::normalize_gender_str(s));
                    }
                }
            } else if let Some(s) = json.as_str() {
                expected_genders.push(Self::normalize_gender_str(s));
            } else if let Some(s) = json.get("value").and_then(|v| v.as_str()) {
                expected_genders.push(Self::normalize_gender_str(s));
            }
        }

        // Jika kriteria mencakup keduanya (pria dan wanita)
        let allows_both = expected_genders.contains(&"male".to_string())
            && expected_genders.contains(&"female".to_string());

        let (is_match, reason) = if allows_both {
            (
                true,
                format!(
                    "Survei terbuka untuk semua gender (pria & wanita); gender persona adalah {}.",
                    fact.normalized_value
                ),
            )
        } else if expected_genders.contains(&persona_gender_norm) {
            (
                true,
                format!(
                    "Gender persona ({}) sesuai dengan kriteria survei.",
                    fact.normalized_value
                ),
            )
        } else {
            (
                false,
                format!(
                    "Gender persona ({}) tidak memenuhi kriteria survei (disyaratkan: {}).",
                    fact.normalized_value, criterion.criterion_text
                ),
            )
        };

        FitCriterionResult {
            criterion_id: criterion.id.clone(),
            criterion_text: criterion.criterion_text.clone(),
            raw_text: criterion.raw_text.clone(),
            criterion_type: criterion.criterion_type,
            operator: criterion.operator,
            match_state: if is_match {
                MatchState::Match
            } else {
                MatchState::Mismatch
            },
            supporting_fact_ids: vec![fact.id.clone()],
            supporting_facts_summary: vec![format!(
                "Jenis Kelamin: {} (confirmed)",
                fact.normalized_value
            )],
            explanation: reason,
        }
    }

    // ==========================================
    // 3. DOMISILI / LOKASI (Location)
    // ==========================================
    fn evaluate_location(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        let matching_facts: Vec<&FactSummary> = facts
            .iter()
            .filter(|f| {
                let k = f.fact_key.to_lowercase();
                let a = f.attribute.to_lowercase();
                let c = f.category.to_lowercase();
                k == "location"
                    || k == "domisili"
                    || k == "kota"
                    || k == "city"
                    || k == "provinsi"
                    || k == "tempat_tinggal"
                    || k == "alamat"
                    || a.contains("domisili")
                    || a.contains("kota")
                    || a.contains("tempat tinggal")
                    || c.contains("domisili")
                    || c.contains("lokasi")
            })
            .collect();

        if matching_facts.is_empty() {
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![],
                supporting_facts_summary: vec![],
                explanation:
                    "Tidak ada data domisili atau lokasi tempat tinggal pada profil persona ini."
                        .to_string(),
            };
        }

        if let Some(err_result) = Self::check_fact_validity(&matching_facts, criterion, "domisili")
        {
            return err_result;
        }

        let confirmed_facts: Vec<&&FactSummary> = matching_facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed_facts.is_empty() {
            let fact_ids = matching_facts.iter().map(|f| f.id.clone()).collect();
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Domisili berstatus '{}' (belum terkonfirmasi)",
                    matching_facts[0].verification_status
                )],
                explanation: "Data domisili ada namun belum terkonfirmasi.".to_string(),
            };
        }

        let fact = confirmed_facts[0];
        let persona_loc = fact.normalized_value.to_lowercase();

        // Kumpulkan daftar wilayah yang disyaratkan
        let mut expected_locations = Vec::new();
        if let Some(json) = &criterion.expected_value_json {
            if let Some(arr) = json.as_array() {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        expected_locations.push(s.to_lowercase());
                    }
                }
            } else if let Some(s) = json.as_str() {
                expected_locations.push(s.to_lowercase());
            }
        }

        // Periksa kesesuaian wilayah (termasuk Jabodetabek)
        let is_match = Self::check_location_match(&persona_loc, &expected_locations);

        let (state, reason) = if is_match {
            (
                MatchState::Match,
                format!(
                    "Domisili persona ({}) sesuai dengan wilayah kriteria survei.",
                    fact.normalized_value
                ),
            )
        } else {
            (
                MatchState::Mismatch,
                format!(
                    "Domisili persona ({}) tidak termasuk dalam wilayah yang disyaratkan ({}).",
                    fact.normalized_value, criterion.criterion_text
                ),
            )
        };

        FitCriterionResult {
            criterion_id: criterion.id.clone(),
            criterion_text: criterion.criterion_text.clone(),
            raw_text: criterion.raw_text.clone(),
            criterion_type: criterion.criterion_type,
            operator: criterion.operator,
            match_state: state,
            supporting_fact_ids: vec![fact.id.clone()],
            supporting_facts_summary: vec![format!(
                "Domisili: {} (confirmed)",
                fact.normalized_value
            )],
            explanation: reason,
        }
    }

    // ==========================================
    // 4. PEKERJAAN & INDUSTRI (Occupation)
    // ==========================================
    fn evaluate_occupation(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        let matching_facts: Vec<&FactSummary> = facts
            .iter()
            .filter(|f| {
                let k = f.fact_key.to_lowercase();
                let a = f.attribute.to_lowercase();
                k == "occupation"
                    || k == "pekerjaan"
                    || k == "profesi"
                    || k == "job"
                    || k == "industri"
                    || k == "bidang_pekerjaan"
                    || a.contains("pekerjaan")
                    || a.contains("profesi")
                    || a.contains("industri")
            })
            .collect();

        if matching_facts.is_empty() {
            let msg = if criterion.operator == CriterionOperator::NotEquals {
                "Tidak ada data pekerjaan persona untuk memvalidasi syarat eksklusi industri."
            } else {
                "Tidak ada data pekerjaan pada profil persona ini."
            };
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![],
                supporting_facts_summary: vec![],
                explanation: msg.to_string(),
            };
        }

        if let Some(err_result) = Self::check_fact_validity(&matching_facts, criterion, "pekerjaan")
        {
            return err_result;
        }

        let confirmed_facts: Vec<&&FactSummary> = matching_facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed_facts.is_empty() {
            let fact_ids = matching_facts.iter().map(|f| f.id.clone()).collect();
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Pekerjaan berstatus '{}' (belum terkonfirmasi)",
                    matching_facts[0].verification_status
                )],
                explanation: "Data pekerjaan ada namun belum terkonfirmasi.".to_string(),
            };
        }

        let fact = confirmed_facts[0];
        let persona_job = fact.normalized_value.to_lowercase();

        // Tangani kriteria eksklusi (NotEquals) seperti "Bukan periklanan, media, riset pasar"
        if criterion.operator == CriterionOperator::NotEquals {
            let mut excluded_terms = Vec::new();
            if let Some(json) = &criterion.expected_value_json {
                if let Some(arr) = json.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            excluded_terms.push(s.to_lowercase());
                        }
                    }
                } else if let Some(s) = json.as_str() {
                    excluded_terms.push(s.to_lowercase());
                }
            }

            let violates_exclusion = excluded_terms.iter().any(|term| {
                persona_job.contains(term)
                    || (term.contains("riset pasar") && persona_job.contains("market research"))
                    || (term.contains("periklanan") && persona_job.contains("advertising"))
            });

            let (state, reason) = if violates_exclusion {
                (
                    MatchState::Mismatch,
                    format!(
                        "Pekerjaan persona ({}) termasuk dalam bidang yang dieksklusikan dari survei ini.",
                        fact.normalized_value
                    ),
                )
            } else {
                (
                    MatchState::Match,
                    format!(
                        "Pekerjaan persona ({}) lolos syarat eksklusi industri.",
                        fact.normalized_value
                    ),
                )
            };

            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: state,
                supporting_fact_ids: vec![fact.id.clone()],
                supporting_facts_summary: vec![format!(
                    "Pekerjaan: {} (confirmed)",
                    fact.normalized_value
                )],
                explanation: reason,
            };
        }

        // Kriteria pekerjaan yang disyaratkan (In / Equals)
        let mut target_jobs = Vec::new();
        if let Some(json) = &criterion.expected_value_json {
            if let Some(arr) = json.as_array() {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        target_jobs.push(s.to_lowercase());
                    }
                }
            } else if let Some(s) = json.as_str() {
                target_jobs.push(s.to_lowercase());
            }
        }

        let is_match = target_jobs
            .iter()
            .any(|t| persona_job.contains(t) || t.contains(&persona_job));

        let (state, reason) = if is_match {
            (
                MatchState::Match,
                format!(
                    "Pekerjaan persona ({}) sesuai dengan kriteria yang disyaratkan.",
                    fact.normalized_value
                ),
            )
        } else {
            (
                MatchState::Mismatch,
                format!(
                    "Pekerjaan persona ({}) tidak memenuhi kriteria yang disyaratkan ({}).",
                    fact.normalized_value, criterion.criterion_text
                ),
            )
        };

        FitCriterionResult {
            criterion_id: criterion.id.clone(),
            criterion_text: criterion.criterion_text.clone(),
            raw_text: criterion.raw_text.clone(),
            criterion_type: criterion.criterion_type,
            operator: criterion.operator,
            match_state: state,
            supporting_fact_ids: vec![fact.id.clone()],
            supporting_facts_summary: vec![format!(
                "Pekerjaan: {} (confirmed)",
                fact.normalized_value
            )],
            explanation: reason,
        }
    }

    // ==========================================
    // 5. PENDIDIKAN (Education)
    // ==========================================
    fn evaluate_education(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        let matching_facts: Vec<&FactSummary> = facts
            .iter()
            .filter(|f| {
                let k = f.fact_key.to_lowercase();
                let a = f.attribute.to_lowercase();
                k == "education"
                    || k == "pendidikan"
                    || k == "pendidikan_terakhir"
                    || a.contains("pendidikan")
            })
            .collect();

        if matching_facts.is_empty() {
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![],
                supporting_facts_summary: vec![],
                explanation: "Tidak ada data jenjang pendidikan pada profil persona ini."
                    .to_string(),
            };
        }

        if let Some(err_result) =
            Self::check_fact_validity(&matching_facts, criterion, "pendidikan")
        {
            return err_result;
        }

        let confirmed_facts: Vec<&&FactSummary> = matching_facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed_facts.is_empty() {
            let fact_ids = matching_facts.iter().map(|f| f.id.clone()).collect();
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Pendidikan berstatus '{}' (belum terkonfirmasi)",
                    matching_facts[0].verification_status
                )],
                explanation: "Data pendidikan ada namun belum terkonfirmasi.".to_string(),
            };
        }

        let fact = confirmed_facts[0];
        let persona_edu_rank = Self::education_level_rank(&fact.normalized_value);

        let required_rank = criterion
            .expected_value_json
            .as_ref()
            .and_then(|v| v.get("min"))
            .and_then(|v| v.as_str())
            .map(Self::education_level_rank)
            .unwrap_or(0);

        let is_match = persona_edu_rank >= required_rank;

        let (state, reason) = if is_match {
            (
                MatchState::Match,
                format!(
                    "Jenjang pendidikan persona ({}) memenuhi kriteria minimal.",
                    fact.normalized_value
                ),
            )
        } else {
            (
                MatchState::Mismatch,
                format!(
                    "Jenjang pendidikan persona ({}) belum memenuhi kriteria minimal ({}).",
                    fact.normalized_value, criterion.criterion_text
                ),
            )
        };

        FitCriterionResult {
            criterion_id: criterion.id.clone(),
            criterion_text: criterion.criterion_text.clone(),
            raw_text: criterion.raw_text.clone(),
            criterion_type: criterion.criterion_type,
            operator: criterion.operator,
            match_state: state,
            supporting_fact_ids: vec![fact.id.clone()],
            supporting_facts_summary: vec![format!(
                "Pendidikan: {} (confirmed)",
                fact.normalized_value
            )],
            explanation: reason,
        }
    }

    // ==========================================
    // 6. PENGHASILAN / PENGELUARAN (Income)
    // ==========================================
    fn evaluate_income(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        let matching_facts: Vec<&FactSummary> = facts
            .iter()
            .filter(|f| {
                let k = f.fact_key.to_lowercase();
                let a = f.attribute.to_lowercase();
                k == "income"
                    || k == "penghasilan"
                    || k == "gaji"
                    || k == "pengeluaran"
                    || k == "ses"
                    || a.contains("penghasilan")
                    || a.contains("gaji")
                    || a.contains("pengeluaran")
            })
            .collect();

        if matching_facts.is_empty() {
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![],
                supporting_facts_summary: vec![],
                explanation: "Tidak ada data penghasilan atau pengeluaran pada profil persona ini."
                    .to_string(),
            };
        }

        if let Some(err_result) =
            Self::check_fact_validity(&matching_facts, criterion, "penghasilan")
        {
            return err_result;
        }

        let confirmed_facts: Vec<&&FactSummary> = matching_facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed_facts.is_empty() {
            let fact_ids = matching_facts.iter().map(|f| f.id.clone()).collect();
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Penghasilan berstatus '{}' (belum terkonfirmasi)",
                    matching_facts[0].verification_status
                )],
                explanation: "Data penghasilan ada namun belum terkonfirmasi.".to_string(),
            };
        }

        let fact = confirmed_facts[0];
        FitCriterionResult {
            criterion_id: criterion.id.clone(),
            criterion_text: criterion.criterion_text.clone(),
            raw_text: criterion.raw_text.clone(),
            criterion_type: criterion.criterion_type,
            operator: criterion.operator,
            match_state: MatchState::Match,
            supporting_fact_ids: vec![fact.id.clone()],
            supporting_facts_summary: vec![format!(
                "Penghasilan: {} (confirmed)",
                fact.normalized_value
            )],
            explanation: format!(
                "Data penghasilan persona ({}) telah tercatat dan terkonfirmasi.",
                fact.normalized_value
            ),
        }
    }

    // ==========================================
    // 7. STATUS PERNIKAHAN (Marital Status)
    // ==========================================
    fn evaluate_marital_status(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        let matching_facts: Vec<&FactSummary> = facts
            .iter()
            .filter(|f| {
                let k = f.fact_key.to_lowercase();
                let a = f.attribute.to_lowercase();
                k == "marital_status"
                    || k == "status_pernikahan"
                    || k == "status_kawin"
                    || a.contains("pernikahan")
                    || a.contains("status kawin")
            })
            .collect();

        if matching_facts.is_empty() {
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![],
                supporting_facts_summary: vec![],
                explanation: "Tidak ada data status pernikahan pada profil persona ini."
                    .to_string(),
            };
        }

        if let Some(err_result) =
            Self::check_fact_validity(&matching_facts, criterion, "status pernikahan")
        {
            return err_result;
        }

        let confirmed_facts: Vec<&&FactSummary> = matching_facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed_facts.is_empty() {
            let fact_ids = matching_facts.iter().map(|f| f.id.clone()).collect();
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Status pernikahan berstatus '{}'",
                    matching_facts[0].verification_status
                )],
                explanation: "Data status pernikahan belum terkonfirmasi.".to_string(),
            };
        }

        let fact = confirmed_facts[0];
        let persona_status = fact.normalized_value.to_lowercase();

        let target_status = criterion
            .expected_value_json
            .as_ref()
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        let is_match =
            persona_status.contains(&target_status) || target_status.contains(&persona_status);

        let (state, reason) = if is_match {
            (
                MatchState::Match,
                format!(
                    "Status pernikahan persona ({}) sesuai dengan kriteria survei.",
                    fact.normalized_value
                ),
            )
        } else {
            (
                MatchState::Mismatch,
                format!(
                    "Status pernikahan persona ({}) tidak sesuai dengan kriteria ({}).",
                    fact.normalized_value, criterion.criterion_text
                ),
            )
        };

        FitCriterionResult {
            criterion_id: criterion.id.clone(),
            criterion_text: criterion.criterion_text.clone(),
            raw_text: criterion.raw_text.clone(),
            criterion_type: criterion.criterion_type,
            operator: criterion.operator,
            match_state: state,
            supporting_fact_ids: vec![fact.id.clone()],
            supporting_facts_summary: vec![format!(
                "Status Pernikahan: {} (confirmed)",
                fact.normalized_value
            )],
            explanation: reason,
        }
    }

    // ==========================================
    // 8. PENGGUNAAN / KEPEMILIKAN PRODUK
    // ==========================================
    fn evaluate_product_usage(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        let crit_text = criterion.criterion_text.to_lowercase();

        let matching_facts: Vec<&FactSummary> = facts
            .iter()
            .filter(|f| {
                let k = f.fact_key.to_lowercase();
                let a = f.attribute.to_lowercase();
                let c = f.category.to_lowercase();
                crit_text.contains(&k)
                    || crit_text.contains(&a)
                    || k.contains("product")
                    || k.contains("produk")
                    || c.contains("consumption")
                    || c.contains("lifestyle")
            })
            .collect();

        if matching_facts.is_empty() {
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: vec![],
                supporting_facts_summary: vec![],
                explanation: format!(
                    "Tidak ditemukan data kepemilikan atau penggunaan produk terkait ({}) pada profil persona.",
                    criterion.criterion_text
                ),
            };
        }

        if let Some(err_result) =
            Self::check_fact_validity(&matching_facts, criterion, "penggunaan produk")
        {
            return err_result;
        }

        let confirmed_facts: Vec<&&FactSummary> = matching_facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed_facts.is_empty() {
            let fact_ids = matching_facts.iter().map(|f| f.id.clone()).collect();
            return FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Unknown,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Data produk berstatus '{}' (belum terkonfirmasi)",
                    matching_facts[0].verification_status
                )],
                explanation: "Data produk ada namun belum terkonfirmasi.".to_string(),
            };
        }

        let fact = confirmed_facts[0];
        FitCriterionResult {
            criterion_id: criterion.id.clone(),
            criterion_text: criterion.criterion_text.clone(),
            raw_text: criterion.raw_text.clone(),
            criterion_type: criterion.criterion_type,
            operator: criterion.operator,
            match_state: MatchState::Match,
            supporting_fact_ids: vec![fact.id.clone()],
            supporting_facts_summary: vec![format!(
                "{}: {} (confirmed)",
                fact.attribute, fact.normalized_value
            )],
            explanation: format!(
                "Fakta profil '{}' bernilai '{}' mendukung kriteria ini.",
                fact.attribute, fact.normalized_value
            ),
        }
    }

    // ==========================================
    // 9. GENERAL (Kriteria Umum)
    // ==========================================
    fn evaluate_general(
        criterion: &OpportunityCriterion,
        facts: &[FactSummary],
    ) -> FitCriterionResult {
        if let Some(attr) = &criterion.normalized_attribute {
            let matching_facts: Vec<&FactSummary> = facts
                .iter()
                .filter(|f| {
                    f.fact_key.eq_ignore_ascii_case(attr) || f.attribute.eq_ignore_ascii_case(attr)
                })
                .collect();

            if !matching_facts.is_empty() {
                if let Some(err_result) =
                    Self::check_fact_validity(&matching_facts, criterion, attr)
                {
                    return err_result;
                }

                let confirmed: Vec<&&FactSummary> = matching_facts
                    .iter()
                    .filter(|f| f.verification_status == "confirmed")
                    .collect();

                if let Some(fact) = confirmed.first() {
                    return FitCriterionResult {
                        criterion_id: criterion.id.clone(),
                        criterion_text: criterion.criterion_text.clone(),
                        raw_text: criterion.raw_text.clone(),
                        criterion_type: criterion.criterion_type,
                        operator: criterion.operator,
                        match_state: MatchState::Match,
                        supporting_fact_ids: vec![fact.id.clone()],
                        supporting_facts_summary: vec![format!(
                            "{}: {} (confirmed)",
                            fact.attribute, fact.normalized_value
                        )],
                        explanation: format!(
                            "Fakta '{}' terkonfirmasi pada profil: {}.",
                            fact.attribute, fact.normalized_value
                        ),
                    };
                }
            }
        }

        FitCriterionResult {
            criterion_id: criterion.id.clone(),
            criterion_text: criterion.criterion_text.clone(),
            raw_text: criterion.raw_text.clone(),
            criterion_type: criterion.criterion_type,
            operator: criterion.operator,
            match_state: MatchState::Unknown,
            supporting_fact_ids: vec![],
            supporting_facts_summary: vec![],
            explanation: "Tidak ada fakta pendukung langsung untuk kriteria umum ini pada profil."
                .to_string(),
        }
    }

    // ==========================================
    // HELPER METHODS
    // ==========================================

    /// Memeriksa apakah sekumpulan fakta memiliki status kadaluarsa (Stale) atau konflik (Conflict)
    fn check_fact_validity(
        facts: &[&FactSummary],
        criterion: &OpportunityCriterion,
        domain_name: &str,
    ) -> Option<FitCriterionResult> {
        let fact_ids: Vec<String> = facts.iter().map(|f| f.id.clone()).collect();

        // 1. Cek Disputed
        if facts
            .iter()
            .any(|f| f.verification_status == "disputed" || f.has_conflict)
        {
            return Some(FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Conflict,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Ditemukan fakta {domain_name} berstatus disputed / memiliki konflik"
                )],
                explanation: format!(
                    "Fakta {domain_name} memiliki status disputed (diperdebatkan) atau ditemukan konflik pada profil."
                ),
            });
        }

        // 2. Cek apakah ada lebih dari satu fakta terkonfirmasi dengan nilai berbeda (Kontradiksi fakta)
        let confirmed: Vec<&&FactSummary> = facts
            .iter()
            .filter(|f| f.verification_status == "confirmed")
            .collect();

        if confirmed.len() > 1 {
            let first_val = &confirmed[0].normalized_value;
            let has_different_values = confirmed.iter().any(|f| &f.normalized_value != first_val);
            if has_different_values {
                return Some(FitCriterionResult {
                    criterion_id: criterion.id.clone(),
                    criterion_text: criterion.criterion_text.clone(),
                    raw_text: criterion.raw_text.clone(),
                    criterion_type: criterion.criterion_type,
                    operator: criterion.operator,
                    match_state: MatchState::Conflict,
                    supporting_fact_ids: fact_ids,
                    supporting_facts_summary: confirmed
                        .iter()
                        .map(|f| format!("{}: {}", f.attribute, f.normalized_value))
                        .collect(),
                    explanation: format!(
                        "Ditemukan beberapa fakta {domain_name} terkonfirmasi yang saling berkontradiksi pada profil."
                    ),
                });
            }
        }

        // 3. Cek Outdated / Expired (Stale)
        if facts
            .iter()
            .all(|f| f.verification_status == "outdated" || !f.is_valid)
        {
            return Some(FitCriterionResult {
                criterion_id: criterion.id.clone(),
                criterion_text: criterion.criterion_text.clone(),
                raw_text: criterion.raw_text.clone(),
                criterion_type: criterion.criterion_type,
                operator: criterion.operator,
                match_state: MatchState::Stale,
                supporting_fact_ids: fact_ids,
                supporting_facts_summary: vec![format!(
                    "Fakta {domain_name} sudah kadaluarsa (outdated)"
                )],
                explanation: format!(
                    "Fakta {domain_name} pada profil persona telah kadaluarsa (outdated) dan perlu diperbarui."
                ),
            });
        }

        None
    }

    /// Ekstraksi nilai usia numerik dari fakta profil
    fn extract_age_value(fact: &FactSummary) -> Option<i64> {
        if let Some(num) = fact.value.as_i64() {
            return Some(num);
        }
        if let Ok(num) = fact.normalized_value.trim().parse::<i64>() {
            return Some(num);
        }

        // Periksa apakah format tanggal lahir YYYY-MM-DD
        let trimmed = fact.normalized_value.trim();
        if trimmed.len() >= 4 {
            let year_str = &trimmed[0..4];
            if let Ok(birth_year) = year_str.parse::<i64>() {
                if (1900..=2026).contains(&birth_year) {
                    let current_year = 2026; // Tahun baseline aplikasi
                    return Some(current_year - birth_year);
                }
            }
        }

        None
    }

    /// Normalisasi string gender menjadi 'male', 'female', atau string aslinya
    fn normalize_gender_str(s: &str) -> String {
        let trimmed = s.trim().to_lowercase();
        match trimmed.as_str() {
            "pria" | "laki-laki" | "cowok" | "male" | "l" => "male".to_string(),
            "wanita" | "perempuan" | "cewek" | "female" | "p" | "w" => "female".to_string(),
            _ => trimmed,
        }
    }

    /// Peringkat jenjang pendidikan untuk komparasi minimal
    fn education_level_rank(s: &str) -> i32 {
        let lower = s.to_lowercase();
        if lower.contains("s3") || lower.contains("doktor") {
            7
        } else if lower.contains("s2") || lower.contains("magister") || lower.contains("master") {
            6
        } else if lower.contains("s1") || lower.contains("sarjana") || lower.contains("d4") {
            5
        } else if lower.contains("d3")
            || lower.contains("diploma")
            || lower.contains("d1")
            || lower.contains("d2")
        {
            4
        } else if lower.contains("sma")
            || lower.contains("smk")
            || lower.contains("slta")
            || lower.contains("smu")
        {
            3
        } else if lower.contains("smp") || lower.contains("sltp") {
            2
        } else if lower.contains("sd") {
            1
        } else {
            0
        }
    }

    /// Evaluasi pencocokan lokasi termasuk kluster metropolitan Jabodetabek
    fn check_location_match(persona_loc: &str, expected_locations: &[String]) -> bool {
        if expected_locations.is_empty() {
            return true;
        }

        let is_jabodetabek_cluster = expected_locations
            .iter()
            .any(|loc| loc.contains("jabodetabek"));

        if is_jabodetabek_cluster {
            let jabo_cities = [
                "jakarta",
                "bogor",
                "depok",
                "tangerang",
                "bekasi",
                "tangsel",
                "tangerang selatan",
                "dki",
                "jabodetabek",
            ];
            if jabo_cities.iter().any(|city| persona_loc.contains(city)) {
                return true;
            }
        }

        // Pencocokan langsung atau substring
        expected_locations.iter().any(|expected| {
            persona_loc.contains(expected)
                || expected.contains(persona_loc)
                || (expected == "indonesia" && !persona_loc.is_empty())
        })
    }
}
