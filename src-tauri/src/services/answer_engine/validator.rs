use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::domain::answer_engine::{
    EvidenceStatus, RawAnswerRecommendation, ValidatedAnswerRecommendation,
};
use crate::domain::error::AppError;
use crate::domain::evidence::{
    EvidenceConfidence, EvidencePackage, EvidenceRecommendation, RetrievedEvidenceItem,
};

const MAX_INTERPRETATION_LEN: usize = 1000;
const MAX_RATIONALE_LEN: usize = 2000;
const MAX_DRAFT_ANSWER_LEN: usize = 4000;
const MAX_CLARIFYING_QUESTION_LEN: usize = 500;

/// Melakukan validasi lokal ketat terhadap rekomendasi mentah dari AI Provider (Local Policy Enforcement)
///
/// Kebijakan Keamanan:
/// 1. Schema & Field Length Validation: membatasi panjang string dan struktur.
/// 2. Evidence Allowlist & Citation Verification (AT-043): menolak fakta halusinasi, cross-persona, atau tidak terdaftar.
/// 3. Local Evidence Policy Enforcement: jika bukti lokal kurang atau bertentangan, menolak draf faktual model dan memaksa abstensi.
/// 4. Prompt Injection Defense (AT-044): mengabaikan klaim status model yang bertentangan dengan bukti terverifikasi.
/// 5. Derivasi Confidence: keyakinan dihitung dari fakta profil riil, bukan keyakinan model.
#[allow(clippy::too_many_arguments)]
pub fn validate_recommendation(
    raw: RawAnswerRecommendation,
    evidence: &EvidencePackage,
    persona_id: &str,
    question_id: Option<String>,
    session_id: Option<String>,
    prompt_version: &str,
    provider_config_id: Option<String>,
    model_id: Option<String>,
) -> Result<ValidatedAnswerRecommendation, AppError> {
    // 1. Parsing status dasar dari model
    let mut status = EvidenceStatus::from_str_loose(&raw.status);

    // 2. Normalisasi & sanitasi field teks
    let interpretation = raw
        .interpretation
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.chars().count() > MAX_INTERPRETATION_LEN {
                s.chars().take(MAX_INTERPRETATION_LEN).collect::<String>()
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| {
            "Analisis pertanyaan survei berdasarkan bukti profil responden.".to_string()
        });

    let question_intent = raw
        .question_intent
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("general_survey_question")
        .to_string();

    let answer_format = raw
        .answer_format
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown")
        .to_string();

    let rationale = raw
        .rationale
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.chars().count() > MAX_RATIONALE_LEN {
                s.chars().take(MAX_RATIONALE_LEN).collect::<String>()
            } else {
                s.to_string()
            }
        });

    let mut clarifying_question = raw
        .clarifying_question
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.chars().count() > MAX_CLARIFYING_QUESTION_LEN {
                s.chars()
                    .take(MAX_CLARIFYING_QUESTION_LEN)
                    .collect::<String>()
            } else {
                s.to_string()
            }
        });

    let mut draft_answer = raw
        .draft_answer
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.chars().count() > MAX_DRAFT_ANSWER_LEN {
                s.chars().take(MAX_DRAFT_ANSWER_LEN).collect::<String>()
            } else {
                s.to_string()
            }
        });

    let mut missing_information = raw.missing_information.unwrap_or_default();
    let mut conflicts = raw.conflicts.unwrap_or_default();
    let assumptions = raw.assumptions.unwrap_or_default();

    // 3. Evidence Allowlist & Citation Verification (AT-043)
    // Bangun peta bukti yang valid dan lolos filter SC-032 untuk persona ini
    let mut allowed_evidence_map: HashMap<&str, &RetrievedEvidenceItem> = HashMap::new();
    let mut allowed_ids: HashSet<&str> = HashSet::new();

    for item in &evidence.retrieved_evidence {
        allowed_evidence_map.insert(&item.fact_id, item);
        allowed_ids.insert(&item.fact_id);
    }

    let mut verified_facts: Vec<RetrievedEvidenceItem> = Vec::new();
    let mut rejected_citations: Vec<String> = Vec::new();

    if let Some(cited_ids) = raw.supporting_fact_ids {
        for cited_id in cited_ids {
            let trimmed = cited_id.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(evidence_item) = allowed_evidence_map.get(trimmed) {
                // Hindari duplikasi jika model mengutip ID yang sama berulang
                if !verified_facts
                    .iter()
                    .any(|f| f.fact_id == evidence_item.fact_id)
                {
                    verified_facts.push((*evidence_item).clone());
                }
            } else {
                // AT-043: Sitasi ID fakta tidak ditemukan di allowlist permintaan
                rejected_citations.push(trimmed.to_string());
            }
        }
    }

    // 4. Penegakan Kebijakan Bukti Lokal (Local Evidence Policy Enforcement)
    // A. Jika model mengklaim 'supported' tapi TIDAK ADA sitasi fakta yang valid lolos allowlist:
    if status.permits_factual_draft() && verified_facts.is_empty() {
        // Pelanggaran: Klaim faktual tanpa bukti atau menggunakan sitasi palsu/halusinasi
        status = EvidenceStatus::InsufficientInformation;
        draft_answer = None;
        missing_information.push(
            "Draf jawaban faktual ditolak oleh sistem verifikasi lokal karena tidak memiliki bukti profil yang sah atau mengutip ID fakta yang tidak valid (AT-043).".to_string(),
        );
    }

    // B. Kebijakan Abstensi Berdasarkan SC-032 EvidencePackage:
    // Jika evidence retrieval engine menyatakan bukti kurang, model tidak boleh memaksa menjawab
    if evidence.recommendation == EvidenceRecommendation::InsufficientEvidence {
        status = EvidenceStatus::InsufficientInformation;
        draft_answer = None;
        if clarifying_question.is_none() {
            clarifying_question = Some(
                "Profil responden saat ini belum memiliki informasi yang cukup untuk menjawab pertanyaan ini secara akurat. Mohon lengkapi fakta profil Anda.".to_string(),
            );
        }
    }

    // C. Kebijakan Konflik Berdasarkan SC-032:
    // Jika terdeteksi konflik kritis di data responden, model dilarang memberi jawaban definitif
    if evidence.recommendation == EvidenceRecommendation::NeedsClarification
        || !evidence.conflict_indicators.is_empty()
    {
        let has_critical_conflict = evidence
            .conflict_indicators
            .iter()
            .any(|c| c.severity == crate::domain::evidence::ConflictSeverity::Critical);

        if has_critical_conflict {
            status = EvidenceStatus::ConflictingEvidence;
            draft_answer = None;

            for ci in &evidence.conflict_indicators {
                let desc = format!("[{:?}] {}", ci.conflict_type, ci.description);
                if !conflicts.contains(&desc) {
                    conflicts.push(desc);
                }
                if clarifying_question.is_none() {
                    if let Some(ref prompt) = ci.resolution_prompt {
                        clarifying_question = Some(prompt.clone());
                    }
                }
            }
        }
    }

    // D. Integritas status non-faktual: jika status adalah insufficient, conflicting, atau user_input_required,
    // draft_answer WAJIB null
    if !status.permits_factual_draft() {
        draft_answer = None;
    }

    // 5. Perhitungan Keyakinan Bukti (Local Evidence Confidence)
    let confidence = match status {
        EvidenceStatus::InsufficientInformation | EvidenceStatus::ConflictingEvidence => {
            EvidenceConfidence::Uncertain
        }
        EvidenceStatus::UserInputRequired | EvidenceStatus::NotApplicable => {
            EvidenceConfidence::Low
        }
        EvidenceStatus::Supported => evidence.evidence_confidence,
        EvidenceStatus::PartiallySupported => {
            if evidence.evidence_confidence == EvidenceConfidence::High {
                EvidenceConfidence::Medium
            } else {
                evidence.evidence_confidence
            }
        }
    };

    Ok(ValidatedAnswerRecommendation {
        id: Uuid::new_v4().to_string(),
        question_id,
        session_id,
        persona_id: persona_id.to_string(),
        status,
        question_intent,
        answer_format,
        interpretation,
        draft_answer,
        user_answer: None,
        review_status: Some("draft".to_string()),
        review_reason: None,
        reviewed_at: None,
        supporting_facts: verified_facts,
        rejected_citations,
        rationale,
        missing_information,
        conflicts,
        assumptions,
        clarifying_question,
        confidence,
        prompt_version: prompt_version.to_string(),
        provider_config_id,
        model_id,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}
