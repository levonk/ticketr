---
type: Practice
title: Coverage Tooling in Rust
description: cargo-llvm-cov for fast, accurate coverage; cargo-tarpaulin for portable fallback. Enforce line thresholds in CI, exclude test code from reports, generate HTML, and upload to codecov or coveralls.
tags: [ci-cd, testing, rust, coverage, cargo-llvm-cov, cargo-tarpaulin, codecov]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Coverage Tooling in Rust

## Failure Mode

Coverage runs ad hoc on a developer laptop. No CI gate enforces a floor, so
coverage decays release over release. Test modules inflate numbers, hiding
uncovered production code. Reports live only in a terminal, so reviewers
cannot see gaps in PRs.

## Practice

Pick **cargo-llvm-cov** as the primary tool. Fall back to cargo-tarpaulin
where llvm-cov's toolchain requirements are unavailable. Enforce a threshold
in CI and upload reports for PR review.

### cargo-llvm-cov vs cargo-tarpaulin

| Tool | Speed | Accuracy | Portability |
|------|-------|----------|-------------|
| cargo-llvm-cov | Fast (native LLVM profiling) | High (line + region) | Needs llvm-tools |
| cargo-tarpaulin | Slower (ptrace-based) | Good (line) | Single binary, no LLVM |

Prefer llvm-cov on Linux and macOS CI runners. Use tarpaulin on targets
where llvm-tools is awkward to install.

### Enforce a Threshold in CI

Fail the build below the floor. Start at 80% lines and ratchet upward.

```bash
cargo llvm-cov --workspace --fail-under-lines 80 \
  --exclude-from-report tests --exclude-from-report benches
```

`--exclude-from-report` keeps test and bench modules out of the denominator
so the number reflects production code only.

### HTML Report Generation

Generate a local HTML report for review without a third-party service.

```bash
cargo llvm-cov --workspace --html --output-dir target/coverage
```

Open `target/coverage/index.html` to browse per-file coverage.

### Lcov Output for CI Upload

Emit lcov.info for codecov, coveralls, or GitHub Actions artifacts.

```bash
cargo llvm-cov --workspace --lcov --output-path target/lcov.info
```

### GitHub Actions Integration

```yaml
# .github/workflows/coverage.yml
name: coverage
on: [pull_request]
jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - run: cargo install cargo-llvm-cov --locked
      - run: cargo llvm-cov --workspace --lcov
            --output-path target/lcov.info
            --fail-under-lines 80
            --exclude-from-report tests
            --exclude-from-report benches
      - uses: codecov/codecov-action@v4
        with:
          files: target/lcov.info
          fail_ci_if_error: true
```

### Threshold Discipline

- Never lower the threshold to make CI pass. Fix the coverage gap instead.
- Raise the threshold only after the new floor is consistently green.
- Exclude generated code (`#[allow(dead_code)]` is not enough — use
  `--exclude-from-report` or `coverage = false` in `Cargo.toml`).

## Related Concepts

- [Rust CI Tooling](rust-ci-tooling.md) — Coverage runs alongside
  cargo-nextest in the same job
- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — Same threshold enforced
  locally and in CI
