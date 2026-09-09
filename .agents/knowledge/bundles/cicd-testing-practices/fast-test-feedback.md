---
type: Practice
title: Fast Test Feedback
description: Keep the test suite fast through parallelization, caching, splitting slow integration/E2E from fast unit tests, and sharding. Never weaken the gate to work around a slow suite — fix the speed. Use git-based change detection and built-in test-dependency tooling (Vitest, Jest, pytest-testmon, cargo-nextest) so automated processes run only the tests that are affected. The same git-based selective-run principle applies to auto-generated documentation.
tags: [testing, performance, parallelization, caching, sharding, selective-testing, test-dependency, git, vitest, jest, pytest-testmon, cargo-nextest, documentation-generation]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
---

# Fast Test Feedback

## Failure Mode

The test suite takes 20 minutes. Developers stop running it locally — they
push and wait for CI. CI becomes the only feedback loop, so every push
blocks for 20 minutes. To "fix" this, someone disables tests, marks them
optional, or splits them into a separate workflow that runs on a schedule.
The gate is weakened. Regressions slip through because the fast path no
longer runs the tests that would have caught them.

Or: the suite runs every test on every push regardless of what changed.
A one-line README edit triggers 2,000 tests. Developers learn to avoid
small commits because the feedback cost is constant regardless of change
size. The suite is correct but slow because it does work it does not need
to do.

Or: auto-generated documentation rebuilds on every commit even when no
source file that feeds the docs changed. The docs job adds 5 minutes to
every CI run for nothing.

The root cause is always the same: the suite does more work than the
change requires, and the response to slowness is to weaken the gate
rather than to make the suite do less unnecessary work.

## Practice

A slow suite is a bug, not a constraint to work around. Fix the speed;
do not weaken the gate. The suite must stay fast enough that developers
run it locally and CI provides feedback in minutes, not tens of minutes.

### Core Principles

1. **Never weaken the gate to work around a slow suite.** Disabling
   tests, marking them optional, moving them behind a schedule, or
   splitting them out of the blocking path are all gate-weakening. If
   the suite is slow, make it faster — parallelize, cache, shard, split
   by speed tier, and run only affected tests. The gate stays full.
2. **Parallelize.** Run tests across all available cores and CI runners.
   Most test runners support parallel execution natively — enable it.
   In CI, shard across multiple runners (matrix + shard) so the suite
   wall-clock time scales with runner count, not test count.
3. **Cache.** Cache dependencies, build artifacts, and test results
   between runs. A cold run may be slow; a warm run should be fast.
   Use the CI provider's cache (GitHub Actions `actions/cache`, GitLab
   CI cache) for dependency directories and build outputs.
4. **Split slow integration/E2E from fast unit tests.** Unit tests are
   fast and run on every change. Integration and E2E tests are slow and
   may run in a separate tier — but still in the blocking path, not
   optional. The split is about ordering and parallelism, not about
   skipping. Run unit tests first for fast failure feedback, then
   integration/E2E in parallel shards.
5. **Shard.** Split the test suite across N runners so each runner
   executes 1/N of the tests. Use a deterministic shard assignment
   (hash test name → shard index) so the same test always runs on the
   same shard — this makes caching and failure diagnosis predictable.
6. **Run only affected tests.** Use git-based change detection to
   determine which files changed, then use the test runner's built-in
   dependency-aware filtering to run only the tests whose code paths
   changed. This is the single highest-impact optimization: a one-line
   change should run a handful of tests, not the whole suite.
7. **Apply the same git-based selective-run principle to auto-generated
   documentation.** Docs generation (API references, type docs, schema
   docs) should use git to detect whether any source file that feeds
   the docs changed. If nothing changed, skip the docs job. If
   something changed, regenerate only the affected docs.

### Do Not Weaken the Gate

