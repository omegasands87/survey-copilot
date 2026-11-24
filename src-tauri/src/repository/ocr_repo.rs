use rusqlite::Connection;

use crate::domain::capture::CaptureMode;
use crate::domain::ocr::{
    ExtractedSurveyQuestion, MatrixStructure, OcrTextBlock, ScaleStructure, UncertainRegion,
};
use crate::domain::survey::{AnswerFormat, ConfirmationStatus, QuestionType};

pub struct OcrRepository;

impl OcrRepository {
    pub fn save_extraction(
        conn: &Connection,
        extraction: &ExtractedSurveyQuestion,
    ) -> Result<bool, rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let options_json =
            serde_json::to_string(&extraction.answer_options).unwrap_or_else(|_| "[]".to_string());

        let structure_val = serde_json::json!({
            "scale": extraction.scale,
            "matrix": extraction.matrix,
            "visibleUnits": extraction.visible_units,
            "isRequired": extraction.is_required,
        });
        let structure_json =
            serde_json::to_string(&structure_val).unwrap_or_else(|_| "{}".to_string());
        let text_blocks_json =
            serde_json::to_string(&extraction.text_blocks).unwrap_or_else(|_| "[]".to_string());
        let uncertain_regions_json = serde_json::to_string(&extraction.uncertain_regions)
            .unwrap_or_else(|_| "[]".to_string());
        let warnings_json =
            serde_json::to_string(&extraction.warnings).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            r#"
            INSERT INTO ocr_extractions (
                id, capture_id, session_id, persona_id, ocr_engine,
                raw_ocr_text, question_text_original, question_text_corrected,
                instructions, options_json, structure_json, text_blocks_json,
                uncertain_regions_json, warnings_json, overall_confidence,
                confirmation_status, extracted_at, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8,
                ?9, ?10, ?11, ?12,
                ?13, ?14, ?15,
                ?16, ?17, ?18, ?19
            )
            ON CONFLICT(id) DO UPDATE SET
                question_text_corrected = excluded.question_text_corrected,
                instructions = excluded.instructions,
                options_json = excluded.options_json,
                structure_json = excluded.structure_json,
                confirmation_status = excluded.confirmation_status,
                updated_at = excluded.updated_at
            "#,
            rusqlite::params![
                extraction.extraction_id,
                extraction.capture_id,
                extraction.session_id,
                extraction.persona_id,
                extraction.ocr_engine,
                extraction.raw_ocr_text,
                extraction.question_text_original,
                extraction.question_text_corrected,
                extraction.instructions,
                options_json,
                structure_json,
                text_blocks_json,
                uncertain_regions_json,
                warnings_json,
                extraction.overall_confidence as f64,
                extraction.confirmation_status.as_str(),
                extraction.extracted_at,
                now,
                now,
            ],
        )?;

        Ok(())
    }

    pub fn get_extraction_by_id(
        conn: &Connection,
        extraction_id: &str,
    ) -> Result<Option<ExtractedSurveyQuestion>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                oe.id, oe.capture_id, oe.session_id, oe.persona_id, oe.ocr_engine,
                oe.raw_ocr_text, oe.question_text_original, oe.question_text_corrected,
                oe.instructions, oe.options_json, oe.structure_json, oe.text_blocks_json,
                oe.uncertain_regions_json, oe.warnings_json, oe.overall_confidence,
                oe.confirmation_status, oe.extracted_at,
                sc.capture_mode, sc.captured_at, sc.application_label
            FROM ocr_extractions oe
            LEFT JOIN screen_captures sc ON oe.capture_id = sc.id
            WHERE oe.id = ?1
            "#,
        )?;

        let mut rows = stmt.query(rusqlite::params![extraction_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_row_to_extraction(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_latest_extraction_by_capture(
        conn: &Connection,
        capture_id: &str,
    ) -> Result<Option<ExtractedSurveyQuestion>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                oe.id, oe.capture_id, oe.session_id, oe.persona_id, oe.ocr_engine,
                oe.raw_ocr_text, oe.question_text_original, oe.question_text_corrected,
                oe.instructions, oe.options_json, oe.structure_json, oe.text_blocks_json,
                oe.uncertain_regions_json, oe.warnings_json, oe.overall_confidence,
                oe.confirmation_status, oe.extracted_at,
                sc.capture_mode, sc.captured_at, sc.application_label
            FROM ocr_extractions oe
            LEFT JOIN screen_captures sc ON oe.capture_id = sc.id
            WHERE oe.capture_id = ?1
            ORDER BY oe.created_at DESC
            LIMIT 1
            "#,
        )?;

        let mut rows = stmt.query(rusqlite::params![capture_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_row_to_extraction(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn update_confirmed_extraction(
        conn: &Connection,
        extraction_id: &str,
        corrected_text: &str,
        instructions: Option<&str>,
        options_json: &str,
        confirmation_status: &str,
    ) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let affected = conn.execute(
            r#"
            UPDATE ocr_extractions SET
                question_text_corrected = ?1,
                instructions = ?2,
                options_json = ?3,
                confirmation_status = ?4,
                updated_at = ?5
            WHERE id = ?6
            "#,
            rusqlite::params![
                corrected_text,
                instructions,
                options_json,
                confirmation_status,
                now,
                extraction_id,
            ],
        )?;
        Ok(affected > 0)
    }

    fn map_row_to_extraction(
        row: &rusqlite::Row,
    ) -> Result<ExtractedSurveyQuestion, rusqlite::Error> {
        let extraction_id: String = row.get(0)?;
        let capture_id: Option<String> = row.get(1)?;
        let session_id: Option<String> = row.get(2)?;
        let persona_id: Option<String> = row.get(3)?;
        let ocr_engine: String = row.get(4)?;
        let raw_ocr_text: String = row.get(5)?;
        let question_text_original: String = row.get(6)?;
        let question_text_corrected: Option<String> = row.get(7)?;
        let instructions: Option<String> = row.get(8)?;
        let options_json: String = row.get(9)?;
        let structure_json: Option<String> = row.get(10)?;
        let text_blocks_json: String = row.get(11)?;
        let uncertain_regions_json: String = row.get(12)?;
        let warnings_json: String = row.get(13)?;
        let overall_confidence: f64 = row.get(14)?;
        let confirmation_status_str: String = row.get(15)?;
        let extracted_at: String = row.get(16)?;

        let source_mode_str: Option<String> = row.get(17)?;
        let captured_at: Option<String> = row.get(18)?;
        let application_label: Option<String> = row.get(19)?;

        let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
        let text_blocks: Vec<OcrTextBlock> =
            serde_json::from_str(&text_blocks_json).unwrap_or_default();
        let uncertain_regions: Vec<UncertainRegion> =
            serde_json::from_str(&uncertain_regions_json).unwrap_or_default();
        let warnings: Vec<String> = serde_json::from_str(&warnings_json).unwrap_or_default();

        let mut scale: Option<ScaleStructure> = None;
        let mut matrix: Option<MatrixStructure> = None;
        let mut visible_units: Option<String> = None;
        let mut is_required: Option<bool> = None;

        if let Some(ref s_json) = structure_json {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(s_json) {
                if let Some(scale_val) = v.get("scale") {
                    scale = serde_json::from_value(scale_val.clone()).ok();
                }
                if let Some(matrix_val) = v.get("matrix") {
                    matrix = serde_json::from_value(matrix_val.clone()).ok();
                }
                if let Some(units_val) = v.get("visibleUnits").and_then(|u| u.as_str()) {
                    visible_units = Some(units_val.to_string());
                }
                if let Some(req_val) = v.get("isRequired").and_then(|r| r.as_bool()) {
                    is_required = Some(req_val);
                }
            }
        }

        let source_mode = match source_mode_str.as_deref() {
            Some("window") => CaptureMode::Window,
            Some("clipboard_image") => CaptureMode::ClipboardImage,
            Some("manual") => CaptureMode::Manual,
            _ => CaptureMode::Region,
        };

        let question_type = if matrix.is_some() {
            QuestionType::Matrix
        } else if scale.is_some() {
            QuestionType::Scale
        } else if !options.is_empty() {
            QuestionType::SingleChoice
        } else {
            QuestionType::Unknown
        };

        let answer_format = if matrix.is_some() {
            AnswerFormat::Matrix
        } else if scale.is_some() {
            AnswerFormat::RatingScale
        } else if !options.is_empty() {
            AnswerFormat::SingleChoice
        } else {
            AnswerFormat::Unknown
        };

        Ok(ExtractedSurveyQuestion {
            extraction_id,
            capture_id,
            session_id,
            persona_id,
            source_mode,
            captured_at: captured_at.unwrap_or_else(|| extracted_at.clone()),
            extracted_at,
            ocr_engine,
            application_label,
            image_width: 800,
            image_height: 600,
            raw_ocr_text,
            question_text_original,
            question_text_corrected,
            instructions,
            answer_options: options,
            question_type,
            answer_format,
            scale,
            matrix,
            visible_units,
            is_required,
            text_blocks,
            uncertain_regions,
            warnings,
            overall_confidence: overall_confidence as f32,
            confirmation_status: ConfirmationStatus::from_str_loose(&confirmation_status_str),
        })
    }
}
