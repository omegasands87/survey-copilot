# Security Architecture

## 1. Objectives

Protect respondent data, persona separation, API credentials, screen captures, and the user's control over survey interactions.

## 2. Data classification

- Public: application version and public documentation.
- Operational: provider health, settings, and non-sensitive diagnostics.
- Personal: persona facts, questions, answer history.
- Sensitive: financial, health, precise location, government identifiers, and other high-impact attributes.

Do not collect credentials for survey websites. Sensitive facts are excluded from external AI context by default.

## 3. Local data

- Store SQLite database in the Windows application data directory.
- Apply restrictive filesystem permissions.
- Use parameterized SQL and foreign keys.
- Back up before migrations and destructive operations.
- Provide export and complete deletion.
- Do not store screenshots permanently by default.
- Temporary capture images must be deleted after processing unless explicitly saved by the user.

## 4. API credentials

- Store secrets natively in Windows Credential Manager (`Advapi32.dll` via `CredWriteW`, `CredReadW`, `CredDeleteW`, `CredFree` with `CRED_TYPE_GENERIC` and `CRED_PERSIST_LOCAL_MACHINE`).
- SQLite stores strictly opaque credential references formatted as `survey_copilot/cred/<id>` in table `app_credentials`.
- Never store API key values in SQLite, frontend state persistence, logs, crash reports, error strings, or exports (verified by automated tests).
- Mask keys after entry; frontend inputs use `type="password"` and immediately zero/clear the secret value from memory upon IPC submission.
- Shared credential protection: deleting a credential referenced by active provider configurations (`usageCount > 0`) is blocked unless explicitly confirmed by the operator (`force = true`).
- An in-memory test store (`InMemoryCredentialStore`) is provided exclusively for automated integration tests and non-Windows developer environments, with strict guards preventing use as production persistence.
- Do not send credentials to custom endpoints other than the endpoint configured for that provider.

## 5. External AI disclosure

Before sending content, identify the provider and disclose that the current question and selected relevant profile facts will be transmitted. Sensitive facts require explicit per-request authorization. Do not send the entire persona or database by default. Provider retention and training statements must reflect current provider terms; do not make unsupported privacy guarantees.

## 6. Screen capture

- Capture only after an explicit user action.
- User selects region or window.
- Show preview before OCR or external image processing.
- Never silently record or poll the screen.
- Do not include screenshots in logs or telemetry.
- External vision is optional and requires explicit consent for the selected image.
- Delete temporary image data according to the documented retention policy.

## 7. Prompt injection and untrusted input

Survey text, OCR output, screenshots, answer options, custom provider responses, and model output are untrusted. They cannot change app policy, request unrelated profile facts, authorize tools, or execute code. Escape content in the UI. Enforce source allowlists locally.

## 8. Custom endpoint restrictions

- HTTPS required by default.
- Allow HTTP only for localhost after explicit user confirmation.
- Reject non-HTTP schemes and invalid URL forms.
- Do not support provider plugins that execute arbitrary code in v1.0.
- Restrict redirects and validate destination behavior to reduce credential exfiltration risk.
- Show the configured destination clearly before saving.

## 9. Threats and mitigations

| Threat | Mitigation |
|---|---|
| API key leakage | Windows Credential Manager; redacted logs |
| Cross-persona leakage | persona-scoped queries and service validation |
| Prompt injection | untrusted-content boundary; local enforcement |
| Hallucinated facts | evidence contract; abstention; source validation |
| Malicious screenshot content | user preview; text-only rendering; no execution |
| Custom endpoint credential theft | HTTPS validation; explicit endpoint display |
| Rate-limit retry storm | bounded retries; cooldown; jitter |
| Unauthorized survey submission | no browser automation or submission capability |
| Local data theft | OS permissions; documented backup/deletion controls |

## 10. Native security

Use Tauri capability allowlists and least privilege. Validate every IPC input. No arbitrary shell execution, unrestricted filesystem access, or browser-control API. Review dependencies and lockfiles before release.
