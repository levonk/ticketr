---
type: Practice
title: Standard Developer UX Flow
description: Three-flow developer UX pattern — direnv → devbox → just (auto-detecting) → [build tool]. Single-target auto-detection via DEVBOX_SHELL_ENABLED eliminates the -internal thinking overhead. Technology-agnostic build tool mapping.
tags: [developer-experience, devbox, just, workflow, ai-agents, build-tools, auto-detection]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-28"
sources:
  - id: levonk-base-boilerplate
    resource: internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md
    title: levonk-base-boilerplate
---

# Standard Developer UX Flow

## Failure Mode

Inconsistent commands across projects create cognitive overhead. AI agents
struggle with environment drift. Developers don't know whether to use `devbox
run`, `just`, or direct language tools. The old two-tier pattern (`just build`
vs `just build-internal`) required constant awareness of whether the devbox
environment was active.

## Practice

Define three standard flows that cover all developer personas. The core
pattern is: `direnv → devbox → just (auto-detecting) → [build tool]`.

The key insight: **there is only one target name per action**. `just build`
auto-detects whether it's inside devbox (via `DEVBOX_SHELL_ENABLED`) and either
runs the implementation directly or re-execs via `devbox run`. No more
"should I use `-internal`?" thinking.

### Technology-Agnostic Build Tools

Each project type uses its native build tools:

| Technology | Build | Test | Lint | Dev |
|------------|-------|------|------|-----|
| Rust | `cargo build` | `cargo test` | `cargo clippy` | `cargo run` |
| Node.js | `nx build` | `nx test` | `nx lint` | `nx dev` |
| Python | `python -m build` | `pytest` | `ruff check` | `uv run python src/main.py` |
| Go | `go build` | `go test` | `golangci-lint run` | `go run` |
| Java | `mvn compile` | `mvn test` | `checkstyle` | `mvn exec:java` |

### Flow 1: AI Agent / CI (Primary)

```bash
just build
just test
just lint
```

`just build` auto-detects the devbox environment. If `DEVBOX_SHELL_ENABLED=1`
(set by `devbox run`, `devbox shell`, or `devbox shellenv`), it runs the
implementation directly. If not, it re-execs via `devbox run -- just build_impl`.
If devbox is missing, it runs `just doctor` and fails with a diagnostic.

No need to prefix with `devbox run --` — the target handles that automatically.

### Flow 2: Novice Developer

```bash
just build
# Flow: just build → _devbox build_impl → devbox run -- just build_impl → cargo build
```

Same command as Flow 1. The auto-detection handles everything. Simpler command
interface for daily development — no environment awareness required.

### Flow 3: Power User (in devbox shell)

```bash
devbox shell
just build
# Flow: just build → _devbox build_impl → (DEVBOX_SHELL_ENABLED=1) → just build_impl → cargo build
```

Already in devbox environment via direnv. `just build` detects
`DEVBOX_SHELL_ENABLED=1` and runs the implementation directly — no `devbox run`
wrapper, no `-internal` suffix to remember.

Power users can also call `just build_impl` directly to skip the detection
overhead entirely (one fewer `just` invocation), but this is never required.

### Bootstrap Flow

```bash
cd project
# direnv auto-activates devbox
# devbox init_hook calls bootstrap_impl
# .envrc async-triggers prime_impl (fire-and-forget warmup)
# bootstrap calls prime_impl for cache warming
```

### Prime Flow (sync checkpoint + async warmup)

`prime_impl` has two phases. See [Async Prime Internal](async-prime-internal.md)
for the full pattern.

**Phase 1 (sync): Git checkpoint** — commits any pending work as a single
checkpoint commit (no push) so there's a safe rollback point before warmup.
Follows the `pre-task-commit-checkpoint` protocol from the
`git-repository-management` skill. Skippable via `PRIME_SKIP_CHECKPOINT=1`.

**Phase 2 (async, fire-and-forget): Cache warmup** — kicks off cache-warming
jobs in parallel:

- **Download packages** (`cargo fetch`, `pnpm install --frozen-lockfile`, `uv sync --frozen`) — warms package cache
- **Build** (`just build_impl`) — warms compiler/build cache
- **List** (`just --list`) — discovers recipes for AI agent context
- **Generate API doc** (if `has_docs`) — warms doc cache

Verification gates (typecheck, test, validate) are NOT run in prime — they
stay synchronous and blocking. The rule: if a failure means the agent should
stop and fix it, it's synchronous; if a failure just means the cache didn't
warm, it's async.

### The `_devbox` Helper (DRY Auto-Detection)

All normal targets delegate to a single `_devbox` helper recipe that handles
devbox environment detection:

```just
_devbox target *args:
    #!/usr/bin/env bash
    if [ "${DEVBOX_SHELL_ENABLED:-0}" = "1" ]; then
        exec just "{{target}}" {{args}}
    elif command -v devbox >/dev/null 2>&1; then
        exec devbox run -- just "{{target}}" {{args}}
    else
        echo "❌ devbox not found in PATH." >&2
        echo "💡 Running doctor to diagnose environment issues..." >&2
        just doctor 2>/dev/null || true
        exit 1
    fi

build:
    just _devbox build_impl

build_impl:
    cargo build
```

### Devbox Scripts Configuration

```json
{
  "scripts": {
    "bootstrap": "just bootstrap_impl",
    "prime": "just prime_impl",
    "doctor": "just doctor",
    "clean": "just clean_impl",
    "build": "just build_impl",
    "lint": "just lint_impl",
    "test": "just test_impl",
    "dev": "just dev_impl"
  }
}
```

Devbox scripts point to `*_impl` targets directly because automated systems are
already in the devbox environment — no need for the `_devbox` detection wrapper.

Note: `doctor` has no `_impl` variant — it runs directly because it's the
fallback when devbox is missing.

## Related Concepts

- [Auto-Detecting Devbox Targets](internal-vs-normal-targets.md) — The `_devbox` helper and `*_impl` naming convention
- [Async Prime Internal](async-prime-internal.md) — The async warmup pattern (downloads + build + list + docs in parallel; verification gates stay sync)
- [Just Over Makefiles](just-over-makefiles.md) — Why just is the task runner
- [Mandatory Testing Workflow](mandatory-testing-workflow.md) — Testing gates for all flows
- [Devbox Script Generation Bug](devbox-script-generation-bug.md) — Why auto-detection makes this bug irrelevant
