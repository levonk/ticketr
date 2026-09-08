---
type: Practice
title: Multi-Language Devbox
description: Managing Python, Rust, and Node in one devbox — multiple language packages, avoiding version conflicts, tree-sitter grammars spanning Rust and Node, PATH ordering, and disk space considerations.
tags: [dev-environment, devbox, rust, python, node, multi-language, reproducibility]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"
sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Multi-Language Devbox

## Failure Mode

Polyglot projects — a Rust core with Python bindings and a Node-based build
tool — end up with three separate environment managers fighting over PATH. Nix
provides the system tools, but each language's native package manager
(`cargo`, `uv`, `pnpm`) owns its own dependency graph. When devbox tries to
provide language libraries too, versions conflict with the native manager's
lockfile. Tree-sitter grammars are the canonical trap: they require Rust to
compile the parser and Node to generate the grammar source, so both toolchains
must coexist at build time.

## Practice

Use devbox for **system tools and language runtimes only**. Each language's
native package manager owns its libraries. devbox provides `rustup`, `python`,
`nodejs`, and `pnpm`; `cargo`, `uv`/`pip`, and `pnpm` provide the rest. Never
declare a language library (e.g. `nixpkgs#python312Packages.requests`) in
devbox — it bypasses the project lockfile and drifts.

### devbox.json — Rust + Node + Python

```json
{
  "packages": [
    "rustup",
    "nodejs_22",
    "pnpm",
    "python312",
    "uv",
    "just",
    "direnv",
    "tree-sitter"
  ],
  "shell": {
    "init_hook": [
      "rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install",
      "pnpm install --frozen-lockfile >/dev/null 2>&1 || true",
      "uv sync --frozen >/dev/null 2>&1 || true"
    ]
  }
}
```

The `init_hook` warms each native package manager's cache but never installs
global libraries — `--frozen-lockfile` and `--frozen` enforce lockfile fidelity.

### Avoiding Version Conflicts

| Layer | Owner | Example |
|-------|-------|---------|
| System tools (just, direnv, tree-sitter CLI) | devbox / nixpkgs | `just`, `direnv` |
| Language runtime (rustc, node, python) | devbox + rust-toolchain.toml | `rustup`, `nodejs_22` |
| Language libraries (crates, npm, pip) | Native package manager | `cargo`, `pnpm`, `uv` |

The rule: if it has a lockfile in the repo, the native manager owns it. devbox
provides the runtime that runs the manager, not the libraries the manager
installs.

### Tree-sitter Grammars (Rust + Node)

Tree-sitter grammars are generated from a Node.js `grammar.js` and compiled into
a Rust crate, so the build requires both toolchains simultaneously:

```bash
pnpm exec tree-sitter generate   # Node: generate grammar source
cargo build -p my-tree-sitter-grammar  # Rust: compile the parser crate
```

devbox provides both `nodejs_22` and `rustup`; the `tree-sitter` CLI from
nixpkgs bridges the two. No cross-manager coordination is needed beyond having
both runtimes on PATH.

### PATH Ordering

When multiple language bin dirs coexist, order matters — project-local
overrides win over devbox-provided runtimes, which win over global installs:

```bash
# .envrc
use devbox
PATH_add "${PWD}/.cargo/bin"        # rustup/cargo project binaries
PATH_add "${PWD}/node_modules/.bin" # pnpm project binaries
PATH_add "${PWD}/.venv/bin"         # uv/venv project binaries
```

devbox's own PATH entries (from `devbox shellenv`) are already present via
`use devbox`; the `PATH_add` lines above prepend project-local dirs so
`cargo-nextest` or a local `tree-sitter` binary shadows the devbox-provided one.

### Disk Space Considerations

Three language toolchains plus their package caches consume real space:
`rustup` toolchains (~1-2 GB per channel), `node_modules` (200 MB - 2 GB),
`.venv` + `uv` cache (100 MB - 1 GB), and nixpkgs closures for the runtimes
(~500 MB). Scope `CARGO_HOME` and the Python venv inside the project directory
(or a shared cache dir) so `devbox shell` exit does not orphan multi-GB caches
in `$HOME`. For CI, use `devbox install --no-init-hook` and let the pipeline
warm caches explicitly.

## Related Concepts

- [Devbox Over Raw Nix](devbox-over-raw-nix.md) — Why devbox manages runtimes, not libraries
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — How `just` recipes route to each language's native tool
