---
type: Practice
title: Worktree Port Allocation
description: Deterministic port assignment for parallel development worktrees so multiple instances of a dev server can run simultaneously without port conflicts or manual coordination.
tags: [worktree, ports, parallel-development, deterministic, dev-server, isolation]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Worktree Port Allocation

## Failure Mode

A developer opens three worktrees to work on three features in parallel.
Each worktree's dev server tries to bind to the same default port (e.g.,
3000, 5173, 8080). The first one succeeds; the rest fail with
`EADDRINUSE`. The developer must manually set a different `PORT` env var
in each terminal, remember which port goes with which worktree, and
update the frontend proxy config to match. This is tedious, error-prone,
and breaks the "just run `dev`" promise.

Worse: an AI agent spinning up a worktree for an automated workflow has
no idea which ports are in use. It either picks a random port (collision
risk) or hardcodes one (no parallelism).

## Practice

Assign ports **deterministically** based on the worktree's filesystem
path. The same worktree always gets the same port — across restarts,
across machines, across agents — without any manual configuration or
coordination protocol.

### Core Principles

1. **Deterministic, not random**: The port is a pure function of the
   worktree path. No state, no locks, no "next available port" scans.
2. **Bounded range**: Map into a reserved range (e.g., 3190–4089) that
   avoids well-known ports and common dev-server defaults.
3. **Collision tolerance**: The range should be large enough that two
   worktrees with different paths almost never hash to the same port.
   When they do, the server's normal "port in use" error surfaces it —
   the developer overrides with an explicit `PORT` env var.
4. **Explicit override wins**: If `PORT` is set in the environment, use
   it. Deterministic allocation is the fallback, not a mandate.
5. **Main checkout uses the default**: The primary working copy (not a
   worktree) uses the project's default port. Only worktrees get
   allocated ports.

### Algorithm

```
if PORT env var is set:
    use PORT (explicit override)
elif CWD is inside a git worktree:
    hash = hash_function(worktree_path)
    offset = hash % range_size + min_offset
    port = base_port + offset
else:
    use default_port
```

The hash function should be fast, uniform, and deterministic. MD5 or
SHA-256 truncated to 16 bits is sufficient — the goal is uniform
distribution across a 900-port range, not cryptographic security.

## Concrete Instances

### Hash-based allocation (TypeScript / Bun)

```typescript
import { createHash } from 'crypto';

function calculatePortOffset(path: string): number {
  const hash = createHash('md5').update(path).digest();
  // 100–999 range → ports 3190–4089 when added to base 3090
  return (hash.readUInt16BE(0) % 900) + 100;
}

async function getPort(): Promise<number> {
  if (process.env.PORT) return Number(process.env.PORT);
  const basePort = 3090;
  const cwd = process.cwd();
  if (await isWorktreePath(cwd)) {
    return basePort + calculatePortOffset(cwd);
  }
  return basePort;
}
```

The server calls `getPort()` at startup. The same worktree path always
produces the same port (e.g., 3742), so the developer can bookmark
`http://localhost:3742` and it works across restarts. The main checkout
uses 3090. The 900-port range (3190–4089) gives a collision probability
of ~0.1% per pair of worktrees — acceptable for single-developer
parallel work.

### Vite auto-increment (frontend dev server)

```javascript
// vite.config.ts
export default defineConfig({
  server: {
    port: 5173,
    strictPort: false, // if 5173 is taken, try 5174, 5175, ...
  },
});
```

Vite's built-in `strictPort: false` tries the configured port, then
increments until it finds a free one. This is simpler but
**non-deterministic**: the same worktree may get a different port each
run depending on what else is running. Suitable for interactive
development where the developer sees the printed URL; less suitable for
automated agents that need to know the port ahead of time.

### Just-based allocation (multi-language)

```just
_dev-port:
    #!/usr/bin/env bash
    if [ -n "$PORT" ]; then
        echo "$PORT"
    elif git rev-parse --is-inside-work-tree >/dev/null 2>&1 && \
         [ "$(git rev-parse --git-common-dir)" != "$(git rev-parse --git-dir)" ]; then
        # We're in a worktree — hash the path
        PATH_HASH=$(echo "$(pwd)" | md5sum | cut -c1-4)
        OFFSET=$(( (0x$PATH_HASH % 900) + 100 ))
        echo $(( 3090 + OFFSET ))
    else
        echo "3090"
    fi

dev:
    PORT=$(just _dev-port) just dev_impl
```

This shell-based approach works for any language: the port is computed
from the worktree path before the dev server starts, then passed as an
environment variable. No language-specific port-allocation code needed.

## Prevention

1. **Detect worktrees reliably** — check whether the current directory is
   a linked worktree (not just a git repository). In git, this means
   checking whether `.git` is a file (worktree) vs a directory (main
   checkout).
2. **Log the allocated port** — at startup, log the worktree path, the
   computed port, and the base port. When a developer reports "my server
   isn't on 3000," the log shows exactly where it is.
3. **Document the port range** — put it in the project's docs so
   firewall rules, proxy configs, and CI runners can account for it.
4. **Reserve the range in CI** — if CI runs worktree-based integration
   tests, ensure the port range is not already bound by other services
   on the runner.

## Related Concepts

- [Git Worktree Isolation](git-worktree-isolation.md) — Worktrees enable
  parallel development; port allocation makes parallel dev servers feasible
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — The
  `just dev` flow that invokes the dev server; port allocation is
  transparent to the developer
