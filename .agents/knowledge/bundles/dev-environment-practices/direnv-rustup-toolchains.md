---
type: Practice
title: direnv rustup Toolchains
description: direnv integration with rustup — RUSTUP_HOME and CARGO_HOME env vars in .envrc, automatic toolchain switching on directory change, PATH management for cargo binaries, and fast loading for Rust workspaces.
tags: [dev-environment, rust, direnv, rustup, toolchain, developer-experience]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"
sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# direnv rustup Toolchains

## Failure Mode

Without direnv, switching Rust projects means manually running
`rustup override set` or remembering which toolchain a project needs. Cargo
binaries installed via `cargo install` (e.g. `cargo-nextest`) land in a global
`~/.cargo/bin` that is shared across toolchains, so a binary built against
nightly leaks into a stable project's PATH. New terminals forget the override
entirely, and contributors debug "wrong rustc" errors that vanish on rerun.

## Practice

Let direnv own the rustup environment variables and PATH augmentation. On
`cd` into a project, `.envrc` exports `RUSTUP_HOME` and `CARGO_HOME` scoped to
the project, loads the devbox environment, and prepends the project's cargo bin
directory. `rust-toolchain.toml` handles the actual toolchain selection —
direnv only sets the sandbox boundaries.

### .envrc for a Rust Project

```bash
# .envrc
use_devbox() {
    watch_file devbox.json devbox.lock rust-toolchain.toml
    eval "$(devbox shellenv)"
}

use devbox

# Scope rustup state to this project (optional, for isolation)
export RUSTUP_HOME="${PWD}/.rustup"
export CARGO_HOME="${PWD}/.cargo"

# Project-local cargo binaries on PATH
PATH_add "${CARGO_HOME}/bin"
```

`watch_file rust-toolchain.toml` ensures direnv reloads when the pinned channel
changes — without it, switching from stable to nightly requires a manual
`direnv reload`.

### Automatic Toolchain Switching

`rust-toolchain.toml` does the switching; direnv does the scoping. When a
contributor `cd`s into the project, rustup sees the pinned channel on the next
`cargo` invocation and installs/activates it automatically. No `rustup override`
commands are needed — the file is the override.

For projects that cannot use `rust-toolchain.toml` (e.g. a monorepo with
per-crate toolchains), fall back to an explicit override in `.envrc`:

```bash
# .envrc (fallback when rust-toolchain.toml is impractical)
rustup override set nightly-2026-07-15 >/dev/null 2>&1
```

### PATH Management for Cargo Binaries

`cargo install` writes to `CARGO_HOME/bin`. With `CARGO_HOME` scoped per
project, installed binaries stay project-local — a `cargo-nextest` built under
nightly never appears on a stable project's PATH. The `PATH_add` line above
makes those binaries discoverable within the project directory only.

For shared, toolchain-agnostic binaries (`cargo-watch`, `cargo-edit`), keep a
global `~/.cargo/bin` and add it after the project-local dir so project
binaries take precedence:

```bash
PATH_add "${CARGO_HOME}/bin"
PATH_add "${HOME}/.cargo/bin"
```

### Loading Speed

Rust projects can slow direnv loading if `.envrc` runs `cargo metadata` or
`rustup show` synchronously. Keep `.envrc` to env exports and `watch_file`
calls — defer any cargo/rustup work to the `devbox` init_hook or to `just prime`
where it runs asynchronously. A clean `.envrc` loads in under 200ms; one that
shells out to cargo can take seconds.

Anti-pattern to avoid:

```bash
# .envrc — SLOW, do not do this
export TOOLCHAIN=$(rustup show active-toolchain)   # blocks on rustup
export CRATE_NAMES=$(cargo metadata --format-version 1 | jq -r ...)  # blocks on cargo
```

### Workspace Projects

For a cargo workspace, one `.envrc` at the workspace root covers all member
crates. `watch_file` the workspace `Cargo.toml` in addition to
`rust-toolchain.toml` so member additions trigger a reload:

```bash
watch_file devbox.json devbox.lock rust-toolchain.toml Cargo.toml
```

## Related Concepts

- [direnv Auto-Activation](direnv-auto-activation.md) — The base `use_devbox` pattern this builds on
- [Devbox Rust Versions](devbox-rust-versions.md) — The `rust-toolchain.toml` pinning direnv watches