The temptation when a suite is slow is to make tests optional, move
them to a nightly run, or split them into a non-blocking workflow.
**These are all gate-weakening.** They trade correctness for speed in
the wrong direction — the slow tests are usually the integration and
E2E tests that catch the most valuable regressions, and moving them
out of the blocking path means those regressions reach production.

The correct response to a slow suite is, in order:

1. **Run only affected tests** (git + dependency-aware tooling) —
   eliminates unnecessary work.
2. **Parallelize and shard** — distributes remaining work across cores
   and runners.
3. **Cache** — eliminates redundant work across runs.
4. **Split by speed tier** — runs fast unit tests first for early
   feedback, slow integration/E2E in parallel.
5. **Fix the slow tests** — if a test is slow because it does too much,
   split it. If it is slow because it hits a real network or database,
   mock or containerize the dependency.

Only after all five are exhausted should you consider whether a
specific test genuinely cannot be made fast — and even then, the
answer is to quarantine and fix it, not to weaken the gate for the
entire suite.

## Concrete Instances

### Git-Based Change Detection

Determine which files changed, then run only the tests whose code
paths are affected. The git tooling figures out whether tests need to
run at all — if no source file changed, skip the test job entirely.

```bash
# Get the list of files changed in this push/PR vs the base branch
CHANGED_FILES=$(git diff --name-only origin/main...HEAD)

# If no source files changed, skip tests entirely
if ! echo "$CHANGED_FILES" | grep -qE '\.(ts|tsx|js|jsx|py|rs|go)$'; then
  echo "No source files changed — skipping test suite"
  exit 0
fi
```

For local development, the same logic applies: compare against the
merge-base of the current branch and the main branch, or use the
staged diff for pre-commit hooks.

### Vitest (TypeScript) — Dependency-Aware Filtering

Vitest tracks test dependencies automatically. Use `--changed` to run
only tests affected by changes since a ref, or `--related` to run
tests related to specific files.

```bash
# Run only tests affected by changes since origin/main
vitest run --changed origin/main

# Run only tests related to a specific changed file
vitest run --related src/components/Button.tsx

# Force re-run of affected tests (ignore cache)
vitest run --changed origin/main --force
```

Vitest's `--changed` uses the module graph to determine which tests
import the changed modules — no manual dependency tracking needed.

### Jest (TypeScript/JavaScript) — Selective Re-runs

Jest supports `--changedSince` and `--changedFilesWithAncestor` for
git-based selective runs, and `--findRelatedTests` for file-based
filtering.

```bash
# Run only tests for files changed since origin/main
jest --changedSince=origin/main

# Run tests related to specific changed files
jest --findRelatedTests src/components/Button.tsx src/utils/format.ts

# Combine with coverage for affected files only
jest --changedSince=origin/main --coverage --collectCoverageFrom='src/**/*.{ts,tsx}'
```

Jest uses its own dependency graph (powered by Metro/Haste) to map
changed source files to the tests that import them.

### pytest-testmon (Python) — Dependency-Aware Selection

`pytest-testmon` tracks which tests cover which lines of code, then
runs only the tests whose covered lines changed.

```bash
# Install
pip install pytest-testmon

# Run only tests affected by changes since last commit
pytest --testmon

# Run with a specific base commit for CI
pytest --testmon --testmon-forceselect  # ignore cache, recompute

# In CI: compare against the merge-base
git fetch origin main
pytest --testmon
```

`pytest-testmon` writes a `.testmondata` file mapping test IDs to
covered source files. On each run, it checks which source files
changed (via git) and runs only the tests that cover them.

For projects without `pytest-testmon`, `pytest --testmon` can be
approximated with `pytest-testmon`'s lighter alternative,
`pytest-picked`, which runs tests for files listed in `git status`:

```bash
pip install pytest-picked
pytest --picked
```

### cargo-nextest (Rust) — Sharding and Filtering

`cargo-nextest` is a faster test runner for Rust that supports
sharding and retry of flaky tests natively.

