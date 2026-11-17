# Product Requirements Document

**Product:** Survey Copilot  
**Status:** Proposed  
**Target:** Windows desktop MVP  
**Product model:** AI copilot with human review and explicit user action

## 1. Problem statement
People answering online surveys may struggle to interpret questions, recall their own facts consistently, and phrase accurate answers. Re-entering profile details across surveys is tedious, while unsupported AI-generated answers can misrepresent the user and create integrity or account risks.

## 2. Product vision
Provide a local-first desktop assistant that helps a person understand a survey question and draft an answer grounded in facts the person has explicitly provided and confirmed.

## 3. Goals
1. Explain survey questions and answer constraints in plain language.
2. Retrieve relevant, confirmed user facts with traceable provenance.
3. Recommend a draft only when sufficient evidence exists.
4. Identify missing, stale, or contradictory information.
5. Let the user review, edit, reject, and copy every recommendation.
6. Keep profile data under the user's control and minimize external disclosure.

## 4. Non-goals
- Automatically completing, clicking through, or submitting surveys.
- Circumventing CAPTCHA, bot detection, access controls, or platform restrictions.
- Fabricating identity, demographics, employment, income, purchases, brand usage, experiences, or opinions.
- Optimizing answers to deceive screeners or manipulate eligibility.
- Guaranteeing survey qualification, rewards, or acceptance.
- Collecting credentials or controlling survey-platform accounts.
- Cloud synchronization or multi-user collaboration in the MVP.

## 5. Target users
- People who periodically complete legitimate surveys and want help understanding questions.
- Users who maintain recurring profile facts and want to avoid inconsistent manual recall.
- Users who need a transparent drafting assistant rather than an autonomous survey agent.

See [personas](USER_PERSONAS.md).

## 6. Product principles
- Truth over completion: abstain rather than invent.
- Evidence before fluency: every factual recommendation links to supporting facts.
- User agency: no external action without an explicit user action.
- Data minimization: retrieve and transmit only the context needed for the current question.
- Clear uncertainty: distinguish missing evidence from low confidence.
- Respect: users can inspect, correct, export, and delete their data.

## 7. MVP scope
- Desktop shell for Windows 10/11, subject to support verification during implementation.
- Local Digital Persona with editable facts and evidence/source notes.
- Manual survey workspace for entering or pasting survey questions.
- Question classification and plain-language explanation.
- Evidence-grounded answer suggestions with support status and citations to local fact IDs.
- User editing, confirmation, copying, and answer history.
- Basic consistency checks across stored facts and prior confirmed answers.
- Settings for AI provider/API key, privacy, and data management.
- Local SQLite persistence.

## 8. Functional requirements
| ID | Requirement | Priority |
|---|---|---|
| FR-001 | Create and edit profile facts | Must |
| FR-002 | Record fact source, date, and verification status | Must |
| FR-003 | Add survey questions manually | Must |
| FR-004 | Classify question intent and constraints | Must |
| FR-005 | Retrieve only relevant facts for a question | Must |
| FR-006 | Return structured recommendation and evidence | Must |
| FR-007 | Abstain on insufficient or conflicting evidence | Must |
| FR-008 | Allow edit, reject, confirm, and copy | Must |
| FR-009 | Store answer history and provenance | Must |
| FR-010 | Detect relevant contradictions | Should |
| FR-011 | Export and delete local data | Must |
| FR-012 | OCR from screenshots | Later |
| FR-013 | Browser integration or autofill | Out of scope |

## 9. Non-functional requirements
- Local-first persistence; no account required for local core workflows.
- Secrets stored in the operating system credential store, not application source or plain settings.
- Structured AI output validated before display.
- Network transmission disclosed and minimized.
- Keyboard-accessible primary workflows.
- Recoverable database migrations and explicit error handling.
- Performance targets are defined in [AI evaluation](../03-ai-engine/AI_EVALUATION.md).

## 10. Success measures
Measure product quality rather than survey outcomes:
- Evidence attribution correctness.
- Rate of unsupported factual recommendations.
- Correct abstention on missing or conflicting facts.
- User correction/rejection rate and reasons.
- Time to review and copy a supported draft.
- Crash rate and data integrity during migrations.

## 11. Risks and mitigations
- **Hallucinated personal claims:** strict evidence contract, abstention, source validation.
- **Sensitive data exposure:** local-first design, minimal context, consent, secret storage.
- **Platform policy violations:** manual workflow; no automation or circumvention.
- **Stale profile facts:** verification dates and expiry/review indicators.
- **Prompt injection in survey text:** treat all survey content as untrusted data.
- **Overconfidence:** label evidence status, not likelihood of survey acceptance.

## 12. Open decisions
- Final supported Windows versions and signing/distribution approach.
- AI provider and model selection after privacy, cost, and quality evaluation.
- Whether encrypted database storage is mandatory for first public release.
- OCR provider and supported image formats.
- Data retention defaults and export format.
