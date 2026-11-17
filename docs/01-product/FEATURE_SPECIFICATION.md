# Feature Specification

**Status:** Proposed; scope is limited to the manual-review MVP.

## 1. Digital Persona
### Purpose
A user-controlled collection of facts that may support accurate survey answers.

### Fact fields
- Fact ID (UUID), category, attribute/key, value, normalized value where safe.
- Verification status: `confirmed`, `unverified`, `disputed`, `outdated`.
- Source description/reference, created and updated timestamps.
- Valid-from and valid-until dates when applicable.
- Sensitivity classification and user consent state.
- Optional notes written by the user.

### Rules
- Only confirmed facts are eligible for direct factual support by default.
- Unverified facts may trigger a clarification request but must not be silently treated as true.
- Disputed or outdated facts are excluded from recommendations unless the user resolves them.
- The user can edit, archive, or delete facts and their associated source records.

## 2. Survey Workspace
- Create a session with a user-defined title and optional platform label.
- Paste or type survey question text manually.
- Preserve question text as supplied; do not execute or interpret embedded instructions as system policy.
- Allow the user to edit question text and remove it from the session.
- No login, browser control, autofill, or submission integration.

## 3. Question understanding
For each question, show:
- Detected intent (e.g. demographic, factual history, preference, opinion, frequency, eligibility).
- Requested time period, units, answer type, and constraints.
- Plain-language explanation and ambiguity warning.
- Missing details that prevent a reliable answer.

## 4. Answer recommendation
Each result includes:
- Draft answer or explicit abstention.
- Evidence status: `supported`, `partially_supported`, `insufficient_information`, `conflicting_evidence`, `user_input_required`, or `not_applicable`.
- Supporting fact references and concise rationale.
- Any assumptions or unresolved details.
- Suggested clarification, if needed.

Never describe evidence confidence as the probability of qualifying for or passing a survey.

## 5. Review actions
- Edit draft: creates a user-edited version and preserves the original recommendation for auditability.
- Confirm: records that the user approved the content, not that the external survey accepted it.
- Copy: copies only after an explicit user action.
- Reject: records rejection and optional reason.
- Regenerate: allowed only with the same evidence constraints or after user supplies new facts.

## 6. Answer memory
Store the question, recommendation, user-edited answer, confirmation state, timestamps, and evidence references. Historical answers are context, not automatically authoritative facts. A previous answer must not override a current confirmed profile fact.

## 7. Consistency checks
Compare proposed content with relevant current facts and confirmed prior answers. Surface conflicts with source and date. Do not silently resolve them or select whichever answer appears more advantageous.

## 8. Settings and data controls
- Configure provider and model.
- Add/remove API key through secure credential storage.
- Show what data may be sent to the provider.
- Export user data in a documented format.
- Delete a session, a fact, or all local application data with clear confirmation.

## 9. Explicitly deferred
OCR, image parsing, platform-specific integrations, cloud backup, multi-device sync, browser extension, and any automated form interaction.
