---
type: Practice
title: Adding New Tools
description: Define the CLI surface, add display-name/config mapping, extend detection/mapping, wire execution path, and add service operations under services/.
tags: [architecture, tooling, cli, extensibility, services]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Adding New Tools

## Practice

- Define the CLI surface in the command layer (args, subcommands).
- Add display-name / config mapping in services or config domain.
- Extend detection / mapping; wire execution path in CLI logic domain.
- Add external service operations as needed under `services/`.
- Keep responsibilities in their domains; update tests and docs alongside changes.

## Why

A consistent procedure for adding tools keeps the CLI surface uniform, the
detection layer coherent, and the service boundary clean. Each new tool
touches the same set of layers in the same order, which makes review and
maintenance predictable.

## See Also

- [Tool Detection Architecture](tool-detection.md) — how tools are discovered on the system.
- [Project Structure](project-structure.md) — where `services/`, `cli/`, and `config/` live.

## Sources

- Migrated from src/current/rules/software-dev/general/architecture/adding-tools.md
