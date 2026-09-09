---
type: Practice
title: Shared Dockerized Quality Scripts
description: Single Docker-based quality script per boilerplate that both pre-commit hooks and GitHub Actions invoke. Configurable scanners, environment variables for fast/full mode, Copier-generated.
tags: [docker, quality, scripts, pre-commit, ci-cd, scanners, trivy, checkov]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: adr-20251218002-shared-quality-scripts
    resource: "internal-docs/adr/adr-20251218002-shared-quality-scripts.md"
    title: "levonk-base-boilerplate"
---


# Shared Dockerized Quality Scripts

## Failure Mode

Separate implementations for pre-commit hooks and CI workflows cause drift,
duplicated dependency pinning, and conflicting scan configurations. Heavyweight
scans (Trivy, Checkov) block urgent iterations when run locally on every commit.

## Practice

Adopt a **single Docker-based quality script** per boilerplate that both
pre-commit hooks and GitHub Actions invoke.

### Script Contract

- Lives in `scripts/run-quality-checks.sh`
- Only place that pins tool images and orchestrates scans
- Accepts boolean env vars: `ENABLE_TRIVY`, `ENABLE_CHECKOV`, etc.
- Composite toggles: `FAST_MODE` (lint only), `FULL_MODE` (everything)
- `MARKDOWN_ONLY=1` for docs-only changes
- Returns non-zero exit code on first failure (fail fast)

### Pre-Commit Hook

- Minimal wrapper that ensures script is executable
- Passes through developer overrides (e.g., `FAST_MODE=1`)
- Documented in generated README sections

### GitHub Actions Workflow

- Checkout → `chmod +x` script → run with `FULL_MODE=1`
- Honors `--security-opt=no-new-privileges` container policy
- Caches scanner directories via Actions cache

### Copier Integration

- Template definitions emit script, hook, and workflow automatically
- Variables in `copier.yml` allow projects to choose which scanners are included

### Benefits

- **Parity**: One script eliminates divergence between local and CI behavior
- **Isolation**: Dockerized tools avoid polluting developer machines
- **Configurability**: Developers tailor local runs to change scope
- **Maintainability**: Update tool versions in one file, propagate via Copier

## Related Concepts

- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — This script enables parity
- [Accessibility Testing](accessibility-testing.md) — axe-core runs through this
  script
