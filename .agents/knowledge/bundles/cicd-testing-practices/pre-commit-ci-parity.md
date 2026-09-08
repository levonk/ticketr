---
type: Practice
title: Pre-Commit CI Parity
description: Same checks run locally and in CI via shared Dockerized quality script. FAST_MODE for local, FULL_MODE for CI. Includes workflow security linting (zizmor + actionlint) as a standard CI check. Eliminates "works on my machine" and ensures consistent enforcement.
tags: [pre-commit, ci-cd, parity, quality, docker, consistency, zizmor, actionlint, workflow-security]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-08-27"
  last-used: "2026-08-27"
sources:
  - id: adr-20251218002-shared-quality-scripts
    resource: "internal-docs/adr/adr-20251218002-shared-quality-scripts.md"
    title: "levonk-base-boilerplate"
  - id: zizmor
    resource: "https://github.com/woodruffw/zizmor"
    title: "zizmor — GitHub Actions workflow security analyzer"
  - id: actionlint
    resource: "https://github.com/rhysd/actionlint"
    title: "actionlint — GitHub Actions workflow syntax linter"
---


# Pre-Commit CI Parity

## Failure Mode

Different checks run locally vs CI. "Works on my machine" syndrome. Drift
between hook and CI tool versions. Developers bypass slow checks by skipping
hooks, then CI fails unexpectedly. Workflow YAML files are never linted for
security or syntax — unpinned actions, excessive permissions, and schema errors
reach GitHub CI undetected.

## Practice

**Same checks run locally and in CI** via the shared Dockerized quality script.
Workflow security linting (zizmor + actionlint) runs as a dedicated CI job on
every push and PR.

### How It Works

1. **Pre-commit hook**: Calls `scripts/run-quality-checks.sh` with `FAST_MODE=1`
   for quick feedback
2. **GitHub Actions**: Calls same script with `FULL_MODE=1` for complete
   validation
3. **Same Docker images**: Same pinned tool versions in both contexts
4. **Same scanner configs**: Same rules, same thresholds
5. **Workflow security job**: A dedicated `workflow-security` job runs
   **zizmor** (workflow security analyzer) and **actionlint** (workflow
   syntax/schema linter) on all `.github/workflows/*.yml`. Independent of the
   test job so security findings are visible even when tests fail.

### Workflow Security Linting

The `workflow-security` job is the standard CI security baseline for every
adopted project. It catches the class of bugs that quality scripts cannot:

- **zizmor**: unpinned actions (mutable `@vN` or `@main` refs), excessive
  permissions (`permissions: write-all`), OIDC token misuse, secret injection
  via `pull_request_target`, untrusted checkout patterns.
- **actionlint**: syntax errors, schema violations, deprecated syntax,
  undefined secrets, invalid expressions, job dependency cycles.

Both tools are available via `nix run nixpkgs#zizmor` and
`nix run nixpkgs#actionlint` (no installation needed if Nix is available).
Fallbacks: `uvx zizmor` (pip) and `cargo binstall actionlint` (Rust).

```yaml
# .github/workflows/ci.yml — dedicated workflow-security job
workflow-security:
  runs-on: ubuntu-latest
  permissions:
    contents: read
  steps:
    - uses: actions/checkout@v4
    - name: Run zizmor
      run: nix run nixpkgs#zizmor -- .github/workflows/
    - name: Run actionlint
      run: nix run nixpkgs#actionlint -- .github/workflows/*.yml
```

### Developer Experience

- `FAST_MODE=1`: Lint only, runs in seconds
- `MARKDOWN_ONLY=1`: For docs-only changes
- `SKIP_RUNTIME_SCAN=1`: Skip heavy runtime scans
- `FULL_MODE=1`: Everything (CI default)
- `workflow-security` job: Runs in CI only — zizmor and actionlint are
  workflow-specific tools that don't have a local pre-commit equivalent in the
  shared quality script. Run them locally with `nix run nixpkgs#zizmor --
  .github/workflows/` before pushing if you changed workflow files.

### Benefits

- **Consistent enforcement**: Same checks, same results
- **Faster onboarding**: One interface to learn
- **Easier upgrades**: Patch tooling centrally in one file
- **No surprises**: If it passes locally, it passes in CI
- **Workflow security**: Unpinned actions and schema errors caught before
  they reach GitHub CI, not after a security incident

## Related Concepts

- [Shared Quality Scripts](shared-quality-scripts.md) — The script that enables
  parity
- [Vitest Unified Runner](vitest-unified-runner.md) — Tests run through this
  parity pipeline
- [Dependency Supply Chain](https://github.com/levonk/skills-releases/blob/main/knowledge/devsecops-codeguard/dependency-supply-chain.md)
  — zizmor extends supply-chain security to GitHub Actions workflow files
