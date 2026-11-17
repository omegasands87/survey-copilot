# Privacy Notice — Draft

**This is a product-planning draft, not a legally reviewed privacy policy.** It must be revised to match the actual implementation, jurisdictions, providers, and applicable law before public release.

## Information the application may store
The planned local application may store profile facts supplied by the user, source notes, survey questions, AI recommendations, user edits, answer history, preferences, and operational metadata.

## Storage
The design is local-first and plans to use a local SQLite database. Exact storage paths, backup behavior, encryption, and deletion semantics must be documented after implementation.

## AI provider requests
If the user configures an AI provider and requests analysis, the application may send the survey question and a minimized subset of relevant profile facts to that provider. The full profile should not be sent by default. Sensitive information requires explicit authorization and a justified need.

The provider's own privacy, retention, and training terms apply to information sent to it. Users should review those terms before configuration. Do not assume providers handle data identically.

## Credentials
Provider API keys should be stored using the operating system credential facility and must not be committed, logged, or stored in plain-text settings.

## User controls and security
The intended product provides controls to inspect, edit, export, and delete local data, and remove provider credentials. The final policy must explain limitations involving backups, diagnostics, and provider-side retention.

The project intends to use least privilege, input validation, redacted diagnostics, and secure secret storage. No software can guarantee absolute security.

## Legal review
Add a maintained security/privacy contact and jurisdiction-specific legal review before release. Replace this draft with the final policy URL and effective date.
