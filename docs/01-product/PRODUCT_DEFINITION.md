# Product Definition — Survey Copilot v1.0

**Status:** Working product baseline  
**Target:** Windows 11 x64  
**Primary language:** Indonesian  
**Business model:** Free, personal-use application

## 1. Product essence

Survey Copilot is a simple desktop AI assistant that reads survey questions from the user's screen or pasted text, understands what they ask, uses the selected real respondent profile, and recommends a truthful, consistent answer. It also helps the user judge which survey opportunities fit a selected profile.

The user opens and operates survey websites normally. Survey Copilot assists alongside them. The user makes the final decision and manually enters and submits every answer.

## 2. Core user experience

1. User selects a respondent profile.
2. User opens a survey website in their browser.
3. User captures the visible question or pastes its text.
4. App extracts the question and answer options and shows them for review.
5. AI interprets the question and retrieves only relevant facts from the selected profile.
6. App displays a recommended answer, explanation, evidence, and uncertainty—or asks for missing information.
7. User copies or uses the recommendation and manually answers the survey.
8. User can review prior questions and answers later.

For survey opportunity screening, the user captures the visible offer and criteria. The app compares explicit criteria against the user's real profiles and explains match, mismatch, or unknown. It never guarantees qualification.

## 3. Owner decisions

- Windows 11 x64 only for v1.0.
- One local operator may maintain multiple distinct real-persona profiles.
- Support survey websites generally; no dependency on a specific survey platform.
- Read survey content through user-selected screenshot capture and manual text input.
- Gemini is the default AI provider.
- Support multiple Gemini API keys in an automatic pool.
- Allow custom providers, API keys, endpoints, and model identifiers through supported adapters.
- Automatically rotate/fail over on rate limits and retryable failures.
- Provide configurable retry behavior.
- Indonesian interface.
- Free personal-use product.
- No earnings, reward, balance, or income tracking feature.

## 4. Product principles

1. Simple user experience; complexity stays in the implementation.
2. Truthful recommendations only; never invent personal facts or opinions.
3. Every factual recommendation must be traceable to the selected persona's confirmed facts.
4. Multiple personas are isolated. Each represents a real person whose information the operator is authorized to manage.
5. Missing, stale, or conflicting evidence must be disclosed; the app asks or abstains.
6. Survey content and OCR text are untrusted data, not instructions to the app.
7. External AI requests use minimum necessary context and disclose what is sent.
8. API keys remain in Windows Credential Manager.
9. User controls all external survey interactions.
10. Do not promise eligibility or survey acceptance.

## 5. Included in v1.0

### 5.1 Digital Persona
- Create and manage multiple real respondent profiles.
- Keep facts, sources, preferences, and answer history separate per persona.
- Store facts with category, value, source, date, verification status, and sensitivity.
- Support confirmed, unverified, disputed, and outdated states.
- Select the active persona before analyzing a survey question.
- Prevent facts from one persona being used for another.
- Ask before using sensitive information in an external AI request.

### 5.2 Screen Intelligence
- Capture a user-selected screen region or application window.
- Read text using OCR and reconstruct visible question layout.
- Identify prompt, instructions, answer options, scales, units, and matrix questions when visible.
- Show extracted content for correction and confirmation before analysis.
- Allow manual paste/type as fallback.
- No background or silent capture.

### 5.3 Answer Copilot
- Explain survey questions in plain Indonesian.
- Classify question type and expected answer format.
- Retrieve relevant confirmed facts from the active persona.
- Check consistency with relevant facts and prior confirmed answers.
- Recommend a concise answer with evidence and rationale.
- Ask a focused question or abstain when evidence is insufficient.
- Allow edit, confirm, copy, reject, and regenerate.
- Preserve answer history without silently turning answers into profile facts.

### 5.4 Survey Opportunity Fit
- Read visible survey description and eligibility criteria from a user-selected capture.
- Compare explicit criteria against each persona.
- Show match, mismatch, unknown, stale, and conflict.
- Explain each result using the facts that support it.
- Rank only with transparent criteria such as explicit eligibility fit and user-selected preferences.
- Keep historical outcome learning separate from factual eligibility matching.
- Show uncertainty and sample size where historical learning is available.

### 5.5 AI Provider Hub
- Gemini as the default provider.
- Multiple API keys per provider.
- Adaptive pool selection and failover.
- Round-robin, weighted, and priority strategies.
- Retry, cooldown, rate-limit handling, cancellation, and per-key health.
- Custom provider, HTTPS endpoint, authentication header, and model configuration through supported adapters.
- Secure secret storage in Windows Credential Manager.
- User can test, enable, disable, edit, and remove configurations.

### 5.6 Local application
- Windows 11 x64 desktop app.
- Local SQLite database.
- Indonesian-first interface.
- Local export and complete data deletion.
- App remains usable for profile and history management without an AI connection.

## 6. Explicitly excluded from v1.0

- Earnings, rewards, points, balances, cash-out, or income tracking.
- Automatic survey filling, clicking, navigation, or submission.
- Browser automation or browser account integration.
- CAPTCHA bypass or circumventing survey-platform controls.
- Fabricated or fictional respondent profiles.
- Deceptive screener-answer optimization.
- Multi-user cloud accounts and profile synchronization.
- Background screen recording.

## 7. Recommendation contract

Every analysis must return:
- Interpreted question and answer format.
- Evidence status.
- Recommended answer or abstention.
- Supporting fact references from the active persona.
- Concise rationale.
- Missing information, assumptions, and conflicts.
- One focused clarification question when necessary.

The app validates the response schema and every fact reference locally before display. Model confidence is evidence confidence, not probability of survey acceptance.

## 8. Technical baseline

- Desktop: Tauri 2.
- Native layer: Rust.
- UI: React + TypeScript.
- Styling: Tailwind CSS and accessible component primitives.
- Local database: SQLite with migrations.
- AI: provider adapter interface, Gemini adapter first.
- Secrets: Windows Credential Manager.
- OCR: evaluated local OCR implementation; optional external vision only with explicit consent.
- Testing: Rust tests, TypeScript tests, integration tests, security tests, and Windows installer tests.

## 9. Release acceptance

v1.0 is ready only when:
- A user can create multiple isolated personas.
- A user can capture a question or paste it manually and correct OCR output.
- AI recommendations cite only valid facts from the active persona.
- Missing or conflicting facts cause clarification or abstention.
- Multiple Gemini keys can fail over safely.
- A compatible custom provider can be configured without changing domain logic.
- No secret is stored in plaintext application settings or logs.
- Survey interaction remains manual.
- The Windows 11 x64 installer passes clean-machine tests.
