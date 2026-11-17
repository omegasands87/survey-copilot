# Release Plan

## Stages
- **Internal alpha:** core local workflows; synthetic test data; no public reliability claims.
- **Private beta:** limited users, explicit feedback, security and usability review.
- **MVP release:** documented supported OS, tested migration path, privacy disclosure, signed package where feasible.

## Release checklist
- [ ] All Must requirements implemented or explicitly descoped.
- [ ] No open critical/high security findings.
- [ ] AI evaluation release gates pass.
- [ ] Export, deletion, backup, and migration recovery tested.
- [ ] API key stored in OS credential facility.
- [ ] Network disclosure and provider terms reviewed.
- [ ] Accessibility review completed.
- [ ] Installation and troubleshooting docs verified.
- [ ] Version, changelog, and support channel published.
- [ ] Build artifacts tested on clean supported Windows environments.

## Versioning and rollback
Use Semantic Versioning once public compatibility expectations exist. Document breaking schema changes. Keep prior release artifacts and database backup/restore instructions. Pause distribution if a release risks data integrity or credential exposure.
