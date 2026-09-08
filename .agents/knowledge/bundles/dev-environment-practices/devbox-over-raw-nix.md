---
type: Practice
title: Devbox Over Raw Nix
description: Replace flake.nix with devbox.json for simpler configuration, familiar CLI workflow, and lower barrier to entry while maintaining Nix reproducibility.
tags: [devbox, nix, developer-experience, configuration, migration]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: levonk-base-boilerplate
    resource: internal-docs/adr/adr-20251226001-devbox-direnv-dev-environment.md
    title: levonk-base-boilerplate
---

# Devbox Over Raw Nix

## Failure Mode

Raw Nix flakes provide reproducibility but the learning curve is steep. Writing
`flake.nix` files requires understanding the Nix language, and the developer
experience for managing packages is often verbose — a 50-line flake for a simple
Node.js project is common.

## Practice

Use **Devbox** (by Jetify) to define the development environment via
`devbox.json`. Devbox uses Nix under the hood, preserving reproducibility while
offering a simpler JSON-based configuration and familiar CLI workflow
(`devbox add <package>`).

### Configuration

```json
{
  "packages": [
    "just",
    "nodejs",
    "pnpm"
  ],
  "shell": {
    "init_hook": [
      "just bootstrap_impl"
    ]
  },
  "scripts": {
    "bootstrap": "just bootstrap_impl",
    "build": "just build_impl",
    "test": "just test_impl",
    "lint": "just lint_impl",
    "dev": "just dev_impl"
  },
  "nix_pkgs": [
    "just"
  ]
}
```

### Why Devbox Wins

1. **Simpler Configuration**: `devbox.json` is easier to read and modify than
   `flake.nix`
2. **Lower Barrier to Entry**: `devbox add <package>` without learning Nix
3. **Performance**: Optimized shell activation and caching
4. **Continuity**: Still uses Nix packages — the breadth of nixpkgs remains
5. **Lock File**: `devbox.lock` ensures reproducible environments across team

## Migration Path

From raw Nix flakes to devbox:

1. Initialize `devbox.json` in the repository root
2. Configure `.envrc` to use `eval "$(devbox generate direnv --print-envrc)"`
3. Update documentation to reflect `devbox` usage instead of raw `nix develop`
4. Map existing flake packages to devbox packages (most names are identical)

## Related Concepts

- [Nix Flake Dev Shells](nix-flake-dev-shells.md) — The predecessor practice
- [direnv Auto-Activation](direnv-auto-activation.md) — Activation layer
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — Workflow built on devbox
