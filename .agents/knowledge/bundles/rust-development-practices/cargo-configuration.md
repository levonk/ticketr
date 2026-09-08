---
type: Practice
title: Cargo Configuration
description: Required Cargo.toml metadata, dependency version pinning, feature flags for optional functionality, and workspace configuration for monorepo integration.
tags: [rust, cargo, cargo-toml, dependencies, features, workspace]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md"
    title: "levonk-base-boilerplate"
---

# Cargo Configuration

## Failure Mode

Missing Cargo.toml metadata causes poor crate discoverability. Unpinned
dependencies lead to reproducibility issues. Missing feature flags force all
functionality on all consumers.

## Practice

### Required Metadata

```toml
[package]
name = "package-slug"
version = "0.1.0"
edition = "2021"
authors = ["Author <email>"]
description = "Package description"
license = "MIT"
repository = "https://github.com/..."
rust-version = "1.70"
```

### Dependency Management

- **Version pinning**: All dependencies must specify exact versions or compatible
  version requirements
- **Feature flags**: Use Cargo features for optional functionality
- **Minimal dependencies**: Prefer standard library over external crates
- **Security**: Regularly audit dependencies with `cargo audit`

### Feature Flags

```toml
[features]
default = []
async = ["tokio"]
serde = ["dep:serde", "serde/derive"]
```

### Workspace Configuration

```toml
[workspace]
members = [
    "packages/active/*/rust/*",
    "apps/active/*/rust/*"
]

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
```

## Related Concepts

- [Project Structure](project-structure.md) — Directory layout for the package
- [Security and Auditing](security-auditing.md) — cargo audit for dependencies
