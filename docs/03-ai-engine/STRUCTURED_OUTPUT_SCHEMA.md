# Structured AI Output Contract

**Version:** 1.0

## 1. Purpose

Every AI operation returns a predictable structure. The application validates this structure before showing a recommendation.

## 2. Question analysis response

Required fields:
- status: supported | partially_supported | insufficient_information | conflicting_evidence | user_input_required | not_applicable
- question_intent: string enum or other
- answer_format: single_choice | multiple_choice | dropdown | numeric | date | rating | ranking | matrix | free_text | unknown
- interpretation: short plain-language explanation
- draft_answer: string or null
- supporting_fact_ids: array of request-scoped fact IDs
- rationale: short explanation
- missing_information: array of strings
- conflicts: array of strings
- assumptions: array of strings
- clarifying_question: string or null

## 3. Validation rules

- Reject missing required fields, invalid enum values, unexpected source IDs, and overlong values.
- For insufficient_information, conflicting_evidence, and user_input_required, draft_answer must be null unless the schema explicitly marks a safe partial answer.
- Every supporting_fact_id must exist in the current request allowlist and belong to the active persona.
- Model confidence is not required. If used internally, it describes evidence support only.
- Do not trust model-generated eligibility, qualification, or success probabilities.
- The app must independently check that the draft is supported by cited facts.

## 4. Survey criteria response

Each criterion contains:
- criterion_text
- normalized_attribute (nullable)
- expected_value (nullable)
- match_state: match | mismatch | unknown | stale | conflict
- supporting_fact_ids
- explanation

The model may extract and normalize criteria, but deterministic local code assigns the final match_state using persona facts. Never accept a model's unsupported claim that a criterion is satisfied.

## 5. Prompt-injection boundary

Survey text, OCR, answer options, and captured descriptions are quoted untrusted data. Model output cannot authorize new tools, access other personas, reveal unrelated facts, or override application policy.
