# Configuration

## AI provider setup

Gemini is the default provider. The user can add multiple API keys. Each key is stored in Windows Credential Manager; the database stores only an opaque credential reference.

Basic setup:
1. Add Gemini API key.
2. Choose model.
3. Test connection.
4. Enable the key.

The pool automatically selects an eligible key and fails over when a key is rate-limited or temporarily unavailable.

## Pool settings

Advanced settings may configure:
- Strategy: adaptive, round-robin, weighted, priority.
- Maximum attempts.
- Retry and timeout.
- Cooldown and concurrency limits.
- Per-key priority and weight.

Defaults should be suitable for normal use. Retry is bounded and cancellation must stop pending attempts.

## Custom providers

The user may add a custom provider name, HTTPS endpoint, adapter type, model ID, authentication header, and timeout. The selected adapter must support the request/response contract. A connection test is required before enabling the configuration. Unsupported protocols are rejected.

## Privacy controls

Before analysis, explain which provider receives the current question and selected relevant facts. Sensitive facts require explicit per-request authorization. Do not send the full persona by default.

## Local data

Show the database location safely. Provide export and delete controls. Do not advise manual database edits while the app is running.

## Diagnostics

Include app version, OS version, provider configuration ID, operation category, and redacted error codes. Exclude raw survey text, profile values, screenshots, and credentials by default.
