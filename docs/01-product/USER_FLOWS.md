# User Flows

## Flow 1 — First launch
1. User opens the desktop app.
2. App explains local storage and the optional AI network request.
3. User chooses whether to configure an AI provider now or use local profile tools first.
4. User creates profile facts and marks each fact's verification status.
5. App shows an empty dashboard with a clear next step.

## Flow 2 — Add a survey question
1. User creates a survey session.
2. User manually pastes or types a question.
3. App displays the captured text for review.
4. User requests analysis.
5. App classifies intent and extracts answer constraints.

## Flow 3 — Generate a grounded recommendation
1. App retrieves relevant confirmed facts only.
2. Consistency service checks current facts and relevant confirmed history.
3. App sends the minimum necessary context to the configured AI provider after clear disclosure.
4. Provider returns structured output.
5. App validates schema, status, and source references.
6. App displays draft, rationale, supporting facts, and unresolved issues.
7. User edits, confirms, rejects, or copies.

## Flow 4 — Insufficient information
1. Retrieval finds no adequate confirmed fact.
2. App returns `insufficient_information` or `user_input_required`.
3. App asks one targeted clarification.
4. User may add a fact and its source.
5. App reruns the recommendation using the updated profile.

## Flow 5 — Conflicting information
1. Consistency service detects two incompatible facts or an answer/profile conflict.
2. App displays both values, dates, and sources.
3. User resolves, updates, or leaves the conflict unresolved.
4. No definitive recommendation is generated while the conflict is material.

## Flow 6 — Export or delete
1. User opens Settings > Data.
2. App explains scope and consequences.
3. User confirms export or deletion.
4. App performs the operation locally and reports completion or a recoverable error.
