# Directory Update Log

## 2026-08-30

* **Ingest**: Authored 1 new concept page covering test-suite speed and
  selective test execution. The bundle had content on what to test and
  how to keep tests deterministic, but no guidance on keeping the suite
  fast or on running only affected tests. This page fills that gap.
  - [fast-test-feedback.md](fast-test-feedback.md) — parallelize, cache,
    split slow integration/E2E from fast unit tests, shard. Never weaken
    the gate to work around a slow suite. Use git-based change detection
    and built-in test-dependency tooling (Vitest `--changed`, Jest
    `--findRelatedTests`, pytest-testmon, cargo-nextest) to run only
    affected tests. Same git-based selective-run principle applies to
    auto-generated documentation.
* **Update**: Updated [overview.md](overview.md.tmpl) testing stack
  diagram and table with a Speed phase.
* **Update**: Updated [index.md](index.md) with 1 new concept entry.

## 2026-08-27

* **Update**: Updated [pre-commit-ci-parity.md](pre-commit-ci-parity.md) to
  include workflow security linting (zizmor + actionlint) as a standard CI
  parity check. Added a dedicated `workflow-security` job to the CI parity
  model — runs zizmor (workflow security analyzer) and actionlint (workflow
  syntax/schema linter) on all `.github/workflows/*.yml`, independent of the
  test job. Added cross-reference to
  [dependency-supply-chain.md](https://github.com/levonk/skills-releases/blob/main/knowledge/devsecops-codeguard/dependency-supply-chain.md)
  in the devsecops-codeguard bundle. Updated frontmatter with zizmor and
  actionlint sources.

## 2026-08-08

* **Ingest**: Authored 2 new UI/UX testing concept pages extending the
  testing stack with two complementary dimensions: requirements coverage
  (deterministic, no AI tokens) and usability (AI-driven, BYOK). Both pages
  grounded against the tool repos on 2026-08-08.
  - [ui-requirements-coverage.md](ui-requirements-coverage.md) —
    agent-browser (web, MIT) and agent-device (mobile, MIT) for
    deterministic accessibility-tree inspection. CLI-first, no API keys.
  - [ux-usability-testing.md](ux-usability-testing.md) — Stagehand (web,
    MIT, BYOK) and finalrun-agent (mobile, Apache-2.0, BYOK) for AI-driven
    natural-language task testing. AutoMobile (Apache-2.0) as MCP-first
    alternative. Graceful missing-key handling documented.
* **Update**: Updated [overview.md](overview.md.tmpl) testing stack diagram
  and table with UI Coverage and UX Usability phases.
* **Update**: Updated [index.md](index.md) with 2 new concept entries.

## 2026-08-05

* **Ingest**: Authored 4 new Rust CI/testing concept pages sourced from the
  project-lint audit. The bundle was previously TypeScript-focused with zero
  Rust CI content; this ingest fills the highest-impact gap. All pages
  grounded against project-lint/Cargo.toml and current tool versions on
  2026-08-05.
  - [rust-ci-tooling.md](rust-ci-tooling.md) — cargo-deny, cargo-audit 0.22.1,
    cargo-nextest, cargo-release, cross-compilation; GitHub Actions workflow
  - [snapshot-testing-rust.md](snapshot-testing-rust.md) — insta crate,
    inline vs external snapshots, snapshot review in CI
  - [coverage-tooling-rust.md](coverage-tooling-rust.md) — cargo-llvm-cov vs
    tarpaulin, coverage thresholds, HTML reports, CI integration
  - [property-based-testing-rust.md](property-based-testing-rust.md) —
    proptest, strategy composition, shrinking, testing invariants
* **Update**: Updated [index.md](index.md) with 4 new concept entries.

## 2026-07-26
* **Migration**: Migrated bundle from OKF v0.1 to OKF v0.2 — bumped `okf_version` in index.md. No `# Citations` sections or `timestamp` fields to migrate.
* **Migration**: Migrated `## Citations` body sections to `sources` frontmatter with stable `id` attributes per OKF v0.2 §13.1.

## 2026-07-18

* **DRY**: Converted [overview.md](overview.md.tmpl) to `overview.md.tmpl` and
  added `{{{ include "includes/tech-stack-table.md" . }}}` so the canonical
  tech-stack choices table is inlined from a single source of truth at
  `src/current/includes/tech-stack-table.md.tmpl`. See the
  typescript-monorepo-best-practices log entry for the full rationale.

## 2026-07-17

* **Initialization**: Created the `cicd-testing-practices` knowledge bundle to consolidate CI/CD and testing practices from three ADRs in levonk-base-boilerplate and the bookkeep-saas PRD.
* **Creation**: Authored 5 concept pages covering the testing stack.
  - [hybrid-playwright-stagehand.md](hybrid-playwright-stagehand.md) — 80/20 split, deterministic vs AI-powered tests
  - [shared-quality-scripts.md](shared-quality-scripts.md) — single Docker-based quality script for hooks and CI
  - [vitest-unified-runner.md](vitest-unified-runner.md) — Vitest for all TypeScript testing
  - [pre-commit-ci-parity.md](pre-commit-ci-parity.md) — same checks locally and in CI
  - [accessibility-testing.md](accessibility-testing.md) — axe-core in CI, WCAG 2.1 AA from day one
* **Creation**: Established [overview.md](overview.md) synthesis and [index.md](index.md) directory listing.
* **Note**: Concepts extracted from ADR-20260104001 (hybrid Playwright/Stagehand, 261 lines), ADR-20251218002 (shared quality scripts, 115 lines), ADR-20251106002 (Vitest, 83 lines) in boilerplate, and bookkeep-saas PRD NFR21-NFR24 (a11y).
