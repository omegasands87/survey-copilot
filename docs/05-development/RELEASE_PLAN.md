# Release Plan

## Release channels

- **Internal CI artifact:** unsigned or otherwise not yet publisher-verified package used only by maintainers and invited testers. Artifact retention is finite; it is not a public release.
- **Private beta:** controlled distribution after security, privacy, installation and recovery reviews. Tester feedback and manual acceptance evidence are required.
- **Public MVP:** only after all MUST requirements are implemented/explicitly descoped, release gates pass, a signed installer is available, and supported Windows behavior is verified.

## Release gates

- [ ] All MUST requirements implemented or explicitly descoped with owner approval.
- [ ] No open Critical/High security or data-integrity findings.
- [ ] AI evidence/abstention evaluation gates pass.
- [ ] Export, deletion, backup, restore, and database migration recovery tested.
- [ ] API secrets stored only in OS credential facility; deletion/retention scope accurately documented.
- [ ] Network disclosure, external vision consent, and provider terms reviewed.
- [ ] Keyboard, screen reader, contrast and accessibility review completed.
- [ ] Installation, upgrade, uninstall, and troubleshooting documentation verified.
- [ ] Version, changelog, support channel, and artifact provenance documented.
- [ ] Clean install, in-place update, uninstall, and data retention tested on supported Windows 11 x64 devices at relevant display scaling.
- [ ] Public installer Authenticode signature and publisher identity verified.
- [ ] Release artifact SHA-256 independently checked and recorded.
- [ ] No synthetic OCR fixture or demo data can appear in production flows.

## Current package workflow

The `Windows NSIS installer package` workflow runs on pull requests to `main` and manual dispatch. It executes the frontend and Rust gates, builds an x64 NSIS installer, validates package naming/size/configuration, and uploads the installer with a SHA-256 manifest as a 30-day CI artifact.

This is an internal testing mechanism only. It does not create a GitHub Release, publish a tag, sign the installer, or complete device acceptance.

## Versioning, rollback, and data protection

Use Semantic Versioning once public compatibility expectations exist. Document breaking database schema changes. Preserve prior signed release artifacts and versioned backup/restore instructions. Stop distribution immediately if a release risks data integrity, credential exposure, deceptive survey responses, or loss of user data.
