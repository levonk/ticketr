---
okf_version: "0.2"
---

# CI/CD Testing Practices

A compounding knowledge base documenting practices for CI/CD pipelines and
testing strategies — hybrid Playwright/Stagehand testing, shared Dockerized
quality scripts, Vitest as unified test runner, and pre-commit/CI parity.

## Concepts

* [Overview](overview.md) - Synthesis of the full CI/CD testing practice set
* [hybrid-playwright-stagehand](hybrid-playwright-stagehand.md) - 80/20 split: Playwright for deterministic, Stagehand for AI-powered resilient tests
* [shared-quality-scripts](shared-quality-scripts.md) - Single Docker-based quality script for pre-commit and CI parity
* [vitest-unified-runner](vitest-unified-runner.md) - Vitest as test runner for unit, integration, and E2E
* [pre-commit-ci-parity](pre-commit-ci-parity.md) - Same checks run locally and in CI, reducing "works on my machine"
* [accessibility-testing](accessibility-testing.md) - axe-core in CI, WCAG 2.1 AA compliance from day one
* [rust-ci-tooling](rust-ci-tooling.md) - cargo-deny, cargo-audit, cargo-nextest, cargo-release, cross-compilation for Rust CI
* [snapshot-testing-rust](snapshot-testing-rust.md) - insta crate, inline vs external snapshots, snapshot review in CI
* [coverage-tooling-rust](coverage-tooling-rust.md) - cargo-llvm-cov vs tarpaulin, coverage thresholds, HTML reports, CI integration
* [property-based-testing-rust](property-based-testing-rust.md) - proptest, strategy composition, shrinking, testing invariants
* [ui-requirements-coverage](ui-requirements-coverage.md) - Verify every documented UI requirement is represented in the running app using deterministic accessibility-tree inspection (agent-browser for web, agent-device for mobile)
* [ux-usability-testing](ux-usability-testing.md) - Test whether users can accomplish tasks without documentation using AI-driven natural-language goals (Stagehand for web, finalrun-agent for mobile)
* [fast-test-feedback](fast-test-feedback.md) - Keep the suite fast: parallelize, cache, split slow integration/E2E from fast unit tests, shard. Never weaken the gate — use git-based change detection and dependency-aware tooling (Vitest, Jest, pytest-testmon, cargo-nextest) to run only affected tests
