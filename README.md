# Survey Copilot

Desktop survey earning copilot for reading survey screens, matching opportunities to multiple truthful personas, recommending evidence-backed answers, and tracking observed survey outcomes and rewards.

- **Product:** AI Copilot
- **Platform:** Windows 11 desktop (v1.0 target)
- **Interaction:** User-initiated screen capture/OCR and visual analysis; user reviews and controls survey actions
- **Core intelligence:** Multi-persona matching, question/quality analysis, earnings tracking, and outcome-based fit learning
- **Architecture direction:** Local-first with optional external AI processing

## Product contract

Survey Copilot helps a user evaluate survey opportunities, read visible survey content through selected screen capture and OCR/vision, select the truthful respondent persona, prepare evidence-grounded answers, and track observed outcomes and rewards.

- Multiple real-personas are supported, with strict isolation of facts and histories.
- Recommendations must be grounded in traceable user-provided evidence.
- Missing, stale, or conflicting evidence must lead to clarification or abstention.
- Screen capture is user-initiated; extracted content is verified before analysis.
- Survey fit is learned from attributed, observed outcomes, with sample size and uncertainty.
- Offered, pending, credited, and withdrawable rewards are tracked separately.
- Users retain control over survey interactions and final submission in v1.0.
- Profile and answer history are local by default.
- Sensitive facts are excluded from external AI requests unless explicitly authorized for that request.
- No browser automation, autofill, auto-submit, CAPTCHA bypass, eligibility manipulation, or fabricated personal information.

## Documentation

Start with the [documentation index](docs/00-INDEX.md).

**The authoritative product baseline is [Product Definition v1.0](docs/01-product/PRODUCT_DEFINITION.md).** Product requirements, feature specifications, and user flows derive from that document. Technical and implementation documents must conform to the product contract and may not silently expand its scope.

The product documentation also specifies Screen Intelligence, Multi-Persona management, Earnings Intelligence, and Survey Fit Learning. The remaining documentation covers architecture, AI engine, UI/UX, development, testing, operations, and architecture decisions.

## Planned technical direction

Tauri 2, Rust, React, TypeScript, Tailwind CSS, shadcn/ui, SQLite, an AI provider adapter, Zod, Zustand, TanStack Query, and optional SQLite FTS5. These are technical direction documents, not a statement that implementation has started or that every architecture decision is final.

## Development status

This repository currently contains product and engineering documentation. The desktop application source code has not yet been added. Documents marked Proposed or Draft describe intended behavior, not shipped functionality. The privacy notice is not a legally reviewed policy.

## Build governance

The project will not enter implementation until the product baseline is approved and the subsequent architecture and implementation specifications are complete, internally consistent, and reviewed. Changes after baseline approval require documented impact analysis and explicit approval.
