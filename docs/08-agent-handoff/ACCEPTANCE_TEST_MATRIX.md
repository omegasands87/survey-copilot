# Acceptance Test Matrix

Use this matrix to convert product requirements into automated tests and manual Windows release tests. Add test IDs to test names where practical.

| ID | Area | Scenario | Expected result | Priority |
|---|---|---|---|---|
| AT-001 | Build | Fresh checkout installs and builds | Frontend and Rust build pass with committed lockfiles | P0 |
| AT-002 | Migration | Fresh DB startup | All initial tables/indexes exist; foreign keys enabled | P0 |
| AT-003 | Migration | App restarts with existing DB | No schema reset or duplicate migration | P0 |
| AT-004 | Migration | Migration fails midway | Existing user data remains recoverable; startup reports safe error | P0 |
| AT-010 | Persona | Create first persona | First persona becomes active | P1 |
| AT-011 | Persona | Create second persona | First remains active until explicit switch | P1 |
| AT-012 | Persona | Switch active persona | Exactly one active persona; existing sessions retain original owner | P1 |
| AT-013 | Persona isolation | Query facts for persona B | No fact from persona A is returned | P1 |
| AT-014 | Persona deletion | Delete persona with child data | Explicit confirmation; documented cascade; no orphaned records | P1 |
| AT-020 | Facts | Create fact and source | Both persist atomically and remain linked to same persona | P1 |
| AT-021 | Facts | Mark fact disputed/outdated | Fact is excluded from definitive answer retrieval | P1 |
| AT-022 | Facts | Delete referenced fact | Defined deletion policy; no unexplained database error | P1 |
| AT-023 | Facts | Store sensitive fact | Excluded from provider payload absent explicit request consent | P1 |
| AT-030 | Credentials | Add API key | Secret stored in Windows Credential Manager; only opaque ref in DB | P2 |
| AT-031 | Credentials | Remove provider | Credential removed; failure surfaced | P2 |
| AT-032 | Provider | Gemini structured request | Normalized response validated and returned | P2 |
| AT-033 | Pool | One key returns 429, another healthy | Respect Retry-After/cooldown and fail over if permitted | P2 |
| AT-034 | Pool | All keys hit account quota | Stop retries for account/project scope; preserve question | P2 |
| AT-035 | Pool | Invalid request/content refusal | Do not rotate through all keys | P2 |
| AT-036 | Pool | User cancels request | Queued retries stop; no hidden request continues | P2 |
| AT-037 | Custom endpoint | HTTP endpoint outside localhost | Reject unless explicitly allowed by policy; default HTTPS only | P2 |
| AT-038 | Security | Redirect to another host | Never forward API credentials to unapproved origin | P2 |
| AT-040 | Question | Paste question | Original text persists and is recoverable after restart | P3 |
| AT-041 | Question | Ambiguous timeframe/unit | Ask clarification; do not infer silently | P3 |
| AT-042 | Retrieval | Persona A has evidence, B does not | B's request contains no A facts and abstains if evidence missing | P3 |
| AT-043 | AI output | Model cites unknown fact ID | Reject or remove unsupported citation; do not show as evidence | P3 |
| AT-044 | AI output | Prompt injection in survey text | Treat as untrusted content; no policy/tool override | P3 |
| AT-045 | AI output | Contradictory profile facts | No definitive answer; disclose conflict | P3 |
| AT-046 | History | User edits generated answer | Original AI text and edited text are both preserved | P3 |
| AT-047 | History | User confirms recommendation | Record approval only; no survey submission | P3 |
| AT-050 | Capture | Capture invoked by user | Capture preview is shown before OCR/analysis | P4 |
| AT-051 | Capture | App is idle/background | No screenshot capture occurs | P4 |
| AT-052 | OCR | Clipped/low-confidence text | Mark uncertainty and require correction/confirmation | P4 |
| AT-053 | OCR | OCR service unavailable | Manual text input remains usable | P4 |
| AT-054 | Capture | Temporary capture discarded | Temporary image file is removed | P4 |
| AT-060 | Survey fit | Criterion matches confirmed current fact | Local deterministic matcher returns match with fact reference | P5 |
| AT-061 | Survey fit | Criterion evidence absent | Return unknown, never match | P5 |
| AT-062 | Survey fit | Criterion fact is stale/disputed | Return stale/conflict as appropriate | P5 |
| AT-063 | Learning | No or sparse historical outcomes | Show insufficient data; no fabricated probability | P5 |
| AT-070 | Privacy | Export all local data | Includes selected data only; never includes API secrets | P6 |
| AT-071 | Deletion | Delete all app data | Reports database, temp files, backups, and credential cleanup outcome | P6 |
| AT-072 | Release | Clean Windows 11 x64 install | App launches and primary local workflows work | P6 |
| AT-073 | Release | Uninstall | Behavior for user data is clearly disclosed and tested | P6 |
| AT-074 | Product boundary | Attempt automated survey action | No such command, dependency, or capability exists | All |
| AT-075 | Product boundary | Earnings/reward tracking request in v1 scope | No such feature or data ledger exists | All |
