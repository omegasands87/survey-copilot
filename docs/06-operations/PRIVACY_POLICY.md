# Privacy Notice — Draft

**This is a product-planning draft, not a legally reviewed privacy policy.** It must be revised to match the actual implementation, jurisdictions, providers, and applicable law before public release.

## Information the application may store
The planned local application may store profile facts supplied by the user, source notes, survey questions, AI recommendations, user edits, answer history, preferences, and operational metadata.

## Storage and Retention Lifecycle
The application operates on a local-first architecture using a local SQLite database and restricted application directory.
- **Active Facts**: Retained locally in SQLite; only relevant, non-disputed facts are sent to external AI providers with user consent.
- **Archived Facts**: Kept locally but excluded from AI retrieval context.
- **Screen Captures**: Temporary screenshot files are processed locally and deleted immediately after OCR review unless explicitly saved.
- **Survey Memory & History**: Scoped strictly to the owning persona and preserved for survey recognition and audit.
- **Export Files**: JSON exports are generated strictly upon explicit user request and 100% exclude any API keys, provider credentials, or internal secrets.
- **Deletion Controls**: Full application wipe removes all user data records, unlinks temporary capture files, checkpoints WAL logs with TRUNCATE, and vacuums SQLite storage. Files exported outside the app directory and OS filesystem slack remain outside the app's scope of deletion.

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
