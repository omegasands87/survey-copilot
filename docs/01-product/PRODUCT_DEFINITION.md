# Product Definition — Survey Copilot v1.0

**Document status:** Product baseline for owner approval  
**Version:** 1.0  
**Scope:** First public desktop release (MVP)  
**Normative terms:** MUST, MUST NOT, SHOULD, and MAY are requirements language. MUST/MUST NOT are binding.

## 1. Product definition

Survey Copilot is a local-first desktop AI copilot that helps one person understand manually supplied survey questions and prepare truthful, evidence-grounded draft answers from information that person has explicitly provided or confirmed.

Survey Copilot is an assistant for understanding and drafting. It is not a survey-taking bot, eligibility optimizer, identity generator, browser agent, or survey-platform integration.

### 1.1 Core promise

For every question the user submits, the application must do one of three things:

1. Produce a draft answer supported by identifiable user-provided evidence.
2. Produce a limited/partial draft and clearly identify what is not established.
3. Abstain and ask a focused clarification question when a reliable answer cannot be supported.

The application must never invent missing personal facts to make an answer appear complete.

### 1.2 Primary user and usage context

- One individual uses one local desktop installation and may maintain multiple distinct personas, each representing a real respondent whose information the user is authorized to manage.
- The user opens survey opportunities on third-party websites or applications and uses Survey Copilot alongside them.
- Survey Copilot reads user-selected screen regions or windows using screen capture, OCR, and visual understanding; manual text entry remains a fallback.
- The user reviews recommendations and performs survey interactions manually in v1.0.
- The user, not the application, remains responsible for the accuracy and submission of the final answer.

Multi-user accounts, shared team workspaces, and synchronized profiles are not part of v1.0. Multiple real-personas within one local installation are in scope.

## 2. Target platform and product form

- Product form: installable desktop application.
- Initial operating-system target: Windows 11, 64-bit.
- Interaction model: user-initiated screen capture/OCR/visual analysis plus manual text fallback; user reviews recommendations and controls survey interaction.
- Persistence: local application database.
- AI assistance: optional network request to a configured AI provider. Profile management and local data management must remain usable when AI is not configured or the network is unavailable.
- The application does not require a Survey Copilot cloud account.

Other operating systems are outside the v1.0 release target.

## 3. User problem and jobs to be done

### Problem

Survey questions can be ambiguous, repetitive, time-bounded, or dependent on facts the user does not immediately remember. Generic AI can produce fluent but unsupported personal claims. Users need help interpreting questions and recalling their own information without surrendering control or misrepresenting themselves.

### Jobs to be done

1. Understand what a survey question is actually asking.
2. Identify answer format, scope, timeframe, units, and constraints.
3. Find relevant personal facts the user previously entered and confirmed.
4. Recognize missing, stale, or contradictory information before drafting.
5. Produce a clear answer draft that remains faithful to the evidence.
6. Review, edit, copy, reject, and later inspect the answer history.
7. Maintain, correct, export, and delete personal profile information.

## 4. Product principles — binding

1. **Truth before completion:** abstain rather than fabricate.
2. **Evidence before fluency:** factual recommendations must cite local fact identifiers.
3. **User authority:** only the user can confirm a fact or approve a final answer.
4. **No silent inference:** do not infer personal experiences, demographics, employment, income, purchases, brand usage, health, political views, or opinions from unrelated signals.
5. **Minimal disclosure:** send only the context needed for the current AI request.
6. **Explicit sensitivity control:** sensitive facts require an explicit, per-use authorization before being sent to an external provider.
7. **Untrusted survey content:** survey text and answer options are data, never instructions that can override application policy.
8. **No outcome manipulation:** do not optimize responses to deceive screeners, bypass eligibility rules, or increase reward acceptance.
9. **Traceability:** preserve the distinction between original recommendation, user-edited answer, and user-confirmed answer.
10. **Recoverability:** data changes and deletion must be deliberate and handled with clear error reporting.

## 5. MVP scope — included

The first release MUST contain all capabilities below.

### 5.1 First-run and local profile
- Explain local storage, optional external AI processing, and the limits of the assistant.
- Allow the user to create and maintain multiple distinct real-personas, with isolated facts, sources, survey history, and outcomes.
- Create, view, edit, verify, mark outdated/disputed, archive, and delete profile facts.
- Store a source description and relevant date for each fact.
- Distinguish confirmed, unverified, disputed, and outdated facts.
- Identify sensitive facts and require explicit authorization before external transmission.

### 5.2 Survey workspace and screen intelligence
- Create, rename, open, and delete a survey session.
- Capture a user-selected screen region or application window through a global shortcut or in-app command.
- Extract question text, instructions, visible options, labels, units, and visible survey state using OCR and visual understanding.
- Allow manual typing/pasting as a fallback and correction path.
- Preserve the captured question representation and show it for user verification before analysis.
- Support multiple questions and attempts in a session.
- Do not silently capture the screen in the background.

