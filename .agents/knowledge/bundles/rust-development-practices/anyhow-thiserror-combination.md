---
type: Practice
title: Anyhow + Thiserror Combination
description: Use anyhow at the application layer and main, thiserror for library-layer structured errors, convert thiserror to anyhow with context(), chain via #[from], and downcast anyhow errors to thiserror types in tests.
tags: [rust, anyhow, thiserror, error-handling, context, downcast]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Anyhow + Thiserror Combination

## Failure Mode

Using `anyhow` everywhere erases error variants; callers cannot match on cause.
Using `thiserror` everywhere forces library users to handle every variant even
when they only care about success. Missing `context()` produces errors like "io
error" with no file path. Tests that assert on error variants cannot reach them
through an `anyhow::Error` barrier.

## Practice

### Layered Responsibility

Use `thiserror` in library crates. Define an enum per concern with `#[from]`
for automatic conversion. Callers match variants and recover.

Use `anyhow` in binary crates and at `main`. It bundles context without
forcing callers to enumerate every failure mode.

```rust
// library crate: lint-core
#[derive(thiserror::Error, Debug)]
pub enum LintError {
    #[error("config parse failed at {path}:{line}: {source}")]
    Config { path: camino::Utf8PathBuf, line: usize, #[source] source: toml::de::Error },
    #[error("rule {rule_id} failed: {0}")]
    Rule { rule_id: String, #[from] source: RuleError },
}
pub type Result<T> = std::result::Result<T, LintError>;
```

### anyhow::Result for main

Return `anyhow::Result` from `main`. The runtime prints the error chain and
exits non-zero. Add `context()` at each boundary so the chain reads top-down.

```rust
fn main() -> anyhow::Result<()> {
    let cfg = load_config().context("loading config")?;
    run(&cfg).context("running lint")?;
    Ok(())
}
```

### Converting thiserror to anyhow with context()

`anyhow::Context` is implemented for any `Result<T, E: std::error::Error>`.
Calling `.context()` on a `LintError` result wraps it without losing the
variant — the original error remains reachable via the source chain.

```rust
use anyhow::Context;
let cfg: Config = load_config().context("project-lint config")?;
```

### The #[from] Chain

In `thiserror` enums, `#[from]` generates `From` so `?` converts automatically.
Chain one library error into another to keep the cause tree intact. Avoid
`Box<dyn Error>` — it breaks the chain and prevents downcasting.

### Downcasting for Tests

Tests need to assert on specific variants. `anyhow::Error::downcast_ref`
recovers the inner `thiserror` type.

```rust
#[test]
fn bad_config_reports_line() {
    let err = run("bad.toml").unwrap_err();
    let lint = err.downcast_ref::<LintError>().expect("not LintError");
    match lint {
        LintError::Config { line, .. } => assert_eq!(*line, 3),
        other => panic!("unexpected variant: {other:?}"),
    }
}
```

When the boundary is pure application code, prefer asserting on the rendered
chain string instead — it is stable across refactorings.

## Related Concepts

- [Error Handling](error-handling.md) — base thiserror patterns
- [Toml Config Validation](toml-config-validation.md) — config errors with line info
- [Testing Strategy](testing-strategy.md) — asserting on error variants
