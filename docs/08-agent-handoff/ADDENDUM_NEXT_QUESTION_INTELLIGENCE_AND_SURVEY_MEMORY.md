# Addendum — Next Question Intelligence & Survey Memory

> **ADDITIONAL SCOPE — READ AFTER THE CURRENT IMPLEMENTATION TASK**
>
> **Non-interruption marker:** This addendum is supplemental product scope. If an Antigravity implementation run is already in progress, do not interrupt, reset, or replace its current task. Finish the active task and its verification first. Then read this addendum, reconcile it with the backlog, and implement it in the normal priority order. Do not silently change an in-flight implementation.

**Status:** Proposed product requirement; not evidence that any feature is implemented.

## 1. Purpose

Survey Copilot should retain a structured, user-reviewable history of surveys the user has actually encountered. The AI may use that history to recognize a previously seen survey, understand its question sequence and branching, prepare the user for likely next questions, and retrieve relevant prior context.

This capability is named **Next Question Intelligence (NQI)** and its supporting store is **Survey Memory**.

The system must distinguish:
- **Detected:** a question actually observed in the current session and confirmed or accepted by the user.
- **Predicted:** a possible next question inferred from prior sequence/branch evidence; it has not yet been observed.
- **Historical:** a question recorded during an earlier session.
- **Unknown:** insufficient evidence to identify or predict the question.

A prediction must never be presented as an observed fact.

## 2. Survey record and provenance

For every survey session, support storing the following fields where available:

- Survey title as displayed or entered by the user.
- Survey source/platform name, if known.
- Source URL or origin/domain, if the user provides it or it is visibly available in the selected capture.
- Optional survey/provider identifier, only when visibly exposed or user-supplied.
- First-seen and last-seen timestamps.
- Active persona and session identifiers.
- Survey language and detected topic/category, with uncertainty.
- Survey recognition fingerprint derived from normalized, non-sensitive survey structure; document the algorithm and version.
- User-supplied notes and recognition status.
- Provenance for each field: user-entered, manually confirmed, OCR-detected, or AI-inferred.

Do not scrape websites, inspect DOM, inspect browser storage, or collect hidden platform metadata. A URL must not be fabricated from a title or inferred from inaccessible browser state. Treat survey title and source as potentially sensitive metadata.

## 3. Per-question history

Store each question as a first-class record, preserving the exact observed text separately from normalized/search text.

For every question, support:

- Stable local question ID.
- Parent survey/session ID and active persona ID.
- Observed sequence/order within the session.
- Exact question/prompt text, instructions, and visible answer options.
- Question type (single choice, multiple choice, free text, scale, matrix, numeric, date, other, unknown).
- Extracted constraints such as units, ranges, timeframe, and required format.
- OCR/capture provenance and extraction confidence, if applicable.
- Whether the text was user-entered, OCR-detected, AI-inferred, or user-confirmed.
- Timestamp observed and optional screenshot reference subject to retention policy.
- Actual answer selected/entered by the user, stored separately from AI suggestions.
- Answer state: draft, user-edited, user-confirmed, skipped, not answered, or unknown.
- Optional branch/transition relationship to the preceding question, with evidence and confidence.
- Links to relevant persona facts/evidence IDs used for a recommendation, without copying facts across personas.

Never label an AI-generated draft as the user's actual answer. Only persist an actual answer as such after the user explicitly records/confirms it. Do not infer an answer from a click or screen state that the app is not authorized to observe.

## 4. Recognizing a previously seen survey

When a new session begins, NQI may compare its visible/user-provided title, source, optional visible identifier, and observed question structure against Survey Memory.

Recognition must return:
- Candidate survey record(s).
- Match evidence (for example title/source/question-structure similarity).
- Match confidence and the algorithm/version used.
- Differences between the historical and current observed questions.
- A clear status: exact structural match, likely match, possible match, or no reliable match.

Do not identify a survey as the same solely because its title matches. Survey versions can change. Confirm recognition with the user when evidence is ambiguous. A match does not imply that old answers are correct for the current respondent or survey version.

## 5. Next Question Intelligence

NQI may predict likely next questions from:
- The current survey's observed question sequence.
- Previously observed sequences for a confidently matched survey/version.
- The current session's user-confirmed answers.
- Explicitly observed branch transitions.
- General question context, only as a lower-confidence fallback.

Prediction output must include:
- Candidate next question(s), not a single fabricated certainty.
- Evidence source: same survey history, same survey version, observed branch, or general inference.
- Prediction confidence and a concise explanation.
- Relevant conditions/answers that may lead to each branch.
- Clear **Predicted — not yet detected** labeling.

After the user manually advances the survey, the user may initiate or authorize a new screen capture. OCR can then identify the actual next question. The app must not click Next, navigate, auto-fill, auto-submit, inspect the DOM, or otherwise control the survey.

If the observed question differs from the prediction, record the actual observation and use it to improve future sequence/branch understanding. Do not rewrite historical observations to fit a prediction.

## 6. Safe use of historical answers and multiple accounts/personas

Survey Memory is for recall, recognition, context, and truthful consistency—not for manufacturing eligibility or defeating screeners.