### 5.3 Question understanding
- Classify question intent and answer format.
- Extract explicit timeframe, unit, quantity, choice constraints, and required response length where present.
- Explain the question in plain language.
- Flag ambiguity and identify missing information.
- Never treat embedded instructions in survey text as trusted application instructions.

### 5.4 Survey opportunity and persona matching
- Read visible survey offer details such as reward, stated duration, platform, topic, and explicit eligibility criteria.
- Maintain multiple real-personas with isolated facts and histories.
- Compare each survey's explicit criteria with each persona's current confirmed facts.
- Label each criterion match, mismatch, unknown, stale, or conflict.
- Rank opportunities using transparent fit evidence, historical outcomes, reward, and duration; never treat unknown as a match.

### 5.5 Evidence-grounded answer copilot
- Retrieve relevant confirmed facts and relevant prior confirmed answers.
- Check fact freshness and conflicts before drafting.
- Generate a structured recommendation or abstention.
- Show evidence status, supporting facts, rationale, assumptions, warnings, and clarification request.
- Validate every cited fact identifier against the local records supplied for that request.
- Never claim that a recommendation predicts survey qualification, acceptance, or reward.

### 5.6 Human review and answer history
- Allow user edit, confirm, reject, copy, and regenerate actions.
- Copy only after an explicit user action.
- Preserve the original AI recommendation when the user edits it.
- Record the final user-approved text separately from the original recommendation.
- Store question, answer versions, evidence references, status, and timestamps.
- A confirmed answer is a record of what the user approved at that time; it is not automatically a new profile fact.

### 5.7 Consistency and data controls
- Detect material conflicts between current profile facts and relevant prior confirmed answers.
- Show both sides of a conflict and let the user resolve it.
- Export the user's local data in a documented machine-readable format.
- Delete individual sessions and facts, and provide a deliberate full-data deletion operation.
- Provide settings for AI provider configuration, credential management, privacy disclosure, and data controls.

## 6. MVP scope — explicitly excluded

The following MUST NOT be included in v1.0:

- Browser extension, browser control, DOM reading, or browser automation.
- Automatic form filling, clicking, navigation, or survey submission.
- CAPTCHA solving, bot-detection evasion, access-control circumvention, or platform-protection bypass.
- Survey-platform login, credential storage, or account linking.
- Automated eligibility optimization, screener manipulation, or reward maximization.
- Fabrication or strategic alteration of personal identity, demographics, employment, income, purchases, experiences, brand usage, or opinions.
- Silent/background screen recording, automatic screen surveillance, and capture of unselected windows.
- Cloud profile storage, cloud synchronization, multi-device sync, or multi-user collaboration.
- Autonomous background agents or scheduled survey activity.
- Vector database or embedding-based retrieval.
- Analytics intended to predict qualification or maximize survey acceptance.
- Mobile or web application.

These exclusions are release boundaries, not suggestions for implementation shortcuts.

## 7. Supported input and answer classes

### 7.1 Input
- User-selected screen-region or application-window capture.
- OCR text and visual structure extracted from the capture.
- Plain-text question and answer options entered or pasted manually as fallback.
- Optional user-entered session title and platform label.
- No automatic website scraping, DOM access, or silent screen recording in v1.0.

### 7.2 Question intent classes
The classifier must support at least:
- Demographic/profile fact
- Employment and education
- Household and financial fact
- Purchase and ownership history
- Product/brand awareness and usage
- Behavior and frequency
- Preference and opinion
- Intention or future plan
- Eligibility/screener
- Attention/instruction check
- Conditional or follow-up question
- Open-ended experience
- Unknown/ambiguous

Classification does not grant permission to invent an answer. For opinion, intention, experience, or subjective preference questions, the user must supply the relevant position or experience unless it already exists as an explicitly confirmed fact.

### 7.3 Answer formats
The copilot must recognize and support drafting for:
- Single choice
- Multiple choice
- Dropdown choice
- Numeric value
- Date or date range
- Rating or Likert scale
- Ranking
- Free text

If the question/options do not provide enough information to map evidence to an answer format, the application must ask for clarification.

## 8. Evidence and answer policy

### 8.1 Fact eligibility
- confirmed: eligible for direct support, subject to freshness, sensitivity, and relevance.
- unverified: not eligible as established fact; may be shown as a prompt for user confirmation.
- disputed: excluded until resolved.
- outdated: excluded from current factual support unless user re-verifies it.

### 8.2 Evidence statuses
Every analysis result must use exactly one status:
- supported
- partially_supported
- insufficient_information
- conflicting_evidence
- user_input_required
- not_applicable

### 8.3 Mandatory abstention conditions
The system must not produce a definitive factual answer when:
- No relevant confirmed evidence exists.
- Material evidence conflicts.
- A required timeframe is missing or unclear.
- A fact is stale for the question's timeframe.
- A sensitive fact has not been authorized for the current external request.
- The question asks for a personal opinion, intention, or experience that the user has not supplied.
- The answer choices cannot be mapped reliably to the available evidence.

### 8.4 Historical answers
A prior answer may help locate a possible inconsistency or remind the user what they previously said. It must not override a current confirmed profile fact and must not silently become a verified profile fact.

