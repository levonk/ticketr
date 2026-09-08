---
type: Practice
title: Auto-Detecting Devbox Targets (formerly Internal vs Normal Targets)
description: Single-target just pattern that auto-detects devbox environment via DEVBOX_SHELL_ENABLED. No more -internal suffix thinking overhead. DRY _devbox helper handles detection, re-exec, and doctor fallback. Implementation lives in _impl targets.
tags: [just, devbox, targets, naming-convention, workflow, auto-detection]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-28"
sources:
  - id: levonk-base-boilerplate
    resource: internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md
    title: levonk-base-boilerplate
---

# Auto-Detecting Devbox Targets

## Failure Mode

The previous two-tier convention (`just build` vs `just build-internal`) required
developers and AI agents to constantly decide: "Am I in a devbox shell? Should I
use the normal target or the internal target?" This cognitive overhead led to:

- AI agents double-wrapping `devbox run` calls (calling `devbox run -- just build`
  when already inside devbox)
- Developers calling `*-internal` targets without the environment active, leading
  to missing tools and cryptic errors
- Inconsistent usage across CI, local dev, and AI agent sessions

## Practice

Use a **single-target auto-detection pattern**. Each public target delegates to
a shared `_devbox` helper that checks `DEVBOX_SHELL_ENABLED` and either runs the
implementation directly, re-execs via `devbox run`, or falls back to `doctor`.

### The `_devbox` Helper (DRY Detection)

One helper recipe handles all devbox detection logic. Every normal target
delegates to it:

```just
# Devbox auto-detection: run impl target directly if in devbox,
# re-exec via devbox run if not, or fail with doctor diagnostic.
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
```

### Normal Targets (Developer Interface)

Normal targets are one-liners that delegate to `_devbox`:

```just
build:
    just _devbox build_impl

test:
    just _devbox test_impl

lint:
    just _devbox lint_impl
```

Flow: `just build` → `_devbox build_impl` → (in devbox) `just build_impl` → `cargo build`
Flow: `just build` → `_devbox build_impl` → (not in devbox) `devbox run -- just build_impl` → `cargo build`

### Implementation Targets (`*_impl` suffix, underscore-prefixed)

Implementation targets contain the **actual commands** — the language-specific
commands that run inside the devbox environment. The underscore prefix hides
them from `just --list`:

```just
build_impl:
    cargo build

test_impl:
    cargo test

lint_impl:
    cargo clippy -- -D warnings
```

### Why `*_impl` Instead of `*-internal`

The `*_impl` naming replaces the old `*-internal` convention:

- **`_` prefix** hides implementation targets from `just --list` output — the
  user never sees them unless they explicitly ask
- **`_impl` suffix** is shorter and clearer ("implementation") than "-internal"
- **No thinking overhead**: the user always types `just build`, never `just build-internal`
- **Power users** can still call `just build_impl` directly if they know they're
  in devbox (e.g., inside `devbox shell`), but it's never required

### The `doctor` Target (Special Case)

`doctor` is the **fallback diagnostic** — it must NOT use `_devbox` because it
runs when devbox is missing. It checks tools directly:

```just
doctor:
    #!/usr/bin/env bash
    echo "🔍 Checking development environment..."
    if ! command -v devbox >/dev/null 2>&1; then
        echo "❌ devbox: NOT FOUND"
        echo "💡 Install: https://www.jetify.com/devbox/docs/installation"
    else
        echo "✅ devbox: $(devbox version 2>&1)"
    fi
    if ! command -v just >/dev/null 2>&1; then
        echo "❌ just: NOT FOUND"
    else
        echo "✅ just: $(just --version 2>&1)"
    fi
    # ... check language-specific tools
```

### Why Devbox Scripts Point to `*_impl`

Devbox scripts in `devbox.json` call `*_impl` targets directly because
automated systems (CI/CD, init_hooks) are already in the devbox environment.
Calling normal targets would add an unnecessary `_devbox` → `just` round-trip:

```json
{
  "scripts": {
    "build": "just build_impl",
    "test": "just test_impl"
  }
}
```

### Required Targets

All boilerplate projects MUST include:

- `build`, `test`, `lint`, `dev`, `typecheck` (normal — delegate to `_devbox`)
- `build_impl`, `test_impl`, `lint_impl`, `dev_impl` (implementation)
- `bootstrap`, `bootstrap_impl` (initialization)
- `prime`, `prime_impl` (async warmup: downloads + build + list + API docs; see [Async Prime Internal](async-prime-internal.md))
- `doctor` (health check — runs directly, NOT via `_devbox`)
- `clean` (cleanup — delegates to `_devbox`)

## Migration from `*-internal` to `*_impl`

For existing projects using the old `*-internal` convention:

1. Rename `build-internal` → `build_impl`, `test-internal` → `test_impl`, etc.
2. Add the `_devbox` helper recipe
3. Replace normal target bodies (`devbox run build`) with `just _devbox build_impl`
4. Update `devbox.json` scripts: `"build": "just build_impl"` (was `"just build-internal"`)
5. Update `.envrc`: `devbox run -- just prime_impl` (was `just prime-internal`)
6. Remove any `*-internal` aliases after confirming no external callers remain

## Related Concepts

- [Standard Developer UX Flow](standard-developer-ux-flow.md) — The three-flow pattern (updated for auto-detection)
- [Async Prime Internal](async-prime-internal.md) — The async warmup job set and fire-and-forget pattern
- [Just Over Makefiles](just-over-makefiles.md) — Why just is the task runner
- [Devbox Script Generation Bug](devbox-script-generation-bug.md) — Why auto-detection makes this bug irrelevant
