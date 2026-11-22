# Testing Strategy

## 1. Test layers

1. Unit: domain rules, classification, normalization, consistency, pool selection, cooldown, retry decisions.
2. Repository: SQLite CRUD, foreign keys, persona scoping, transactions, migrations, deletion.
3. Integration: capture-to-OCR pipeline and AI workflow with mocked providers.
4. Contract: Gemini/custom adapter request and response normalization; strict AI output schema.
5. Security: prompt injection, secret redaction, IPC validation, endpoint validation, SQL injection, filesystem scope.
6. UI: primary workflows, keyboard navigation, accessible labels, error states, persona switching.
7. Release: packaged startup, upgrade, uninstall, export, restore, Windows Credential Manager behavior.

## 2. Critical persona scenarios

- Persona A facts never appear in Persona B request.
- Session ownership remains unchanged when active persona switches.
- Unverified, disputed, outdated, and sensitive facts are filtered correctly.
- Conflicts prevent definitive recommendation.
- Historical answers are not automatically promoted to facts.

## 3. Critical screen scenarios

- Region and window capture.
- OCR misreads, clipped text, negation, numbers, units, and matrix layout.
- User correction is preserved.
- AI analysis cannot start before user confirmation.
- No background capture.
- Temporary screenshots are deleted according to retention policy.

## 4. Critical provider scenarios

- Several Gemini keys with adaptive and round-robin selection.
- 429 causes cooldown and failover.
- Retry-After is respected.
- 401/403 marks configuration as needing attention.
- All keys unavailable preserves the question and shows a retry action.
- Invalid request is not retried across every key.
- Cancellation stops queued retries.
- Custom HTTPS endpoint is validated.
- API keys never appear in logs, exports, or frontend bundles.

## 5. AI quality scenarios

No relevant facts; only unverified/outdated facts; conflicting facts; unknown source ID; unsupported claim; embedded prompt injection; ambiguous question; subjective opinion request; sensitive fact without consent; provider timeout; malformed JSON.

## 6. Test data and CI

Use synthetic personas and deterministic fixtures. Never commit real survey answers, API keys, or profile exports. Run format, lint, typecheck, unit, repository, contract, and security tests in CI. Provider-dependent tests must be mocked or explicitly opt-in.
