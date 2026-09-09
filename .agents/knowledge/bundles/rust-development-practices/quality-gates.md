---
type: Practice
title: Quality Gates
description: Pre-commit hooks for fmt/clippy/test, CI/CD with multiple Rust versions (stable/beta/nightly), cross-platform testing, documentation builds, security audits, cargo outdated drift detection, and weekly dependency update PRs.
tags: [rust, quality-gates, pre-commit, ci-cd, cross-platform, testing, cargo-outdated, cargo-audit]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-08-20"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md"
    title: "levonk-base-boilerplate"
  - id: levonk-base-boilerplate-cargo-audit-wiring
    resource: "internal-docs/feature/todo/cargo-audit-wiring/feat-202608201814-cargo-audit-wiring.md"
    title: "levonk-base-boilerplate"
---

# Quality Gates

## Failure Mode

Without quality gates, unformatted code, linting failures, and untested changes
land in the repository. Single-version testing misses compatibility issues.
Missing security audits let vulnerable dependencies ship. Dependency drift
accumulates with no visibility — projects fall behind security patches silently.

## Practice

### Pre-commit Hooks

```bash
#!/bin/sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

### Validation Pipeline (`just validate`)

The boilerplate's `just validate` target runs the full 6-step pipeline:

```just
validate:
    just format-check      # cargo fmt --check (blocking)
    just lint              # cargo clippy -D warnings (blocking)
    just test              # cargo test (blocking)
    just doc-no-open       # cargo doc (blocking)
    just audit             # cargo audit (blocking)
    just outdated          # cargo outdated --exit-code 0 (non-blocking)
```

The outdated check is non-blocking (`--exit-code 0` with `|| log_warn`) so
drift reports never abort validate — they warn and continue. The blocking
version is the weekly CI cron job.

### CI/CD Requirements

- **Multiple Rust versions**: Test against stable, beta, and nightly
- **Cross-platform**: Test on Linux, macOS, and Windows
- **Documentation**: Ensure docs build without warnings
- **Security**: Run `cargo audit` in CI (blocking)
- **Outdated check**: Run `cargo outdated` in CI (non-blocking,
  `continue-on-error: true`)

### Shared CI Workflow

Both Rust boilerplates (CLI and package) consume the shared CI workflow at
`_shared/.github/workflows/rust-ci.yml.jinja`, which provides 5 jobs:

- `test` — cargo test (matrix: ubuntu/macos/windows)
- `lint` — clippy + fmt check
- `build` — cargo build --release (matrix)
- `security` — cargo audit (blocking)
- `outdated-check` — cargo outdated (non-blocking, `continue-on-error: true`)

### Weekly Dependency Update Workflow

A weekly scheduled workflow at `_shared/.github/workflows/rust-weekly-outdated.yml.jinja`
runs every Monday at 09:00 UTC:

1. `cargo update` (no dry-run — actually updates Cargo.lock)
2. `cargo audit` (verifies no new vulnerabilities introduced)
3. `cargo build --all-features` (verifies the project still builds)
4. `cargo test --all-features` (verifies tests still pass)
5. Creates a pull request with the updated `Cargo.lock` via
   `peter-evans/create-pull-request@v6`

This is the blocking counterpart to the non-blocking `cargo outdated` in
validate — instead of just reporting drift, it applies the update and
verifies it doesn't break anything.

### Validation Criteria

A Rust package is considered complete when:

1. `cargo check` passes without warnings
2. `cargo test` passes all tests
3. `cargo clippy` passes without warnings
4. `cargo fmt --check` passes
5. `cargo doc` generates docs without warnings
6. `cargo build --release` succeeds
7. `cargo audit` passes (no known vulnerabilities)
8. `cargo outdated` reports no critical drift
9. Docker image builds and runs successfully
10. `nix build` succeeds in development shell

## Related Concepts

- [Rustfmt and Clippy](rustfmt-clippy-config.md) — Formatting and linting config
- [Testing Strategy](testing-strategy.md) — Test types that gates enforce
- [Security and Auditing](security-auditing.md) — cargo audit + cargo outdated in CI
