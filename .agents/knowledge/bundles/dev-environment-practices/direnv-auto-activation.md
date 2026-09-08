---
type: Practice
title: direnv Auto-Activation
description: Automatic environment loading on cd using direnv with devbox; watch_file for config changes, use_devbox pattern, and .envrc configuration.
tags: [direnv, devbox, auto-activation, environment, developer-experience]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: adr-20251226001-devbox-direnv-dev-environment
    resource: internal-docs/adr/adr-20251226001-devbox-direnv-dev-environment.md
    title: levonk-base-boilerplate
  - id: adr-20260131001-standard-developer-ux-flow
    resource: internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md
    title: levonk-base-boilerplate
---

# direnv Auto-Activation

## Failure Mode

Manual environment sourcing leads to forgotten activation, stale shells, and
inconsistent state. Developers run `nix develop` or `source .envrc` manually,
forget to do it in new terminals, and end up with missing tools or wrong
versions.

## Practice

Use **direnv** to automatically activate the devbox environment on `cd` into the
project directory. direnv watches configuration files and reloads the
environment when they change.

### Configuration

```bash
# .envrc
use_devbox() {
    watch_file devbox.json
    eval "$(devbox shellenv)"
}

use devbox
```

### How It Works

1. Developer enters project directory (`cd project`)
2. direnv detects `.envrc` and evaluates it
3. `use_devbox` calls `devbox shellenv` to get environment variables
4. `watch_file devbox.json` ensures direnv reloads when devbox config changes
5. Tools from `devbox.json` are now on PATH

### Baseline Makefile Targets (Historical)

The original Nix flake ADR defined baseline Makefile targets that map to the
modern just equivalents:

- **bootstrap**: install Nix (if missing), install devbox, install direnv, run
  `devbox install`
- **clean**: `rm -rf .devbox` and clean up caches
- **build**: `devbox run build`
- **lint**: `devbox run lint`
- **doctor**: verify `devbox`, `nix`, and `direnv` installation

## Related Concepts

- [Devbox Over Raw Nix](devbox-over-raw-nix.md) — The environment direnv activates
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — The workflow that
  relies on auto-activation
