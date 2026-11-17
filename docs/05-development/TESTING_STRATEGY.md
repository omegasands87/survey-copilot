# Testing Strategy

## Test layers
1. Unit: domain rules, validators, classifiers, normalization, status transitions.
2. Repository: SQLite CRUD, foreign keys, transactions, migrations, deletion.
3. Integration: AI pipeline with mocked provider responses.
4. Contract: provider adapter and structured output schemas.
5. Security: prompt injection, secret redaction, IPC validation, SQL injection, filesystem scope.
6. UI: primary flows, keyboard navigation, accessible labels, error states.
7. Release: packaged startup, upgrade, uninstall, export and restore.

## Critical scenarios
No relevant confirmed facts; only unverified/outdated facts; conflicting facts; model cites an unknown ID; model emits unsupported claims; survey text asks to reveal unrelated facts; provider timeout/malformed JSON; user deletes a cited fact; app closes during write or migration.

## Test data
Use synthetic personas and deterministic fixtures. Never commit real survey answers, API keys, or profile exports.

## Automation
Run format, lint, typecheck, unit, repository, and security tests in CI. Keep provider-dependent tests mocked or opt-in to avoid unpredictable costs.
