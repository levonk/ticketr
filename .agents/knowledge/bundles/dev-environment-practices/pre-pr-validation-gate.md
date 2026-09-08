---
type: Practice
title: Pre-PR Validation Gate
description: A single command runs all checks — type-check, lint, format, tests, and bundled-asset verification — before a pull request is opened. One gate, zero bypass, CI parity by construction.
tags: [validation, pre-pr, quality-gate, ci-parity, single-command, developer-experience]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Pre-PR Validation Gate

## Failure Mode

Developers run lint locally, skip type-check because it is slow, forget
format checking, and push. CI then fails on a check the developer never
ran — sometimes hours later, after context has shifted. The feedback loop
stretches from seconds to hours, and the developer must context-switch back
to a change they thought was done.

The root cause is **check enumeration drift**: the set of checks that CI
runs lives in a workflow file, while the set of checks a developer runs
lives in their head (or in scattered `just lint`, `just test` invocations).
No mechanism guarantees the two sets are identical.

## Practice

Define a **single command** that runs every check CI will run, in the same
order, with the same thresholds. This command is the only acceptable
pre-PR verification step. If it passes locally, CI passes — by
construction, not by hope.

### Core Principles

1. **One command, all checks**: No partial runs. The gate either runs
   everything or it is not the gate.
2. **CI parity by construction**: The gate invokes the same scripts (or
   script invocations) that CI invokes. See
   [Pre-Commit CI Parity](https://github.com/levonk/skills-releases/blob/main/knowledge/cicd-testing-practices/pre-commit-ci-parity.md)
   for the shared-script pattern that makes this trivially true.
3. **Zero-tolerance thresholds**: Lint runs with `--max-warnings 0` (or
   equivalent). A warning is a failure. No "I will fix it later."
4. **Bundled-asset verification**: If the project ships generated or
   bundled assets (schemas, default configs, skill bundles), the gate
   verifies they are up to date — not just that the code compiles.
5. **Fail fast, report all**: The gate stops on the first failure with a
   clear message. It does not silently continue past broken checks.

### What the Gate Includes

A complete gate covers every check that CI enforces:

| Check | Purpose |
|-------|---------|
| Bundled-asset check | Generated files are in sync with their sources |
| Type-check | No type errors across the workspace |
| Lint (zero warnings) | Code style and correctness rules enforced |
| Format check | Whitespace and formatting match project config |
| Tests | All unit, integration, and installer tests pass |

### Relationship to Existing Practices

This page adds the **single-gate** angle: the assertion that a pre-PR
check is only trustworthy when it is one command that covers everything.
Two existing pages cover complementary concerns:

- [Mandatory Testing Workflow](mandatory-testing-workflow.md) defines
  **what** must be tested (TDD, regression tests, coverage requirements)
  and the quality-gate mindset. The validation gate is the **mechanism**
  that enforces it as a single command.
- [Pre-Commit CI Parity](https://github.com/levonk/skills-releases/blob/main/knowledge/cicd-testing-practices/pre-commit-ci-parity.md)
  defines the **shared-script** pattern that makes local and CI run the
  same checks. The validation gate is the **entry point** that invokes
  those shared scripts in sequence.

The gate does not replace either page — it sits on top of them, providing
the single-command contract that eliminates "I forgot to run X."

## Concrete Instances

### Bun workspace (TypeScript)

```json
{
  "scripts": {
    "validate": "bun run check:bundled && bun run check:bundled-skill && bun run check:bundled-schema && bun run type-check && bun run lint --max-warnings 0 && bun run format:check && bun run test:install && bun run test"
  }
}
```

`bun run validate` runs 7+ checks in sequence: bundled-defaults
verification, bundled-skill verification, bundled-schema verification,
type-check, lint with `--max-warnings 0`, format check, installer tests,
and the full test suite. All must pass for CI to succeed. The command is
documented in the project AGENTS.md as the mandatory pre-PR step.

### Just + Devbox (multi-language)

```just
validate:
    just _devbox validate_impl

validate_impl:
    just check-bundled
    just typecheck
    just lint
    just format-check
    just test
```

`just validate` auto-detects devbox (via the `_devbox` helper from
[Standard Developer UX Flow](standard-developer-ux-flow.md)) and runs all
checks inside the reproducible Nix environment. The same `validate_impl`
target is called by CI, ensuring parity.

### Cargo Make (Rust)

```toml
[tasks.validate]
dependencies = ["fmt-check", "clippy", "test"]
run_task = { name = "audit" }
```

`cargo make validate` chains format check, clippy with `-D warnings`,
tests, and a security audit into a single invocation. CI calls the same
task, so local and CI are identical by construction.

## Prevention

1. **Document the gate in AGENTS.md** — state explicitly: "Always run
   before creating a pull request." Make it the first thing a new
   contributor reads.
2. **Never bypass** — if the gate fails, fix the issue. Do not comment
   out a check to "unblock" a PR. A bypassed gate is worse than no gate
   because it creates the illusion of verification.
3. **Keep the gate fast** — if the full gate takes more than a few
   minutes, provide a fast subset for iterative development (e.g.,
   `FAST_MODE=1` in the shared quality script), but the full gate
   remains mandatory before PR creation.
4. **Update the gate when CI changes** — any new check added to CI must
   be added to the gate in the same commit. The gate and CI are a
   single contract split across two locations.

## Related Concepts

- [Mandatory Testing Workflow](mandatory-testing-workflow.md) — Defines
  what must be tested; the validation gate is the mechanism that enforces it
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — The
  `_devbox` auto-detection pattern that makes `just validate` work inside
  and outside devbox
- [Pre-Commit CI Parity](https://github.com/levonk/skills-releases/blob/main/knowledge/cicd-testing-practices/pre-commit-ci-parity.md)
  — The shared-script pattern that makes local and CI run identical checks
- [Shared Quality Scripts](https://github.com/levonk/skills-releases/blob/main/knowledge/cicd-testing-practices/shared-quality-scripts.md)
  — The Dockerized script that both the gate and CI invoke
