---
type: Practice
title: Error Handling
description: Use thiserror for structured error types, anyhow for context, implement From traits, and never panic in library code except for unrecoverable logic errors.
tags: [rust, error-handling, thiserror, anyhow, panic, result]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md"
    title: "levonk-base-boilerplate"
---

# Error Handling

## Failure Mode

Panics in library code crash applications. Unstructured errors (String, Box<dyn
Error>) prevent callers from matching on error variants. Missing From
implementations force callers to manually convert errors.

## Practice

### Structured Error Types with thiserror

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, PackageError>;
```

### Patterns

- Use `thiserror` for structured error types
- Implement `From` traits for error conversion (via `#[from]`)
- Provide context with `anyhow`'s `context()` method when appropriate
- **Never** use `panic!` in library code except for unrecoverable logic errors

## Related Concepts

- [Project Structure](project-structure.md) — error.rs module placement
- [Cargo Configuration](cargo-configuration.md) — thiserror as workspace dependency
