# Memory System

## 1. Memory categories
- **Profile facts:** user-maintained statements about the user, with verification and provenance.
- **Answer history:** drafts, edits, confirmations, and rejections tied to specific survey questions.
- **Session context:** temporary information relevant to the current survey session.

These categories must remain distinct.

## 2. Authority order
For factual claims: current confirmed profile facts with provenance take precedence. A prior answer is not automatically a fact. User-provided clarification may update a fact only after explicit review and confirmation.

## 3. Lifecycle
- Create: user enters a fact and optionally records source.
- Verify: user explicitly marks it confirmed.
- Review: app flags age, expiry, or context-dependent staleness.
- Update: preserve audit-relevant history where appropriate.
- Archive/delete: user controls retention and removal.

## 4. Retrieval
Retrieve by semantic relevance and structured attributes. Filter by verification status, validity dates, sensitivity, and consent. Return the smallest sufficient set. FTS5 can support lexical search; semantic/vector retrieval is not an MVP dependency.

## 5. Retention
Define retention for sessions, answer history, logs, exports, and backups. Provide user-visible deletion. Deletion must address derived indexes and references, not just hide a record.

## 6. No silent learning
The app must not infer or promote a new personal fact from a survey response, model suggestion, or copied answer without the user's explicit review and confirmation.
