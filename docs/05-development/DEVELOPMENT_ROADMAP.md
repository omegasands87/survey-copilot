# Development Roadmap

**Planning baseline:** 8 weeks; estimates require validation after engineering setup.

| Week | Focus | Deliverable |
|---|---|---|
| 1 | Foundation | Tauri app shell, React/TS setup, linting, CI, SQLite migration framework |
| 2 | Digital Persona | Fact CRUD, source metadata, verification states |
| 3 | Survey workspace and history | Session/question management, history model |
| 4 | AI workflow | Classifier, provider adapter, structured response validation |
| 5 | Evidence and consistency | Retrieval filters, source validation, conflict handling |
| 6 | Copilot UX | Review/edit/confirm/copy/reject interactions |
| 7 | Privacy and hardening | Credential storage, data export/delete, threat tests |
| 8 | QA and release prep | Regression, accessibility, packaging, documentation |

## Milestones
- **M1 Foundation:** app launches and persists local data.
- **M2 Persona:** facts and provenance can be managed.
- **M3 Grounded answer:** recommendation is schema-validated and evidence-linked.
- **M4 Review workflow:** user actions are explicit and recorded.
- **M5 Release candidate:** privacy, security, testing, and packaging gates pass.

## Dependencies and risks
AI provider selection, Windows credential integration, packaging/signing, and encryption decisions may change the schedule. Do not compress security and data-integrity work to preserve a calendar date.
