# Coding Standards

## TypeScript and React
Enable strict TypeScript. Avoid `any`; use discriminated unions for domain states. Keep components focused and business logic in services/domain modules. Validate external data at runtime. Never place API keys or privileged secrets in frontend code.

## Rust
Use explicit error types; avoid panics in normal paths. Validate all command arguments. Keep Tauri commands narrow and delegate to services. Avoid shell execution and broad filesystem permissions. Use maintained crates.

## Database
Parameterize every query. Use transactions for multi-step mutations. Add immutable migrations and tests. Enforce foreign keys and relevant uniqueness constraints.

## AI integration
Treat provider output as untrusted. Use strict schemas and validate source IDs locally. Do not log raw personal context or secrets by default. Version and regression-test prompts. No model output directly triggers privileged actions.

## Git workflow
Use focused branches and pull requests. Include tests and documentation updates. Keep commits descriptive. Never commit local databases, credentials, build artifacts, or user data.

## Naming
Use descriptive domain names such as `profileFact`, `evidenceStatus`, and `surveyQuestion` rather than ambiguous abbreviations.