Mandatory rules:
- Persona boundaries remain absolute. Historical personal answers and facts from persona A must never be retrieved as personal evidence for persona B.
- Reuse a historical answer as a suggested draft only when it belongs to the same real person/persona, is still current, is relevant to the exact question, and is supported by current verified facts. Show its source/date and require user review.
- If a different real person is using another persona or account, the AI may reuse non-personal survey structure (title, source, question wording, order, and observed branching) as navigation/context knowledge only. It must formulate that person's answer independently from that person's own verified facts.
- Do not copy another person's answer, profile, demographics, experiences, preferences, or outcome into the active persona.
- Do not tailor, falsify, or coach answers to pass a screener, evade quota/eligibility restrictions, impersonate another respondent, evade account restrictions, or guarantee survey acceptance.
- Never claim that repeating a historical answer will make a user qualify or complete a survey.
- Where the survey's rules prohibit multiple accounts or repeat participation, do not advise circumventing those rules.

The feature may help a user recognize that a survey has appeared before and prepare for its likely structure. It must not promise to make a different account/person pass “perfectly.”

## 7. Learning and evaluation

Keep deterministic observations separate from model-generated hypotheses.

- Historical questions and confirmed transitions are observations.
- Predicted questions and inferred branches are hypotheses.
- Track prediction outcomes only when the actual next question is later observed.
- Evaluate prediction quality on chronological holdout data; do not train/evaluate on future questions from the same session in a way that leaks the answer.
- Report sample size, coverage, calibration, and uncertainty before presenting aggregate prediction metrics.
- Sparse or conflicting history must produce “insufficient evidence.”
- Do not optimize a model against screener pass rate or recommend deceptive answers.
- AI confidence means evidence/prediction confidence, not probability of survey acceptance.

## 8. Privacy, security, retention, and controls

Survey titles, URLs, question text, answers, and screenshots may contain personal or sensitive information.

Required controls:
- Survey history is scoped to its owning persona and session; enforce ownership in service and repository layers, not only UI.
- Do not send historical question/answer content to an external AI provider by default. Require clear, explicit user consent for the relevant request and disclose what will be sent.
- Minimize retained data. Store screenshots only when necessary and under an explicit retention policy; prefer storing reviewed extracted text and provenance rather than indefinite raw images.
- Provide user controls to view, correct, export, archive, and delete survey records and their question/answer history.
- Deletion must account for child question records, answer records, screenshot files, indexes, caches, and backups according to the documented deletion policy.
- Redact sensitive values from logs and diagnostics.
- Treat survey text, OCR output, historical content, and model output as untrusted data and defend against prompt injection.
- Do not place API keys, credentials, or secrets in survey history.

## 9. UI requirements

Add a **Survey History / Survey Memory** view that lets users:
- Browse and search prior surveys by title, source, and date.
- Open a survey to review its ordered question history.
- Inspect the actual answer the user recorded separately from any AI draft.
- See the source/provenance and timestamp for each question.
- See which questions were detected versus predicted.
- Review known branch transitions and confidence.
- Correct metadata, mark a survey version as changed, and delete/archive records.

Add a **Next Question Intelligence** panel in the active session showing:
- Current detected question.
- Predicted next question(s), if evidence supports a prediction.
- Evidence and confidence for each prediction.
- Historical match and version warning, if applicable.
- Actual newly detected question after the user manually advances and initiates/authorizes capture.

## 10. Acceptance criteria to add to the backlog/test matrix

Do not mark this addendum complete until tests verify at least:

1. Survey title, source, URL, timestamps, and provenance can be recorded without inventing missing metadata.
2. Each observed question is persisted in order with exact text and visible options.
3. User-entered/confirmed answers are distinct from AI-generated drafts.
4. Survey recognition reports evidence and uncertainty; title-only collisions do not produce a definitive match.
5. Predicted questions are visibly distinct from detected questions.
6. Branch predictions reflect observed transitions and do not claim certainty.
7. A newly observed question can correct a prior prediction without altering the historical record.
8. Persona A's historical answer is inaccessible as personal evidence to persona B.
9. Persona B can use non-personal survey structure while generating answers only from persona B's own evidence.
10. External AI transmission of survey history is blocked until explicit consent.
11. User export and deletion include survey/question/answer records and governed attachments.
12. Prompt injection embedded in historical survey text cannot override system or product policy.
13. No test or metric rewards screener deception, account evasion, or fabricated answers.
14. Sparse history returns insufficient evidence rather than an invented prediction.

## 11. Implementation sequencing

This feature depends on reliable session/question persistence, persona isolation, OCR provenance, privacy controls, and answer-history separation. Do not implement it as an isolated AI prompt or an in-memory cache.

After the currently active Antigravity task is complete:
1. Review this addendum and update the implementation backlog with appropriately sequenced work items.
2. Extend the database schema through versioned migrations.
3. Implement survey/session/question/answer persistence and provenance.
4. Implement survey recognition and history UI.
5. Implement prediction and branch modeling only after enough reliable observations exist.
6. Add acceptance tests and privacy/security review.

**Definition of done:** the feature stores verifiable survey history, distinguishes observed facts from predictions, preserves strict persona isolation, and provides useful preparation without copying another person's identity or coaching deceptive screener answers.
