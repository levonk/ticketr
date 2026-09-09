---
type: Practice
title: Nix Flake Dev Shells
description: Per-project reproducible tooling via flake.nix devShells; the original approach before migrating to devbox. Documents the Nix foundation that devbox builds on.
tags: [nix, flake, dev-shell, reproducible, direnv, tooling]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: levonk-base-boilerplate
    resource: internal-docs/adr/adr-20251219001-nix-direnv-dev-environment.md
    title: levonk-base-boilerplate
---

# Nix Flake Dev Shells

## Failure Mode

Missing developer tools create friction and lead to inconsistent local setups.
Without per-project tool management, developers install tools globally, versions
diverge across machines, and "works on my machine" problems proliferate.

## Practice

Use `flake.nix` to define a reproducible dev shell that provides required tools.
When a tool is missing, the fix is to add it to `flake.nix` — not to install it
globally.

### Configuration

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            universal-ctags
            ripgrep
            fd
          ];
        };
      });
}
```

### direnv Integration

```bash
# .envrc
use_flake() {
  watch_file flake.nix flake.lock
  eval "$(nix print-dev-env --impure)"
}

use flake
```

Contributors run `direnv allow` once per clone. The environment auto-activates on
every `cd` into the project.

## Why It Was Replaced

Nix flakes provide reproducibility but the learning curve for writing
`flake.nix` files is steep, and the developer experience for managing packages
is verbose. This practice was superseded by
[Devbox Over Raw Nix](devbox-over-raw-nix.md), which maintains the Nix foundation
while simplifying configuration to JSON.

## Related Concepts

- [Devbox Over Raw Nix](devbox-over-raw-nix.md) — The successor practice
- [direnv Auto-Activation](direnv-auto-activation.md) — Shared activation layer
