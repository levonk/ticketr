---
type: Practice
title: Wrapper Probe Caching — Cache Per-Command Subprocess Probes
description: Shell scripts that wrap every command through environment detection (devbox, rtk, cli-tool-discovery) must cache probe results. Un-cached probes spawn a subprocess per command invocation, and each subprocess may pay a 15s timeout. Under parallel agent load, N commands × M sessions × timeout compounds into multi-minute hangs. Cache probe results for the script lifetime; honor known-broken flags in downstream probes to skip re-probing entirely.
tags: [shell-scripting, caching, devbox, wrapper, probe, subprocess, parallel-agents, performance, rtk, cli-tool-discovery]
date:
  created: "2026-08-16"
  knowledge-basis: "2026-08-16"
  last-used: "2026-08-16"
sources:
  - id: rtk-helpers-uncached-probe-hang
    resource: src/current/includes/rtk-helpers.sh.tmpl
    title: 'rtk-helpers.sh: rtk_available() and rtk_prefix() were un-cached, spawning cli-tool-discovery.sh on every git_cmd() call'
  - id: wrapper-helpers-cached-prefix
    resource: src/current/includes/wrapper-helpers.sh.tmpl
    title: 'wrapper-helpers.sh: wrapper_prefix() was already cached — the pattern that rtk-helpers should have followed'
---

# Wrapper Probe Caching — Cache Per-Command Subprocess Probes

## Failure Mode

Shell scripts that wrap commands through environment detection layers
(devbox, rtk, cli-tool-discovery.sh) call probe functions inside hot
loops — once per `git_cmd()`, `rtk_wrap()`, or `devbox_run()` invocation.
When those probe functions spawn subprocesses (e.g., `cli-tool-discovery.sh`
which probes devbox with a 15s timeout), the cost compounds:

- **Single session, no load**: 20 commands × 1s probe = 20s. Tolerable.
- **Single session, devbox broken**: 20 commands × 15s timeout = 300s.
  The startup `probe_devbox()` disables devbox, but if the per-command
  probe doesn't honor the disabled flag, it re-probes devbox every time.
- **10 parallel sessions, devbox broken**: 20 commands × 15s timeout ×
  nix store lock contention = 30+ minutes. Each session's probe
  serializes on the nix daemon, and the effective per-probe latency
  exceeds the 15s timeout.

### Symptoms

```bash
$ devbox run -- git-collect.sh
# Appears to hang — no output for 30+ minutes
# CPU not pinned (waiting on nix store lock / devbox probe timeout)
# No error message (the probe times out silently and retries)
```

The script was invoked through `devbox run --` correctly. The startup
`probe_devbox()` fired and completed. The hang is in the per-command
probe loop, not in the startup probe.

### Root Cause

Two missing optimizations in the wrapper helper chain:

1. **Un-cached probe results**: `rtk_available()` called
   `cli-tool-discovery.sh` on every invocation. Unlike
   `wrapper_prefix()` which cached its result in
   `WRAPPER_PREFIX_CACHE`, `rtk_available()` and `rtk_prefix()` had no
   cache. A script making 20 `git_cmd()` calls spawned 20
   `cli-tool-discovery.sh` subprocesses.

2. **Downstream probes ignoring known-broken flags**: When
   `probe_devbox()` set `WRAPPER_DEVBOX_DISABLED=1` (devbox hung or
   failed), `rtk_available()` still called `cli-tool-discovery.sh rtk`,
   which re-probed devbox from scratch — paying the 15s timeout again
   on every call. The disabled flag was honored by `wrapper_prefix()`
   but not by the rtk probe chain.

## Practice

### Cache Every Per-Command Probe

Any function called inside a hot loop that spawns a subprocess must
cache its result for the script lifetime. The probe result does not
change mid-script — devbox availability, rtk availability, and rtk
command coverage are stable for the duration of a single invocation.

**Pattern** (from `wrapper-helpers.sh`):

```bash
wrapper_prefix() {
    # Return cached result if available
    if [[ -n "${WRAPPER_PREFIX_CACHE+x}" ]]; then
        printf '%s' "$WRAPPER_PREFIX_CACHE"
        return
    fi
    # ... probe logic ...
    WRAPPER_PREFIX_CACHE="$result"
    printf '%s' "$result"
}
```

**Apply the same pattern to every probe in the chain**:

| Function | Cache variable | Cache granularity |
|----------|---------------|-------------------|
| `wrapper_prefix()` | `WRAPPER_PREFIX_CACHE` | Per-script (one probe) |
| `rtk_available()` | `RTK_AVAILABLE_CACHE` | Per-script (one probe) |
| `rtk_prefix()` | `RTK_PREFIX_CACHE__<tool>` | Per-tool (one `rtk rewrite` per tool) |