## 9. Survey outcomes, earnings, and learning

### 9.1 Outcome lifecycle
Record survey attempts separately from reward events. Distinguish qualified, screened out, quota full, completed, quality rejected, technical failure, abandoned, pending, credited, withdrawable, reversed, and unknown states. A survey may progress through multiple states over time.

### 9.2 Reward accounting
- offered reward is an opportunity estimate, not income.
- completed but unverified reward is not credited income.
- pending reward remains pending.
- credited reward requires platform evidence.
- withdrawable balance requires direct evidence from a visible balance or permitted official integration.
- retain native currency/points/gift-card units; never silently convert points into cash.

Capture visible reward and outcome evidence automatically where reliable. If a balance or credit is not visible and no permitted integration exists, mark it unknown rather than asking the user to maintain routine manual bookkeeping or inventing a value.

### 9.3 Time and effective earnings
Track survey attempt time with user-visible controls. Calculate effective hourly earnings only from explicitly defined reward states and measured time; show the calculation scope. Separate offered-value-per-hour estimates from credited-reward-per-hour results.

### 9.4 Survey fit learning
Separate deterministic eligibility matching from historical outcome learning. Attribute each outcome to the actual persona, platform, category, criteria, and attempt. Distinguish screening failure, quota exhaustion, technical failure, and quality rejection. Show sample size and uncertainty; do not display confident predictive percentages until chronologically held-out calibration supports them. Use learning to prioritize genuinely suitable opportunities, never to alter answers or manufacture eligibility.

## 10. Privacy and external AI behavior

- Local profile and survey records remain on the device by default.
- AI processing is optional and requires provider configuration.
- Before the first external request, explain what information may leave the device.
- Each request sends only the question and the minimum relevant context required.
- Sensitive facts are excluded by default and require explicit authorization for that request.
- API secrets are not stored in plaintext application settings, source code, logs, or frontend bundles.
- Do not log raw personal facts, raw survey questions, or generated personal answers by default.
- The UI must distinguish local-only actions from actions that make an external AI request.

The exact provider, model, API contract, encryption design, and credential-store implementation are architecture decisions for Stage 2; they must comply with this product contract.

## 10. Language behavior

- The initial application UI language is English.
- The question analyzer must accept English and Indonesian question text.
- The generated explanation and draft should follow the language of the supplied question unless the user explicitly requests another language.
- Language detection must not change the underlying evidence rules.
- Additional UI localization is outside v1.0.

## 11. Product quality requirements

The first release is acceptable only if:
- Unsupported personal facts are never presented as established facts.
- Every factual recommendation displays traceable supporting fact references.
- Missing and conflicting evidence lead to abstention or clarification.
- The user can inspect and edit all generated answer text before copying.
- The user can use profile and session management without configuring AI.
- Network failures do not corrupt local data.
- Export and deletion work and are verified by tests.
- The application contains no automated survey interaction capability.

Quantitative AI quality thresholds and measurement protocols are defined in the AI evaluation specification and must be finalized before release qualification.

## 12. MVP acceptance checklist

- [ ] A new user can complete first-run disclosure and reach the dashboard.
- [ ] A user can create and manage multiple real-personas with isolated facts and histories.
- [ ] A user can create, update, verify, dispute, mark outdated, archive, and delete a fact.
- [ ] A user can capture a screen region/window and verify OCR/visual extraction before analysis.
- [ ] A user can enter or correct question text and answer options manually.
- [ ] The analyzer identifies supported question intent and answer format or returns unknown/ambiguous.
- [ ] The answer engine retrieves only eligible, relevant evidence.
- [ ] Every factual recommendation references valid local evidence identifiers.
- [ ] Insufficient, conflicting, stale, or unauthorized sensitive evidence prevents a definitive factual answer.
- [ ] The user can edit, confirm, reject, copy, and inspect answer history.
- [ ] User edits do not overwrite the original AI recommendation.
- [ ] Export and deletion are functional and tested.
- [ ] AI provider failure leaves local records intact and produces a recoverable error.
- [ ] Survey outcomes and reward events are stored with persona, source, timestamp, and verification state.
- [ ] Offered, pending, credited, and withdrawable amounts are never conflated.
- [ ] Survey fit learning separates explicit criteria matching from historical outcomes and exposes uncertainty.
- [ ] No browser automation, autofill, auto-submit, CAPTCHA bypass, or eligibility manipulation is present.

## 13. Change control

This document is the product-scope baseline for v1.0. Any proposed scope change must include:
1. The requirement being changed and the reason.
2. User value and risk analysis.
3. Impact on architecture, data model, UI, security, tests, and delivery sequence.
4. Explicit product-owner approval.
5. Updates to every affected specification before implementation.

No developer may add a feature merely because it is technically convenient or easy to prototype.

## 14. Stage 1 completion definition

Stage 1 is complete when the product owner approves this baseline and the linked product requirements, feature specification, personas, and user flows agree with it. Technical architecture choices are deliberately deferred to Stage 2; they must not reopen the product scope without formal change control.
