# User Flows

**Version:** 1.0 MVP  
**Normative source:** PRODUCT_DEFINITION.md

## Flow 1 — First launch

1. User launches the Windows 11 app.
2. App explains local storage, AI data transmission, sensitive-data handling, and manual survey interaction.
3. User creates the first persona.
4. User may configure Gemini now or later.
5. App opens Beranda.

## Flow 2 — Manage personas

1. User creates a persona for a real respondent.
2. User adds facts, source descriptions, dates, and sensitivity labels.
3. User confirms facts that are accurate.
4. User can edit, dispute, mark outdated, archive, or delete facts.
5. User selects the active persona before starting a survey session.

## Flow 3 — Read a survey question

1. User opens a survey in their browser.
2. User invokes screen capture and selects a region/window, or pastes question text.
3. App runs OCR and reconstructs prompt, instructions, options, units, and visible layout.
4. App shows the extracted question and uncertain text.
5. User corrects and confirms the content.

## Flow 4 — Get an answer recommendation

1. User selects/validates the active persona.
2. App classifies the question and extracts answer constraints.
3. App retrieves relevant confirmed facts from that persona only.
4. App checks missing, stale, sensitive, and conflicting facts.
5. App minimizes the context and sends it to the selected provider.
6. Provider returns structured output.
7. App validates schema and source IDs locally.
8. App shows the recommendation, evidence, explanation, and uncertainty—or asks a focused clarification.

## Flow 5 — Review answer

1. User edits, confirms, rejects, regenerates, or copies the recommendation.
2. App stores original recommendation and user-edited text separately.
3. User switches to the survey and manually enters the answer.
4. User manually submits the survey if they choose.

## Flow 6 — Compare survey fit

1. User captures the visible survey description and eligibility criteria.
2. App extracts explicit criteria.
3. App compares each criterion against each selected persona.
4. App shows match, mismatch, unknown, stale, or conflict with evidence.
5. User decides which survey and truthful respondent profile to use.

## Flow 7 — Configure API pool

1. User opens Settings > AI Provider.
2. User adds one or more Gemini API keys.
3. App stores each key in Windows Credential Manager and tests the configuration.
4. User may add a custom compatible provider, endpoint, and model.
5. App automatically selects an available key using the configured pool strategy.
6. On rate limit or transient failure, app cools down the key and tries another eligible configuration.
7. If all configurations fail, app preserves the question and offers retry.

## Flow 8 — Export or delete data

1. User opens Settings > Data.
2. User selects export or deletion.
3. App explains scope and consequences.
4. User confirms destructive actions.
5. App performs the operation and verifies the result.

## Flow 9 — Work without AI

1. User opens the app while providers are unavailable.
2. App still allows persona and history management.
3. Captured or pasted question content remains available locally.
4. User can retry when a provider becomes available.
