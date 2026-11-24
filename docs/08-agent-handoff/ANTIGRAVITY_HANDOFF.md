# Antigravity Engineering Handoff

**Project:** Survey Copilot  
**Repository:** `omegasands87/survey-copilot`  
**Target:** Windows 11 x64  
**Document status:** Authoritative implementation handoff  
**Product baseline:** `docs/01-product/PRODUCT_DEFINITION.md`

## 1. Mission

Implement Survey Copilot v1.0 as a reliable, Indonesian-first Windows desktop application. The application reads survey questions from a user-selected screen region/window or pasted text, interprets the question, retrieves relevant facts from the selected real respondent profile, and recommends a truthful answer with evidence. The user remains responsible for all survey website interaction and submission.

This repository already contains a starter Tauri/React application, an initial SQLite migration, and basic persona/fact commands. It is not a finished product. Continue from the existing implementation; do not restart or replace the repository.

## 2. Source-of-truth order

If specifications appear inconsistent, resolve them in this order:

1. This handoff for agent workflow and implementation sequencing.
2. `docs/01-product/PRODUCT_DEFINITION.md` for product scope and owner decisions.
3. `docs/01-product/PRODUCT_REQUIREMENTS.md` for normative requirements.
4. Architecture contracts under `docs/02-architecture/`.
5. AI contracts under `docs/03-ai-engine/`.
6. UX and design documents under `docs/04-design/`.
7. Development and operations documents.
8. Existing code, which may be incomplete or contain starter placeholders.

Existing code does not override a product requirement. A lower-level document or code comment must not silently change product scope. When a genuine conflict is found, stop the affected work, document the conflict, and request a decision rather than guessing.

## 3. Confirmed product decisions

- Windows 11 x64 only for v1.0.
- Indonesian-first UI.
- Free personal-use product.
- One local operator can manage multiple distinct, real respondent profiles.
- A persona is a real person whose information the operator is authorized to manage; no fictional profiles or screener deception.
- Survey input: explicit user-selected screenshot capture or manual paste/type.
- AI provider: Gemini first; multiple Gemini API configurations; compatible custom provider/endpoint/model support through adapters.
- AI suggests; the user reviews, edits, copies, and manually enters/submits survey answers.
- Local SQLite persistence.
- API secrets in Windows Credential Manager.
- No earnings, reward, points, balance, payout, or income tracking in v1.0.
- No browser automation, DOM inspection, auto-click, auto-fill, navigation, auto-submit, CAPTCHA bypass, or circumvention of survey-platform controls.
- No silent/background screen capture.

## 4. Current repository state

### Baseline at initial handoff
At project inception, the repository contained a starter Tauri/React shell, unmigrated initial schema, and basic persona stubs without OCR, AI pooling, answer recommendation, or installer.

### Current implementation status (v0.1.0-beta Testing Release)
All planned milestones from **SC-001 through SC-061** have been implemented and verified by automated test suites. Milestone **SC-062** (Testing Release) is packaged and ready for manual user verification:
- **CI / CD Pipeline:** Fully green on Windows GitHub Actions runner (23/23 steps passed, including `npm ci`, `npm test`, `cargo fmt`, `cargo check`, and `cargo test`).
- **Database & Migrations:** Sequential, transactional migration runner (`Database::initialize`) managing migrations `0001` through `0013` with baseline adoption and foreign key enforcement.
- **Respondent Data & Isolation:** Multi-persona lifecycle, typed profile facts, source attribution, verification states, and strict cross-persona boundary isolation.
- **Security & Secrets:** Windows Credential Manager integration via `WindowsCredentialStore`, storing opaque credential references in SQLite. Zero API keys in plaintext or logs.
- **Provider Pooling:** Gemini and OpenAI-compatible provider adapters, bounded provider pool, automatic failover, cooldowns, and retry budgets.
- **Screen Intelligence:** Explicit user-invoked screen capture with preview dialog, in-memory capture codec, and local OCR layout reconstructor.
- **AI Recommendation Engine:** Question classifier, evidence engine, consistency checks, structured answer generator with abstention contracts, and audit review history.
- **Survey Opportunities & Fit Evaluation:** Opportunity text/OCR capture, criteria extractor, deterministic persona eligibility fit evaluator, and historical outcome learning.
- **Survey Memory & NQI:** Pattern recognition, session branching transitions, and Next Question Intelligence prediction.
- **Privacy & Diagnostics:** Outbound data transmission transparency modal, per-request consent checks, safe pre-redacted diagnostic logging (`LogRedactor`), and screenshot retention reclamation.
- **Accessibility & Polish:** WCAG 2.1 AA focus rings, keyboard dismiss (Escape), offline detection banner, responsive design down to 860px, and standard Indonesian copy.
- **Windows Release:** NSIS installer compiled (`release-testing/Survey Copilot_0.1.0_x64-setup.exe`, `installMode: "currentUser"`). Ready for manual verification per [Manual Testing Guide](05-development/MANUAL_TESTING_GUIDE.md).

For detailed granular criteria, refer to the [Implementation Backlog](IMPLEMENTATION_BACKLOG.md).

## 5. Architecture boundaries

### Frontend
React + TypeScript. Owns presentation, user interaction, and transient view state. It must not access SQLite directly or hold provider secrets. Use typed wrappers around Tauri commands; keep domain logic in testable modules rather than large React components.

### Native backend
Rust/Tauri. Owns database access, capture, OCR orchestration, Windows Credential Manager, provider HTTP calls, filesystem operations, and domain validation. Tauri commands should be thin adapters over service/repository layers.

### Persistence
SQLite in the app's local application-data directory. Use foreign keys, parameterized SQL, transactions, versioned migrations, UTC ISO-8601 timestamps, and explicit persona scoping. Every query that accesses persona-owned data must constrain the persona in the query itself and validate parent/child ownership in the service layer.

