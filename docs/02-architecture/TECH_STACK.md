# Technology Stack

**Target:** Windows 11 x64

| Layer | Choice | Notes |
|---|---|---|
| Desktop shell | Tauri 2 | Native desktop packaging |
| Native backend | Rust | Capture, database, secrets, provider dispatch |
| Frontend | React + TypeScript | UI and workflow state |
| Styling | Tailwind CSS | Consistent visual system |
| UI primitives | shadcn/ui or equivalent accessible primitives | Keyboard and contrast review required |
| Database | SQLite | Local-first with migrations |
| Validation | Zod + Rust validation | Validate at UI and native boundaries |
| State | Zustand or equivalent lightweight store | Avoid unnecessary global state |
| AI provider | Gemini adapter first | Multiple keys via provider pool |
| Extensibility | Provider adapter interface | Custom compatible endpoints and models |
| Secrets | Windows Credential Manager | No plaintext API keys |
| OCR | Windows OCR or Tesseract after evaluation | Select based on accuracy, language coverage, packaging, and license |
| Screen capture | Windows capture APIs through Rust/Tauri | Explicit user-initiated only |
| Tests | Vitest/React Testing Library + Rust + integration tests | Include Windows installer smoke tests |

## Engineering constraints

- Pin dependencies and commit lockfiles.
- Use maintained stable releases at project bootstrap.
- Keep provider SDKs behind adapters.
- Keep database access in Rust/repository services.
- Do not add a vector database before measured need.
- Do not add browser automation or form-filling dependencies.
- Do not add earnings/reward tracking in v1.0.
