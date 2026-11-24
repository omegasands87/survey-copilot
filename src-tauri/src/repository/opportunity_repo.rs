use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::opportunity::{
    CriterionOperator, CriterionType, OpportunityCriterion, OpportunityDetail,
    OpportunityProvenance, OpportunityStatus, SurveyOpportunity,
};

pub struct OpportunityRepository;

impl OpportunityRepository {
    /// Menyimpan entitas peluang survei
    pub fn save_opportunity(
        conn: &Connection,
        opp: &SurveyOpportunity,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            r#"
            INSERT INTO survey_opportunities (
                id, title, platform_label, topic, description,
                raw_text, provenance, status, source_capture_id,
                source_ocr_id, captured_at, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                ?10, ?11, ?12, ?13
            )
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                platform_label = excluded.platform_label,
                topic = excluded.topic,
                description = excluded.description,
                raw_text = excluded.raw_text,
                status = excluded.status,
                updated_at = excluded.updated_at
            "#,
            params![
                opp.id,
                opp.title,
                opp.platform_label,
                opp.topic,
                opp.description,
                opp.raw_text,
                opp.provenance.as_str(),
                opp.status.as_str(),
                opp.source_capture_id,
                opp.source_ocr_id,
                opp.captured_at,
                opp.created_at,
                opp.updated_at,
            ],
        )?;

