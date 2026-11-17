# Troubleshooting

The application is not yet implemented. This document defines intended support guidance and will be updated with verified UI labels and error codes.

## AI provider unavailable
Check network connectivity and provider status. Confirm provider/model configuration. Re-enter the credential through Settings; do not paste it into survey fields or issue reports. Retry once after a transient failure. Continue manually if the provider remains unavailable.

## Recommendation says information is missing
This is expected when no current confirmed fact supports the question. Add or verify the relevant fact only if accurate. Do not invent a value to obtain a draft.

## Conflicting information
Review values, dates, units, and sources. Update the profile only after determining which value is correct for the requested time period.

## Database or migration error
Stop repeated retries if data integrity may be affected. Preserve the database file, avoid manual edits, and use the documented backup/recovery procedure once implemented. Redact personal data before sharing diagnostics.

## Report a bug
Include app version, OS version, reproduction steps, expected behavior, and redacted error output. Never attach API keys, full profile exports, or private survey content unless a secure support process explicitly requires it.