The `rtk_prefix()` cache is per-tool, not per-command, because rtk's
coverage for a given tool (e.g., `git`) is stable — if `rtk rewrite --
git status` returns "supported", all common `git` subcommands are
supported. The edge case (supports some subcommands but denies others)
over-wraps the denied subcommands, but rtk passes denied commands
through unchanged — the only cost is a no-op rtk spawn, not incorrect
behavior.

### Honor Known-Broken Flags in Downstream Probes

When an upstream probe declares a wrapper broken (e.g.,
`WRAPPER_DEVBOX_DISABLED=1`), every downstream probe that would re-test
the same wrapper must short-circuit and fall back to a cheaper
alternative.

```bash
rtk_available() {
    # ... cache check ...
    if [[ "${WRAPPER_DEVBOX_DISABLED:-0}" -eq 1 ]]; then
        # Devbox known broken — skip cli-tool-discovery.sh (it would
        # re-probe devbox, 15s timeout). Fall back to PATH check only.
        command -v rtk >/dev/null 2>&1 && result="rtk"
    else
        # ... full cli-tool-discovery probe ...
    fi
}
```

Without this, the startup probe's work is wasted — it disables devbox,
but the per-command probes ignore the flag and re-probe devbox anyway.

### Provide a Cache Refresh Override

Allow callers to force a re-probe for edge cases where the environment
changes mid-script (rare, but useful for long-running scripts):

```bash
if [[ -n "${RTK_PREFIX_CACHE__${safe_tool}+x}" && "${RTK_PREFIX_REFRESH:-0}" -eq 0 ]]; then
    printf '%s' "${!cache_var}"
    return
fi
```

Set `RTK_PREFIX_REFRESH=1` or `WRAPPER_PREFIX_REFRESH=1` to bypass the
cache on the next call.

## Detection

### Timing Test

```bash
# Create a fake cli-tool-discovery.sh that sleeps 5s
cat > /tmp/fake-cli-tool-discovery.sh <<'EOF'
#!/usr/bin/env bash
sleep 5
echo "FOUND:rtk"
EOF
chmod +x /tmp/fake-cli-tool-discovery.sh

# Run the script with the fake slow probe
time WRAPPER_DEVBOX_DISABLED=1 \
     CLI_TOOL_DISCOVERY=/tmp/fake-cli-tool-discovery.sh \
     bash git-collect.sh /path/to/repo

# Without caching: 20 calls × 5s = 100s
# With caching: <5s (probe called once or never)
```

### Marker File Test

For deterministic verification (no timing flakiness), have the fake
probe write a marker file and assert it was never created:

```bash
cat > /tmp/fake-cli-tool-discovery.sh <<EOF
#!/usr/bin/env bash
touch /tmp/probe-called.marker
echo "FOUND:rtk"
EOF

WRAPPER_DEVBOX_DISABLED=1 \
     CLI_TOOL_DISCOVERY=/tmp/fake-cli-tool-discovery.sh \
     bash git-collect.sh /path/to/repo

[[ ! -f /tmp/probe-called.marker ]] && echo "PASS: probe was not called" \
    || echo "FAIL: probe was called despite WRAPPER_DEVBOX_DISABLED=1"
```

## Prevention

1. **Audit every probe function for a cache** — if it spawns a
   subprocess and is called inside a loop, it must cache. Grep for
   `command -v`, `bash "$CLI_TOOL_DISCOVERY"`, and `$(...)` inside
   functions that are called per-command.
2. **Propagate known-broken flags through the entire probe chain** —
   when `probe_devbox()` sets `WRAPPER_DEVBOX_DISABLED=1`, every
   function that might re-probe devbox must check the flag first.
3. **Add timing regression tests** — the existing
   `wrapper_prefix cache prevents repeated devbox probes` test pattern
   should be replicated for every cached probe. Use the marker-file
   variant for deterministic CI.
4. **Profile under parallel load** — a script that completes in 5s
   single-session may take 30+ min under 10 parallel sessions if
   probes contend for the same lock (nix store, file lock, network).
   Test with `parallel -j10` or multiple terminal sessions.

## Related Concepts

- [Devbox Broken Override](devbox-broken-override.md) — when devbox
  cannot build the environment at all, override devbox-wrapped commands
  with direct equivalents. This concept (wrapper-probe-caching) is the
  performance fix that prevents the per-command probe from re-discovering
  the breakage on every call.
- [Shell Scripting Best Practices](shell-scripting-best-practices.md) —
  strict mode, PATH guards, and subprocess minimization. This concept
  extends the subprocess-minimization practice to probe functions
  specifically.
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — the
  three-flow pattern that assumes devbox works. This concept ensures
  the wrapper detection layer doesn't become a performance bottleneck
  when devbox is broken or under load.