### AI provider boundary
The answer engine depends on a normalized provider interface, not Gemini SDK types. Provider adapters normalize requests, responses, and errors. Provider pool is bounded, observable, cancellation-aware, and respects provider account/project quotas and terms. Key rotation must not be used to evade account-level quota, billing, or policy limits.

### Screen boundary
Capture only after explicit user invocation. Show a preview. OCR and layout extraction are read-only. Do not expose commands that click, type, navigate, inspect browser DOM, or submit forms. Delete temporary image data according to the documented retention policy.

## 6. Required domain flow

1. User selects or confirms active persona.
2. User pastes question text or explicitly captures a screen region/window.
3. App extracts question, instructions, options, units, and layout.
4. User reviews/corrects and confirms extracted content.
5. App creates or selects a session owned by that persona.
6. Question classifier extracts intent, answer format, timeframe, units, constraints, and ambiguity.
7. Local retrieval selects only relevant, current, confirmed, non-disputed facts from that persona.
8. Sensitive facts are excluded by default and require explicit per-request consent before external transmission.
9. Consistency checks detect conflict, stale facts, and unsupported assumptions.
10. Provider adapter returns structured output.
11. Local validation checks schema, lengths, status semantics, and source IDs against the exact request allowlist.
12. Local evidence policy determines whether a draft can be shown or the app must ask/abstain.
13. User edits, confirms, rejects, regenerates, or copies.
14. Original AI output and user-edited text are stored separately. No survey action occurs automatically.

## 7. Persona and evidence invariants

- All persona-owned rows must have a direct or verifiable parent relationship to one persona.
- A session, question, attempt, fit result, fact, source, and history entry must never be silently reassigned to another persona.
- Switching active persona cannot change the persona ownership of an existing session.
- Retrieval must filter by persona ID in SQL and revalidate ownership in Rust.
- Only facts with appropriate verification state, freshness, relevance, and sensitivity consent may support a factual draft.
- Unverified, disputed, stale, and conflicting facts must not be treated as confirmed evidence.
- The model cannot add arbitrary fact IDs; local code validates each ID against the request-scoped allowlist.
- A previous survey answer is not automatically a verified fact.
- Subjective opinion/experience questions require the respondent's own input unless an explicit, current preference is stored.
- Model confidence is evidence confidence only, never survey acceptance probability.

## 8. Provider pool invariants

- Secrets are written to Windows Credential Manager by native code and never returned to the frontend.
- Database stores only opaque credential references.
- Never log secret values, authorization headers, raw profile context, screenshots, or full prompts.
- Distinguish invalid request, content refusal, auth failure, rate limit, quota exhaustion, transient network/server failure, timeout, and cancellation.
- Retry only retryable errors. Never retry malformed local input, invalid request, or refusal across every key.
- Bound attempts, concurrency, timeout, exponential backoff, and cooldown.
- Respect `Retry-After` when provided.
- Fail over between keys only when the provider's error semantics permit it.
- If provider indicates account/project-level quota or billing restriction, stop pool retries for that scope; do not rotate keys to evade it.
- Validate custom endpoint scheme and host; require HTTPS except explicit localhost development mode. Prevent credential forwarding across untrusted redirects.
- Preserve the user's question when all providers fail.

## 9. Screen capture and OCR invariants

- Capture must be directly initiated by the user and visible.
- Support region and selected-window capture on Windows 11.
- Preview the captured image before OCR/AI processing.
- Keep screenshot data temporary unless the user explicitly chooses to retain it for a documented purpose.
- OCR output must preserve reading order and distinguish question, instructions, options, scales, matrix rows/columns, and units when possible.
- Flag low-confidence or clipped text. Never silently invent missing characters or options.
- Require user confirmation before AI analysis.
- Manual paste/type must work without capture/OCR.
- No background capture, keylogging, browser DOM access, or survey interaction.

## 10. Implementation sequencing

Follow `IMPLEMENTATION_BACKLOG.md`. Implement vertical slices in this order:

1. Stabilize scaffold, lockfiles, CI, migration runner, and architecture layers.
2. Complete persona/fact/source lifecycle and isolation tests.
3. Implement Windows Credential Manager and provider configuration.
4. Implement Gemini adapter and provider pool.
5. Implement question input/session persistence and structured answer workflow.
6. Implement screen capture, OCR, preview, and correction.
7. Implement survey opportunity fit and outcome history.
8. Complete export/delete, privacy controls, settings, accessibility, packaging, and release validation.

If dependencies or platform constraints require a sequence change, explain it in a short ADR and update the backlog.

## 11. Build and test commands

From repository root:

```powershell
npm install
npm run build
npm run tauri dev
npm run tauri build
```

Rust checks:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Generate and commit lockfiles. Once `package-lock.json` exists, CI should use `npm ci` rather than `npm install`.

Do not claim the Windows installer works unless it has been built and launched on a clean Windows 11 x64 environment. If Antigravity's environment is not Windows, run platform-independent tests and clearly leave Windows-specific verification pending.

## 12. Required reporting

At the end of each implementation increment, report:
- Backlog item ID and scope.
- Files created/changed.
- User-visible behavior now implemented.
- Database migration impact.
- Exact test commands run and their results.
- Tests not run and why.
- Security/privacy implications.
- Known limitations.
- Next unblocked backlog item.

## 13. Definition of release readiness

Release readiness requires all MUST requirements in the Product Requirements, all release-blocking acceptance tests, successful Windows x64 build and clean-machine install/upgrade/uninstall tests, no critical/high unresolved security findings, no cross-persona leakage, and verified manual-only survey interaction.
