# Multi-Persona Specification

**Version:** 1.0 product specification  
**Status:** Required MVP capability  
**Normative source:** PRODUCT_DEFINITION.md

## 1. Purpose

Allow the user to maintain multiple distinct, truthful respondent profiles and select the profile that genuinely represents the person whose answers are being provided for a survey.

A persona is a structured respondent profile, not a fictional identity or a strategy for bypassing survey eligibility rules.

## 2. Persona model

Each persona has:
- Unique persona ID and user-defined display name.
- Relationship/ownership context (self or another real person whose information the user is authorized to manage).
- Active, archived, or disabled state.
- Profile facts, sources, verification states, sensitivity labels, and validity periods.
- Persona-specific survey sessions, answer history, and observed outcomes.
- Creation and modification timestamps.

The application must support multiple personas without an artificial low fixed limit. Practical limits may be set based on performance.

## 3. Truthfulness and authorization

- Each persona corresponds to a real individual.
- The user must have authority and, where appropriate, consent to store and use another person's information.
- The application must not generate fictional demographics, employment, income, purchases, opinions, or experiences.
- A persona must not be selected merely because it is more likely to pass a screener if it does not truthfully represent the actual respondent.
- Facts, sources, sessions, and outcomes remain isolated by persona.
- The active persona is conspicuous before analysis.
- Switching persona never carries another persona's facts into the active request.

## 4. Survey-to-persona matching

Compare explicit survey criteria with persona facts. Each criterion is classified as match, mismatch, unknown, stale, or conflict. Show supporting facts or missing information. Unknown and mismatch must not be guessed or rewritten. Ranking must be transparent and must not encourage misrepresentation.

## 5. Persona-specific learning

Historical outcomes are attributed only to the persona actually used in that survey attempt. They are not personal facts and must never be used as facts about another persona.

## 6. Lifecycle and controls

- Create, edit, activate, archive, and delete a persona.
- Add, verify, update, dispute, mark outdated, archive, and delete persona facts.
- Export persona data separately or as part of a full export.
- Delete a persona and associated history according to the deletion policy.
- Confirm before switching persona during an active survey session.

## 7. Acceptance criteria

- [x] User can create and manage multiple personas (CRUD, archive, restore, safe deletion).
- [x] Facts and sources are isolated by persona ID (proven by cross-persona isolation tests).
- [ ] Survey matching shows match, mismatch, unknown, stale, and conflict.
- [x] Active persona is visible before answer generation.
- [x] Another persona's facts are never included in the current request.
- [ ] Each persona has separate sessions, answer history, and observed outcomes (in-progress with survey memory).
- [x] Persona records represent real people and do not facilitate screener deception.
