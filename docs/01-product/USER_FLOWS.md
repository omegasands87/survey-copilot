# User Flows

**Version:** 1.0 MVP  
**Normative source:** [Product Definition](PRODUCT_DEFINITION.md)

## Flow 1 — First launch and local readiness

1. User launches the Windows 11 desktop application.
2. Application explains local storage, optional AI processing, sensitive-data handling, and manual-only survey interaction.
3. User acknowledges the explanation and continues.
4. Application initializes local storage.
5. User may configure an AI provider now or skip setup.
6. User reaches the dashboard. Profile and session features are available even if AI is not configured.

**Failure:** If local storage cannot initialize, show a blocking recoverable error; do not pretend that changes can be saved.

## Flow 2 — Create or maintain Digital Persona

1. User opens Digital Persona.
2. User creates a fact with category, attribute, value, source description, date/validity where known, sensitivity, and verification state.
3. User reviews the fact and explicitly marks it confirmed if accurate.
4. Application saves the fact and displays its current state.
5. User may edit, mark disputed/outdated, archive, or delete it later.
6. Any change that affects support eligibility is reflected in subsequent retrieval.

**Rule:** Only current, relevant confirmed facts can directly support factual recommendations.

## Flow 3 — Create survey session and enter a question

1. User creates a session and optionally enters a title/platform label.
2. User invokes the screen-capture shortcut and selects the question region/window, or uses manual text input.
3. OCR and visual analysis extract prompt, instructions, visible options, labels, and layout.
4. Application displays extracted content and uncertain regions for review.
5. User corrects any extraction error.
6. User explicitly requests analysis.

**Rule:** The application does not inspect the survey website or retrieve question text from a URL.

## Flow 4 — Analyze and recommend

1. Application classifies intent, answer format, language, timeframe, units, and constraints.
2. Application retrieves relevant current confirmed facts and relevant historical context.
3. Consistency checks evaluate freshness and material conflicts.
4. Application determines the minimum context needed for the AI request.
5. Sensitive facts remain excluded unless the user explicitly authorizes their use for this request.
6. Application sends the request to the configured provider.
7. Provider returns structured output.
8. Application validates schema, evidence status, and every cited source identifier against the local request allowlist.
9. Application displays the interpretation, answer or abstention, evidence, rationale, uncertainty, and warnings.

**Failure:** Provider/network/schema failure is shown as a recoverable error. Local records remain unchanged.

## Flow 5 — Insufficient evidence or user input required

1. Retrieval finds no adequate confirmed fact or the question requires the user's own opinion, intention, or experience.
2. Application returns insufficient_information or user_input_required.
3. Application asks one focused clarification question.
4. User may supply the missing information and explicitly save/confirm it as a profile fact, or answer only for the current question.
5. Application reruns analysis using the newly authorized information.

**Rule:** A response supplied for one question is not silently promoted into a permanent profile fact.

## Flow 6 — Conflicting evidence

1. Consistency checking detects materially incompatible facts or a conflict with a prior confirmed answer.
2. Application shows both records, values, dates, and sources.
3. User resolves, updates, disputes, or leaves the conflict unresolved.
4. While a material conflict remains unresolved, application does not produce a definitive factual recommendation.
5. The resolution is recorded and used in later analysis.

## Flow 7 — Review and act on a recommendation

1. User inspects the answer draft, evidence, status, and warnings.
2. User edits, confirms, rejects, or requests regeneration.
3. User edits are stored separately from the original AI recommendation.
4. User explicitly selects Copy to copy the approved text.
5. User manually switches to the external survey and decides what to do there.
6. Application records the review state and history.

**Rule:** Copy is the only transfer action in v1.0. There is no autofill or submit action.

## Flow 8 — Export or delete data

1. User opens Settings > Data.
2. User selects export, deletion of an individual record, or full local-data deletion.
3. Application explains the scope and consequences.
4. User explicitly confirms a destructive action.
5. Application performs the local operation and reports success or a recoverable failure.
6. Export output is validated and deletion is verified against local records.

## Flow 9 — Work without AI availability

1. User opens the application while AI is unconfigured or offline.
2. Application allows local profile and survey-session management.
3. If the user requests AI analysis, application explains that provider/network setup is required.
4. No local data is lost or blocked by the unavailable provider.


## Flow 10 — Compare survey opportunity across personas

1. User opens a survey listing or offer page.
2. User invokes capture for the visible offer and eligibility details.
3. Application extracts platform, topic, reward, stated duration, and explicit criteria.
4. Application compares each criterion with every active persona.
5. Application shows match, mismatch, unknown, stale, or conflict with supporting facts.
6. Application presents a transparent opportunity ranking and identifies missing information.
7. User selects the persona that truthfully represents the actual respondent and decides whether to start.

## Flow 11 — Track survey attempt and reward

1. User starts a survey attempt and selects the active persona.
2. Application starts an elapsed/active-time record with visible controls.
3. Screen Intelligence detects visible progress, completion, screening, or reward states when possible.
4. Application records each observation with source, timestamp, amount, unit, persona, and confidence.
5. Offered reward remains an offer; completion without credit remains unverified or pending.
6. Credited and withdrawable values are recorded only when supported by visible platform evidence or a permitted official integration.
7. User can correct an observation only when the application cannot reliably determine it; correction is recorded as user-supplied evidence.
8. Dashboard updates from the event ledger without requiring routine manual bookkeeping.

## Flow 12 — Learn persona-specific survey fit

1. Before an attempt, the application stores the criteria and persona facts available at that time.
2. After an observable outcome, the application attaches the outcome to that exact attempt and persona.
3. The learning system separates screened-out, quota-full, technical failure, quality rejection, completion, and reward-credit events.
4. The system updates historical category/platform/criteria statistics when evidence is sufficient.
5. Sparse data is labeled insufficient history; no confident prediction is shown.
6. Any predictive ranking is evaluated on later, chronologically held-out attempts and shows sample size and uncertainty.
7. The system uses the result to prioritize future suitable opportunities, never to alter personal facts or recommend deceptive answers.
