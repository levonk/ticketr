---
type: Practice
title: Devbox Broken Override — Direct Package-Manager Fallback
description: When devbox is broken on a platform (nixpkgs pin missing a package, architecture-specific failure), override devbox-wrapped commands with direct package-manager equivalents rather than blocking. Document the override so subagents use the direct form without re-discovering the breakage.
tags: [devbox, broken, override, fallback, pnpm, nixpkgs, platform, intel-mac, troubleshooting]
date:
  created: "2026-07-30"
  knowledge-basis: "2026-07-30"
  last-used: "2026-07-30"
sources:
  - id: seodata-execute-workflow-2c23734
    resource: https://github.com/levonk/seodata/commit/2c23734
    title: 'seodata-execute.md: devbox-broken override guidance'
  - id: devbox-script-generation-bug
    resource: devbox-script-generation-bug.md
    title: 'Companion concept: devbox script generation bug (different failure mode, same family)'
---

# Devbox Broken Override — Direct Package-Manager Fallback

## Failure Mode

Devbox fails to activate or build the environment on a specific platform —
typically because a nixpkgs pin references a package that no longer builds on
that architecture (e.g., the `jujitsu` package missing from the nixpkgs pin on
Intel Macs). The failure is **platform-specific**: devbox works on other
machines, so the project's `devbox.json` is not wrong, but on the affected
machine every `devbox run --` command fails.

### Symptoms

```bash
$ devbox run -- just build
# Error: failed to build devbox environment
# Error: package 'jujitsu' not found in nixpkgs pin
```

Every `devbox run -- <cmd>` fails with the same root cause. The project's
`just` targets that auto-detect devbox (via the `_devbox` helper) also fail
because they re-exec through `devbox run --`.

### Root Cause

A nixpkgs version pinned in `devbox.json` (or `devbox.lock`) references a
package that has been removed or fails to build on the target architecture.
This is an upstream nixpkgs issue, not a project configuration issue — the same
`devbox.json` works on other platforms.

### Distinct from the Script Generation Bug

[Devbox Script Generation Bug](devbox-script-generation-bug.md) covers a
devbox v0.14.x regression where `devbox run <script>` fails but `devbox run --
<cmd>` still works. This concept covers the broader case where devbox itself
cannot build the environment at all — neither `devbox run <script>` nor
`devbox run -- <cmd>` works. The workaround is different: instead of routing
around the broken script generation, you route around devbox entirely.

## Practice

> **Before bypassing devbox, check whether per-platform pinning can fix the
> issue.** [Per-Platform Devbox Package Entries](devbox-per-platform-packages.md)
> documents a pattern where broken packages on one platform get a pinned
> `github:nixos/nixpkgs/<channel>#pkg` entry for that platform only, while
> `@latest` is used everywhere else. This keeps devbox functional on ALL
> platforms. The bypass below is the escalation path when per-platform
> pinning is not viable (too many packages broken on one platform).

### Override Devbox-Wrapped Commands with Direct Equivalents

When devbox is broken on a platform, do not block — override the
devbox-wrapped commands with direct package-manager equivalents. The
project's tools (pnpm, tsc, eslint, vitest, cargo, etc.) are typically
available outside devbox via the user's global install or nix-profile. Use
them directly.

**Decision table** — replace each `devbox run -- just <target>` with the
underlying command the target wraps:

| Devbox-wrapped | Direct fallback | Notes |
|----------------|-----------------|-------|
| `devbox run -- just test-internal` | `pnpm exec vitest run` | Replace `just` target with the underlying test runner |
| `devbox run -- just typecheck-internal` | `pnpm exec tsc --noEmit` | Replace with the underlying typechecker |
| `devbox run -- just lint-internal` | `pnpm exec eslint src --ext .ts` | Replace with the underlying linter |
| `devbox run -- just build-internal` | `pnpm exec <build-cmd>` | Replace with the underlying build tool |
| `devbox run -- just <custom>` | Read the `justfile` to find the underlying command | The `justfile` is the source of truth for what each target wraps |

For non-JavaScript projects, adapt the fallback to the project's toolchain:

| Ecosystem | Direct fallback |
|-----------|-----------------|
| Rust | `cargo build`, `cargo test`, `cargo clippy` |
| Python | `uv run pytest`, `uv run ruff check`, `uv run mypy` |
| Go | `go build ./...`, `go test ./...`, `golangci-lint run` |

### Document the Override

Record the override in the project's execution workflow file (e.g.,
`.agents/workflows/<project>-execute.md`) so every subagent uses the direct
form without re-discovering the breakage on every story:

```markdown
- **Devbox is currently broken** (<reason>). Use `pnpm` and `pnpm exec`
  directly for all commands. Override the story files' acceptance criteria
  commands: replace `devbox run -- just test-internal` with
  `pnpm exec vitest run`, `devbox run -- just typecheck-internal` with
  `pnpm exec tsc --noEmit`, `devbox run -- just lint-internal` with
  `pnpm exec eslint src --ext .ts`.
```

Without this documentation, each subagent independently hits the devbox
failure, tries to diagnose it, and either blocks or re-discovers the
fallback — wasting time on every story in the pipeline.

### When to Fix Devbox vs Override

| Situation | Action |
|-----------|--------|
| Devbox broken on one developer's machine, works for others | Override locally; the nixpkgs pin will be fixed upstream eventually |
| Devbox broken for the whole team | Fix the `devbox.json` pin (update `devbox.lock`) — this is a project-level fix, not a local override |
| Devbox broken in CI only | Fix CI — CI should not depend on a broken environment; pin a known-good nixpkgs revision |
| Devbox broken on a new architecture (e.g., aarch64-darwin) | File an upstream issue; override locally until the pin is updated |

## Prevention

1. **Pin nixpkgs to a stable channel** — use `nixos-unstable` or a specific
   revision rather than `nixpkgs-unstable` to reduce the chance of pulling a
   broken package.
2. **Test devbox on all target architectures** before merging a `devbox.json`
   change — CI should run on every architecture the team uses.
3. **Keep the override table in the workflow file** — even when devbox is
   working, having the fallback documented means a future breakage is a
   zero-discovery event: the subagent reads the workflow file and uses the
   direct form.
4. **Update `devbox.lock` regularly** — `devbox update` pulls the latest
   nixpkgs revisions that are known to build. Stale locks accumulate
   breakage.

## Detection

```bash
# Quick check: does devbox build the environment?
devbox run -- echo "devbox ok"
# If this fails, devbox is broken on this machine

# Check which package is failing
devbox run -- nix-store --query --references $(which devbox) 2>&1 | head
# Or read the error message — it names the missing package
```

## Related Concepts

- [Per-Platform Devbox Package Entries](devbox-per-platform-packages.md) —
  the preferred solution before escalating to the bypass. When only one or a
  few packages are broken on one platform, per-platform pinning keeps devbox
  functional everywhere. The bypass is the escalation when pinning is not
  viable.
- [Devbox Script Generation Bug](devbox-script-generation-bug.md) — a
  different devbox failure mode where `devbox run <script>` fails but
  `devbox run -- <cmd>` works. The workaround there is to use `just` targets
  directly; here, the workaround is to bypass devbox entirely.
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — the
  three-flow pattern that assumes devbox works. This concept is the fallback
  when Flow 1 (devbox shell) is unavailable.
- [Auto-Detecting Devbox Targets](internal-vs-normal-targets.md) — the
  `_devbox` helper that auto-detects devbox. When devbox is broken, the
  helper's `devbox run --` re-exec fails; the override bypasses the helper.
