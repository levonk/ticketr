---
type: Practice
title: Devbox Rust Versions
description: Pinning the Rust toolchain in devbox — rust-toolchain.toml vs devbox rust-package, rustup integration, component selection, and reproducible channel+date locking.
tags: [dev-environment, rust, devbox, rustup, reproducibility, toolchain]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"
sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Devbox Rust Versions

## Failure Mode

Rust toolchain drift is silent. One contributor builds on `stable` 1.82, another
on `nightly-2026-07-01`, and CI on whatever `rustup default` resolved to at
image-build time. Clippy lints pass locally and fail in CI; `cargo` flags
introduced in a newer release are rejected on older toolchains; and a devbox
that only declares `rust` (unpinned) floats to whatever nixpkgs shipped at
`devbox gen` time — reproducible across machines but not across months.

## Practice

Pin the Rust toolchain at two layers: devbox provides the `rustup` binary and
the default channel, while `rust-toolchain.toml` locks the exact channel, date,
and components per project. The two files cooperate — devbox ensures rustup is
present; `rust-toolchain.toml` ensures rustup installs the right toolchain on
first `cargo` invocation.

### devbox.json — Toolchain Manager

```json
{
  "packages": [
    "rustup",
    "just",
    "direnv"
  ],
  "shell": {
    "init_hook": [
      "rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install"
    ]
  }
}
```

Declare `rustup` (not `rust` or `cargo`) so the project controls the channel via
`rust-toolchain.toml` rather than inheriting whatever nixpkgs pinned. This
decouples the toolchain manager version from the compiler version.

### rust-toolchain.toml — Channel + Components

```toml
[toolchain]
channel = "1.82.0"
components = ["rustfmt", "clippy", "rust-analyzer"]
targets = ["aarch64-unknown-linux-gnu"]
profile = "minimal"
```

For projects requiring nightly, pin the dated channel — never bare `nightly`,
which floats and breaks reproducibility within a day:

```toml
[toolchain]
channel = "nightly-2026-07-15"
components = ["rustfmt", "clippy", "rust-analyzer", "miri"]
```

### rustup Integration

`rustup` reads `rust-toolchain.toml` automatically on any `cargo`/`rustc`
invocation in the project directory. If the pinned toolchain is missing, rustup
downloads it transparently. The devbox `init_hook` above pre-installs it so the
first `just build` does not pay the download cost.

```bash
# Manual install (mirrors what the init_hook automates)
rustup toolchain install 1.82.0 --component rustfmt,clippy,rust-analyzer
rustup override set 1.82.0
```

### Multi-Version Management

Different projects pin different channels. `rustup` holds them side by side in
`RUSTUP_HOME/toolchains/`; switching directories switches the active toolchain
via each project's `rust-toolchain.toml`. Never run `rustup default nightly`
globally — it shadows the per-project pin and causes the silent drift this
practice prevents.

### Component Selection

Use `profile = "minimal"` and list components explicitly. The default profile
installs `rust-docs` (hundreds of MB) that CI and most contributors do not need.
Explicit components keep `rustup toolchain install` fast and disk usage
predictable.

### Reproducibility Checklist

1. devbox.lock pins the `rustup` package version
2. `rust-toolchain.toml` pins channel + date (for nightly) or exact semver (for
   stable)
3. `components` and `targets` are explicit — no implicit defaults
4. No `rustup default` commands in shell rc files or devbox init_hook

## Related Concepts

- [Devbox Over Raw Nix](devbox-over-raw-nix.md) — Why devbox provides the toolchain manager
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — How the pinned toolchain flows through `just` recipes
