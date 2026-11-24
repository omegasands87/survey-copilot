use std::collections::HashSet;

use crate::domain::classifier::{
    AmbiguityReason, AmbiguityReport, AmbiguityType, ClassificationConfidence,
    ClassifyQuestionInput, ConstraintProvenance, ExtractedConstraints, NumericRangeConstraint,
    QuestionCategory, QuestionClassificationResult, ScaleAnchor, ScaleConstraint, ScaleDirection,
    SelectionMode, TextLengthConstraint, TimeframeConstraint, TimeframeType, UnitConstraint,
};
use crate::domain::survey::{AnswerFormat, QuestionType};

pub const CLASSIFIER_VERSION: &str = "1.0.0";

/// Layanan Parser dan Classifier Deterministik Pertanyaan Survei (SC-031)
/// Menggunakan parsing deterministik murni tanpa dependensi eksternal
pub struct QuestionClassifierService;

impl QuestionClassifierService {
    /// Mengklasifikasi pertanyaan survei dari DTO input bertipe
    pub fn classify(input: &ClassifyQuestionInput) -> QuestionClassificationResult {
        Self::classify_raw(
            &input.question_text,
            input.instructions.as_deref(),
            input.options.as_deref(),
        )
    }

    /// Mengklasifikasi pertanyaan dari komponen teks mentah tanpa mengubah teks asli
    pub fn classify_raw(
        raw_question: &str,
        raw_instructions: Option<&str>,
        raw_options: Option<&[String]>,
    ) -> QuestionClassificationResult {
        let normalized_text = Self::normalize_text(raw_question);
        let normalized_instructions = raw_instructions.map(Self::normalize_text);
        let normalized_options: Vec<String> = raw_options
            .unwrap_or(&[])
            .iter()
            .map(|s| Self::normalize_text(s))
            .filter(|s| !s.is_empty())
            .collect();

        // 1. Deteksi Kategori (Multi-label)
        let categories = Self::detect_categories(
            &normalized_text,
            normalized_instructions.as_deref(),
            &normalized_options,
        );

        let primary_category = categories
            .first()
            .copied()
            .unwrap_or(QuestionCategory::OtherUnknown);

        // 2. Ekstraksi Batasan (Constraints)
        let constraints = Self::extract_constraints(
            &normalized_text,
            normalized_instructions.as_deref(),
            &normalized_options,
        );

        // 3. Tentukan QuestionType & AnswerFormat
        let (question_type, answer_format) = Self::infer_type_and_format(
            &categories,
            &constraints,
            &normalized_options,
            &normalized_text,
            normalized_instructions.as_deref(),
        );

        // 4. Deteksi Ambiguitas (Ambiguity Detection & AT-041)
        let ambiguity = Self::detect_ambiguity(
            raw_question,
            &normalized_text,
            normalized_instructions.as_deref(),
            &normalized_options,
            &categories,
            &constraints,
        );

        // 5. Hitung Tingkat Keyakinan (Confidence)
        let confidence =
            Self::determine_confidence(&categories, &constraints, &ambiguity, &normalized_options);

        QuestionClassificationResult {
            classifier_version: CLASSIFIER_VERSION.to_string(),
            normalized_text,
            categories,
            primary_category,
            confidence,
            question_type,
            answer_format,
            constraints,
            ambiguity,
        }
    }

    // ==========================================
    // NORMALISASI DETERMINISTIK
    // ==========================================

    /// Menormalkan teks tanpa memodifikasi teks sumber asli
    pub fn normalize_text(text: &str) -> String {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let mut result = String::with_capacity(trimmed.len());
        let mut last_was_whitespace = false;

        for c in trimmed.chars() {
            if c.is_whitespace() {
                if !last_was_whitespace {
                    result.push(' ');
                    last_was_whitespace = true;
                }
            } else {
                for lc in c.to_lowercase() {
                    result.push(lc);
                }
                last_was_whitespace = false;
            }
        }

        result
    }

    /// Mencocokkan kata kunci: jika keyword <= 3 karakter (singkatan), harus cocok sebagai kata utuh
    pub fn text_matches_keyword(text: &str, keyword: &str) -> bool {
        if keyword.len() <= 3 {
            text.split(|c: char| !c.is_alphanumeric())
                .any(|token| token.eq_ignore_ascii_case(keyword))
        } else {
            text.contains(keyword)
        }
    }

    // ==========================================
    // DETEKSI KATEGORI (MULTI-LABEL)
    // ==========================================

    fn detect_categories(
        question: &str,
        instructions: Option<&str>,
        options: &[String],
    ) -> Vec<QuestionCategory> {
        let mut matched = Vec::new();
        let mut seen = HashSet::new();

        let q_lower = question.to_lowercase();
        let instr_lower = instructions.map(|s| s.to_lowercase()).unwrap_or_default();
        let combined_text = format!("{q_lower} {instr_lower}");

        let mut add_cat = |cat: QuestionCategory| {
            if seen.insert(cat) {
                matched.push(cat);
            }
        };

        // Format checks
        if Self::is_matrix_grid(&q_lower, &instr_lower, options) {
            add_cat(QuestionCategory::MatrixGrid);
        }

        if Self::is_satisfaction_rating(&combined_text, options) {
            add_cat(QuestionCategory::SatisfactionRating);
        }

        if Self::is_date(&combined_text, options) {
            add_cat(QuestionCategory::Date);
        }

        if Self::is_demographic(&combined_text, options) {
            add_cat(QuestionCategory::Demographic);
        }

        if Self::is_brand_product_usage(&combined_text, options) {
            add_cat(QuestionCategory::BrandProductUsage);
        }

        if Self::is_employment_education(&combined_text, options) {
            add_cat(QuestionCategory::EmploymentEducation);
        }

        if Self::is_financial_economic(&combined_text, options) {
            add_cat(QuestionCategory::FinancialEconomic);
        }

        if Self::is_consumption_habits(&combined_text, options) {
            add_cat(QuestionCategory::ConsumptionHabits);
        }

        if Self::is_asset_ownership(&combined_text, options) {
            add_cat(QuestionCategory::AssetOwnership);
        }

        if Self::is_experience_behavior(&combined_text, options) {
            add_cat(QuestionCategory::ExperienceBehavior);
        }

        if Self::is_preference_opinion(&combined_text, options) {
            add_cat(QuestionCategory::PreferenceOpinion);
        }

        if Self::is_numeric(&combined_text, options) {
            add_cat(QuestionCategory::Numeric);
        }

        if !options.is_empty() {
            add_cat(QuestionCategory::MultipleChoice);
        } else if Self::is_free_text(&combined_text) {
            add_cat(QuestionCategory::FreeText);
        }

        if matched.is_empty() {
            matched.push(QuestionCategory::OtherUnknown);
        }

        matched
    }

