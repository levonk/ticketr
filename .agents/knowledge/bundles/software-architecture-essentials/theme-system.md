---
type: Practice
title: Theme System
description: Centralize theme definitions (palettes, variants) in a single source of truth with runtime switching and consistent semantic colors across the UI.
tags: [architecture, theme-system, ui, theming, design-tokens]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Theme System

## Practice

- Define themes (palettes, variants) in a single source of truth.
- Maintain global theme state; support runtime switching without restart when possible.
- Ensure consistent colors for `success` / `warn` / `error` / `info` across the UI.
- Integrate themes with menus, progress indicators, error handling, and ASCII/branding as needed.

## Why

A single source of truth for themes prevents color drift, makes runtime
switching (light/dark, brand variants) trivial, and gives every component
a consistent semantic vocabulary for state.

## Sources

- Migrated from src/current/rules/software-dev/general/architecture/theme-system.md
