# Feature Specification

**Version:** 1.0 MVP  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)  
**Status:** Product baseline

## 1. First-run experience

### Purpose
Explain the application's operating model before the user stores personal data or sends information to an AI provider.

### Required behavior
1. State that profile, sessions, and answer history are stored locally by default.
2. Explain that AI analysis sends the current question and selected relevant context to the configured provider.
3. Explain that sensitive facts are excluded unless explicitly authorized for that request.
4. State that the application does not fill or submit surveys.
5. Let the user continue to local profile management without configuring AI.
6. Show the dashboard after onboarding.

### Failure behavior
If local storage initialization fails, do not show a usable dashboard that implies data can be saved. Show a recoverable error and preserve diagnostic details without personal content.

## 2. Multi-Persona Digital Persona

### Purpose
Maintain multiple user-controlled, truthful respondent profiles. Each persona represents a real person whose information the user is authorized to manage.

### Fact lifecycle
- Create: user enters a fact, value, category, source description, relevant date, sensitivity, and verification state.
- Confirm: user explicitly attests that the fact is accurate.
- Edit: changes are saved as a new current value; prior value/audit treatment is defined in the data model specification.
- Mark disputed/outdated: immediately removes the fact from direct recommendation support.
- Archive: removes the fact from active retrieval while retaining it according to the data-retention design.
- Delete: removes the fact and dependent source records according to the deletion policy.

### Fact states
- confirmed
- unverified
- disputed
- outdated

Only confirmed, current, relevant facts may directly support factual answers. Sensitive confirmed facts still require per-request authorization before external transmission.

### Required fact metadata
Fact identifier, category, attribute/key, value, source description, source date or verification date where known, status, sensitivity classification, created/updated timestamps, and optional validity period/notes.

## 3. Survey Workspace

### Session operations
Create, rename, open, and delete a session. A session may contain multiple question records.

### Question operations
Manually type or paste question text; optionally enter answer options; edit or delete the question; request analysis only after reviewing the captured text.

### Required behavior
- Preserve the exact original text as entered.
- Keep answer options distinct from the question prompt.
- Show the text to the user before analysis.
- Treat all question text and options as untrusted input.
- Do not fetch URLs, inspect browser tabs, or read external pages.
- Do not interpret text inside a survey question as application policy.

## 4. Question analysis

### Required outputs
- Intent classification or unknown/ambiguous.
- Answer-format classification or unknown/ambiguous.
- Explicit timeframe, unit, quantity, and constraints when detectable.
- Plain-language interpretation.
- Ambiguity and missing-information flags.
- Language identification for English or Indonesian.

### Supported intent classes
Demographic/profile; employment/education; household/financial; purchase/ownership; brand awareness/usage; behavior/frequency; preference/opinion; intention/future plan; eligibility/screener; attention/instruction check; conditional/follow-up; open-ended experience; unknown/ambiguous.

### Supported answer formats
Single choice, multiple choice, dropdown, numeric, date/date range, rating/Likert, ranking, free text.

### Safety behavior
Classification does not establish the truth of a personal claim. Opinions, intentions, and experiences require user-supplied evidence. An eligibility question must be answered truthfully; the product must not recommend deception or optimize for acceptance.

## 5. Answer recommendation

### Required result fields
- Question classification.
- Recommended answer, nullable.
- Evidence status.
- Supporting fact references.
- Concise rationale.
- Assumptions and unresolved details.
- Warnings.
- Clarification question, nullable.
- User confirmation required flag.

### Evidence statuses
supported, partially_supported, insufficient_information, conflicting_evidence, user_input_required, not_applicable.

### Recommendation rules
1. Retrieve relevant current confirmed facts only for direct factual support.
2. Use prior confirmed answers as historical context, never as an automatic source of truth.
3. Check freshness and conflicts before drafting.
4. Exclude sensitive facts from external context unless authorized for the current request.
5. Do not produce a definitive answer when evidence is insufficient, stale, conflicting, or unauthorized.
6. Validate all returned source identifiers locally before display.
7. Display source evidence and uncertainty with the draft.
8. A model confidence score, if retained internally, must represent evidence confidence only and must never be described as qualification probability.

## 6. Human review actions

### Edit
User changes the draft. The original AI output remains immutable in history; the edited text is stored as a separate version.

### Confirm
Records that the user approved the displayed answer. It does not indicate external acceptance or submission.

### Copy
Copies only the currently displayed user-approved text after an explicit user action. Copy must not trigger any external navigation or submission.

### Reject
Marks the recommendation as rejected. An optional reason may be recorded.

### Regenerate
Runs a new analysis using the same allowed evidence or newly supplied facts. It must not weaken evidence policy or silently broaden data disclosure.

## 7. Answer History

Store the question text, answer options, original recommendation, user-edited versions, user confirmation/rejection state, supporting fact IDs, relevant timestamps, and provider/model metadata needed for troubleshooting without logging raw secrets.

Historical answers are not automatically promoted into Digital Persona facts. If a user wants a prior answer to become a fact, the user must explicitly create or confirm a separate profile fact.

## 8. Consistency Engine

Compare the current candidate answer against relevant confirmed facts and relevant prior confirmed answers. Detect material disagreement in value, timeframe, unit, or scope. Present the conflicting records and ask the user to resolve them. Never silently choose the more favorable answer.

## 9. Settings and data controls

- Configure the supported AI provider and model.
- Add, replace, and remove the provider API credential through secure credential storage.
- Explain the data categories sent in each AI request.
- Export local records in a documented format.
- Delete individual facts and sessions.
- Delete all local application data after explicit confirmation.
- Show recoverable errors for provider, network, storage, and export/delete failures.

## 10. Screen Intelligence

- User invokes a global shortcut or in-app capture command.
- User selects a screen region or application window.
- OCR extracts visible text; visual analysis reconstructs grouping, option relationships, scales, tables, and controls when needed.
- The application presents extracted content and uncertainty for correction before analysis.
- The capture path must not silently record the screen.
- No hidden dropdown options or off-screen content may be represented as captured.
- Screen capture is for reading only; no click, fill, navigation, or submission automation is included in v1.0.

## 11. Survey Opportunity Analyzer

Read visible opportunity details such as platform, topic, offered reward, stated duration, and explicit eligibility criteria. Compare these criteria with all eligible personas. Show match, mismatch, unknown, stale, and conflict per criterion, with evidence. Rank opportunities transparently using fit evidence, historical outcomes, reward, duration, and user preferences. Unknown criteria must not be treated as matches.

## 12. Earnings Intelligence

Track survey attempts, time, completion states, reward states, amounts, native units, source evidence, and timestamps. Keep offered, completed-unverified, pending, credited, withdrawable, reversed, and unknown states separate. Dashboard totals must be reproducible from the underlying ledger. Routine values visible on screen should be captured without manual re-entry; invisible balances must remain unknown unless a permitted official integration supplies evidence.

## 13. Survey Fit Learning

Separate deterministic eligibility matching from historical outcome learning. Attribute outcomes to the actual persona, platform, category, criteria, and attempt. Distinguish screened out, quota full, quality rejected, technical failure, abandoned, completed, pending, and credited. Show sample size and uncertainty. Do not display calibrated percentages until chronologically held-out validation supports them. Never recommend false answers or persona manipulation.

## 14. Explicitly excluded from v1.0

Browser integration, DOM scraping, browser automation, autofill, auto-submit, platform account credential collection, silent background screen recording, cloud sync, team/multi-user workspace, mobile/web, autonomous agents, vector retrieval, and survey qualification guarantees.
