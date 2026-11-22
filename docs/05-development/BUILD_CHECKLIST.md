# Build and Release Checklist

## Foundation
- [ ] Tauri 2 / Rust / React / TypeScript scaffold committed
- [ ] CI format, lint, typecheck, Rust check, and tests
- [ ] SQLite migrations and backup/restore
- [ ] Typed Tauri command layer and capability allowlist
- [ ] Windows Credential Manager integration

## Personas
- [ ] Multiple persona CRUD
- [ ] Fact/source lifecycle
- [ ] Strict persona isolation tests
- [ ] Sensitive-data consent
- [ ] Export and deletion

## Screen Intelligence
- [ ] Region capture
- [ ] Window capture
- [ ] OCR
- [ ] Question/option/layout reconstruction
- [ ] Preview and correction
- [ ] Manual text fallback
- [ ] Temporary screenshot deletion
- [ ] No background capture

## AI Provider Hub
- [ ] Gemini adapter
- [ ] Multiple API keys per provider
- [ ] Adaptive pool
- [ ] Round-robin, weighted, priority
- [ ] 429 cooldown and failover
- [ ] Bounded retries and cancellation
- [ ] Custom compatible endpoint/model
- [ ] Key redaction and secure deletion

## Answer Copilot
- [ ] Question classifier
- [ ] Fact retrieval
- [ ] Consistency engine
- [ ] Structured output validation
- [ ] Source ID allowlist validation
- [ ] Edit/copy/confirm/reject/regenerate
- [ ] History

## Survey Fit
- [ ] Visible criteria extraction
- [ ] Deterministic match/mismatch/unknown/stale/conflict
- [ ] Transparent explanation
- [ ] Historical fit learning with sample size and uncertainty
- [ ] No deceptive optimization

## Release
- [ ] Indonesian-first UI
- [ ] Accessibility review
- [ ] Security and privacy review
- [ ] Clean Windows 11 x64 install test
- [ ] Upgrade and uninstall test
- [ ] Release notes and support docs
- [ ] No feature claimed as complete without test evidence
