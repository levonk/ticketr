---
type: Practice
title: Architecture Philosophy
description: Domain-based modular architecture with clear separation of concerns, focused modules, and coordination via local index modules.
tags: [architecture, philosophy, modular, separation-of-concerns]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Architecture Philosophy

- Domain-based modular architecture with clear separation of concerns.
- Clear entry points per domain; thin top-level (`main`/`lib`) that delegate.
- Focused modules with single responsibility; small, testable units.
- Coordination via local index/re-export modules; avoid cross-domain leakage.
- Benefits: maintainability, testability, extensibility, refactoring safety.

See also: `dot_config/ai/rules/software-dev/general/architecture/project-structure.md`.

## Sources

- Migrated from src/current/rules/software-dev/general/architecture/philosophy.md
