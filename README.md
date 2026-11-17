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

Start with the [documentation index](docs/00-INDEX.md). The documentation set covers:

- Product requirements, feature scope, personas, and user flows.
- System architecture, database schema, AI architecture, security, and technology stack.
- AI prompt contract, question classification, answer generation, memory, consistency, and evaluation.
- UI/UX specification, design system, and desktop layout.
- Development roadmap, implementation plan, coding standards, testing, and release planning.
- Installation, configuration, draft privacy notice, troubleshooting, and architecture decision records.

The original [technical blueprint](docs/BLUEPRINT.md) remains the high-level overview.

## Planned stack

Tauri 2, Rust, React, TypeScript, Tailwind CSS, shadcn/ui, SQLite, an AI provider adapter, Zod, Zustand, TanStack Query, optional SQLite FTS5, and deferred OCR.

## Development status

This repository currently contains planning and engineering documentation. The desktop application source code has not yet been added. Documents marked **Proposed** or **Draft** describe intended behavior, not shipped functionality. The privacy notice is not a legally reviewed policy.

## Initial implementation sequence

1. Initialize Tauri 2 + React + TypeScript.
2. Set up linting, formatting, tests, and CI.
3. Implement SQLite migrations and repositories.
4. Build Digital Persona CRUD and verification.
5. Build Survey Workspace and answer history.
6. Integrate the AI provider adapter and structured Answer Engine.
7. Add evidence display, consistency checks, and user review.
8. Add privacy controls, security hardening, and release packaging.

See the [development roadmap](docs/05-development/DEVELOPMENT_ROADMAP.md) and [implementation plan](docs/05-development/IMPLEMENTATION_PLAN.md).
