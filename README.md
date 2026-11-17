# Survey Copilot

Desktop AI copilot for understanding survey questions and recommending accurate, evidence-backed answers from user-confirmed profile information.

- **Product:** AI Copilot
- **Platform:** Desktop (Tauri 2)
- **Interaction:** Manual assistance — review, edit, confirm, and copy
- **Architecture:** Local-first with optional AI cloud integration

## Product principles

- Recommendations are grounded in user-confirmed facts.
- The assistant discloses uncertainty and asks for clarification when evidence is missing.
- Users retain control over every answer.
- Profile and answer history are local by default.
- No automatic form filling/submission, CAPTCHA bypass, or fabricated personal information.

## Documentation

Read the [complete technical blueprint](docs/BLUEPRINT.md) for product scope, architecture, data model, AI workflow, Digital Persona, memory and consistency, desktop UX, security, testing, MVP scope, roadmap, and release checklist.

## Planned stack

Tauri 2, Rust, React, TypeScript, Tailwind CSS, shadcn/ui, SQLite, OpenAI API, Zod, Zustand, TanStack Query, SQLite FTS5, and optional Tesseract OCR.

## Development status

The repository currently contains the product and engineering blueprint. Application implementation has not yet been added.

## Initial implementation sequence

1. Initialize Tauri 2 + React + TypeScript.
2. Set up linting, formatting, tests, and CI.
3. Implement SQLite migrations and repositories.
4. Build Digital Persona CRUD and verification.
5. Build Survey Workspace and answer history.
6. Integrate the AI client and structured Answer Engine.
7. Add evidence display, consistency checks, and user review.
8. Add OCR, privacy controls, and release packaging.

See [docs/BLUEPRINT.md](docs/BLUEPRINT.md) for full details.
