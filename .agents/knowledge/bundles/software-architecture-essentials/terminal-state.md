---
type: Practice
title: Terminal State Management
description: Use minimal terminal control sequences, prepare terminal state for tools requiring input focus, and centralize theme-aware terminal helpers.
tags: [architecture, terminal, tty, state-management, cli]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Terminal State Management

## Practice

- Use minimal terminal control sequences; avoid clearing buffers unexpectedly.
- Prepare terminal state for tools that require input focus.
- Add short, intentional delays when launching tools to avoid race conditions.
- Centralize terminal state helpers; make them theme-aware when applicable.

## Why

Terminal state is shared global state. Sloppy control sequences break scroll
back history, race conditions corrupt input handling, and uncoordinated
launches can leave the terminal in a bad state for the next tool.

## Sources

- Migrated from src/current/rules/software-dev/general/architecture/terminal-state.md
