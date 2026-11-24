use crate::domain::opportunity::{
    CriterionOperator, CriterionType, OpportunityCriterion, OpportunityProvenance,
    OpportunityStatus, SurveyOpportunity,
};
use uuid::Uuid;

/// Hasil ekstraksi awal dari teks mentah peluang survei
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedOpportunity {
    pub title: Option<String>,
    pub platform_label: Option<String>,
    pub topic: Option<String>,
    pub description: Option<String>,
    pub criteria: Vec<ExtractedCriterion>,
}

/// Kriteria hasil ekstraksi teks yang dapat diverifikasi dan diedit
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedCriterion {
    pub criterion_text: String,
    pub raw_text: String,
    pub criterion_type: CriterionType,
    pub normalized_attribute: Option<String>,
    pub expected_value_json: Option<serde_json::Value>,
    pub operator: CriterionOperator,
    pub criterion_order: i64,
}

/// Layanan Ekstraksi Kriteria Survei Deterministik (SC-050)
///
/// Mengekstrak informasi peluang survei dan kriteria eksplisit secara objektif
/// dari teks yang terlihat tanpa mereka-reka kriteria atau menyimpulkan aturan tersembunyi.
pub struct CriteriaExtractorService;

impl CriteriaExtractorService {
    /// Platform survei umum yang dapat dideteksi secara otomatis dari teks
    const KNOWN_PLATFORMS: &'static [&'static str] = &[
        "YouGov",
        "Milieu",
        "Milieu Insight",
        "Toluna",
        "LifePoints",
        "Nusaresearch",
        "Ipsos",
        "Ipsos iSay",
        "Jakpat",
        "AttaPoll",
        "Google Opinion Rewards",
        "SurveyMonkey",
        "Qualtrics",
        "Cint",
        "Dynata",
        "PopSurvey",
        "SurveyOn",
    ];

    /// Topik umum survei
    const KNOWN_TOPICS: &'static [(&'static str, &'static str)] = &[
        ("makanan", "Makanan & Kuliner"),
        ("minuman", "Minuman & Konsumsi"),
        ("kopi", "Minuman (Kopi)"),
        ("otomotif", "Otomotif & Kendaraan"),
        ("mobil", "Otomotif (Mobil)"),
        ("motor", "Otomotif (Sepeda Motor)"),
        ("keuangan", "Keuangan & Finansial"),
        ("bank", "Perbankan"),
        ("gadget", "Teknologi & Gadget"),
        ("smartphone", "Smartphone & Telekomunikasi"),
        ("elektronik", "Elektronik Konsumen"),
        ("kesehatan", "Kesehatan & Farmasi"),
        ("gaya hidup", "Gaya Hidup & Hiburan"),
        ("belanja online", "E-Commerce & Belanja"),
        ("e-commerce", "E-Commerce & Belanja"),
        ("pendidikan", "Pendidikan"),
        ("pekerjaan", "Ketenagakerjaan"),
    ];

    /// Mengekstrak peluang survei dan kriteria dari teks mentah
    pub fn extract(raw_text: &str, platform_hint: Option<&str>) -> ExtractedOpportunity {
        let lines: Vec<&str> = raw_text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        // 1. Deteksi Platform
        let platform_label = platform_hint
            .filter(|h| !h.trim().is_empty())
            .map(|h| h.trim().to_string())
            .or_else(|| Self::detect_platform(raw_text));

        // 2. Deteksi Judul & Topik
        let title = Self::detect_title(&lines);
        let topic = Self::detect_topic(raw_text);

        // 3. Deteksi Deskripsi / Konteks
        let description = Self::extract_description(&lines, title.as_deref());

        // 4. Ekstraksi Kriteria Eksplisit
        let criteria = Self::extract_criteria(&lines);

        ExtractedOpportunity {
            title,
            platform_label,
            topic,
            description,
            criteria,
        }
    }

    /// Mengubah teks mentah menjadi entitas domain lengkap siap simpan
    pub fn build_opportunity_with_criteria(
        raw_text: &str,
        platform_hint: Option<&str>,
        provenance: OpportunityProvenance,
        source_capture_id: Option<String>,
        source_ocr_id: Option<String>,
    ) -> (SurveyOpportunity, Vec<OpportunityCriterion>) {
        let extracted = Self::extract(raw_text, platform_hint);
        let now = chrono::Utc::now().to_rfc3339();
        let opp_id = Uuid::new_v4().to_string();

        let opportunity = SurveyOpportunity {
            id: opp_id.clone(),
            title: extracted.title,
            platform_label: extracted.platform_label,
            topic: extracted.topic,
            description: extracted.description,
            raw_text: Some(raw_text.to_string()),
            provenance,
            status: OpportunityStatus::CriteriaExtracted,
            source_capture_id,
            source_ocr_id,
            captured_at: now.clone(),
            created_at: now.clone(),
            updated_at: now,
        };

        let criteria: Vec<OpportunityCriterion> = extracted
            .criteria
            .into_iter()
            .map(|c| OpportunityCriterion {
                id: Uuid::new_v4().to_string(),
                opportunity_id: opp_id.clone(),
                criterion_text: c.criterion_text,
                raw_text: Some(c.raw_text),
                criterion_type: c.criterion_type,
                normalized_attribute: c.normalized_attribute,
                expected_value_json: c.expected_value_json,
                operator: c.operator,
                criterion_order: c.criterion_order,
            })
            .collect();

        (opportunity, criteria)
    }

    /// Deteksi platform survei
    fn detect_platform(raw_text: &str) -> Option<String> {
        let text_lower = raw_text.to_lowercase();
        for &platform in Self::KNOWN_PLATFORMS {
            if text_lower.contains(&platform.to_lowercase()) {
                return Some(platform.to_string());
            }
        }
        None
    }

    /// Deteksi judul peluang survei
    fn detect_title(lines: &[&str]) -> Option<String> {
        for line in lines {
            let lower = line.to_lowercase();
            // Prefix eksplisit judul
            if let Some(_stripped) = lower.strip_prefix("judul:") {
                let title = line[6..].trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
            if let Some(_stripped) = lower.strip_prefix("survei:") {
                let title = line[7..].trim();
                if !title.is_empty() {
                    return Some(format!("Survei: {}", title));
                }
            }
            if let Some(_stripped) = lower.strip_prefix("survey:") {
                let title = line[7..].trim();
                if !title.is_empty() {
                    return Some(format!("Survey: {}", title));
                }
            }
            // Baris berawalan kata kunci survei
            if (lower.starts_with("survei ")
                || lower.starts_with("survey ")
                || lower.starts_with("riset "))
                && line.len() <= 120
            {
                return Some(line.to_string());
            }
        }

        // Fallback: baris pertama jika pendek dan bukan header kriteria
        if let Some(first) = lines.first() {
            let lower = first.to_lowercase();
            if !Self::is_criteria_header(&lower)
                && first.len() <= 80
                && !Self::is_bullet_line(first)
            {
                return Some(first.to_string());
            }
        }

        None
    }

    /// Deteksi topik survei berdasarkan kata kunci umum
    fn detect_topic(raw_text: &str) -> Option<String> {
        let lower = raw_text.to_lowercase();
        for &(keyword, label) in Self::KNOWN_TOPICS {
            if lower.contains(keyword) {
                return Some(label.to_string());
            }
        }
        None
    }

    /// Mengekstrak ringkasan deskripsi jika ada kalimat penjelasan
    fn extract_description(lines: &[&str], title: Option<&str>) -> Option<String> {
        let mut desc_parts = Vec::new();
        for line in lines {
            let lower = line.to_lowercase();
            if Self::is_criteria_header(&lower) || Self::is_bullet_line(line) {
                continue;
            }
            if let Some(t) = title {
                if *line == t {
                    continue;
                }
            }
            if lower.starts_with("deskripsi:") {
                let d = line[10..].trim();
                if !d.is_empty() {
                    return Some(d.to_string());
                }
            }
            if line.len() > 20 && !lower.starts_with("platform:") {
                desc_parts.push(*line);
            }
        }

        if desc_parts.is_empty() {
            None
        } else {
            Some(desc_parts.join(" "))
        }
    }

    /// Mengekstrak seluruh kriteria eksplisit dari teks
    fn extract_criteria(lines: &[&str]) -> Vec<ExtractedCriterion> {
        let mut criteria = Vec::new();
        let mut inside_criteria_section = false;
        let mut order = 0;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let lower = trimmed.to_lowercase();

            // Cek apakah baris merupakan header bagian kriteria
            if Self::is_criteria_header(&lower) {
                inside_criteria_section = true;
                // Jika header memiliki isi inline seperti "Kriteria: Pria usia 20-30", proses bagian inlinenya
                if let Some(idx) = trimmed.find(':') {
                    let inline_content = trimmed[idx + 1..].trim();
                    if !inline_content.is_empty() {
                        let parsed_subcriteria = Self::parse_clause_or_line(inline_content, order);
                        for c in parsed_subcriteria {
                            criteria.push(c);
                            order += 1;
                        }
                    }
                }
                continue;
            }

            // Cek jika bagian kriteria berakhir karena ada header lain
            if inside_criteria_section
                && (lower.starts_with("reward:")
                    || lower.starts_with("poin:")
                    || lower.starts_with("kontak:"))
            {
                inside_criteria_section = false;
                continue;
            }

            let is_bullet = Self::is_bullet_line(trimmed);
            let has_criterion_keywords = Self::contains_criterion_keywords(&lower);

            // Ekstrak jika berada di dalam seksi kriteria, berupa bullet, atau mengandung kata kunci kriteria kuat
            if inside_criteria_section || is_bullet || has_criterion_keywords {
                let cleaned_line = Self::strip_bullet_marker(trimmed);
                if cleaned_line.is_empty() {
                    continue;
                }

                let parsed_items = Self::parse_clause_or_line(cleaned_line, order);
                for c in parsed_items {
                    criteria.push(c);
                    order += 1;
                }
            }
        }

        criteria
    }

    /// Memeriksa apakah baris adalah header kriteria
    fn is_criteria_header(lower: &str) -> bool {
        lower.starts_with("kriteria")
            || lower.starts_with("syarat")
            || lower.starts_with("persyaratan")
            || lower.starts_with("target:")
            || lower.starts_with("target responden")
            || lower.starts_with("profil responden")
            || lower.starts_with("eligibility")
            || lower.starts_with("dicari:")
            || lower.starts_with("requirements")
    }

    /// Memeriksa apakah baris berformat bullet
    fn is_bullet_line(line: &str) -> bool {
        let t = line.trim_start();
        t.starts_with('-')
            || t.starts_with('*')
            || t.starts_with('•')
            || t.starts_with("[ ]")
            || (t.chars().next().is_some_and(|c| c.is_ascii_digit())
                && (t.contains(". ") || t.contains(") ")))
    }

    /// Menghilangkan bullet marker dari awal baris
    fn strip_bullet_marker(line: &str) -> &str {
        let mut t = line.trim();
        if t.starts_with('-') || t.starts_with('*') || t.starts_with('•') {
            t = t[1..].trim();
        } else if t.starts_with("[ ]") {
            t = t[3..].trim();
        } else if let Some(idx) = t.find(". ") {
            if t[..idx].chars().all(|c| c.is_ascii_digit()) {
                t = t[idx + 2..].trim();
            }
        } else if let Some(idx) = t.find(") ") {
            if t[..idx]
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_alphabetic())
            {
                t = t[idx + 2..].trim();
            }
        }
        t
    }

    /// Memeriksa apakah teks baris memiliki kata kunci kriteria yang kuat
    fn contains_criterion_keywords(lower: &str) -> bool {
        lower.contains("usia")
            || lower.contains("umur")
            || lower.contains("gender")
            || lower.contains("jenis kelamin")
            || lower.contains("domisili")
            || lower.contains("pekerjaan")
            || lower.contains("pengguna")
            || lower.contains("mengkonsumsi")
            || lower.contains("pendidikan")
            || lower.contains("penghasilan")
            || lower.contains("pengeluaran")
            || lower.contains("bukan pekerja")
            || lower.contains("tidak bekerja di")
    }

    /// Mem-parsing klausa atau baris menjadi satu atau lebih kriteria terstruktur
    fn parse_clause_or_line(text: &str, current_order: i64) -> Vec<ExtractedCriterion> {
        let mut results = Vec::new();
        let lower = text.to_lowercase();
        let mut order = current_order;

        // 1. Ekstraksi Usia (Age)
        if lower.contains("usia") || lower.contains("umur") || lower.contains("age") {
            if let Some(c) = Self::parse_age_criterion(text, &lower, order) {
                results.push(c);
                order += 1;
            }
        }

        // 2. Ekstraksi Gender
        if lower.contains("gender")
            || lower.contains("jenis kelamin")
            || lower.contains("pria")
            || lower.contains("wanita")
            || lower.contains("laki-laki")
            || lower.contains("perempuan")
        {
            if let Some(c) = Self::parse_gender_criterion(text, &lower, order) {
                results.push(c);
                order += 1;
            }
        }

        // 3. Ekstraksi Domisili / Lokasi
        if lower.contains("domisili")
            || lower.contains("lokasi")
            || lower.contains("tinggal di")
            || lower.contains("jabodetabek")
            || lower.contains("jakarta")
            || lower.contains("surabaya")
            || lower.contains("bandung")
        {
            if let Some(c) = Self::parse_location_criterion(text, &lower, order) {
                results.push(c);
                order += 1;
            }
        }

        // 4. Ekstraksi Pekerjaan / Larangan Industri
        if lower.contains("pekerjaan")
            || lower.contains("profesi")
            || lower.contains("karyawan")
            || lower.contains("bukan pekerja")
            || lower.contains("tidak bekerja di")
        {
            if let Some(c) = Self::parse_occupation_criterion(text, &lower, order) {
                results.push(c);
                order += 1;
            }
        }

        // 5. Ekstraksi Penggunaan Produk / Kepemilikan
        if lower.contains("pengguna")
            || lower.contains("menggunakan")
            || lower.contains("mengkonsumsi")
            || lower.contains("memiliki")
            || lower.contains("pemilik")
        {
            if let Some(c) = Self::parse_product_usage_criterion(text, &lower, order) {
                results.push(c);
                order += 1;
            }
        }

        // 6. Ekstraksi Pendidikan
        if lower.contains("pendidikan")
            || lower.contains("lulusan")
            || lower.contains("sarjana")
            || lower.contains("s1")
            || lower.contains("diploma")
        {
            if let Some(c) = Self::parse_education_criterion(text, &lower, order) {
                results.push(c);
                order += 1;
            }
        }

        // 7. Ekstraksi Finansial / Pengeluaran / Pendapatan
        if lower.contains("penghasilan")
            || lower.contains("pengeluaran")
            || lower.contains("pendapatan")
            || lower.contains("gaji")
        {
            if let Some(c) = Self::parse_income_criterion(text, &lower, order) {
                results.push(c);
                order += 1;
            }
        }

        // 8. Ekstraksi Status Pernikahan
        if lower.contains("menikah")
            || lower.contains("lajang")
            || lower.contains("single")
            || lower.contains("berkeluarga")
        {
            if let Some(c) = Self::parse_marital_criterion(text, &lower, order) {
                results.push(c);
            }
        }

        // 9. Fallback: Kriteria Umum (General) jika tidak ada pola kategori khusus yang cocok
        if results.is_empty() {
            results.push(ExtractedCriterion {
                criterion_text: text.to_string(),
                raw_text: text.to_string(),
                criterion_type: CriterionType::General,
                normalized_attribute: None,
                expected_value_json: Some(serde_json::json!({ "requirement": text })),
                operator: CriterionOperator::Equals,
                criterion_order: current_order,
            });
        }

        results
    }

    /// Ekstraksi spesifik kriteria usia
    fn parse_age_criterion(text: &str, lower: &str, order: i64) -> Option<ExtractedCriterion> {
        let numbers: Vec<i64> = extract_integers(lower);

        let (operator, value_json, display) = if numbers.len() >= 2 {
            let min = numbers[0];
            let max = numbers[1];
            (
                CriterionOperator::Range,
                serde_json::json!({ "min": min, "max": max }),
                format!("Usia {min} - {max} tahun"),
            )
        } else if numbers.len() == 1 {
            let n = numbers[0];
            if lower.contains("minimal")
                || lower.contains("min")
                || lower.contains("di atas")
                || lower.contains(">=")
            {
                (
                    CriterionOperator::GreaterThanOrEqual,
                    serde_json::json!({ "min": n }),
                    format!("Usia minimal {n} tahun"),
                )
            } else if lower.contains("maksimal")
                || lower.contains("maks")
                || lower.contains("di bawah")
                || lower.contains("<=")
            {
                (
                    CriterionOperator::LessThanOrEqual,
                    serde_json::json!({ "max": n }),
                    format!("Usia maksimal {n} tahun"),
                )
            } else {
                (
                    CriterionOperator::Equals,
                    serde_json::json!({ "age": n }),
                    format!("Usia {n} tahun"),
                )
            }
        } else {
            (
                CriterionOperator::Equals,
                serde_json::json!({ "raw": text }),
                text.to_string(),
            )
        };

        Some(ExtractedCriterion {
            criterion_text: display,
            raw_text: text.to_string(),
            criterion_type: CriterionType::Age,
            normalized_attribute: Some("age".to_string()),
            expected_value_json: Some(value_json),
            operator,
            criterion_order: order,
        })
    }

    /// Ekstraksi spesifik kriteria gender
    fn parse_gender_criterion(text: &str, lower: &str, order: i64) -> Option<ExtractedCriterion> {
        let has_pria =
            lower.contains("pria") || lower.contains("laki-laki") || lower.contains("male");
        let has_wanita =
            lower.contains("wanita") || lower.contains("perempuan") || lower.contains("female");

        let (operator, val, display) = if has_pria && has_wanita {
            (
                CriterionOperator::In,
                serde_json::json!({ "allowed": ["pria", "wanita"] }),
                "Jenis Kelamin: Pria atau Wanita".to_string(),
            )
        } else if has_pria {
            (
                CriterionOperator::Equals,
                serde_json::json!({ "allowed": ["pria"] }),
                "Jenis Kelamin: Pria".to_string(),
            )
        } else if has_wanita {
            (
                CriterionOperator::Equals,
                serde_json::json!({ "allowed": ["wanita"] }),
                "Jenis Kelamin: Wanita".to_string(),
            )
        } else {
            (
                CriterionOperator::Equals,
                serde_json::json!({ "raw": text }),
                text.to_string(),
            )
        };

        Some(ExtractedCriterion {
            criterion_text: display,
            raw_text: text.to_string(),
            criterion_type: CriterionType::Gender,
            normalized_attribute: Some("gender".to_string()),
            expected_value_json: Some(val),
            operator,
            criterion_order: order,
        })
    }

    /// Ekstraksi spesifik kriteria lokasi / domisili
    fn parse_location_criterion(text: &str, lower: &str, order: i64) -> Option<ExtractedCriterion> {
        let mut locations = Vec::new();
        let target_cities = &[
            ("jabodetabek", "Jabodetabek"),
            ("jakarta", "Jakarta"),
            ("bogor", "Bogor"),
            ("depok", "Depok"),
            ("tangerang", "Tangerang"),
            ("bekasi", "Bekasi"),
            ("bandung", "Bandung"),
            ("surabaya", "Surabaya"),
            ("semarang", "Semarang"),
            ("yogyakarta", "Yogyakarta"),
            ("jogja", "Yogyakarta"),
            ("medan", "Medan"),
            ("makassar", "Makassar"),
            ("denpasar", "Denpasar"),
            ("bali", "Bali"),
            ("palembang", "Palembang"),
            ("indonesia", "Indonesia"),
        ];

        for &(city_key, city_name) in target_cities {
            if lower.contains(city_key) && !locations.contains(&city_name.to_string()) {
                locations.push(city_name.to_string());
            }
        }

        let display = if !locations.is_empty() {
            format!("Domisili: {}", locations.join(", "))
        } else {
            text.to_string()
        };

        Some(ExtractedCriterion {
            criterion_text: display,
            raw_text: text.to_string(),
            criterion_type: CriterionType::Location,
            normalized_attribute: Some("location".to_string()),
            expected_value_json: Some(serde_json::json!({ "locations": locations })),
            operator: CriterionOperator::In,
            criterion_order: order,
        })
    }

    /// Ekstraksi spesifik kriteria pekerjaan
    fn parse_occupation_criterion(
        text: &str,
        lower: &str,
        order: i64,
    ) -> Option<ExtractedCriterion> {
        let is_exclusion = lower.contains("bukan")
            || lower.contains("tidak bekerja")
            || lower.contains("exclude")
            || lower.contains("dilarang");

        let (operator, display) = if is_exclusion {
            (CriterionOperator::NotEquals, format!("Eksklusi: {}", text))
        } else {
            (CriterionOperator::In, text.to_string())
        };

        Some(ExtractedCriterion {
            criterion_text: display,
            raw_text: text.to_string(),
            criterion_type: CriterionType::Occupation,
            normalized_attribute: Some("occupation".to_string()),
            expected_value_json: Some(serde_json::json!({
                "raw": text,
                "is_exclusion": is_exclusion,
            })),
            operator,
            criterion_order: order,
        })
    }

    /// Ekstraksi spesifik kriteria kepemilikan/penggunaan produk
    fn parse_product_usage_criterion(
        text: &str,
        _lower: &str,
        order: i64,
    ) -> Option<ExtractedCriterion> {
        Some(ExtractedCriterion {
            criterion_text: text.to_string(),
            raw_text: text.to_string(),
            criterion_type: CriterionType::ProductUsage,
            normalized_attribute: Some("product_usage".to_string()),
            expected_value_json: Some(serde_json::json!({ "requirement": text })),
            operator: CriterionOperator::Contains,
            criterion_order: order,
        })
    }

    /// Ekstraksi spesifik kriteria pendidikan
    fn parse_education_criterion(
        text: &str,
        lower: &str,
        order: i64,
    ) -> Option<ExtractedCriterion> {
        let (min_level, op) = if lower.contains("s1") || lower.contains("sarjana") {
            ("S1", CriterionOperator::GreaterThanOrEqual)
        } else if lower.contains("d3") || lower.contains("diploma") {
            ("D3", CriterionOperator::GreaterThanOrEqual)
        } else if lower.contains("sma") || lower.contains("smk") {
            ("SMA/SMK", CriterionOperator::GreaterThanOrEqual)
        } else {
            ("Pendidikan Umum", CriterionOperator::Equals)
        };

        Some(ExtractedCriterion {
            criterion_text: text.to_string(),
            raw_text: text.to_string(),
            criterion_type: CriterionType::Education,
            normalized_attribute: Some("education".to_string()),
            expected_value_json: Some(serde_json::json!({ "min_level": min_level })),
            operator: op,
            criterion_order: order,
        })
    }

    /// Ekstraksi spesifik kriteria finansial / pengeluaran
    fn parse_income_criterion(text: &str, _lower: &str, order: i64) -> Option<ExtractedCriterion> {
        Some(ExtractedCriterion {
            criterion_text: text.to_string(),
            raw_text: text.to_string(),
            criterion_type: CriterionType::Income,
            normalized_attribute: Some("income".to_string()),
            expected_value_json: Some(serde_json::json!({ "requirement": text })),
            operator: CriterionOperator::GreaterThanOrEqual,
            criterion_order: order,
        })
    }

    /// Ekstraksi spesifik kriteria status pernikahan
    fn parse_marital_criterion(text: &str, lower: &str, order: i64) -> Option<ExtractedCriterion> {
        let status = if lower.contains("belum menikah")
            || lower.contains("lajang")
            || lower.contains("single")
        {
            "lajang"
        } else if lower.contains("menikah")
            || lower.contains("kawin")
            || lower.contains("berkeluarga")
        {
            "menikah"
        } else {
            "lainnya"
        };

        Some(ExtractedCriterion {
            criterion_text: text.to_string(),
            raw_text: text.to_string(),
            criterion_type: CriterionType::MaritalStatus,
            normalized_attribute: Some("marital_status".to_string()),
            expected_value_json: Some(serde_json::json!({ "status": status })),
            operator: CriterionOperator::Equals,
            criterion_order: order,
        })
    }
}

/// Helper untuk mengekstrak bilangan bulat dari string
fn extract_integers(s: &str) -> Vec<i64> {
    let mut numbers = Vec::new();
    let mut current = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            current.push(c);
        } else if !current.is_empty() {
            if let Ok(n) = current.parse::<i64>() {
                // Batas wajar usia 1..120
                if (1..=120).contains(&n) {
                    numbers.push(n);
                }
            }
            current.clear();
        }
    }

    if !current.is_empty() {
        if let Ok(n) = current.parse::<i64>() {
            if (1..=120).contains(&n) {
                numbers.push(n);
            }
        }
    }

    numbers
}
