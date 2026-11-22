# Multi-Persona Specification

**Version:** 1.0 product specification  
**Status:** Required MVP capability  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)

## 1. Purpose

Allow the user to maintain multiple distinct, truthful respondent profiles and select the profile that genuinely represents the person whose answers are being provided for a survey.

A persona is a structured respondent profile, not a fictional identity or a strategy for bypassing survey eligibility rules.

## 2. Persona model

Each persona has:
- Unique persona ID and user-defined display name.
- Relationship/ownership context (self or another real person whose information the user is authorized to manage).
- Active, archived, or disabled state.
- Profile facts, sources, verification states, sensitivity labels, and validity periods.
- Persona-specific survey history and outcome history.
- Creation and modification timestamps.

The application must support more than one persona without imposing an artificial low fixed limit. Practical limits may be defined in technical design based on performance.

## 3. Truthfulness and authorization rules

- Each persona must correspond to a real individual.
- The user must have authority and, where appropriate, consent to store and use another person's information.
- The application must not generate fictional demographics, employment, income, purchases, opinions, or experiences.
- A persona must not be selected merely because it is more likely to pass a screener if it does not truthfully represent the actual respondent.
- Survey history, facts, and outcomes must remain isolated by persona.
- The UI must make the currently selected persona conspicuous before analysis.
- Switching persona must never silently carry another persona's facts into the active request.

## 4. Survey-to-persona matching

For each survey, the application extracts explicit eligibility criteria and compares them with persona facts.

Each criterion is classified as:
- match: supported by current confirmed evidence.
- mismatch: contradicted by current confirmed evidence.
- unknown: no adequate evidence.
- stale: evidence is no longer current for the criterion.
- conflict: multiple material facts disagree.

The matching result must show the criteria and supporting fact references. A mismatch or unknown criterion must not be rewritten or guessed. The application may rank truthful matches for user convenience, but must explain the ranking and must not encourage misrepresentation.

## 5. Persona-specific learning

Historical outcomes are attributed only to the persona actually used in that survey attempt. They must not be pooled across personas unless an explicitly defined aggregate view is statistically appropriate and privacy-safe. A persona's outcomes must never be used as facts about another persona.

## 6. Lifecycle and controls

- Create, edit, activate, archive, and delete a persona.
- Add, verify, update, dispute, mark outdated, archive, and delete persona facts.
- Export persona data separately or as part of a full export.
- Delete a persona and its associated history according to the documented deletion policy.
- Show a confirmation when switching the active persona during an in-progress survey session.

## 7. Acceptance criteria

- [ ] User can create and manage multiple personas.
- [ ] Facts and sources are isolated by persona ID.
- [ ] Survey matching shows match, mismatch, unknown, stale, and conflict states.
- [ ] The active persona is visible before answer generation.
- [ ] Another persona's facts are never included in the current request.
- [ ] Each persona has separate survey and reward history.
- [ ] Persona records represent real people and do not facilitate fabricated identities or screener deception.
