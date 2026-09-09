---
type: Practice
title: Config-Driven Tool Design
description: Layered config precedence with profile-based activation, versioned schema migration, atomic hot-reload, and cross-platform config discovery — synthesized across the architecture and Rust bundles.
tags: [architecture, rust, configuration, config, toml, hot-reload, cross-platform]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Config-Driven Tool Design

## Failure Mode

A tool reads one config file from one hardcoded path. Users override with
env vars that silently win over the file, so the file lies. A schema change
breaks every existing config with an opaque parse error. A config edit
requires restarting the tool, so the feedback loop is seconds instead of
milliseconds. On macOS the tool writes to `~/.config` and on Windows it
looks in `~/.config` too, and nobody can find their settings.

## Practice

### Layered config precedence

Precedence is fixed, documented, and applied in the same order every run.
Highest to lowest:

1. **CLI args** — `--set key=val` and flags.
2. **Env vars** — `APP__SECTION__KEY` (double underscore = nesting).
3. **Project config** — `./.app/config.toml`, checked into the repo.
4. **User config** — per-user overrides under the platform config dir.
5. **Built-in defaults** — compiled into the binary.

Each layer is parsed independently and merged with a deep-merge; later
layers never partially overwrite a table from an earlier one.

```rust
pub fn load_config(cli: &CliArgs) -> Result<Config, ConfigError> {
    let mut cfg = Config::default();
    merge_user(&mut cfg)?;
    merge_project(&mut cfg)?;
    merge_env(&mut cfg)?;
    merge_cli(&mut cfg, cli)?;
    validate(&cfg)?;
    Ok(cfg)
}
```

### Profile-based activation

A config carries one or more named profiles. Each profile enables a rule
set and may override severity. The active profile is selected by CLI flag or
env var — never inferred from the directory, because inference makes the
output depend on where you ran the binary.

```toml
[profiles.strict]
inherit = "default"
rules = { import-order = "error", license-header = "error" }

[profiles.library]
inherit = "default"
rules = { public-api-docs = "warn" }
```

### Schema validation and migration

The config schema is versioned with a `schema_version` field. On load, the
tool runs the migration chain: `v1 → v2 → v3`, each migration a pure
function from the prior shape to the next. A config that skips versions is
migrated forward automatically; a config ahead of the tool's max version
fails with a clear "upgrade the tool" message.

```rust
fn migrate(raw: serde_json::Value) -> Result<Config, ConfigError> {
    let v = raw["schema_version"].as_u64().unwrap_or(1);
    let raw = (v..CURRENT_SCHEMA_VERSION).fold(Ok(raw), |acc, _| {
        acc.and_then(step_migrate)
    })?;
    serde_json::from_value::<Config>(raw).map_err(ConfigError::from)
}
```

Validate the migrated config against a schema before use; never run on a
config that failed validation. See
[toml-config-validation](https://github.com/levonk/skills-releases/blob/main/knowledge/rust-development-practices/toml-config-validation.md)
for the per-field validation pattern.

### Hot-reload with atomic swap

A `notify` 6.1 file watcher observes the active config files. On change,
the tool reloads, validates, and — only on success — swaps the live config
behind an `Arc<ArcSwap<Config>>`. A failed reload keeps the old config and
logs the error; the tool never runs with a half-parsed config.

```rust
let cfg = Arc::new(ArcSwap::from_pointee(load_config(&cli)?));
let cfg_for_watcher = Arc::clone(&cfg);
notify::watcher(move |res| {
    if let Ok(new) = load_config(&cli) {
        cfg_for_watcher.store(Arc::new(new));
    } else {
        tracing::warn!("config reload failed; keeping previous config");
    }
})?;
```

### Cross-platform config discovery

Use the `directories` crate (or equivalent) so the user config path follows
the platform convention, not a hardcoded `~/.config`.

| Platform | User config path |
|----------|------------------|
| Linux    | `$XDG_CONFIG_HOME/app/config.toml` (default `~/.config/app/`) |
| macOS    | `~/Library/Application Support/app/config.toml` |
| Windows  | `%APPDATA%\app\config.toml` |

Project config is always `./.app/config.toml` relative to the working
directory — this is consistent across platforms because it is repo-relative,
not user-relative.

## Related Concepts

- [Configuration System](configuration-system.md) — the base layered-config
  pattern this page extends with profiles, migration, and hot-reload.
- [toml-config-validation](https://github.com/levonk/skills-releases/blob/main/knowledge/rust-development-practices/toml-config-validation.md)
  — per-field TOML validation in Rust, used by the schema-validation step
  above.
