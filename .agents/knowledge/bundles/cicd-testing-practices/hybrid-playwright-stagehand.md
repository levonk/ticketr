---
type: Practice
title: Hybrid Playwright/Stagehand Testing
description: 80/20 split — Playwright for deterministic, performance-critical tests (80%), Stagehand for brittle flows, complex navigation, third-party integrations (20%). Stagehand built on Playwright.
tags: [playwright, stagehand, testing, e2e, ai-testing, hybrid, browser-automation]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: adr-20260104001-hybrid-webui-testing-playwright-stagehand
    resource: "internal-docs/adr/adr-20260104001-hybrid-webui-testing-playwright-stagehand.md"
    title: "levonk-base-boilerplate"
---


# Hybrid Playwright/Stagehand Testing

## Failure Mode

Pure Playwright tests break when UI selectors change. Pure Stagehand tests are
slow (2-10s per LLM call), expensive (token costs), and flaky (non-zero
hallucination chance).

## Practice

Use a **hybrid approach** with an 80/20 split:

### Playwright (80%) — Deterministic

- High-volume smoke tests (page loads, API responses)
- Performance-critical tests (milliseconds, not seconds)
- Stable infrastructure with `data-testid` attributes
- Deep assertions (exact values in tables)

### Stagehand (20%) — AI-Powered

- Brittle flows (frequently changing UI, A/B tested pages)
- Complex navigation (would require 20+ lines of wait/scroll logic)
- Third-party integrations (Stripe portal, Google login popups)
- Cross-app workflows (navigating through multiple websites)

### Integration

Stagehand is built on top of Playwright. Use Playwright as primary framework.
When a specific test is constantly breaking, wrap that test or step in
Stagehand. Stagehand's page object can pass directly into Playwright logic —
switch between AI mode and deterministic mode in the same script.

### Running Stagehand for Free

1. **Google Gemini 2.0 Flash** — free tier cloud API
2. **Local LLMs via Ollama** — self-hosted, no token costs
3. **Stagehand caching** — skip AI calls after first run

### Package

`@job-aide/tools-node-testing-webui` at
`packages/active/tools/node/testing/webui/typescript`

## Related Concepts

- [Vitest Unified Runner](vitest-unified-runner.md) — Vitest orchestrates these
  tests
- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — These tests run in CI
