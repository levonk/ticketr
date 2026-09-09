---
type: Practice
title: Serde Serialization
description: Optional serde integration with feature flags, multiple format support (JSON, TOML), skip_serializing_if for optional fields, and custom serialization for complex types.
tags: [rust, serde, serialization, json, toml, feature-flags]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md"
    title: "levonk-base-boilerplate"
---

# Serde Serialization

## Failure Mode

Hardcoding serde as a mandatory dependency forces all consumers to pay the
compile-time cost. Missing `skip_serializing_if` bloats output with null fields.
Single-format support limits interoperability.

## Practice

### Optional Serde Integration

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
toml = { version = "0.8", optional = true }

[features]
default = []
serde = ["dep:serde", "serde/derive"]
```

### Patterns

- Derive `Serialize` and `Deserialize` for public types (behind feature flag)
- Use `serde(skip_serializing_if = "Option::is_none")` for optional fields
- Provide custom serialization for complex types
- Support multiple formats (JSON, TOML) when applicable

## Related Concepts

- [Cargo Configuration](cargo-configuration.md) — Feature flag pattern
- [Error Handling](error-handling.md) — Parse errors in deserialization
