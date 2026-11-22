# Target User and Usage Model

**Version:** 1.0 MVP  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)

## 1. Primary target user

A single individual who periodically completes online surveys and wants help interpreting questions, recalling personal facts, and drafting truthful responses without automating interaction with survey platforms.

The application is designed around one local operator who may maintain multiple distinct personas representing real respondents whose information the operator is authorized to manage. Multiple personas are in scope; shared team workspaces, synchronized profiles, and account-based collaboration are excluded from v1.0.

## 2. Core user needs

### Need A — Understand the question
The user needs a plain-language interpretation, recognized answer format, timeframe, units, and constraints.

### Need B — Recall personal facts accurately
The user needs to store facts once, identify their source and verification status, and retrieve only relevant information for a question.

### Need C — Avoid unsupported answers
The user needs the application to abstain or ask for clarification rather than generate plausible personal claims.

### Need D — Retain control
The user needs to inspect, edit, confirm, reject, and copy every answer manually.

### Need E — Protect personal information
The user needs local-first storage, visibility into external AI requests, and explicit control over sensitive information.

## 3. Design implications

- Evidence must be visible next to the recommendation.
- Missing evidence must be represented as a normal outcome, not hidden behind fluent text.
- The application must distinguish confirmed facts from unverified, disputed, and outdated records.
- User edits and confirmations must be auditable and separate from AI output.
- No external action may occur as a side effect of generating an answer.
- Local profile and session workflows must remain available without an AI provider.

## 4. Product assumptions to validate

These assumptions inform usability research but do not change the v1.0 scope without formal approval:
- Users are willing to manually paste survey questions.
- Users will maintain a small set of profile facts.
- Users value evidence transparency and correctness over automatic completion.
- Users understand that the final external survey response remains their responsibility.

## 5. Out-of-target users for v1.0

- Organizations managing survey response teams.
- Users seeking automated survey completion or reward maximization.
- Users requiring browser extensions, mobile apps, cloud-synced profiles, or multi-user collaboration.
- Users expecting the application to infer unknown personal information.


## 6. Multiple respondent personas

A user may create multiple personas when they correspond to distinct real people and the user has authority and, where appropriate, consent to manage their information. Each persona has isolated facts, sources, verification states, survey attempts, and reward outcomes.

The application must display the active persona before analysis and must not mix facts across personas. A persona is never a fictional identity or a means to bypass a survey's eligibility criteria. The actual respondent must be represented truthfully.

## 7. Survey opportunity and outcome learning

Survey fit is evaluated first by matching explicit eligibility criteria to current confirmed persona facts. Historical outcomes may then inform opportunity prioritization, but must be attributed to the exact persona and attempt. Sparse history must be labeled insufficient. Screening failure, quota exhaustion, technical failure, quality rejection, completion, and reward credit are distinct outcomes.
