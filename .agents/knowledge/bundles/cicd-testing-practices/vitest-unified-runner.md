---
type: Practice
title: Vitest Unified Runner
description: Vitest as primary framework for all TypeScript testing — unit, integration, and E2E test runner with Stagehand/Playwright. Fast, ESM-native, Jest-compatible, monorepo-integrated.
tags: [vitest, testing, typescript, unified, runner, jest, esm]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: adr-20251106002-vitest-for-testing
    resource: "internal-docs/adr/adr-20251106002-vitest-for-testing.md"
    title: "levonk-base-boilerplate"
---


# Vitest Unified Runner

## Failure Mode

Using different test runners for different test types fragments the developer
experience. Jest is slower than modern alternatives and requires complex
configuration for TypeScript and ESM.

## Practice

Use **Vitest** as the primary framework for all TypeScript testing.

### Test Types

- **Unit & Integration**: Vitest directly
- **E2E**: Vitest as test runner, orchestrating Stagehand/Playwright

### Configuration

- Global `vitest.config.ts` at monorepo root
- Individual packages extend root configuration
- `package.json` scripts: `"test"`, `"test:e2e"` use `vitest` commands
- New features must include unit tests written with Vitest

### Why Vitest Over Alternatives

- **Jest**: Slower, complex TS/ESM config
- **Mocha & Chai**: Lacks integrated "all-in-one" experience
- **Playwright Test Runner**: Excellent for E2E but doesn't unify all test types

### Benefits

1. **Performance**: Built on Vite, significantly faster than Jest
2. **Modern Tooling**: Out-of-the-box TypeScript and ESM support
3. **Unified Experience**: Single API for all test types
4. **Jest Compatibility**: API largely compatible for easy migration
5. **Monorepo Integration**: Workspace feature works with pnpm/Nx

## Related Concepts

- [Hybrid Playwright/Stagehand](hybrid-playwright-stagehand.md) — E2E tests
  orchestrated by Vitest
- [Shared Quality Scripts](shared-quality-scripts.md) — Quality script invokes
  Vitest
