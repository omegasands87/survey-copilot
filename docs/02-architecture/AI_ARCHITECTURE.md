# AI Architecture

## 1. Design objective
Use a language model to interpret wording and draft language, while deterministic application logic controls evidence eligibility, provenance, validation, and user actions.

## 2. Components
- Question classifier.
- Constraint extractor.
- Local fact retriever.
- Consistency checker.
- Context minimizer.
- Provider adapter.
- Structured-output parser and validator.
- Source-reference validator.
- Recommendation presenter.

## 3. Separation of responsibility
**Model may:** classify intent, explain wording, identify requested details, draft from supplied evidence, and ask a targeted clarification.  
**Model may not:** create facts, mark facts verified, decide a user is eligible, override conflicts, invent source IDs, or submit survey responses.

**Application must:** select eligible facts, enforce privacy filters, validate every source reference, apply abstention rules, and gate all user actions.

## 4. Evidence policy
Confirmed, current, relevant facts are the default evidence set. Unverified facts can be surfaced as questions. Disputed/outdated facts cannot support a definitive draft. Prior answers are secondary context and never automatically become verified profile facts.

## 5. Output contract
Return a schema-validated object containing:
- `status`
- `draft_answer` (nullable)
- `question_intent`
- `rationale`
- `supporting_fact_ids`
- `missing_information`
- `conflicts`
- `assumptions`
- `clarifying_question` (nullable)

Allowed statuses are specified in [Answer Engine](../03-ai-engine/ANSWER_ENGINE.md). The local app rejects unknown statuses, malformed data, and source IDs not included in the request's allowed evidence set.

## 6. Provider abstraction
Define a provider interface for structured generation, timeout, cancellation, and usage metadata. Keep provider-specific request formats out of domain logic. Provider/model selection remains configurable until evaluation supports a decision.

## 7. Resilience
Set request timeout and bounded retries for transient errors only. Do not retry invalid or unsafe content indefinitely. Keep local input when network calls fail. Avoid duplicate billing on retry where idempotency is supported.

## 8. Observability
Capture latency, provider/model identifier, token counts when available, validation outcome, and error category. Do not log raw personal prompts by default.
