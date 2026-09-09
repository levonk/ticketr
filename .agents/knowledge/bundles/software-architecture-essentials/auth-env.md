---
type: Practice
title: Authentication and Environment Management
description: Detect CI/SSH/Docker/Codespaces, prevent browser auth in headless terminals, provide API key guidance, and centralize environment helpers.
tags: [architecture, authentication, environment, ci, headless, api-keys]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Authentication and Environment Management

## Practice

- Detect CI, SSH, Docker, and Codespaces; gate features accordingly.
- Prevent browser-based auth prompts in headless terminals.
- Provide clear API key guidance and secure storage patterns.
- Keep logic centralized; expose helpers for consumers to query environment.

## Why

Auth strategies that work in a browser fail silently in CI, SSH sessions, and
containers. Centralized environment detection lets every consumer ask "am I
headless?" once and pick the right auth path — instead of every consumer
re-deriving the answer and getting it wrong in different ways.

## Sources

- Migrated from src/current/rules/software-dev/general/architecture/auth-env.md
