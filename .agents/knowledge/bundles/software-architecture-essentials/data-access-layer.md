---
type: Practice
title: Data Access Layer
description: Centralized data access layer that abstracts data sources, provides a single source of truth, and enforces authorization.
tags: [architecture, data-access, dal, security, abstraction]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Data Access Layer (DAL)

A Data Access Layer (DAL) is a dedicated, centralized part of your application responsible for all data-related operations. It abstracts the underlying data source (e.g., database, API) from your business logic.

- **Centralize Logic**: Consolidate all data fetching, caching, and mutation logic into a single location (e.g., a `src/data/` or `src/lib/data` directory).
- **Single Source of Truth**: By centralizing data access, you create a single source of truth for how data is retrieved and modified. This simplifies debugging and maintenance.
- **Security Checkpoint**: The DAL provides a natural and effective checkpoint to enforce authorization and validate user permissions before any data is accessed or returned.

## See Also

- [Database Scaling](database-scaling.md) — the distributed-systems view of
  scaling the data sources the DAL abstracts.
- [Caching Strategies](caching-strategies.md) — the DAL is the natural place to
  centralize cache logic in front of the database.
- [System Security Basics](system-security-basics.md) — the DAL as the
  authorization checkpoint in a zero-trust architecture.

## Sources

- Migrated from src/current/rules/software-dev/general/architecture/data-access-layer.md
