# ADR-001: Use Tauri 2 for the Desktop Shell

**Status:** Proposed

## Context
Survey Copilot is a desktop application with local database and OS credential-store needs. It requires a manageable native boundary without a browser automation stack.

## Decision
Adopt Tauri 2 as the proposed desktop shell, with Rust for native commands and React/TypeScript for the frontend.

## Consequences
**Positive:** native integration, constrained command boundary, and relatively lean packaging.  
**Trade-offs:** Rust toolchain complexity, platform-specific packaging, WebView differences, and careful capability configuration.

## Revisit if
Required platform support, accessibility, packaging, or native integrations cannot be delivered reliably with Tauri 2.