    // ==========================================
    // ATURAN DETEKSI KATEGORI SPESIFIK
    // ==========================================

    fn is_demographic(text: &str, _options: &[String]) -> bool {
        static DEMO_WORDS: &[&str] = &[
            "usia",
            "umur",
            "jenis kelamin",
            "gender",
            "tempat lahir",
            "tanggal lahir",
            "domisili",
            "tempat tinggal",
            "kota tempat tinggal",
            "provinsi",
            "status pernikahan",
            "status perkawinan",
            "jumlah anggota keluarga",
            "tanggungan keluarga",
            "kepala keluarga",
            "suku",
            "ras",
            "age",
            "date of birth",
            "gender",
            "marital status",
            "household size",
            "how old are you",
            "where do you live",
            "city of residence",
        ];
        DEMO_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_employment_education(text: &str, _options: &[String]) -> bool {
        static EMP_WORDS: &[&str] = &[
            "pekerjaan",
            "profesi",
            "jabatan",
            "bidang kerja",
            "status kepegawaian",
            "karyawan",
            "pegawai",
            "wirausaha",
            "wiraswasta",
            "pns",
            "asn",
            "tni",
            "polri",
            "buruh",
            "freelance",
            "freelancer",
            "menganggur",
            "ibu rumah tangga",
            "pendidikan terakhir",
            "jenjang pendidikan",
            "tingkat pendidikan",
            "lulusan",
            "sd",
            "smp",
            "sma",
            "smk",
            "diploma",
            "d3",
            "sarjana",
            "s1",
            "magister",
            "s2",
            "doktor",
            "s3",
            "occupation",
            "employment",
            "job title",
            "work status",
            "education level",
            "highest education",
            "bachelor",
            "master degree",
            "high school",
            "current employment",
        ];
        EMP_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_financial_economic(text: &str, _options: &[String]) -> bool {
        static FIN_WORDS: &[&str] = &[
            "pendapatan",
            "penghasilan",
            "gaji",
            "upah",
            "pengeluaran",
            "omset",
            "omzet",
            "tabungan",
            "rekening",
            "rekening bank",
            "kartu kredit",
            "pinjaman",
            "kredit",
            "investasi",
            "kelas ekonomi",
            "status ekonomi",
            "ses",
            "rupiah per bulan",
            "idr per bulan",
            "income",
            "salary",
            "revenue",
            "household income",
            "monthly spending",
            "expenses",
            "financial status",
            "credit card",
            "bank account",
            "investment",
            "earnings",
        ];
        FIN_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_consumption_habits(text: &str, _options: &[String]) -> bool {
        static CONSUME_WORDS: &[&str] = &[
            "kebiasaan konsumsi",
            "mengonsumsi",
            "konsumsi",
            "sering meminum",
            "sering memakan",
            "belanja mingguan",
            "belanja bulanan",
            "belanja online",
            "rokok",
            "kopi",
            "minuman",
            "makanan",
            "camilan",
            "snack",
            "susu",
            "berlangganan",
            "paket data",
            "pulsa",
            "layanan streaming",
            "consume",
            "consumption",
            "groceries",
            "grocery",
            "grocery shopping",
            "online shopping",
            "daily habit",
            "weekly habit",
            "how often do you drink",
            "how often do you eat",
            "subscription",
        ];
        CONSUME_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_asset_ownership(text: &str, _options: &[String]) -> bool {
        static ASSET_WORDS: &[&str] = &[
            "kepemilikan",
            "memiliki",
            "apakah anda memiliki",
            "kendaraan yang anda miliki",
            "rumah pribadi",
            "sepeda motor",
            "mobil",
            "laptop",
            "komputer",
            "smartphone",
            "hp",
            "ac",
            "kulkas",
            "mesin cuci",
            "televisi",
            "smart tv",
            "properti",
            "tanah",
            "apartemen",
            "do you own",
            "ownership",
            "vehicles owned",
            "household items",
            "own or rent",
            "personal property",
        ];
        ASSET_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_brand_product_usage(text: &str, _options: &[String]) -> bool {
        static BRAND_WORDS: &[&str] = &[
            "merek",
            "brand",
            "produk",
            "pernah anda beli",
            "pernah anda gunakan",
            "sedang anda gunakan",
            "merek yang paling sering",
            "brand awareness",
            "paling anda sukai dari merek",
            "top of mind",
            "which brand",
            "brands have you heard",
            "brands do you use",
            "product used",
            "have you purchased",
        ];
        BRAND_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_experience_behavior(text: &str, _options: &[String]) -> bool {
        static EXP_WORDS: &[&str] = &[
            "pengalaman",
            "perilaku",
            "pernahkah anda",
            "apakah anda pernah",
            "seberapa sering",
            "frekuensi",
            "kegiatan",
            "aktivitas",
            "riwayat perjalanan",
            "naik pesawat",
            "menginap di hotel",
            "belanja online",
            "berolahraga",
            "liburan",
            "berkunjung ke",
            "have you ever",
            "experience",
            "behavior",
            "frequency",
            "how often do you",
            "how often",
            "how many times have you",
            "how many times did you",
            "how many times",
            "travel history",
            "visited",
        ];
        EXP_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_preference_opinion(text: &str, _options: &[String]) -> bool {
        static PREF_WORDS: &[&str] = &[
            "menurut anda",
            "pendapat anda",
            "opini",
            "pandangan",
            "preferensi",
            "lebih menyukai",
            "lebih memilih",
            "setujukah anda",
            "apakah anda setuju",
            "alasan anda",
            "mengapa anda memilih",
            "tanggapan anda",
            "in your opinion",
            "what do you think",
            "preference",
            "do you prefer",
            "do you agree",
            "why did you choose",
            "reason for",
        ];
        PREF_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_satisfaction_rating(text: &str, options: &[String]) -> bool {
        static SAT_WORDS: &[&str] = &[
            "seberapa puas",
            "tingkat kepuasan",
            "kepuasan",
            "rating",
            "nilai kualitas",
            "skala penilaian",
            "net promoter score",
            "nps",
            "merekomendasikan kepada",
            "seberapa besar kemungkinan anda merekomendasikan",
            "skala 1 sampai",
            "skala 1 hingga",
            "bintang 1 sampai",
            "how satisfied",
            "satisfaction",
            "rate your",
            "on a scale of",
            "how likely are you to recommend",
            "nps score",
        ];

        if SAT_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
        {
            return true;
        }

        // Cek jika opsi memiliki pola Likert scale eksplisit
        let has_likert = options.iter().any(|opt| {
            let o = opt.to_lowercase();
            o.contains("sangat puas")
                || o.contains("sangat setuju")
                || o.contains("tidak puas")
                || o.contains("tidak setuju")
                || o.contains("strongly agree")
                || o.contains("strongly disagree")
                || o.contains("very satisfied")
        });

        has_likert
    }

    fn is_matrix_grid(question: &str, instructions: &str, _options: &[String]) -> bool {
        static MATRIX_WORDS: &[&str] = &[
            "matriks",
            "tabel berikut",
            "matrix",
            "grid",
            "untuk setiap pernyataan di bawah",
            "pada setiap baris",
            "for each row",
            "for each statement",
            "rate each of the following",
        ];
        MATRIX_WORDS
            .iter()
            .any(|&w| question.contains(w) || instructions.contains(w))
    }

    fn is_date(text: &str, _options: &[String]) -> bool {
        static DATE_WORDS: &[&str] = &[
            "tanggal lahir",
            "tanggal",
            "kapan anda",
            "tahun berapa",
            "bulan apa",
            "dd/mm/yyyy",
            "format tanggal",
            "date of",
            "what date",
            "what year",
            "which month",
        ];
        DATE_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_numeric(text: &str, _options: &[String]) -> bool {
        static NUM_WORDS: &[&str] = &[
            "masukkan angka",
            "isikan angka",
            "dalam bentuk angka",
            "tuliskan dalam angka",
            "dalam angka",
            "berapa jumlah",
            "berapa nominal",
            "dalam nominal",
            "nominal rupiah",
            "berapa usia",
            "berapa umur",
            "berapa banyak",
            "berapa kali",
            "enter a number",
            "numeric value",
            "amount in numbers",
            "how many (in numbers)",
            "how many",
            "how much",
        ];
        NUM_WORDS
            .iter()
            .any(|&w| Self::text_matches_keyword(text, w))
    }

    fn is_free_text(text: &str) -> bool {
        static TEXT_WORDS: &[&str] = &[
            "tuliskan",
            "jelaskan",
            "ceritakan",
            "sebutkan",
            "apa komentar anda",
            "uraikan",
            "tuliskan alasan",
            "describe",
            "explain",
            "please specify",
            "write down",
            "comments",
            "in your own words",
        ];
        TEXT_WORDS.iter().any(|&w| text.contains(w))
    }

    // ==========================================
    // EKSTRAKSI BATASAN (CONSTRAINTS)
    // ==========================================

    fn extract_constraints(
        question: &str,
        instructions: Option<&str>,
        options: &[String],
    ) -> ExtractedConstraints {
        let q_lower = question.to_lowercase();
        let instr_lower = instructions.map(|s| s.to_lowercase()).unwrap_or_default();
        let combined = format!("{q_lower} {instr_lower}");

        let units = Self::extract_units(&q_lower, &instr_lower, options);
        let timeframe = Self::extract_timeframe(&q_lower, &instr_lower, options);
        let (min_sel, max_sel, mode) =
            Self::extract_selection_limits(&q_lower, &instr_lower, options);
        let is_required = Self::extract_is_required(question, instructions);
        let expected_format =
            Self::determine_expected_format(&mode, options, &q_lower, &instr_lower);
        let numeric_range = Self::extract_numeric_range(&combined, options);
        let scale = Self::extract_scale(&combined, options);
        let special_instructions = Self::extract_special_instructions(instructions, question);
        let text_length = Self::extract_text_length(&combined);

        ExtractedConstraints {
            units,
            timeframe,
            min_selections: min_sel,
            max_selections: max_sel,
            selection_mode: mode,
            is_required,
            expected_format,
            numeric_range,
            scale,
            special_instructions,
            text_length,
        }
    }

    fn extract_units(
        question: &str,
        instructions: &str,
        options: &[String],
    ) -> Option<UnitConstraint> {
        static UNIT_PATTERNS: &[(&str, &str)] = &[
            ("rupiah", "Rupiah"),
            ("rp", "Rupiah"),
            ("idr", "IDR"),
            ("ribu", "Ribu"),
            ("juta", "Juta"),
            ("miliar", "Miliar"),
            ("tahun", "Tahun"),
            ("thn", "Tahun"),
            ("bulan", "Bulan"),
            ("bln", "Bulan"),
            ("minggu", "Minggu"),
            ("hari", "Hari"),
            ("jam", "Jam"),
            ("menit", "Menit"),
            ("kali", "Kali"),
            ("kg", "Kilogram"),
            ("gram", "Gram"),
            ("liter", "Liter"),
            ("persen", "Persen"),
            ("%", "Persen"),
            ("orang", "Orang"),
            ("unit", "Unit"),
            ("lembar", "Lembar"),
            ("porsi", "Porsi"),
            ("gelas", "Gelas"),
            ("botol", "Botol"),
            ("batang", "Batang"),
            ("bungkus", "Bungkus"),
            ("pack", "Pack"),
            ("dollars", "USD"),
            ("usd", "USD"),
            ("$", "USD"),
            ("years", "Years"),
            ("months", "Months"),
            ("days", "Days"),
            ("hours", "Hours"),
            ("times", "Times"),
        ];

        let combined = format!("{question} {instructions}");
        let words: Vec<&str> = combined
            .split(|c: char| !c.is_alphanumeric() && c != '$' && c != '%')
            .filter(|w| !w.is_empty())
            .collect();

        for &(token, name) in UNIT_PATTERNS {
            if words.iter().any(|&w| w.eq_ignore_ascii_case(token)) {
                return Some(UnitConstraint {
                    unit_name: name.to_string(),
                    raw_text: token.to_string(),
                    provenance: ConstraintProvenance::Explicit,
                });
            }
        }

        if !options.is_empty() {
            for &(token, name) in UNIT_PATTERNS {
                let count = options
                    .iter()
                    .filter(|opt| {
                        opt.split(|c: char| !c.is_alphanumeric() && c != '$' && c != '%')
                            .any(|w| w.eq_ignore_ascii_case(token))
                    })
                    .count();
                if count >= (options.len() / 2).max(1) {
                    return Some(UnitConstraint {
                        unit_name: name.to_string(),
                        raw_text: token.to_string(),
                        provenance: ConstraintProvenance::Inferred,
                    });
                }
            }
        }

        None
    }

    fn extract_timeframe(
        question: &str,
        instructions: &str,
        options: &[String],
    ) -> Option<TimeframeConstraint> {
        let text = format!("{question} {instructions}");
        let words: Vec<&str> = text.split_whitespace().collect();

        // 1. Cek frasa umum
        if text.contains("30 hari terakhir") || text.contains("past 30 days") {
            return Some(TimeframeConstraint {
                raw_text: "30 hari terakhir".to_string(),
                timeframe_type: TimeframeType::PastNDays,
                count: Some(30),
                provenance: ConstraintProvenance::Explicit,
            });
        }

        if text.contains("12 bulan terakhir") || text.contains("past 12 months") {
            return Some(TimeframeConstraint {
                raw_text: "12 bulan terakhir".to_string(),
                timeframe_type: TimeframeType::PastNMonths,
                count: Some(12),
                provenance: ConstraintProvenance::Explicit,
            });
        }

        if text.contains("sebulan terakhir")
            || text.contains("1 bulan terakhir")
            || text.contains("last month")
        {
            return Some(TimeframeConstraint {
                raw_text: "sebulan terakhir".to_string(),
                timeframe_type: TimeframeType::PastNMonths,
                count: Some(1),
                provenance: ConstraintProvenance::Explicit,
            });
        }

        if text.contains("seminggu terakhir")
            || text.contains("1 minggu terakhir")
            || text.contains("last week")
        {
            return Some(TimeframeConstraint {
                raw_text: "seminggu terakhir".to_string(),
                timeframe_type: TimeframeType::PastNWeeks,
                count: Some(1),
                provenance: ConstraintProvenance::Explicit,
            });
        }

        if text.contains("setahun terakhir")
            || text.contains("1 tahun terakhir")
            || text.contains("last year")
        {
            return Some(TimeframeConstraint {
                raw_text: "setahun terakhir".to_string(),
                timeframe_type: TimeframeType::PastNYears,
                count: Some(1),
                provenance: ConstraintProvenance::Explicit,
            });
        }

        if text.contains("setiap hari")
            || text.contains("daily")
            || text.contains("setiap minggu")
            || text.contains("weekly")
            || text.contains("setiap bulan")
            || text.contains("monthly")
            || text.contains("rutin")
        {
            return Some(TimeframeConstraint {
                raw_text: "rutin/berulang".to_string(),
                timeframe_type: TimeframeType::Recurring,
                count: None,
                provenance: ConstraintProvenance::Explicit,
            });
        }

        // Cek pola token: N (hari|minggu|bulan|tahun) (terakhir|yang lalu)
        for i in 0..words.len() {
            if let Ok(num) = words[i].parse::<u32>() {
                if i + 1 < words.len() {
                    let unit_word = words[i + 1].trim_matches(|c: char| !c.is_alphanumeric());
                    let is_recent = if i + 2 < words.len() {
                        let next_word = words[i + 2].trim_matches(|c: char| !c.is_alphanumeric());
                        next_word == "terakhir" || next_word == "lalu" || next_word == "past"
                    } else {
                        false
                    };

                    if is_recent
                        || words
                            .iter()
                            .any(|&w| w == "terakhir" || w == "past" || w == "last")
                    {
                        match unit_word {
                            "hari" | "day" | "days" => {
                                return Some(TimeframeConstraint {
                                    raw_text: format!("{num} {unit_word} terakhir"),
                                    timeframe_type: TimeframeType::PastNDays,
                                    count: Some(num),
                                    provenance: ConstraintProvenance::Explicit,
                                });
                            }
                            "minggu" | "pekan" | "week" | "weeks" => {
                                return Some(TimeframeConstraint {
                                    raw_text: format!("{num} {unit_word} terakhir"),
                                    timeframe_type: TimeframeType::PastNWeeks,
                                    count: Some(num),
                                    provenance: ConstraintProvenance::Explicit,
                                });
                            }
                            "bulan" | "month" | "months" => {
                                return Some(TimeframeConstraint {
                                    raw_text: format!("{num} {unit_word} terakhir"),
                                    timeframe_type: TimeframeType::PastNMonths,
                                    count: Some(num),
                                    provenance: ConstraintProvenance::Explicit,
                                });
                            }
                            "tahun" | "year" | "years" => {
                                return Some(TimeframeConstraint {
                                    raw_text: format!("{num} {unit_word} terakhir"),
                                    timeframe_type: TimeframeType::PastNYears,
                                    count: Some(num),
                                    provenance: ConstraintProvenance::Explicit,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Cek opsi untuk inferensi timeframe
        for opt in options {
            let o_lower = opt.to_lowercase();
            if o_lower.contains("terakhir") || o_lower.contains("past") || o_lower.contains("last")
            {
                if o_lower.contains("hari") || o_lower.contains("day") {
                    return Some(TimeframeConstraint {
                        raw_text: opt.clone(),
                        timeframe_type: TimeframeType::PastNDays,
                        count: None,
                        provenance: ConstraintProvenance::Inferred,
                    });
                }
                if o_lower.contains("bulan") || o_lower.contains("month") {
                    return Some(TimeframeConstraint {
                        raw_text: opt.clone(),
                        timeframe_type: TimeframeType::PastNMonths,
                        count: None,
                        provenance: ConstraintProvenance::Inferred,
                    });
                }
                if o_lower.contains("tahun") || o_lower.contains("year") {
                    return Some(TimeframeConstraint {
                        raw_text: opt.clone(),
                        timeframe_type: TimeframeType::PastNYears,
                        count: None,
                        provenance: ConstraintProvenance::Inferred,
                    });
                }
            }
        }

        None
    }

    fn extract_selection_limits(
        question: &str,
        instructions: &str,
        options: &[String],
    ) -> (Option<u32>, Option<u32>, SelectionMode) {
        let text = format!("{question} {instructions}");
        let words: Vec<&str> = text.split_whitespace().collect();

        // 1. Single selection explicit
        static SINGLE_PATTERNS: &[&str] = &[
            "pilih salah satu",
            "pilih hanya satu",
            "pilih hanya 1",
            "pilih 1 jawaban",
            "pilih satu jawaban",
            "select only one",
            "select one answer",
            "choose one",
            "single choice",
        ];
        if SINGLE_PATTERNS.iter().any(|&p| text.contains(p)) {
            return (Some(1), Some(1), SelectionMode::Single);
        }

        // 2. Both Minimal and Maksimal (Range of selections)
        let mut min_val_found = None;
        let mut max_val_found = None;

        for i in 0..words.len() {
            let w = words[i];
            if (w == "minimal" || w == "minimum" || w == "setidaknya") && i + 1 < words.len() {
                let next = words[i + 1].trim_matches(|c: char| !c.is_ascii_digit());
                if let Ok(n) = next.parse::<u32>() {
                    if n > 0 {
                        min_val_found = Some(n);
                    }
                }
            } else if (w == "maksimal" || w == "maximum" || w == "hingga" || w == "sampai")
                && i + 1 < words.len()
            {
                let next = words[i + 1].trim_matches(|c: char| !c.is_ascii_digit());
                if let Ok(n) = next.parse::<u32>() {
                    if n > 0 {
                        max_val_found = Some(n);
                    }
                }
            }
        }

        if let (Some(min_n), Some(max_n)) = (min_val_found, max_val_found) {
            return (Some(min_n), Some(max_n), SelectionMode::Multiple);
        }

        // 3. Exact N selections
        for i in 0..words.len() {
            if (words[i] == "pilih" || words[i] == "select") && i + 1 < words.len() {
                let mut target_idx = i + 1;
                if words[target_idx] == "tepat" || words[target_idx] == "exactly" {
                    target_idx += 1;
                }
                if target_idx < words.len() {
                    if let Ok(n) = words[target_idx].parse::<u32>() {
                        if target_idx + 1 < words.len()
                            && (words[target_idx + 1].starts_with("jawaban")
                                || words[target_idx + 1].starts_with("pilihan")
                                || words[target_idx + 1].starts_with("answer"))
                        {
                            return (Some(n), Some(n), SelectionMode::Exact(n));
                        }
                    }
                }
            }
        }

        // 4. At Most N (Maksimal N)
        if let Some(max_n) = max_val_found {
            return (Some(1), Some(max_n), SelectionMode::AtMost(max_n));
        }

        // 5. At Least N (Minimal N)
        if let Some(min_n) = min_val_found {
            let opt_len = options.len() as u32;
            let max_val = if opt_len > 0 { Some(opt_len) } else { None };
            return (Some(min_n), max_val, SelectionMode::AtLeast(min_n));
        }

        // 5. Multiple selection without exact count
        static MULTI_PATTERNS: &[&str] = &[
            "pilih semua yang sesuai",
            "pilih semua yang berlaku",
            "dapat memilih lebih dari satu",
            "boleh memilih lebih dari satu",
            "select all that apply",
            "choose all that apply",
            "multiple answers allowed",
        ];
        if MULTI_PATTERNS.iter().any(|&p| text.contains(p)) {
            let opt_len = options.len() as u32;
            let max_val = if opt_len > 0 { Some(opt_len) } else { None };
            return (Some(1), max_val, SelectionMode::Multiple);
        }

        // 6. Inferensi default: jika ada opsi jawaban, default ke Single choice
        if !options.is_empty() {
            (Some(1), Some(1), SelectionMode::Single)
        } else {
            (None, None, SelectionMode::Unspecified)
        }
    }

    fn extract_is_required(question: &str, instructions: Option<&str>) -> Option<bool> {
        let text = format!("{question} {}", instructions.unwrap_or_default());
        let lower = text.to_lowercase();

        if lower.contains("(opsional)")
            || lower.contains("opsional")
            || lower.contains("(optional)")
            || lower.contains("optional")
        {
            return Some(false);
        }

        if text.contains('*')
            || lower.contains("(wajib)")
            || lower.contains("wajib diisi")
            || lower.contains("harus diisi")
            || lower.contains("required")
            || lower.contains("mandatory")
        {
            return Some(true);
        }

        None
    }

    fn determine_expected_format(
        mode: &SelectionMode,
        options: &[String],
        question: &str,
        instructions: &str,
    ) -> AnswerFormat {
        if Self::is_matrix_grid(question, instructions, options) {
            return AnswerFormat::Matrix;
        }

        if Self::is_satisfaction_rating(&format!("{question} {instructions}"), options) {
            return AnswerFormat::RatingScale;
        }

        if !options.is_empty() {
            match mode {
                SelectionMode::Multiple
                | SelectionMode::AtLeast(_)
                | SelectionMode::AtMost(_)
                | SelectionMode::Exact(_) => AnswerFormat::MultipleSelection,
                _ => AnswerFormat::SingleChoice,
            }
        } else if Self::is_numeric(question, options) {
            AnswerFormat::Number
        } else if Self::is_date(question, options) {
            AnswerFormat::Unknown
        } else {
            AnswerFormat::Text
        }
    }

    fn extract_numeric_range(text: &str, _options: &[String]) -> Option<NumericRangeConstraint> {
        let words: Vec<&str> = text.split_whitespace().collect();

        // Cari pola "antara X sampai Y" atau "between X and Y" atau "X - Y"
        for i in 0..words.len() {
            let w = words[i];
            if (w == "antara" || w == "between" || w == "rentang") && i + 3 < words.len() {
                let min_str = words[i + 1].replace(',', ".");
                let separator = words[i + 2];
                let max_str = words[i + 3].replace(',', ".");

                if separator == "sampai"
                    || separator == "hingga"
                    || separator == "dan"
                    || separator == "and"
                    || separator == "to"
                    || separator == "-"
                {
                    if let (Ok(min), Ok(max)) = (min_str.parse::<f64>(), max_str.parse::<f64>()) {
                        return Some(NumericRangeConstraint {
                            min: Some(min),
                            max: Some(max),
                            step: None,
                            provenance: ConstraintProvenance::Explicit,
                        });
                    }
                }
            } else if (w == "minimal" || w == "minimum" || w == "at least") && i + 1 < words.len() {
                let clean = words[i + 1]
                    .trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != ',');
                if let Ok(min) = clean.replace(',', ".").parse::<f64>() {
                    return Some(NumericRangeConstraint {
                        min: Some(min),
                        max: None,
                        step: None,
                        provenance: ConstraintProvenance::Explicit,
                    });
                }
            } else if (w == "maksimal" || w == "maximum" || w == "at most") && i + 1 < words.len() {
                let clean = words[i + 1]
                    .trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != ',');
                if let Ok(max) = clean.replace(',', ".").parse::<f64>() {
                    return Some(NumericRangeConstraint {
                        min: None,
                        max: Some(max),
                        step: None,
                        provenance: ConstraintProvenance::Explicit,
                    });
                }
            }
        }

        None
    }

    fn extract_scale(text: &str, options: &[String]) -> Option<ScaleConstraint> {
        if !Self::is_satisfaction_rating(text, options) && !options.is_empty() {
            let parsed_nums: Vec<i32> = options
                .iter()
                .filter_map(|opt| opt.trim().parse::<i32>().ok())
                .collect();
            if parsed_nums.len() == options.len() && parsed_nums.len() >= 3 {
                let min_val = *parsed_nums.iter().min().unwrap_or(&1);
                let max_val = *parsed_nums.iter().max().unwrap_or(&5);
                return Some(ScaleConstraint {
                    min_scale: min_val,
                    max_scale: max_val,
                    direction: ScaleDirection::LowToHigh,
                    anchors: vec![
                        ScaleAnchor {
                            value: min_val,
                            label: format!("{min_val}"),
                        },
                        ScaleAnchor {
                            value: max_val,
                            label: format!("{max_val}"),
                        },
                    ],
                    provenance: ConstraintProvenance::Inferred,
                });
            }
        }

        // Cek pola teks skala: "dari X ... hingga Y ..." atau "skala X sampai Y"
        if let Some(scale) = Self::parse_scale_from_text(text) {
            return Some(scale);
        }

        // Likert anchors dari opsi
        if !options.is_empty() {
            let anchors: Vec<ScaleAnchor> = options
                .iter()
                .enumerate()
                .map(|(i, opt)| ScaleAnchor {
                    value: (i + 1) as i32,
                    label: opt.clone(),
                })
                .collect();

            let min_scale = 1;
            let max_scale = options.len() as i32;

            let first_lower = options[0].to_lowercase();
            let last_lower = options[options.len() - 1].to_lowercase();

            let direction = if first_lower.contains("sangat tidak")
                || first_lower.contains("tidak puas")
                || first_lower.contains("strongly disagree")
            {
                ScaleDirection::LowToHigh
            } else if last_lower.contains("sangat tidak")
                || last_lower.contains("tidak puas")
                || last_lower.contains("strongly disagree")
            {
                ScaleDirection::HighToLow
            } else {
                ScaleDirection::Unspecified
            };

            return Some(ScaleConstraint {
                min_scale,
                max_scale,
                direction,
                anchors,
                provenance: ConstraintProvenance::Inferred,
            });
        }

        None
    }

    fn parse_scale_from_text(text: &str) -> Option<ScaleConstraint> {
        let t_lower = text.to_lowercase();
        let sep_candidates = [" hingga ", " sampai ", " to ", " s/d "];
        for sep in sep_candidates {
            if let Some(pos) = t_lower.find(sep) {
                let left_part = &text[..pos];
                let right_part = &text[pos + sep.len()..];

                let left_words: Vec<&str> = left_part.split_whitespace().collect();
                let mut min_num: Option<i32> = None;
                let mut min_idx = 0;
                for (idx, w) in left_words.iter().enumerate().rev() {
                    let clean = w.trim_matches(|c: char| !c.is_ascii_digit());
                    if let Ok(n) = clean.parse::<i32>() {
                        min_num = Some(n);
                        min_idx = idx;
                        break;
                    }
                }

                let right_words: Vec<&str> = right_part.split_whitespace().collect();
                let mut max_num: Option<i32> = None;
                let mut max_idx = 0;
                for (idx, w) in right_words.iter().enumerate() {
                    let clean = w.trim_matches(|c: char| !c.is_ascii_digit());
                    if let Ok(n) = clean.parse::<i32>() {
                        max_num = Some(n);
                        max_idx = idx;
                        break;
                    }
                }

                if let (Some(min_s), Some(max_s)) = (min_num, max_num) {
                    if min_s < max_s {
                        let mut anchors = Vec::new();

                        let left_snippet = left_words[min_idx..].join(" ");
                        if let (Some(start), Some(end)) =
                            (left_snippet.find('('), left_snippet.find(')'))
                        {
                            if start < end {
                                anchors.push(ScaleAnchor {
                                    value: min_s,
                                    label: left_snippet[start + 1..end].trim().to_string(),
                                });
                            }
                        }

                        let right_snippet = right_words[max_idx..].join(" ");
                        if let (Some(start), Some(end)) =
                            (right_snippet.find('('), right_snippet.find(')'))
                        {
                            if start < end {
                                anchors.push(ScaleAnchor {
                                    value: max_s,
                                    label: right_snippet[start + 1..end].trim().to_string(),
                                });
                            }
                        }

                        let direction = if let Some(a0) = anchors.first() {
                            let l = a0.label.to_lowercase();
                            if l.contains("tidak") || l.contains("disagree") || l.contains("poor") {
                                ScaleDirection::LowToHigh
                            } else {
                                ScaleDirection::HighToLow
                            }
                        } else {
                            ScaleDirection::LowToHigh
                        };

                        return Some(ScaleConstraint {
                            min_scale: min_s,
                            max_scale: max_s,
                            direction,
                            anchors,
                            provenance: ConstraintProvenance::Explicit,
                        });
                    }
                }
            }
        }
        None
    }

    fn extract_special_instructions(instructions: Option<&str>, question: &str) -> Vec<String> {
        let mut list = Vec::new();

        if let Some(instr) = instructions {
            let trimmed = instr.trim();
            if !trimmed.is_empty() {
                list.push(trimmed.to_string());
            }
        }

        // Ekstraksi instruksi khusus di dalam tanda kurung
        let mut chars = question.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '(' {
                let mut inner = String::new();
                for inner_c in chars.by_ref() {
                    if inner_c == ')' {
                        break;
                    }
                    inner.push(inner_c);
                }
                let trimmed_inner = inner.trim();
                let lower = trimmed_inner.to_lowercase();
                let is_instruction = lower.contains("pilih")
                    || lower.contains("select")
                    || lower.contains("wajib")
                    || lower.contains("maksimal")
                    || lower.contains("minimal")
                    || lower.contains("hanya")
                    || lower.contains("jawaban");

                if is_instruction && !list.iter().any(|s| s.eq_ignore_ascii_case(trimmed_inner)) {
                    list.push(trimmed_inner.to_string());
                }
            }
        }

        list
    }

    fn extract_text_length(text: &str) -> Option<TextLengthConstraint> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut max_chars = None;
        let mut max_words = None;

        for i in 0..words.len() {
            let w = words[i];
            if (w == "maksimal" || w == "max") && i + 2 < words.len() {
                let count_str = words[i + 1].trim_matches(|c: char| !c.is_ascii_digit());
                let unit = words[i + 2].trim_matches(|c: char| !c.is_alphabetic());

                if let Ok(n) = count_str.parse::<u32>() {
                    if unit.starts_with("karakter") || unit.starts_with("character") {
                        max_chars = Some(n);
                    } else if unit.starts_with("kata") || unit.starts_with("word") {
                        max_words = Some(n);
                    }
                }
            }
        }

        if max_chars.is_some() || max_words.is_some() {
            Some(TextLengthConstraint {
                min_chars: None,
                max_chars,
                max_words,
                provenance: ConstraintProvenance::Explicit,
            })
        } else {
            None
        }
    }

    fn infer_type_and_format(
        categories: &[QuestionCategory],
        constraints: &ExtractedConstraints,
        options: &[String],
        question: &str,
        instructions: Option<&str>,
    ) -> (QuestionType, AnswerFormat) {
        if categories.contains(&QuestionCategory::MatrixGrid) {
            return (QuestionType::Matrix, AnswerFormat::Matrix);
        }

        if categories.contains(&QuestionCategory::SatisfactionRating) {
            return (QuestionType::Scale, AnswerFormat::RatingScale);
        }

        if categories.contains(&QuestionCategory::Date) {
            return (QuestionType::Date, AnswerFormat::Unknown);
        }

        if !options.is_empty() {
            if options.len() == 2 {
                let opt0 = options[0].to_lowercase();
                let opt1 = options[1].to_lowercase();
                let is_bool = (opt0.starts_with("ya")
                    || opt0.starts_with("yes")
                    || opt0.starts_with("benar")
                    || opt0.starts_with("true"))
                    && (opt1.starts_with("tidak")
                        || opt1.starts_with("no")
                        || opt1.starts_with("salah")
                        || opt1.starts_with("false"));
                let is_bool_rev = (opt1.starts_with("ya")
                    || opt1.starts_with("yes")
                    || opt1.starts_with("benar")
                    || opt1.starts_with("true"))
                    && (opt0.starts_with("tidak")
                        || opt0.starts_with("no")
                        || opt0.starts_with("salah")
                        || opt0.starts_with("false"));
                if is_bool || is_bool_rev {
                    return (QuestionType::SingleChoice, AnswerFormat::Boolean);
                }
            }

            match constraints.selection_mode {
                SelectionMode::Multiple
                | SelectionMode::AtLeast(_)
                | SelectionMode::AtMost(_)
                | SelectionMode::Exact(_) => (
                    QuestionType::MultipleChoice,
                    AnswerFormat::MultipleSelection,
                ),
                _ => (QuestionType::SingleChoice, AnswerFormat::SingleChoice),
            }
        } else {
            let combined =
                format!("{question} {}", instructions.unwrap_or_default()).to_lowercase();
            if categories.contains(&QuestionCategory::Numeric)
                || combined.contains("dalam angka")
                || combined.contains("tuliskan dalam angka")
                || combined.contains("masukkan angka")
                || combined.contains("dalam nominal")
                || combined.contains("nominal rupiah")
            {
                (QuestionType::Numeric, AnswerFormat::Number)
            } else {
                (QuestionType::FreeText, AnswerFormat::Text)
            }
        }
    }

    // ==========================================
    // DETEKSI AMBIGUITAS (AMBIGUITY DETECTION - AT-041)
    // ==========================================

    fn detect_ambiguity(
        raw_question: &str,
        normalized_question: &str,
        instructions: Option<&str>,
        options: &[String],
        categories: &[QuestionCategory],
        constraints: &ExtractedConstraints,
    ) -> AmbiguityReport {
        let mut reasons = Vec::new();
        let q_lower = normalized_question.to_lowercase();
        let instr_lower = instructions.map(|s| s.to_lowercase()).unwrap_or_default();
        let combined = format!("{q_lower} {instr_lower}");

        // 1. Incomplete Question
        if raw_question.trim().len() < 8 || raw_question.trim().ends_with("...") {
            reasons.push(AmbiguityReason {
                ambiguity_type: AmbiguityType::IncompleteQuestion,
                explanation: "Teks pertanyaan tampak belum lengkap atau terpotong".to_string(),
                context_snippet: Some(raw_question.to_string()),
            });
        }

        // 2. Conflicting Instructions
        let has_single_phrase = combined.contains("pilih hanya 1")
            || combined.contains("pilih 1 jawaban")
            || combined.contains("pilih 1 jawaban saja")
            || combined.contains("pilih salah satu")
            || combined.contains("select only one");

        let has_multi_phrase = combined.contains("pilih semua yang sesuai")
            || combined.contains("pilih semua yang berlaku")
            || combined.contains("pilih semua")
            || combined.contains("pilih lebih dari satu")
            || combined.contains("select all that apply");

        if has_single_phrase && has_multi_phrase {
            reasons.push(AmbiguityReason {
                ambiguity_type: AmbiguityType::ConflictingInstructions,
                explanation: "Instruksi pertanyaan saling bertentangan antara pilihan tunggal (single) dan pilihan ganda (multiple)".to_string(),
                context_snippet: Some(combined.clone()),
            });
        }

        if let Some(min_s) = constraints.min_selections {
            if !options.is_empty() && (options.len() as u32) < min_s {
                reasons.push(AmbiguityReason {
                    ambiguity_type: AmbiguityType::ConflictingInstructions,
                    explanation: format!(
                        "Instruksi mensyaratkan minimal {min_s} pilihan, namun hanya tersedia {} opsi jawaban",
                        options.len()
                    ),
                    context_snippet: Some(format!("Min required: {min_s}, options count: {}", options.len())),
                });
            }
        }

        // 3. AT-041: Missing Timeframe
        let is_behavioral = categories.contains(&QuestionCategory::ExperienceBehavior)
            || categories.contains(&QuestionCategory::ConsumptionHabits);

        let asks_frequency = combined.contains("berapa kali")
            || combined.contains("seberapa sering")
            || combined.contains("how many times")
            || combined.contains("how often")
            || combined.contains("frekuensi");

        if (is_behavioral || asks_frequency) && constraints.timeframe.is_none() {
            reasons.push(AmbiguityReason {
                ambiguity_type: AmbiguityType::MissingTimeframe,
                explanation: "Pertanyaan menanyakan frekuensi atau perilaku tanpa menyebutkan periode referensi waktu yang jelas (misal: 30 hari terakhir, sebulan terakhir, atau setahun terakhir)".to_string(),
                context_snippet: Some(normalized_question.to_string()),
            });
        }

        // 4. AT-041: Missing Unit
        let is_financial = categories.contains(&QuestionCategory::FinancialEconomic);
        let asks_amount = combined.contains("berapa penghasilan")
            || combined.contains("berapa pendapatan")
            || combined.contains("berapa gaji")
            || combined.contains("berapa pengeluaran")
            || combined.contains("berapa biaya")
            || combined.contains("berapa nominal");

        if (is_financial || asks_amount) && constraints.units.is_none() && options.is_empty() {
            reasons.push(AmbiguityReason {
                ambiguity_type: AmbiguityType::MissingUnit,
                explanation: "Pertanyaan meminta nilai kuantitatif atau finansial namun tidak menyebutkan satuan pengukuran atau periode (misal: Rupiah per bulan atau Rupiah per tahun)".to_string(),
                context_snippet: Some(normalized_question.to_string()),
            });
        }

        // 5. Mismatched Options
        if !options.is_empty() {
            let is_yes_no = options.len() == 2
                && options
                    .iter()
                    .any(|o| o.eq_ignore_ascii_case("ya") || o.eq_ignore_ascii_case("yes"))
                && options
                    .iter()
                    .any(|o| o.eq_ignore_ascii_case("tidak") || o.eq_ignore_ascii_case("no"));

            let asks_age_or_salary = combined.contains("berapa usia")
                || combined.contains("berapa umur")
                || combined.contains("berapa penghasilan");

            if is_yes_no && asks_age_or_salary {
                reasons.push(AmbiguityReason {
                    ambiguity_type: AmbiguityType::MismatchedOptions,
                    explanation: "Pilihan jawaban (Ya/Tidak) tidak sesuai dengan tipe pertanyaan yang menanyakan nilai kuantitas/umur".to_string(),
                    context_snippet: Some(format!("Options: {:?}", options)),
                });
            }
        }

        // 6. Unknown Category
        if categories.contains(&QuestionCategory::OtherUnknown) {
            reasons.push(AmbiguityReason {
                ambiguity_type: AmbiguityType::MultipleInterpretations,
                explanation: "Teks pertanyaan tidak cocok dengan kategori survei yang dikenali"
                    .to_string(),
                context_snippet: Some(normalized_question.to_string()),
            });
        }

        let is_ambiguous = !reasons.is_empty();
        let needs_clarification = is_ambiguous;

        let clarification_prompt = if needs_clarification {
            Some(Self::generate_clarification_prompt(&reasons))
        } else {
            None
        };

        AmbiguityReport {
            is_ambiguous,
            needs_clarification,
            reasons,
            clarification_prompt,
        }
    }

    fn generate_clarification_prompt(reasons: &[AmbiguityReason]) -> String {
        let mut prompts = Vec::new();
        for r in reasons {
            match r.ambiguity_type {
                AmbiguityType::MissingTimeframe => {
                    prompts.push("Mohon tentukan periode waktu yang dimaksud untuk pertanyaan ini (misalnya: dalam 30 hari terakhir, 6 bulan terakhir, atau 1 tahun terakhir).");
                }
                AmbiguityType::MissingUnit => {
                    prompts.push("Mohon tentukan satuan nilai atau mata uang yang diinginkan (misalnya: Rupiah per bulan atau Rupiah per tahun).");
                }
                AmbiguityType::ConflictingInstructions => {
                    prompts.push("Terdapat instruksi yang saling bertentangan pada pertanyaan ini. Apakah Anda ingin memilih satu jawaban atau beberapa jawaban?");
                }
                AmbiguityType::IncompleteQuestion => {
                    prompts.push("Pertanyaan tampak terpotong atau tidak lengkap. Silakan lengkapi teks pertanyaan survei.");
                }
                AmbiguityType::MismatchedOptions => {
                    prompts.push("Pilihan jawaban tidak sesuai dengan isi pertanyaan. Mohon periksa kembali opsi yang tersedia pada survei.");
                }
                AmbiguityType::MultipleInterpretations => {
                    prompts.push("Pertanyaan memiliki lebih dari satu penafsiran. Mohon klarifikasi maksud spesifik pertanyaan.");
                }
            }
        }

        prompts.join(" ")
    }

    fn determine_confidence(
        categories: &[QuestionCategory],
        _constraints: &ExtractedConstraints,
        ambiguity: &AmbiguityReport,
        _options: &[String],
    ) -> ClassificationConfidence {
        if ambiguity.is_ambiguous {
            return ClassificationConfidence::Uncertain;
        }

        if categories.is_empty()
            || (categories.len() == 1 && categories[0] == QuestionCategory::OtherUnknown)
        {
            return ClassificationConfidence::Uncertain;
        }

        if !categories.is_empty() && categories[0] != QuestionCategory::OtherUnknown {
            return ClassificationConfidence::High;
        }

        ClassificationConfidence::Medium
    }
}