```bash
# Install
cargo install cargo-nextest

# Run with sharding — split across N runners
cargo nextest run --partition count:1/4  # shard 1 of 4
cargo nextest run --partition count:2/4  # shard 2 of 4
# ... in CI, matrix.shard index selects the partition

# Run only tests matching a filter (e.g., affected crate)
cargo nextest run -p my-changed-crate

# Run with retry for flaky test detection
cargo nextest run --retries 2
```

For git-based selective runs in Rust, use `cargo-nextest` with a
filter derived from changed crates:

```bash
CHANGED_CRATES=$(git diff --name-only origin/main...HEAD | \
  grep -E '^crates/[^/]+/' | cut -d/ -f2 | sort -u)
for crate in $CHANGED_CRATES; do
  cargo nextest run -p "$crate"
done
```

### Auto-Generated Documentation — Git-Based Selective Runs

The same git-based selective-run principle applies to documentation
generation. API docs, type docs, and schema docs should only
regenerate when the source files that feed them change.

```bash
# GitHub Actions — skip docs job if no source files changed
docs:
  needs: detect-changes  # or use dorny/paths-filter action
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Check if docs sources changed
      id: docs-changed
      run: |
        CHANGED=$(git diff --name-only origin/main...HEAD | \
          grep -qE '^(src/|docs/api/|schemas/)' && echo "true" || echo "false")
        echo "changed=$CHANGED" >> "$GITHUB_OUTPUT"
    - name: Generate docs
      if: steps.docs-changed.outputs.changed == 'true'
      run: pnpm run docs:generate
```

Using `dorny/paths-filter` for cleaner multi-path detection:

```yaml
- uses: dorny/paths-filter@v3
  id: filter
  with:
    filters: |
      docs:
        - 'src/**/*.ts'
        - 'schemas/**/*'
        - 'docs/api/**/*'
- name: Generate API docs
  if: steps.filter.outputs.docs == 'true'
  run: pnpm run docs:generate
```

## Prevention

1. **Set a wall-clock budget.** Define a target suite time (e.g., unit
   tests under 30 seconds, full suite under 5 minutes). When the suite
   exceeds the budget, treat it as a performance regression — fix it
   before merging new features.
2. **Run affected tests by default.** Make `--changed` / `--testmon` /
   `--findRelatedTests` the default in local and CI runs. Run the full
   suite nightly or on the main branch to catch cross-cutting
   regressions.
3. **Shard in CI.** Split the suite across 4-8 runners using
   deterministic shard assignment. The wall-clock time should scale
   with runner count, not test count.
4. **Cache aggressively.** Dependency directories, build outputs, and
   test result caches should be restored between runs. A warm CI run
   should be significantly faster than a cold one.
5. **Split by speed tier.** Run unit tests first (fast, fail fast),
   then integration and E2E in parallel. Never make the slow tier
   optional — it stays in the blocking path, just parallelized.
6. **Monitor suite time.** Track suite wall-clock time over time. A
   slow upward trend means tests are being added without parallelism or
   selectivity — catch it before it crosses the budget.
7. **Apply git-based selective runs to docs generation.** Docs jobs
   that regenerate on every commit waste CI minutes. Gate them on
   whether the source files that feed the docs actually changed.

## Related Concepts

- [Pre-Commit CI Parity](pre-commit-ci-parity.md) — Fast feedback
  starts locally; the pre-commit hook runs the same affected-test
  selection as CI
- [CI Matrix Strategy](ci-matrix-strategy.md) — Sharding is a matrix
  strategy; `fail-fast: false` ensures every shard reports
- [Vitest Unified Runner](vitest-unified-runner.md) — Vitest's
  `--changed` and `--related` flags provide built-in dependency-aware
  filtering
- [Test Determinism](test-determinism.md) — Selective re-runs require
  deterministic tests; a flaky test that is skipped cannot be trusted
- [Test Isolation — Mock Pollution](test-isolation-mock-pollution.md)
  — Parallel execution requires test isolation; mock pollution causes
  cross-test interference under parallelism
