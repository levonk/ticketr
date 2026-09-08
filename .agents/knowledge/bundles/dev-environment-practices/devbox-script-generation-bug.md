---
type: Practice
title: Devbox Script Generation Bug
description: Known devbox v0.14.x regression where script generation fails with "command not found". With the auto-detection pattern, this bug is largely irrelevant — just targets handle devbox detection themselves. Workarounds still documented for edge cases.
tags: [devbox, bug, workaround, script-generation, regression, troubleshooting, auto-detection]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-28"
sources:
  - id: levonk-base-boilerplate-troubleshooting-section
    resource: internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md
    title: levonk-base-boilerplate (Troubleshooting section)
---

# Devbox Script Generation Bug

## Failure Mode

`devbox run <command>` fails with "command not found" error despite proper
`devbox.json` configuration. The generated script in
`.devbox/gen/scripts/.cmd.sh` fails to execute just commands.

### Symptoms

```bash
devbox run view-web
# Error: /path/to/.devbox/gen/scripts/.cmd.sh: line 7: view-web: command not found
```

### Root Cause

Known devbox regression bug introduced in v0.14.x series affecting script
generation. This is a confirmed upstream bug, not a configuration issue.

### Affected Versions

devbox 0.14.0, 0.14.2, 0.16.0 (and likely other 0.14.x+ versions)

### GitHub Issues

- #2517: `error: tool 'git' not found` after upgrading to 0.14.0
- #2108: Running script inside devbox shell throws `file not found` error
- #2607: Cannot run devbox script if another script is sourced in the init hook

## Practice

### Why Auto-Detection Makes This Bug Largely Irrelevant

With the [auto-detection pattern](internal-vs-normal-targets.md), `just build`
handles devbox environment detection itself via the `_devbox` helper. You don't
need `devbox run <script>` to work — you just use `just <target>` directly:

```bash
just build       # Instead of: devbox run build
just test        # Instead of: devbox run test
just view-web    # Instead of: devbox run view-web
```

If you're not in a devbox shell, `just build` calls `devbox run -- just build_impl`
internally. This uses `devbox run --` (the raw form) rather than
`devbox run <script-name>`, which bypasses the broken script generation path.

If you ARE in a devbox shell (via `devbox shell` or direnv), `just build` detects
`DEVBOX_SHELL_ENABLED=1` and runs the implementation directly — no `devbox run`
at all.

### Workaround 1: Use just Directly (Recommended)

Bypass broken devbox scripts entirely:

```bash
just view-web    # Instead of: devbox run view-web
just run         # Instead of: devbox run run
just export      # Instead of: devbox run export
```

### Workaround 2: devbox shell + direct targets (Most Reliable)

Enter devbox shell first, then use targets directly:

```bash
devbox shell
just bootstrap_impl
just view-web_impl
just run_impl
```

### Workaround 3: Version Rollback

```bash
export DEVBOX_USE_VERSION=0.13.7
devbox run view-web  # Now works
```

## Prevention

1. **Prefer `just <target>` commands**: The auto-detection pattern means `just`
   handles devbox detection — `devbox run <script>` is never needed for standard
   targets
2. **Test devbox scripts after setup**: Verify `devbox run <script>` works for
   any custom scripts not covered by just targets
3. **Add bug comments**: Include known bug reference in devbox.json files
4. **Monitor devbox issues**: Track upstream fixes for script generation regression

## Detection

```bash
devbox run --help       # Should show available scripts
devbox run <script>     # Test each script
just --list             # Shows available just targets (includes auto-detecting ones)
```

## Related Concepts

- [Auto-Detecting Devbox Targets](internal-vs-normal-targets.md) — The `_devbox`
  helper pattern that makes this bug largely irrelevant
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — Flows that
  account for this bug via auto-detection
