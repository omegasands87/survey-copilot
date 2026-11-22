# Feature Specification — Survey Copilot v1.0

**Normative source:** PRODUCT_DEFINITION.md

## 1. First launch

- Explain that persona data, questions, and history are stored locally.
- Explain that AI analysis sends the current question and selected relevant facts to the configured provider.
- Explain that sensitive facts are excluded unless the user authorizes their use.
- Explain that all survey interaction remains manual.
- Allow the user to create a persona and configure AI later.

## 2. Digital Persona

Each persona represents a real respondent whose information the operator is authorized to manage.

A persona contains:
- Display name and optional notes.
- Facts grouped by category.
- Source and relevant date for each fact.
- Verification state: confirmed, unverified, disputed, outdated.
- Sensitivity classification.

Only current, confirmed, relevant facts can directly support factual recommendations. Persona switching must not change ownership of sessions or facts. A fact or history item from another persona must never enter the current analysis context.

## 3. Screen Intelligence

User can capture a screen region or visible application window. The app previews the capture, runs OCR, reconstructs the visible question and options, and displays uncertain regions. User can edit extracted content. AI analysis begins only after explicit confirmation.

Manual paste/type is always available. The app does not capture in the background or interact with the browser.

## 4. Question understanding

Classify:
- Intent: demographic, employment, financial, purchase/ownership, brand awareness/usage, behavior, preference/opinion, intention, eligibility, instruction check, follow-up, open-ended experience, unknown.
- Format: single choice, multiple choice, dropdown, numeric, date, rating/Likert, ranking, matrix, free text, unknown.

Extract only visible or explicitly supplied timeframe, unit, range, scope, and response constraints. Explain the question in plain Indonesian. Flag ambiguity rather than silently guessing.

## 5. Answer recommendation

Output:
- Question interpretation.
- Answer format.
- Evidence status.
- Recommended answer or abstention.
- Supporting fact references.
- Short rationale.
- Missing information, assumptions, and conflicts.
- One focused clarification question when useful.

Evidence statuses:
- supported
- partially_supported
- insufficient_information
- conflicting_evidence
- user_input_required
- not_applicable

Do not invent facts, experiences, opinions, purchases, employment, income, or brand usage. Subjective questions require the respondent's own stated view. Sensitive facts require explicit authorization for the current external request.

## 6. User review actions

- Edit: save user-edited text separately from the original recommendation.
- Copy: copy only after explicit user action.
- Confirm: record that the user approved the displayed answer; this is not survey submission.
- Reject: record rejection.
- Regenerate: use the same evidence policy and data minimization.
- Clarify: user can answer for the current question or explicitly save/confirm a profile fact.

No answer is silently promoted into a permanent profile fact.

## 7. Survey Opportunity Fit

Extract visible survey description and explicit eligibility criteria from a user-selected capture. Compare each criterion with the selected persona using:
- match
- mismatch
- unknown
- stale
- conflict

Display the fact or missing information behind each status. Historical outcomes may inform fit estimates only when attributed to the correct persona and supported by sufficient data. Show sample size and uncertainty. Never recommend changing a fact or lying to pass screening.

## 8. Provider Hub

Gemini is the default. Users can add multiple API keys and configure a pool. The pool supports adaptive, round-robin, weighted, and priority strategies; bounded retry; rate-limit cooldown; failover; cancellation; and provider health.

Users can add custom providers, compatible endpoints, and model IDs through supported adapters. Arbitrary custom APIs are not assumed compatible and must pass a connection/format test. API secrets are stored in Windows Credential Manager, not app settings.

## 9. Prohibited behavior

No browser automation, DOM inspection, autofill, auto-click, auto-navigation, auto-submit, CAPTCHA bypass, or platform-control circumvention. No fictional personas, deceptive screener strategy, hidden capture, or earnings/reward tracking in v1.0.
