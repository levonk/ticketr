---
type: Practice
title: TOML Config Validation
description: Validate TOML with serde schemas, merge user/project/defaults with explicit precedence, version configs with a schema_version field, report line/column from toml::de::Error, and hot-reload via file watcher.
tags: [rust, toml, serde, config, validation, hot-reload]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# TOML Config Validation

## Failure Mode

Untyped config tables drift into runtime panics. Merged configs silently shadow
defaults in surprising orders. Schema changes break old configs with no signal.
Parse errors without line numbers force users to grep for the typo. Cold restarts
to pick up config edits kill long-running linters.

## Practice

### Serde Schema Validation

Define a typed config struct with serde derives. Mark optional fields with
`Option<T>` and `#[serde(default)]`. Reject unknown keys with
`#[serde(deny_unknown_fields)]` so typos surface immediately.

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LintConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub rules: RuleSet,
    #[serde(default)]
    pub exclude: Vec<std::path::PathBuf>,
}
```

### Profile Merging

Layer configs with explicit precedence: defaults < user (XDG) < project <
CLI args. Merge in that order so the most specific source wins.

```rust
fn merged() -> LintConfig {
    let defaults = LintConfig::defaults();
    let user = load_user().unwrap_or_else(|_| defaults.clone());
    let project = load_project().unwrap_or_else(|_| user.clone());
    project.overlaid_with(user).overlaid_with(defaults)
}
```

Implement `overlaid_with` field by field. For collections, prefer "replace"
semantics over "extend" unless the field is explicitly additive.

### Schema Versioning

Carry a `schema_version` field. On load, branch on the version: migrate forward
or reject with a clear message. Never silently interpret an older schema with a
newer struct.

```rust
match raw.schema_version {
    1 => migrate_v1_to_v2(raw),
    2 => Ok(raw),
    v => Err(ConfigError::UnknownSchema(v)),
}
```

### Error Reporting with Line/Column

`toml::de::Error` carries span info. Surface it so users can jump to the
offending line.

```rust
let cfg: LintConfig = match toml::from_str(&text) {
    Ok(c) => c,
    Err(e) => {
        let span = e.span().unwrap_or(0..0);
        return Err(anyhow::anyhow!(
            "config parse error at line {}, col {}: {}",
            line_of(&text, span.start),
            col_of(&text, span.start),
            e
        ));
    }
};
```

### Hot-Reload via File Watcher

Pair the config loader with a notify watcher. On write events, debounce, reload,
and swap the active config behind an `Arc<ArcSwap<LintConfig>>`. Emit a
structured log line so users see the reload.

## Related Concepts

- [Clap CLI Patterns](clap-cli-patterns.md) — CLI args override config
- [File Watcher Patterns](file-watcher-patterns.md) — debounced reload events
- [Serde Serialization](serde-serialization.md) — derive patterns for config types
- [Config-Driven Tool Design](https://github.com/levonk/skills-releases/blob/main/software-architecture-essentials/config-driven-tool-design.md) — architectural rationale