        Ok(())
    }

    /// Menyimpan daftar kriteria untuk sebuah peluang survei
    pub fn save_criteria(
        conn: &Connection,
        criteria: &[OpportunityCriterion],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            INSERT INTO survey_opportunity_criteria (
                id, opportunity_id, criterion_text, raw_text,
                criterion_type, normalized_attribute, expected_value_json,
                operator, criterion_order
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7,
                ?8, ?9
            )
            ON CONFLICT(id) DO UPDATE SET
                criterion_text = excluded.criterion_text,
                raw_text = excluded.raw_text,
                criterion_type = excluded.criterion_type,
                normalized_attribute = excluded.normalized_attribute,
                expected_value_json = excluded.expected_value_json,
                operator = excluded.operator,
                criterion_order = excluded.criterion_order
            "#,
        )?;

        for c in criteria {
            let val_json_str = c
                .expected_value_json
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()));

            stmt.execute(params![
                c.id,
                c.opportunity_id,
                c.criterion_text,
                c.raw_text,
                c.criterion_type.as_str(),
                c.normalized_attribute,
                val_json_str,
                c.operator.as_str(),
                c.criterion_order,
            ])?;
        }

        Ok(())
    }

    /// Menghapus seluruh kriteria yang terhubung dengan peluang survei
    pub fn delete_criteria_by_opportunity_id(
        conn: &Connection,
        opportunity_id: &str,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "DELETE FROM survey_opportunity_criteria WHERE opportunity_id = ?1",
            params![opportunity_id],
        )?;
        Ok(())
    }

    /// Menyimpan peluang survei beserta seluruh kriterianya secara atomik
    pub fn save_opportunity_with_criteria(
        conn: &mut Connection,
        opp: &SurveyOpportunity,
        criteria: &[OpportunityCriterion],
    ) -> Result<(), rusqlite::Error> {
        let tx = conn.transaction()?;

        tx.execute(
            r#"
            INSERT INTO survey_opportunities (
                id, title, platform_label, topic, description,
                raw_text, provenance, status, source_capture_id,
                source_ocr_id, captured_at, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                ?10, ?11, ?12, ?13
            )
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                platform_label = excluded.platform_label,
                topic = excluded.topic,
                description = excluded.description,
                raw_text = excluded.raw_text,
                status = excluded.status,
                updated_at = excluded.updated_at
            "#,
            params![
                opp.id,
                opp.title,
                opp.platform_label,
                opp.topic,
                opp.description,
                opp.raw_text,
                opp.provenance.as_str(),
                opp.status.as_str(),
                opp.source_capture_id,
                opp.source_ocr_id,
                opp.captured_at,
                opp.created_at,
                opp.updated_at,
            ],
        )?;

        // Hapus kriteria lama jika ada pembaruan
        tx.execute(
            "DELETE FROM survey_opportunity_criteria WHERE opportunity_id = ?1",
            params![opp.id],
        )?;

        // Simpan kriteria baru
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO survey_opportunity_criteria (
                    id, opportunity_id, criterion_text, raw_text,
                    criterion_type, normalized_attribute, expected_value_json,
                    operator, criterion_order
                ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, ?7,
                    ?8, ?9
                )
                "#,
            )?;

            for c in criteria {
                let val_json_str = c
                    .expected_value_json
                    .as_ref()
                    .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()));

                stmt.execute(params![
                    c.id,
                    c.opportunity_id,
                    c.criterion_text,
                    c.raw_text,
                    c.criterion_type.as_str(),
                    c.normalized_attribute,
                    val_json_str,
                    c.operator.as_str(),
                    c.criterion_order,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Mengambil peluang survei berdasarkan ID
    pub fn get_opportunity_by_id(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<SurveyOpportunity>, rusqlite::Error> {
        conn.query_row(
            r#"
            SELECT
                id, title, platform_label, topic, description,
                raw_text, provenance, status, source_capture_id,
                source_ocr_id, captured_at, created_at, updated_at
            FROM survey_opportunities
            WHERE id = ?1
            "#,
            params![id],
            Self::map_opportunity_row,
        )
        .optional()
    }

    /// Mengambil daftar seluruh peluang survei terurut waktu penangkapan terbaru
    pub fn list_opportunities(
        conn: &Connection,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SurveyOpportunity>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, title, platform_label, topic, description,
                raw_text, provenance, status, source_capture_id,
                source_ocr_id, captured_at, created_at, updated_at
            FROM survey_opportunities
            ORDER BY captured_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let rows = stmt.query_map(params![limit, offset], Self::map_opportunity_row)?;
        let mut opportunities = Vec::new();
        for r in rows {
            opportunities.push(r?);
        }
        Ok(opportunities)
    }

    /// Mengambil seluruh kriteria untuk sebuah peluang survei terurut
    pub fn get_criteria_by_opportunity_id(
        conn: &Connection,
        opportunity_id: &str,
    ) -> Result<Vec<OpportunityCriterion>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                id, opportunity_id, criterion_text, raw_text,
                criterion_type, normalized_attribute, expected_value_json,
                operator, criterion_order
            FROM survey_opportunity_criteria
            WHERE opportunity_id = ?1
            ORDER BY criterion_order ASC
            "#,
        )?;

        let rows = stmt.query_map(params![opportunity_id], Self::map_criterion_row)?;
        let mut criteria = Vec::new();
        for r in rows {
            criteria.push(r?);
        }
        Ok(criteria)
    }

    /// Mengambil detail lengkap peluang survei beserta seluruh kriterianya
    pub fn get_opportunity_detail(
        conn: &Connection,
        id: &str,
    ) -> Result<Option<OpportunityDetail>, rusqlite::Error> {
        let opp = match Self::get_opportunity_by_id(conn, id)? {
            Some(o) => o,
            None => return Ok(None),
        };

        let criteria = Self::get_criteria_by_opportunity_id(conn, id)?;

        Ok(Some(OpportunityDetail {
            opportunity: opp,
            criteria,
        }))
    }

    /// Menghapus peluang survei (kriteria terhapus otomatis berkat ON DELETE CASCADE)
    pub fn delete_opportunity(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
        let rows_affected = conn.execute(
            "DELETE FROM survey_opportunities WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    fn map_opportunity_row(row: &rusqlite::Row) -> Result<SurveyOpportunity, rusqlite::Error> {
        let provenance_str: String = row.get(6)?;
        let status_str: String = row.get(7)?;

        Ok(SurveyOpportunity {
            id: row.get(0)?,
            title: row.get(1)?,
            platform_label: row.get(2)?,
            topic: row.get(3)?,
            description: row.get(4)?,
            raw_text: row.get(5)?,
            provenance: OpportunityProvenance::from_str_loose(&provenance_str),
            status: OpportunityStatus::from_str_loose(&status_str),
            source_capture_id: row.get(8)?,
            source_ocr_id: row.get(9)?,
            captured_at: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }

    fn map_criterion_row(row: &rusqlite::Row) -> Result<OpportunityCriterion, rusqlite::Error> {
        let type_str: String = row.get(4)?;
        let val_json_str: Option<String> = row.get(6)?;
        let op_str: String = row.get(7)?;

        let expected_value_json = val_json_str.and_then(|s| serde_json::from_str(&s).ok());

        Ok(OpportunityCriterion {
            id: row.get(0)?,
            opportunity_id: row.get(1)?,
            criterion_text: row.get(2)?,
            raw_text: row.get(3)?,
            criterion_type: CriterionType::from_str_loose(&type_str),
            normalized_attribute: row.get(5)?,
            expected_value_json,
            operator: CriterionOperator::from_str_loose(&op_str),
            criterion_order: row.get(8)?,
        })
    }
}
