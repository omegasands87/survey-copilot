# Survey Copilot

Desktop AI copilot for understanding survey questions and recommending accurate, evidence-backed answers from user-confirmed profile information.

- **Product:** AI Copilot
- **Platform:** Windows 11 desktop (v1.0 target)
- **Interaction:** Manual assistance — review, edit, confirm, and copy
- **Architecture direction:** Local-first with optional external AI processing

## Product contract

Survey Copilot helps one person understand manually supplied survey questions and prepare truthful answers from information they have explicitly provided or confirmed.

- Recommendations must be grounded in traceable user-provided evidence.
- Missing, stale, or conflicting evidence must lead to clarification or abstention.
- Users retain control over every answer and manually transfer it to the external survey.
- Profile and answer history are local by default.
- Sensitive facts are excluded from external AI requests unless explicitly authorized for that request.
- No browser automation, autofill, auto-submit, CAPTCHA bypass, eligibility manipulation, or fabricated personal information.

## Documentation

Start with the [documentation index](docs/00-INDEX.md).

**The authoritative product baseline is [Product Definition v1.0](docs/01-product/PRODUCT_DEFINITION.md).** Product requirements, feature specifications, and user flows derive from that document. Technical and implementation documents must conform to the product contract and may not silently expand its scope.

The documentation set also includes architecture, AI-engine, UI/UX, development, testing, operations, and architecture-decision documents.

## Planned technical direction

Tauri 2, Rust, React, TypeScript, Tailwind CSS, shadcn/ui, SQLite, an AI provider adapter, Zod, Zustand, TanStack Query, and optional SQLite FTS5. These are technical direction documents, not a statement that implementation has started or that every architecture decision is final.

## Development status

This repository currently contains product and engineering documentation. The desktop application source code has not yet been added. Documents marked Proposed or Draft describe intended behavior, not shipped functionality. The privacy notice is not a legally reviewed policy.

## Build governance

The project will not enter implementation until the product baseline is approved and the subsequent architecture and implementation specifications are complete, internally consistent, and reviewed. Changes after baseline approval require documented impact analysis and explicit approval.
