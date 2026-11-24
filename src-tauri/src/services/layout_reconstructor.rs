use crate::domain::capture::CaptureMode;
use crate::domain::error::AppError;
use crate::domain::ocr::{
    BlockRole, ExtractedSurveyQuestion, MatrixStructure, OcrLine, OcrRawResult, OcrTextBlock,
    ScaleStructure, UncertainRegion, UncertaintyReason,
};
use crate::domain::survey::{AnswerFormat, ConfirmationStatus, QuestionType};
use crate::services::question_classifier::QuestionClassifierService;

pub const LAYOUT_RECONSTRUCTOR_VERSION: &str = "1.0.0";

/// Layanan Rekonstruksi Tata Letak & Struktur Pertanyaan Survei dari OCR (SC-041)
pub struct LayoutReconstructorService;

impl LayoutReconstructorService {
    /// Merekonstruksi struktur pertanyaan survei dari hasil mentah OCR
    pub fn reconstruct(
        raw_result: &OcrRawResult,
        capture_id: Option<String>,
        session_id: Option<String>,
        persona_id: Option<String>,
        source_mode: CaptureMode,
        captured_at: Option<String>,
        application_label: Option<String>,
    ) -> Result<ExtractedSurveyQuestion, AppError> {
        let extracted_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let captured_at = captured_at.unwrap_or_else(|| extracted_at.clone());
        let extraction_id = uuid::Uuid::new_v4().to_string();

        let img_w = raw_result.image_width as f64;
        let img_h = raw_result.image_height as f64;

        // 1. Urutkan baris OCR secara spasial (atas ke bawah, toleransi baseline vertikal)
        let mut sorted_lines = raw_result.lines.clone();
        sorted_lines.sort_by(|a, b| {
            let y_diff = a.bounding_box.y - b.bounding_box.y;
            if y_diff.abs() > 8.0 {
                a.bounding_box
                    .y
                    .partial_cmp(&b.bounding_box.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a.bounding_box
                    .x
                    .partial_cmp(&b.bounding_box.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        // 2. Deteksi teks terpotong (Clipping), keyakinan rendah, dan ambiguitas (AT-052)
        let mut uncertain_regions = Vec::new();
        let mut warnings = Vec::new();
        let mut total_word_confidence = 0.0f32;
        let mut word_count = 0usize;

        for (line_idx, line) in sorted_lines.iter().enumerate() {
            for word in &line.words {
                total_word_confidence += word.confidence;
                word_count += 1;

                // Deteksi low confidence (< 0.70)
                if word.confidence < 0.70 {
                    uncertain_regions.push(UncertainRegion {
                        id: format!("unc-low-conf-{line_idx}-{}", word.text),
                        reason: UncertaintyReason::LowConfidence,
                        bounding_box: Some(word.bounding_box),
                        text: Some(word.text.clone()),
                        details: format!(
                            "Keyakinan OCR rendah ({:.1}%) pada kata '{}'.",
                            word.confidence * 100.0,
                            word.text
                        ),
                    });
                    warnings.push(format!(
                        "Kata '{}' memiliki keyakinan OCR rendah ({:.1}%). Harap periksa ejaan.",
                        word.text,
                        word.confidence * 100.0
                    ));
                }

                // Deteksi kemungkinan teks terpotong di tepi screenshot (Clipping)
                let touches_left = word.bounding_box.x <= 3.0;
                let touches_right = img_w > 0.0 && word.bounding_box.right() >= (img_w - 5.0);
                let touches_bottom = img_h > 0.0 && word.bounding_box.bottom() >= (img_h - 5.0);

                if touches_left || touches_right || touches_bottom {
                    let side = if touches_right {
                        "sisi kanan"
                    } else if touches_left {
                        "sisi kiri"
                    } else {
                        "sisi bawah"
                    };

                    uncertain_regions.push(UncertainRegion {
                        id: format!("unc-clipped-{line_idx}-{}", word.text),
                        reason: UncertaintyReason::LikelyClipped,
                        bounding_box: Some(word.bounding_box),
                        text: Some(word.text.clone()),
                        details: format!(
                            "Teks '{}' berada di tepi {} gambar dan kemungkinan terpotong.",
                            word.text, side
                        ),
                    });
                    warnings.push(format!(
                        "Teks di {} gambar kemungkinan terpotong pada kata '{}'.",
                        side, word.text
                    ));
                }
            }
        }

        // 3. Deteksi Unit yang terlihat (Currency, Waktu, Ukuran, dll.)
        let visible_units = Self::detect_units(&raw_result.raw_text);

        // 4. Deteksi Indikator Wajib (Required)
        let is_required = Self::detect_required_indicator(&raw_result.raw_text, &sorted_lines);

        // 5. Cek apakah ini tata letak Matriks / Tabel Kisi (Matrix / Grid)
        let matrix_opt = Self::try_parse_matrix(&sorted_lines, img_w);

        // 6. Cek apakah ini Skala Penilaian (Rating Scale)
        let scale_opt = if matrix_opt.is_none() {
            Self::try_parse_scale(&sorted_lines, &raw_result.raw_text)
        } else {
            None
        };

        // 7. Ekstraksi Pertanyaan, Instruksi, dan Opsi
        let mut prompt_parts = Vec::new();
        let mut instruction_parts = Vec::new();
        let mut options = Vec::new();
        let mut text_blocks = Vec::new();

        if let Some(ref matrix) = matrix_opt {
            // Pada format Matrix, baris prompt adalah teks sebelum baris kolom / tabel
            let mut prompt_done = false;
            for (idx, line) in sorted_lines.iter().enumerate() {
                let trimmed = line.text.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Jika baris ini merupakan salah satu row atau col matrix, lewati prompt
                let is_part_of_matrix = matrix
                    .columns
                    .iter()
                    .any(|c| c.contains(trimmed) || trimmed.contains(c))
                    || matrix
                        .rows
                        .iter()
                        .any(|r| r.contains(trimmed) || trimmed.contains(r));

                if is_part_of_matrix {
                    prompt_done = true;
                }

                if !prompt_done {
                    if Self::is_instruction_line(trimmed) {
                        instruction_parts.push(trimmed.to_string());
                        text_blocks.push(OcrTextBlock {
                            id: format!("block-inst-{idx}"),
                            text: trimmed.to_string(),
                            bounding_box: line.bounding_box,
                            confidence: line.confidence,
                            words: line.words.clone(),
                            role: BlockRole::Instruction,
                        });
                    } else {
                        prompt_parts.push(trimmed.to_string());
                        text_blocks.push(OcrTextBlock {
                            id: format!("block-prompt-{idx}"),
                            text: trimmed.to_string(),
                            bounding_box: line.bounding_box,
                            confidence: line.confidence,
                            words: line.words.clone(),
                            role: BlockRole::Prompt,
                        });
                    }
                }
            }
        } else if let Some(ref _scale) = scale_opt {
            // Pada format Scale, baris pertama adalah prompt, instruksi terpisah, dan label skala teridentifikasi
            for (idx, line) in sorted_lines.iter().enumerate() {
                let trimmed = line.text.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if idx == 0 || (prompt_parts.is_empty() && !Self::is_instruction_line(trimmed)) {
                    prompt_parts.push(trimmed.to_string());
                    text_blocks.push(OcrTextBlock {
                        id: format!("block-prompt-{idx}"),
                        text: trimmed.to_string(),
                        bounding_box: line.bounding_box,
                        confidence: line.confidence,
                        words: line.words.clone(),
                        role: BlockRole::Prompt,
                    });
                } else if Self::is_instruction_line(trimmed) {
                    instruction_parts.push(trimmed.to_string());
                    text_blocks.push(OcrTextBlock {
                        id: format!("block-inst-{idx}"),
                        text: trimmed.to_string(),
                        bounding_box: line.bounding_box,
                        confidence: line.confidence,
                        words: line.words.clone(),
                        role: BlockRole::Instruction,
                    });
                } else {
                    text_blocks.push(OcrTextBlock {
                        id: format!("block-scale-{idx}"),
                        text: trimmed.to_string(),
                        bounding_box: line.bounding_box,
                        confidence: line.confidence,
                        words: line.words.clone(),
                        role: BlockRole::ScaleLabel,
                    });
                }
            }
        } else {
            // Format Pilihan Ganda / Isian / Biasa
            let mut in_options = false;

            for (idx, line) in sorted_lines.iter().enumerate() {
                let trimmed = line.text.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if Self::is_instruction_line(trimmed) {
                    instruction_parts.push(trimmed.to_string());
                    text_blocks.push(OcrTextBlock {
                        id: format!("block-inst-{idx}"),
                        text: trimmed.to_string(),
                        bounding_box: line.bounding_box,
                        confidence: line.confidence,
                        words: line.words.clone(),
                        role: BlockRole::Instruction,
                    });
                } else if !prompt_parts.is_empty()
                    && (Self::is_option_line(trimmed)
                        || (in_options && Self::looks_like_option_continuation(trimmed)))
                {
                    in_options = true;
                    let cleaned_opt = Self::clean_option_text(trimmed);
                    if !cleaned_opt.is_empty() {
                        options.push(cleaned_opt);
                        text_blocks.push(OcrTextBlock {
                            id: format!("block-opt-{idx}"),
                            text: trimmed.to_string(),
                            bounding_box: line.bounding_box,
                            confidence: line.confidence,
                            words: line.words.clone(),
                            role: BlockRole::Option,
                        });
                    }
                } else if !in_options {
                    prompt_parts.push(trimmed.to_string());
                    text_blocks.push(OcrTextBlock {
                        id: format!("block-prompt-{idx}"),
                        text: trimmed.to_string(),
                        bounding_box: line.bounding_box,
                        confidence: line.confidence,
                        words: line.words.clone(),
                        role: BlockRole::Prompt,
                    });
                } else {
                    text_blocks.push(OcrTextBlock {
                        id: format!("block-misc-{idx}"),
                        text: trimmed.to_string(),
                        bounding_box: line.bounding_box,
                        confidence: line.confidence,
                        words: line.words.clone(),
                        role: BlockRole::Unknown,
                    });
                }
            }
        }

        // Teks pertanyaan utama yang bersih
        let raw_prompt = if !prompt_parts.is_empty() {
            prompt_parts.join(" ")
        } else if let Some(first_line) = sorted_lines.first() {
            first_line.text.trim().to_string()
        } else {
            raw_result.raw_text.trim().to_string()
        };

        let question_text_original = Self::clean_prompt_text(&raw_prompt);
        let instructions = if !instruction_parts.is_empty() {
            Some(instruction_parts.join(" "))
        } else {
            None
        };

        // 8. Integrasi dengan Question Classifier SC-031
        let classification = QuestionClassifierService::classify_raw(
            &question_text_original,
            instructions.as_deref(),
            if !options.is_empty() {
                Some(&options)
            } else {
                None
            },
        );

        // Tentukan QuestionType & AnswerFormat final dengan memadukan deteksi visual dan NLP
        let (question_type, answer_format) = if matrix_opt.is_some() {
            (QuestionType::Matrix, AnswerFormat::Matrix)
        } else if scale_opt.is_some() {
            (QuestionType::Scale, AnswerFormat::RatingScale)
        } else if !options.is_empty() {
            if Self::is_multi_select_instruction(instructions.as_deref().unwrap_or(""))
                || classification.answer_format == AnswerFormat::MultipleSelection
            {
                (
                    QuestionType::MultipleChoice,
                    AnswerFormat::MultipleSelection,
                )
            } else {
                (QuestionType::SingleChoice, AnswerFormat::SingleChoice)
            }
        } else {
            (classification.question_type, classification.answer_format)
        };

        // 9. Cek Ambiguitas Layout
        if let Some(ref m) = matrix_opt {
            if m.rows.is_empty() || m.columns.is_empty() {
                uncertain_regions.push(UncertainRegion {
                    id: "unc-matrix-unbalanced".to_string(),
                    reason: UncertaintyReason::AmbiguousLayout,
                    bounding_box: None,
                    text: None,
                    details: "Struktur tabel matriks memiliki baris atau kolom yang tidak terpetakan secara seimbang.".to_string(),
                });
                warnings.push("Tabel matriks terdeteksi dengan kolom atau baris yang ambigu. Harap periksa tabel.".to_string());
            }
        }

        // Hitung overall confidence
        let mut overall_confidence = if word_count > 0 {
            total_word_confidence / (word_count as f32)
        } else {
            0.85
        };

        // Berikan penalti jika ada teks terpotong atau layout ambigu
        if uncertain_regions
            .iter()
            .any(|u| u.reason == UncertaintyReason::LikelyClipped)
        {
            overall_confidence = (overall_confidence - 0.15).max(0.1);
        }
        if uncertain_regions
            .iter()
            .any(|u| u.reason == UncertaintyReason::AmbiguousLayout)
        {
            overall_confidence = (overall_confidence - 0.20).max(0.1);
        }

        Ok(ExtractedSurveyQuestion {
            extraction_id,
            capture_id,
            session_id,
            persona_id,
            source_mode,
            captured_at,
            extracted_at,
            ocr_engine: raw_result.engine.clone(),
            application_label,
            image_width: raw_result.image_width,
            image_height: raw_result.image_height,
            raw_ocr_text: raw_result.raw_text.clone(),
            question_text_original,
            question_text_corrected: None,
            instructions,
            answer_options: options,
            question_type,
            answer_format,
            scale: scale_opt,
            matrix: matrix_opt,
            visible_units,
            is_required: if is_required { Some(true) } else { None },
            text_blocks,
            uncertain_regions,
            warnings,
            overall_confidence,
            confirmation_status: ConfirmationStatus::Draft,
        })
    }

    // ==========================================
    // DETEKSI MATRIKS / GRID TABLE DETERMINISTIK
    // ==========================================

    fn try_parse_matrix(lines: &[OcrLine], img_w: f64) -> Option<MatrixStructure> {
        let full_text_lower: String = lines
            .iter()
            .map(|l| l.text.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
        let has_matrix_keyword = full_text_lower.contains("berikut:")
            || full_text_lower.contains("pernyataan berikut")
            || full_text_lower.contains("aspek berikut")
            || full_text_lower.contains("matrix")
            || full_text_lower.contains("tabel")
            || full_text_lower.contains("setiap pernyataan");

        // Cari baris header kolom
        // Ciri header kolom: sebuah baris yang memiliki beberapa opsi skala/evaluasi tersebar horizontal
        let known_scale_words = [
            "sangat tidak setuju",
            "tidak setuju",
            "netral",
            "setuju",
            "sangat setuju",
            "strongly disagree",
            "disagree",
            "neutral",
            "agree",
            "strongly agree",
            "sangat buruk",
            "buruk",
            "cukup",
            "baik",
            "sangat baik",
            "very poor",
            "poor",
            "fair",
            "good",
            "excellent",
            "tidak pernah",
            "jarang",
            "kadang-kadang",
            "sering",
            "selalu",
            "never",
            "rarely",
            "sometimes",
            "often",
            "always",
        ];

        let mut header_line_idx: Option<usize> = None;
        let mut columns = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            let line_lower = line.text.to_lowercase();
            let mut matched_headers = Vec::new();

            for kw in &known_scale_words {
                if line_lower.contains(kw) {
                    matched_headers.push(kw);
                }
            }

            // Jika sebuah baris memuat minimal 2 kata skala, atau baris memiliki kata-kata berkolom lebar
            if matched_headers.len() >= 2 {
                header_line_idx = Some(idx);
                // Ekstraksi kolom dari urutan kemunculan di baris
                // Pisahkan kolom berdasarkan kata kunci skala atau pembagian kata
                for kw in &known_scale_words {
                    if line_lower.contains(kw) {
                        matched_headers.retain(|&h| h != kw);
                        matched_headers.push(kw);
                    }
                }
                break;
            }
        }

        // Alternatif deteksi baris kolom dari baris yang memuat "1 2 3 4 5" atau banyak kata dengan x berbeda jauh
        if header_line_idx.is_none() {
            for (idx, line) in lines.iter().enumerate() {
                if idx == 0 {
                    continue;
                }
                let words: Vec<&str> = line.text.split_whitespace().collect();
                if words.len() >= 3 && words.iter().all(|w| w.chars().all(|c| c.is_ascii_digit())) {
                    header_line_idx = Some(idx);
                    columns = words.into_iter().map(|s| s.to_string()).collect();
                    break;
                }
            }
        }

        let header_idx = header_line_idx?;

        // Jika kolom belum terisi dari digit, isi dari baris teks
        if columns.is_empty() {
            let header_line = &lines[header_idx];
            let raw_h = &header_line.text;

            // Coba pisahkan dengan pola skala umum
            let parsed_cols = Self::split_scale_column_headers(raw_h);
            if parsed_cols.len() >= 2 {
                columns = parsed_cols;
            } else {
                columns = raw_h
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if columns.len() < 2 {
                    columns = raw_h
                        .split("  ")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }

        if columns.len() < 2 {
            return None;
        }

        // Baris-baris setelah header kolom merupakan row statements dari matriks
        let mut rows = Vec::new();
        for line in &lines[(header_idx + 1)..] {
            let trimmed = line.text.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Abaikan tombol submit atau navigasi di bawah
            if trimmed.eq_ignore_ascii_case("kirim")
                || trimmed.eq_ignore_ascii_case("submit")
                || trimmed.eq_ignore_ascii_case("next")
                || trimmed.eq_ignore_ascii_case("kembali")
            {
                continue;
            }

            // Jika baris berada di sisi kiri (x < 0.65 * img_w) dan bukan angka tunggal
            if img_w <= 0.0 || line.bounding_box.x < (0.65 * img_w) {
                let cleaned_row = Self::clean_matrix_row_text(trimmed);
                if !cleaned_row.is_empty() {
                    rows.push(cleaned_row);
                }
            }
        }

        if rows.is_empty() && !has_matrix_keyword {
            return None;
        }

        // Jika rows kosong tapi ada matriks keyword, buat placeholder row dari konteks
        if rows.is_empty() {
            rows.push("Item 1".to_string());
        }

        // Bangun kisi sel (cell_matrix) berdimensi rows x columns
        let cell_matrix = vec![vec![None; columns.len()]; rows.len()];

        Some(MatrixStructure {
            rows,
            columns,
            cell_matrix,
            is_multi_select: false,
        })
    }

    fn split_scale_column_headers(header_text: &str) -> Vec<String> {
        let patterns = [
            vec![
                "sangat tidak setuju",
                "tidak setuju",
                "netral",
                "setuju",
                "sangat setuju",
            ],
            vec![
                "strongly disagree",
                "disagree",
                "neutral",
                "agree",
                "strongly agree",
            ],
            vec!["sangat buruk", "buruk", "cukup", "baik", "sangat baik"],
            vec!["very poor", "poor", "fair", "good", "excellent"],
            vec![
                "tidak pernah",
                "jarang",
                "kadang-kadang",
                "sering",
                "selalu",
            ],
            vec!["never", "rarely", "sometimes", "often", "always"],
        ];

        let lower = header_text.to_lowercase();
        for group in &patterns {
            let matched: Vec<String> = group
                .iter()
                .filter(|&item| lower.contains(item))
                .map(|s| {
                    // Title-case kata-katanya
                    s.split_whitespace()
                        .map(|w| {
                            let mut c = w.chars();
                            match c.next() {
                                None => String::new(),
                                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect();

            if matched.len() >= 2 {
                return matched;
            }
        }

        Vec::new()
    }

    // ==========================================
    // DETEKSI SKALA PENILAIAN (RATING SCALE)
    // ==========================================

    fn try_parse_scale(lines: &[OcrLine], full_text: &str) -> Option<ScaleStructure> {
        let lower = full_text.to_lowercase();

        // 1. Cek pola rentang eksplisit: "skala 1 - 5", "1 (sangat buruk) sampai 5 (sangat baik)"
        // atau "scale from 1 (poor) to 10 (excellent)"
        let has_scale_keyword = lower.contains("skala")
            || lower.contains("scale")
            || lower.contains("seberapa puas")
            || lower.contains("seberapa setuju")
            || lower.contains("rate your")
            || lower.contains("rating");

        // Cek baris yang memuat rangkaian angka horizontal: "1 2 3 4 5" atau "0 1 2 3 4 5 6 7 8 9 10"
        for line in lines {
            let words: Vec<&str> = line.text.split_whitespace().collect();
            if words.len() >= 3 {
                let nums: Vec<i64> = words.iter().filter_map(|w| w.parse::<i64>().ok()).collect();
                if nums.len() == words.len() && is_consecutive_sequence(&nums) {
                    let min_v = *nums.first().unwrap();
                    let max_v = *nums.last().unwrap();
                    let (min_l, max_l) = extract_scale_anchors(&lower);
                    return Some(ScaleStructure {
                        min_value: min_v,
                        max_value: max_v,
                        min_label: min_l,
                        max_label: max_l,
                        all_labels: words.iter().map(|s| s.to_string()).collect(),
                        step: Some(1),
                    });
                }
            }
        }

        if has_scale_keyword {
            // Coba ekstrak min dan max dari teks
            if let Some((min_v, max_v)) = extract_numeric_scale_range(&lower) {
                let (min_l, max_l) = extract_scale_anchors(&lower);
                return Some(ScaleStructure {
                    min_value: min_v,
                    max_value: max_v,
                    min_label: min_l,
                    max_label: max_l,
                    all_labels: (min_v..=max_v).map(|n| n.to_string()).collect(),
                    step: Some(1),
                });
            }
        }

        None
    }

    // ==========================================
    // DETEKSI HELPER: UNIT, WAJIB, INSTRUKSI, OPSI
    // ==========================================

    fn detect_units(text: &str) -> Option<String> {
        let units = [
            ("rp", "IDR"),
            ("idr", "IDR"),
            ("usd", "USD"),
            ("$", "USD"),
            ("tahun", "tahun"),
            ("years", "years"),
            ("bulan", "bulan"),
            ("months", "months"),
            ("hari", "hari"),
            ("days", "days"),
            ("jam", "jam"),
            ("hours", "hours"),
            ("kg", "kg"),
            ("gram", "gram"),
            ("meter", "meter"),
            ("km", "km"),
            ("liter", "liter"),
            ("%", "%"),
            ("persen", "%"),
            ("orang", "orang"),
            ("kali", "kali"),
        ];

        let lower = text.to_lowercase();
        for (token, label) in &units {
            // Cari kata utuh atau simbol
            if *token == "$" || *token == "%" {
                if lower.contains(token) {
                    return Some(label.to_string());
                }
            } else {
                for word in lower.split_whitespace() {
                    let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                    if clean == *token {
                        return Some(label.to_string());
                    }
                }
            }
        }

        None
    }

    fn detect_required_indicator(full_text: &str, lines: &[OcrLine]) -> bool {
        let lower = full_text.to_lowercase();
        if lower.contains("(wajib)")
            || lower.contains("(required)")
            || lower.contains("wajib diisi")
            || lower.contains("pertanyaan wajib")
        {
            return true;
        }

        // Cek simbol '*' pada baris pertama (biasanya judul pertanyaan ber-asterisk)
        if let Some(first_line) = lines.first() {
            if first_line.text.contains('*') {
                return true;
            }
        }

        false
    }

    fn is_instruction_line(text: &str) -> bool {
        let lower = text.to_lowercase();
        (lower.starts_with('(') && lower.ends_with(')'))
            || lower.starts_with("pilihlah")
            || lower.starts_with("pilih")
            || lower.starts_with("select")
            || lower.starts_with("choose")
            || lower.starts_with("centang")
            || lower.starts_with("catatan:")
            || lower.starts_with("note:")
            || lower.contains("pilih satu")
            || lower.contains("pilih semua")
            || lower.contains("select all")
            || lower.contains("select one")
    }

    fn is_multi_select_instruction(text: &str) -> bool {
        let lower = text.to_lowercase();
        lower.contains("pilih semua")
            || lower.contains("select all")
            || lower.contains("lebih dari satu")
            || lower.contains("centang semua")
            || lower.contains("semua yang sesuai")
            || lower.contains("all that apply")
    }

    fn is_option_line(text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.len() < 2 {
            return false;
        }

        // Teks yang diakhiri tanda tanya (?) adalah kalimat pertanyaan, BUKAN pilihan jawaban
        if trimmed.ends_with('?') {
            return false;
        }

        // Teks instruksi bukan opsi
        if Self::is_instruction_line(trimmed) {
            return false;
        }

        // Cek format prefix: A., B., C., a), b), 1., 2., [ ], ( ), o, *
        let prefixes = [
            "a.", "b.", "c.", "d.", "e.", "f.", "g.", "h.", "a)", "b)", "c)", "d)", "e)", "f)",
            "g)", "h)", "[ ]", "[x]", "( )", "(x)", "o ", "* ", "- ",
        ];

        let lower = trimmed.to_lowercase();
        for p in &prefixes {
            if lower.starts_with(p) {
                return true;
            }
        }

        // Cek digit diikuti titik atau kurung
        if let Some(first_char) = trimmed.chars().next() {
            if first_char.is_ascii_digit() {
                if let Some(second_char) = trimmed.chars().nth(1) {
                    if second_char == '.' || second_char == ')' || second_char == '-' {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn looks_like_option_continuation(text: &str) -> bool {
        let trimmed = text.trim();
        // Bukan instruksi, bukan pertanyaan baru yang ber-tanda tanya
        !trimmed.ends_with('?')
            && !Self::is_instruction_line(trimmed)
            && !trimmed.eq_ignore_ascii_case("kirim")
            && !trimmed.eq_ignore_ascii_case("submit")
            && !trimmed.eq_ignore_ascii_case("next")
    }

    fn clean_option_text(text: &str) -> String {
        let trimmed = text.trim();
        let prefixes = ["[ ]", "[x]", "( )", "(x)", "o ", "* ", "- "];

        let mut cleaned = trimmed;
        for p in &prefixes {
            if cleaned.to_lowercase().starts_with(p) {
                cleaned = &cleaned[p.len()..];
                break;
            }
        }

        cleaned = cleaned.trim();

        // Hapus prefix A. atau 1. jika ada spasi sesudahnya
        if cleaned.len() >= 3 {
            let first = cleaned.chars().next().unwrap();
            let second = cleaned.chars().nth(1).unwrap();
            let third = cleaned.chars().nth(2).unwrap();
            if (first.is_alphabetic() || first.is_ascii_digit())
                && (second == '.' || second == ')')
                && third.is_whitespace()
            {
                cleaned = &cleaned[3..];
            }
        }

        cleaned.trim().to_string()
    }

    fn clean_prompt_text(text: &str) -> String {
        let trimmed = text.trim();
        let mut cleaned = trimmed;

        // Bersihkan prefix nomor pertanyaan seperti "1.", "Q1:", "Pertanyaan 1:"
        let lower = cleaned.to_lowercase();
        if lower.starts_with("pertanyaan ") {
            if let Some(colon_pos) = cleaned.find(':') {
                cleaned = cleaned[colon_pos + 1..].trim();
            }
        } else if lower.starts_with("q")
            && lower
                .chars()
                .nth(1)
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            if let Some(colon_pos) = cleaned.find(':') {
                cleaned = cleaned[colon_pos + 1..].trim();
            } else if let Some(dot_pos) = cleaned.find('.') {
                cleaned = cleaned[dot_pos + 1..].trim();
            }
        } else if let Some(first_char) = cleaned.chars().next() {
            if first_char.is_ascii_digit() {
                if let Some(dot_pos) = cleaned.find('.') {
                    if dot_pos <= 3 {
                        cleaned = cleaned[dot_pos + 1..].trim();
                    }
                }
            }
        }

        // Hapus asterisk required jika ada di ujung prompt
        cleaned = cleaned.trim_end_matches('*').trim();

        cleaned.to_string()
    }

    fn clean_matrix_row_text(text: &str) -> String {
        let trimmed = text.trim();
        let mut cleaned = trimmed;

        // Bersihkan prefix nomor item seperti "a.", "1.", "-"
        if cleaned.len() >= 3 {
            let first = cleaned.chars().next().unwrap();
            let second = cleaned.chars().nth(1).unwrap();
            let third = cleaned.chars().nth(2).unwrap();
            if (first.is_alphabetic() || first.is_ascii_digit())
                && (second == '.' || second == ')')
                && third.is_whitespace()
            {
                cleaned = &cleaned[3..];
            }
        }

        if cleaned.starts_with('-') || cleaned.starts_with('•') {
            cleaned = cleaned[1..].trim();
        }

        cleaned.trim().to_string()
    }
}

// ==========================================
// PENDUKUNG MATEMATIS & PARSER RENTANG
// ==========================================

fn is_consecutive_sequence(nums: &[i64]) -> bool {
    if nums.len() < 2 {
        return false;
    }
    for i in 1..nums.len() {
        if nums[i] != nums[i - 1] + 1 {
            return false;
        }
    }
    true
}

fn extract_numeric_scale_range(text: &str) -> Option<(i64, i64)> {
    // Cari pola seperti "1 - 5", "1 sampai 5", "1 to 10", "1 - 10"
    let tokens: Vec<&str> = text.split_whitespace().collect();
    for i in 0..tokens.len() {
        if let Ok(min_n) = tokens[i].parse::<i64>() {
            if i + 2 < tokens.len() {
                let mid = tokens[i + 1];
                if mid == "-" || mid == "sampai" || mid == "to" || mid == "hingga" || mid == "s/d" {
                    if let Ok(max_n) = tokens[i + 2].parse::<i64>() {
                        if max_n > min_n && (max_n - min_n) <= 10 {
                            return Some((min_n, max_n));
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_scale_anchors(text: &str) -> (Option<String>, Option<String>) {
    let mut min_label = None;
    let mut max_label = None;

    if text.contains("sangat tidak puas")
        || text.contains("sangat buruk")
        || text.contains("sangat tidak setuju")
    {
        if text.contains("sangat tidak puas") {
            min_label = Some("Sangat Tidak Puas".to_string());
        } else if text.contains("sangat tidak setuju") {
            min_label = Some("Sangat Tidak Setuju".to_string());
        } else {
            min_label = Some("Sangat Buruk".to_string());
        }
    } else if text.contains("strongly disagree")
        || text.contains("very poor")
        || text.contains("very dissatisfied")
    {
        if text.contains("very dissatisfied") {
            min_label = Some("Very Dissatisfied".to_string());
        } else if text.contains("strongly disagree") {
            min_label = Some("Strongly Disagree".to_string());
        } else {
            min_label = Some("Very Poor".to_string());
        }
    }

    if text.contains("sangat puas")
        || text.contains("sangat baik")
        || text.contains("sangat setuju")
    {
        if text.contains("sangat puas") {
            max_label = Some("Sangat Puas".to_string());
        } else if text.contains("sangat setuju") {
            max_label = Some("Sangat Setuju".to_string());
        } else {
            max_label = Some("Sangat Baik".to_string());
        }
    } else if text.contains("strongly agree")
        || text.contains("very good")
        || text.contains("very satisfied")
        || text.contains("excellent")
    {
        if text.contains("very satisfied") {
            max_label = Some("Very Satisfied".to_string());
        } else if text.contains("strongly agree") {
            max_label = Some("Strongly Agree".to_string());
        } else {
            max_label = Some("Excellent".to_string());
        }
    }

    (min_label, max_label)
}
