---
type: Practice
title: Rust CI Tooling
description: cargo-deny, cargo-audit, cargo-outdated, cargo-nextest, cargo-release, and cross form the CI backbone for Rust projects. License, advisory, ban, and drift checks gate every PR; parallel test execution with JUnit output feeds CI dashboards; weekly dependency update PRs keep Cargo.lock fresh.
tags: [ci-cd, testing, rust, cargo-deny, cargo-audit, cargo-outdated, cargo-nextest, cargo-release, cross]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-20"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
  - id: levonk-base-boilerplate
    resource: "_shared/.github/workflows/rust-ci.yml.jinja"
    title: "levonk-base-boilerplate"
  - id: levonk-base-boilerplate-weekly
    resource: "_shared/.github/workflows/rust-weekly-outdated.yml.jinja"
    title: "levonk-base-boilerplate"
---

# Rust CI Tooling

## Failure Mode

Rust projects inherit no license, advisory, or ban checks by default. A
transitive dependency pulls in a restricted license or a known CVE and ships
to production unnoticed. `cargo test` runs serially, starves CI cores, and
emits no machine-readable result for dashboards. Publishing relies on manual
`cargo publish` steps that drift across maintainers. Dependency drift
accumulates with no automated update mechanism — `Cargo.lock` falls behind
security patches, and no PR is ever created to bring it current.

## Practice

Wire six cargo subcommands into every Rust CI pipeline. Run them in the
same script locally and in CI so behavior stays identical.

### cargo-deny — License, Advisory, and Ban Checks

Add a `deny.toml` at the repo root. Fail the build on any vulnerability or
denied license.

```toml
# deny.toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC", "Unicode-DFS-2016"]
deny = ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]
[bans]
multiple-versions = "warn"
wildcards = "deny"
```

### cargo-audit — RustSec Advisory Scan

`cargo-audit 0.22.1` gives a focused, fast advisory pass alongside
cargo-deny's license and ban policy. `--deny warnings` turns unmaintained
and yanked crates into hard failures.

```bash
cargo install cargo-audit --locked --version 0.22.1
cargo audit --deny warnings
```

### cargo-outdated — Dependency Drift Detection

`cargo-outdated` reports when dependencies have newer versions available.
In CI, run it as a non-blocking job (`continue-on-error: true`) so drift
is visible but doesn't gate merges. The blocking version is the weekly
cron job that actually applies updates.

```bash
cargo install cargo-outdated --locked
cargo outdated --exit-code 0  # non-blocking: reports drift only
```

### cargo-nextest — Parallel Test Execution

`cargo-nextest` runs tests in parallel processes, reuses builds, and emits
JUnit XML for CI dashboards. Prefer it over `cargo test` in every CI job.

```bash
cargo nextest run --workspace --profile ci \
  --junitfile target/nextest-junit.xml
```

Set `retries = 2` and `fail-fast = false` in `.config/nextest.toml` under
`[profile.ci]` to absorb flaky tests without hiding real failures.

### cargo-release — Publishing with Trusted Publishing

`cargo-release` automates version bump, tag, and publish. Pair it with
trusted publishing on crates.io so no long-lived API token sits in CI
secrets: `cargo release patch --execute --no-confirm`.

### cross — Cross-Compilation Matrix

`cross` runs tests and builds under Docker for target triples CI cannot run
natively. Fill a matrix without self-hosted runners:

```bash
cross test --target aarch64-unknown-linux-gnu
cross build --target x86_64-pc-windows-gnu
```

### Shared CI Workflow (boilerplate)

The levonk-base-boilerplate provides a shared CI workflow at
`_shared/.github/workflows/rust-ci.yml.jinja` that both Rust boilerplates
(CLI and package) consume via Jinja include. It provides 5 jobs:

- `test` — cargo test (matrix: ubuntu/macos/windows)
- `lint` — clippy + fmt check
- `build` — cargo build --release (matrix)
- `security` — cargo audit (blocking)
- `outdated-check` — cargo outdated (non-blocking, `continue-on-error: true`)

### Weekly Dependency Update Workflow

The boilerplate also provides a weekly scheduled workflow at
`_shared/.github/workflows/rust-weekly-outdated.yml.jinja` that runs every
Monday at 09:00 UTC:

1. `cargo update` (no dry-run — actually updates Cargo.lock)
2. `cargo audit` (verifies no new vulnerabilities introduced)
3. `cargo build --all-features` (verifies the project still builds)
4. `cargo test --all-features` (verifies tests still pass)
5. Creates a pull request with the updated `Cargo.lock` via
   `peter-evans/create-pull-request@v6`

This is the blocking counterpart to the non-blocking `cargo outdated` in
the CI workflow — instead of just reporting drift, it applies the update
and verifies it doesn't break anything.

### GitHub Actions Workflow

```yaml
# .github/workflows/rust-ci.yml
name: rust-ci
on: [pull_request]
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-deny,cargo-audit,cargo-outdated,cargo-nextest
      - run: cargo deny check
      - run: cargo audit --deny warnings
      - run: cargo outdated --exit-code 0
        continue-on-error: true
      - run: cargo nextest run --workspace --profile ci
          --junitfile target/nextest-junit.xml
      - uses: dorny/test-reporter@v1
        if: always()
        with:
          path: target/nextest-junit.xml
          reporter: java-junit
```

## Related Concepts

- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — Run these same checks locally via the shared quality script
- [Shared Quality Scripts](shared-quality-scripts.md) — Single entry point for both hook and CI
- [Quality Gates](https://github.com/levonk/skills-releases/blob/main/knowledge/rust-development-practices/quality-gates.md) — Full validation pipeline + weekly update workflow
- [Security and Auditing](https://github.com/levonk/skills-releases/blob/main/knowledge/rust-development-practices/security-auditing.md) — cargo audit + cargo outdated in the validation pipeline
