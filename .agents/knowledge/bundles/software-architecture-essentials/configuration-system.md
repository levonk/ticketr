---
type: Practice
title: Configuration System
description: Layered config precedence with schema validation, caching, and documented override rules.
tags: [architecture, configuration, config, toml, validation]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Configuration System

- Config precedence (highest first):
  1. Project config (e.g., `./app-config.toml`)
  2. User config (e.g., `~/.config/app/config.toml`)
  3. Built-in defaults
- Provide a schema example and validate on load.
- Cache parsed config; expose read-only runtime view.
- Avoid surprising env side effects; document override rules.

Example TOML shape (adapt to app):

```toml
[tools]
example = { enabled = true, auto_update = false }

[ui]
theme = "T.JARVIS"
center_output = true

[templates]
repository = "org/templates"
auto_sync = true
```

## Sources

- Migrated from src/current/rules/software-dev/general/architecture/configuration-system.md
