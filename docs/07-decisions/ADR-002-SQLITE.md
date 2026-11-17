# ADR-002: Use SQLite for Local Persistence

**Status:** Proposed

## Context
The MVP is single-user and local-first. It needs relational integrity, transactions, migrations, and portable storage without operating a database server.

## Decision
Use SQLite with explicit migrations, foreign keys, parameterized queries, and repository-level access.

## Consequences
**Positive:** simple deployment, transactional consistency, mature tooling.  
**Trade-offs:** local backup/encryption responsibilities, write concurrency constraints, and no built-in multi-device sync.

## Revisit if
Multi-user collaboration, concurrent remote access, or synchronization becomes a validated product requirement.
