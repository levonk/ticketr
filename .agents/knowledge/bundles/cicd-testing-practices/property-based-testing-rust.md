---
type: Practice
title: Property-Based Testing in Rust
description: proptest crate for property-based tests. Compose strategies with prop_compose, let proptest shrink failing cases automatically, and target invariants and round-trip properties where example-based tests fall short.
tags: [ci-cd, testing, rust, proptest, property-based]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Property-Based Testing in Rust

## Failure Mode

Example-based tests cover the cases the author imagined. Edge cases — empty
input, max values, unicode boundaries, nested config — slip through. Bugs
hide in the gaps between examples. Writing enough examples to cover the input
space by hand is impractical.

## Practice

Use the **proptest** crate. Define properties over input strategies; let
proptest generate cases, find failures, and shrink them to minimal
reproductions.

### Strategy Composition with prop_compose

Build strategies from primitives, then compose them.

```rust
use proptest::prelude::*;

prop_compose! {
    fn arb_config()
        (port in 1u16..65535,
         host in "[a-z]{1,10}\\.example\\.com",
         retries in 0u32..10)
        -> Config {
        Config { host, port, retries }
    }
}

proptest! {
    #[test]
    fn config_serializes_roundtrip(c in arb_config()) {
        let json = serde_json::to_string(&c).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(c, back);
    }
}
```

### Shrinking

When a case fails, proptest minimizes it. A panic on a 4 KB string becomes a
failure on a 1-character string. The shrunk case is written to
`proptest-regressions/` so future runs replay it first.

Commit `proptest-regressions/` files. They lock in the discovered edge cases
as permanent regression tests.

### Testing CLI Argument Parsing

Generate arbitrary argument vectors and assert the parser either succeeds
with a valid config or returns a typed error — never panics.

```rust
proptest! {
    #[test]
    fn parser_never_panics(args in prop::collection::vec(".+", 0..10)) {
        let result = parse_args(&args);
        prop_assert!(result.is_ok() || result.is_err());
    }
}
```

### Testing Config Validation

Feed invalid configs through validation and assert the error matches the
violation, not an arbitrary panic.

```rust
proptest! {
    #[test]
    fn invalid_port_rejected(port in 65536u32..=100_000) {
        let cfg = Config { port: port as u16, ..Default::default() };
        prop_assert!(cfg.validate().is_err());
    }
}
```

### When to Use Property Tests vs Example-Based Tests

- **Invariants and round-trips**: serialization round-trips, parser totality,
  monotonic counters, commutative operations.
- **Specific known bugs**: use example tests with fixed expected output. They
  stay readable and document intent.
- Do not replace all example tests with properties. Add property tests
  alongside them for the input space examples cannot cover.

### CI Integration

Run proptest under cargo-nextest. Treat new regressions files as a CI
failure that requires review:

```bash
cargo nextest run --workspace
git diff --exit-code proptest-regressions/
```

## Related Concepts

- [Rust CI Tooling](rust-ci-tooling.md) — Run proptest under cargo-nextest
- [Testing Strategy](https://github.com/levonk/skills-releases/blob/main/knowledge/rust-development-practices/testing-strategy.md) — Where property tests fit in the overall Rust test pyramid
