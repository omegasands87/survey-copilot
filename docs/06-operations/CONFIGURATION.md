# Configuration

## AI provider
Provider and model are user-configurable once support is implemented. Store API keys in the OS credential store. Show the selected provider/model, never display or log the saved secret, and remove/replace credentials through explicit controls. Removing a credential disables provider requests until another is configured.

## Privacy controls
Explain whether question text and relevant profile facts may be sent to the provider. Sensitive facts require explicit per-request authorization. Local-only features should remain usable where possible without an API key.

## Local data
Show the database location safely. Provide export and delete controls with clear explanations. Do not advise manual database edits while the app is running.

## Diagnostics
Include app version, OS version, operation category, and redacted error codes. Exclude raw survey text, profile values, and credentials by default.
