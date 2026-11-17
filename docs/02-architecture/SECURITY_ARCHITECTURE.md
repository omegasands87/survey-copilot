# Security Architecture

## 1. Security objectives
Protect personal profile information, provider credentials, local database integrity, and the user's control over external actions.

## 2. Data classification
- **Public:** application version, public documentation.
- **Internal:** non-sensitive operational preferences.
- **Personal:** profile facts, survey questions, answer history.
- **Sensitive personal:** financial, health, precise location, government identifiers, or other high-impact attributes.

Sensitive data should be excluded from AI context unless essential, explicitly authorized, and supported by a documented use case. Avoid collecting government identifiers or credentials.

## 3. Local storage
- Store database in the OS-appropriate application data directory.
- Apply restrictive filesystem permissions where supported.
- Use parameterized SQL and enable SQLite foreign keys.
- Back up before migrations and destructive operations.
- Define retention and deletion semantics for database, logs, exports, and backups.

## 4. Secrets
- Store provider API keys in Windows Credential Manager, macOS Keychain, or an appropriate Linux secret service.
- Never hardcode keys, commit them, expose them to browser JavaScript, or include them in logs/crash reports.
- Provide explicit remove/replace controls.
- Do not sync secrets.

## 5. Network and AI disclosure
Before a request, make clear that selected question text and a minimized set of relevant facts may be sent to the configured provider. Provide a way to inspect the context where practical. Do not transmit the full profile by default. Document provider retention and training settings based on the provider's current terms; do not make unsupported privacy promises.

## 6. Prompt injection and untrusted content
Treat survey questions, pasted instructions, OCR text, and model output as untrusted data. They cannot change system policy, request secrets, authorize tool use, or override evidence rules. Keep instructions separate from quoted survey content. Do not expose unrelated profile facts in response to embedded requests.

## 7. Native application security
- Apply Tauri capability allowlists and least privilege.
- Validate all IPC command inputs and outputs.
- Restrict filesystem access to required paths.
- Avoid arbitrary shell execution and unrestricted navigation.
- Review dependencies and update vulnerable packages.

## 8. Threat scenarios
| Threat | Mitigation |
|---|---|
| API key leakage | OS credential store; redacted logs |
| Prompt injection | Untrusted-content boundary; no authority transfer |
| Hallucinated personal data | Evidence contract; abstention; source validation |
| Malicious survey text | Render as text; no HTML execution |
| Local data theft | OS permissions; encryption decision; user guidance |
| SQL injection | Parameterized queries |
| Unauthorized form submission | No browser automation or submission capability |
| Dependency compromise | Lockfiles, audits, minimal dependencies |

## 9. Incident response
Provide a way to disable provider access, remove credentials, export/delete local data, and report security issues. Document supported versions and a process for communicating material vulnerabilities.
