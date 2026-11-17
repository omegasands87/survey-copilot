# Technology Stack

**Status:** Proposed; confirm compatibility and versions at implementation time.

| Layer | Proposed choice | Rationale / constraints |
|---|---|---|
| Desktop shell | Tauri 2 | Native desktop packaging with a constrained Rust boundary |
| Native backend | Rust | OS integration, secure commands, SQLite access |
| Frontend | React + TypeScript | Mature component ecosystem and typed UI |
| Styling | Tailwind CSS | Consistent utility-based styling |
| UI primitives | shadcn/ui | Accessible primitives; audit keyboard behavior |
| Database | SQLite | Local-first, portable, transactional |
| Validation | Zod + Rust validation | Validate at both UI and native boundaries |
| UI state | Zustand | Lightweight local state |
| Async state | TanStack Query | Request lifecycle and caching where appropriate |
| Search | SQLite FTS5 | Optional local full-text search |
| Charts | Recharts | Only if usage analytics are added |
| OCR | Tesseract or evaluated provider | Deferred; quality and privacy evaluation required |

## Engineering constraints
- Pin dependencies and commit lockfiles.
- Use supported stable releases at project bootstrap; avoid copying version numbers from stale docs.
- Keep native commands small and typed.
- Keep database access in Rust/repository services, not UI components.
- Keep provider SDK behind a replaceable adapter.
- Do not add a vector database until measured retrieval needs justify it.
- Do not add browser automation, autofill, or auto-submit packages.
