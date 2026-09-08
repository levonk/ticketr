---
type: Practice
title: Testing Strategy
description: Multi-level Rust testing — inline unit tests, integration tests in tests/, doc tests in comments, benchmarks with criterion, and property-based testing with proptest.
tags: [rust, testing, unit-tests, integration-tests, doc-tests, benchmarks, proptest]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md"
    title: "levonk-base-boilerplate"
---

# Testing Strategy

## Failure Mode

Rust packages with only unit tests miss integration failures. Missing doc tests
let examples go stale. No benchmarks mean performance regressions go unnoticed.

## Practice

### Test Organization

- **Unit tests**: Inline in source files using `#[cfg(test)]`
- **Integration tests**: In `tests/` directory for black-box testing
- **Documentation tests**: In doc comments using `///` with code examples
- **Benchmarks**: In `benches/` directory for performance testing

### Required Test Dependencies

```toml
[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"
serial_test = "2.0"
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.4"
```

### Async Test Helper

```rust
#[macro_export]
macro_rules! async_test {
    ($test_name:ident, $test_body:block) => {
        #[tokio::test]
        async fn $test_name() {
            let _ = setup_test_logger();
            $test_body
        }
    };
}
```

### Test Utilities

```rust
// tests/common/mod.rs
pub fn setup_test_logger() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}
```

## Related Concepts

- [Quality Gates](quality-gates.md) — CI runs all test types
- [Async Patterns](async-patterns.md) — Async test macro pattern
