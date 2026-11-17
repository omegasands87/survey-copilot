# Survey Copilot Technical Blueprint

Complete product and engineering specification will be maintained in this document.

## Product
Desktop AI copilot for understanding survey questions and recommending evidence-backed answers from user-confirmed facts. Users review, edit, confirm, and copy answers manually. No auto-fill, auto-submit, CAPTCHA bypass, or fabricated personal information.

## Stack
- Tauri 2, Rust
- React, TypeScript, Tailwind CSS, shadcn/ui
- SQLite, migrations, repositories, FTS5
- OpenAI API with structured output and Zod validation
- Zustand, TanStack Query
- Tesseract OCR (optional P1)

## Core modules
1. Dashboard
2. Digital Persona and fact verification
3. Survey Workspace
4. Question Classifier
5. Profile and Answer Memory retrieval
6. AI Answer Engine
7. Consistency Engine
8. Answer History
9. Settings and privacy
10. Screenshot OCR (P1)

## Architecture
Local-first desktop application. Profile facts, survey sessions, questions, recommendations, and answer history are stored locally in SQLite. The AI request sends only relevant, permitted context over HTTPS. API credentials belong in the operating-system credential store, never in source code or frontend bundles.

## Data model
Core tables: users, profile_facts, fact_sources, user_preferences, platform_accounts, survey_sessions, survey_questions, answer_recommendations, answer_history, answer_sources, ai_usage_logs, app_settings. Use UUID text primary keys, ISO-8601 timestamps, foreign keys, versioned migrations, and deletion policies. Profile facts have category, field_key, JSON value, confidence, verification_status (unverified/confirmed/disputed/outdated), sensitivity, and validity dates. Historical answers are not automatically promoted to profile facts.

## AI workflow
1. Parse user-entered question and options.
2. Classify question type and answer constraints.
3. Retrieve only relevant confirmed facts and relevant answer history.
4. Check conflicts, freshness, and sensitivity permissions.
5. If evidence is missing or conflicting, ask the user for clarification.
6. Generate structured recommendation with answer, evidence status, confidence, reasoning, source IDs, and warnings.
7. Validate schema and verify source IDs against local records.
8. Display recommendation for user review; user may edit, confirm, copy, or discard.

Supported question types include demographic, employment, financial, behavioral, brand awareness/usage, preference, opinion, purchase intent, attention check, conditional, and open-ended. Supported answer formats include single/multiple choice, dropdown, rating/Likert, numeric, date, ranking, and free text.

## Digital Persona rules
Only confirmed facts are eligible as factual evidence by default. Unverified, disputed, and outdated facts require clarification. Sensitive facts are excluded from cloud requests unless explicitly needed and authorized. Never infer sensitive attributes from proxies or fabricate personal experiences.

## Recommendation output
Structured fields: question_type, recommended_answer (nullable), confidence (0..1), evidence_status, reasoning, sources, consistency_check, warnings, requires_user_confirmation, clarification_question. Evidence statuses: supported, partially_supported, insufficient_information, conflicting_evidence, user_input_required, not_applicable. Confidence is evidence confidence, not probability of survey acceptance.

## Consistency and memory
Maintain profile memory, answer memory, and session context separately. Use exact field matching, SQLite FTS5, category/status filters, and recency rules. Conflicts trigger a clarification prompt; never silently choose or rewrite a fact. Add embeddings only if measured retrieval quality requires them.

## Desktop UX
Dashboard; Digital Persona CRUD and verification; Copilot workspace with question/options center panel and evidence/recommendation panel; session manager; answer history; consistency warnings; settings for AI provider, privacy, export, and deletion. OCR import is user-initiated and requires correction/review before AI processing.

## Security and privacy
- TLS for provider requests.
- OS credential store for API secrets.
- Minimize context sent to AI and request consent for sensitive data.
- No sensitive prompt or answer logging by default.
- Tauri capabilities and filesystem permissions are least-privilege.
- Validate all input/output and treat survey text as untrusted data (prompt-injection defense).
- Provide data export and deletion, including attachments.
- Do not store survey platform passwords.
- No automatic form filling/submission, CAPTCHA solving, or platform-protection bypass.

## MVP scope
P0: desktop shell, SQLite, Digital Persona CRUD/verification, survey sessions, manual question input, classifier, profile retrieval, structured recommendation, evidence display, consistency checks, edit/confirm/copy, history, API credential handling, privacy controls.
P1: screenshot OCR, analytics, advanced search, export improvements, macOS/Linux support.
Out of scope: browser automation, auto-fill/submit, cloud sync, multi-user workspace, autonomous agent, vector database before FTS5 evaluation.

## Testing and acceptance
Unit tests for classifier, retrieval, validators, and consistency logic; integration tests for repositories, migrations, and AI orchestration; UI tests for profile/session/review flows; security tests for secret leakage and prompt injection; AI evaluation on synthetic or properly anonymized labeled examples. Initial targets: classification accuracy >=95%, structured output validity >=99%, evidence attribution >=95%, contradiction recall >=90%, unsupported answer rate <=2%. Define dataset and measurement protocol before claiming these metrics.

## Roadmap
Week 1 foundation and SQLite; week 2 Digital Persona; week 3 survey workspace/history; week 4 AI integration/classifier/retrieval; week 5 memory/consistency; week 6 Copilot UI; week 7 OCR/security; week 8 QA and Windows packaging. Estimates assume an experienced full-stack developer and are not guarantees.

## Release checklist
- [ ] Desktop app installs and launches.
- [ ] Profile facts can be created, edited, verified, and deleted.
- [ ] Survey sessions and manual questions persist.
- [ ] AI classifies and recommends using relevant confirmed facts.
- [ ] Missing/conflicting evidence triggers clarification.
- [ ] Source references are validated locally.
- [ ] User can edit, confirm, copy, and review history.
- [ ] Secrets are stored outside source/frontend bundles.
- [ ] Data export/deletion works.
- [ ] No auto-fill or auto-submit exists.
- [ ] Core tests and installer tests pass.

This document is the baseline product and engineering specification. Record material architectural changes as Architecture Decision Records (ADRs).
