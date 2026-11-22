# AI Architecture

## 1. Objective

Use AI to understand survey wording and formulate recommendations, while deterministic local logic controls which facts may be used, persona isolation, privacy, provider routing, and user actions.

## 2. Pipeline

1. Receive user-confirmed question content.
2. Resolve active persona and session ownership.
3. Classify question and extract answer constraints.
4. Retrieve only relevant current confirmed facts from that persona.
5. Check consistency and sensitivity.
6. Build minimal request context and an allowed source-ID list.
7. Select a compatible provider configuration from the pool.
8. Generate structured output.
9. Validate schema, statuses, length, and cited source IDs.
10. Run local evidence/consistency checks.
11. Display recommendation or abstention.
12. Persist only user review actions and required redacted metadata.

## 3. Model responsibilities

Model may:
- Explain question intent.
- Identify response format and constraints.
- Draft wording based on supplied facts.
- Identify missing information and ambiguity.
- Ask one targeted clarification.
- Explain visible survey criteria.

Model may not:
- Invent personal facts, preferences, experiences, or opinions.
- Create or verify profile facts.
- Decide eligibility as a fact.
- Use facts from another persona.
- Override conflicting evidence.
- Invent source IDs.
- Trigger tools, browser actions, form entry, or submission.

## 4. Evidence policy

Only current confirmed relevant facts may support a factual recommendation. Unverified facts may be shown as candidates for user confirmation, not as established truth. Disputed and outdated facts cannot support a definitive answer. Prior answers are historical context only.

## 5. Structured output

Return:
- status
- question_intent
- answer_format
- interpretation
- draft_answer (nullable)
- supporting_fact_ids
- rationale
- missing_information
- conflicts
- assumptions
- clarifying_question (nullable)

Allowed statuses are defined in ANSWER_ENGINE.md. The app rejects unknown fields/statuses where strict schema is enabled and rejects any source ID not in the request allowlist.

## 6. Provider routing

Provider selection is an infrastructure concern, not a model decision. The model cannot select a key, endpoint, or provider. The provider hub filters by operation/model compatibility, enabled state, credential availability, cooldown, concurrency, and configured strategy. Only one provider configuration receives a request at a time.

## 7. Prompt injection defense

Question text, OCR output, answer options, and captured content are quoted data. They cannot change system instructions, request unrelated profile facts, or authorize tool use. Enforce the boundary both in prompt construction and local application logic.

## 8. Resilience

Use bounded retries only for retryable errors. Rate limits cause key cooldown and failover. Authentication failures disable that configuration pending user attention. Invalid requests and refusals are not blindly retried. Cancellation stops pending retries. Preserve question content locally on every failure.

## 9. Observability

Record provider ID, model ID, latency, token counts if available, normalized error category, and schema validation result. Do not log API keys, raw personal prompts, complete answers, or screenshots by default.
