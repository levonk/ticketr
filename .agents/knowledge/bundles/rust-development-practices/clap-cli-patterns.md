---
type: Practice
title: Clap CLI Patterns
description: Use clap derive for ergonomics, enum-based subcommands for modular linters, value_parser for validation, consistent help text, and clap_complete for shell completions.
tags: [rust, clap, cli, derive, subcommands, shell-completion]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Clap CLI Patterns

## Failure Mode

Builder-style clap code grows verbose and drifts from help text. Flat argument
lists become unmanageable once a tool adds a second concern. Missing shell
completions force users to hand-type subcommands. Inconsistent help strings
erode trust in the binary.

## Practice

### Derive vs Builder

Prefer the derive API for application CLIs. Struct fields map to flags, types
drive parsing, and `#[command(...)]` attributes keep metadata co-located with
the data. Reserve the builder API for generated CLIs or runtime-shaped argument
sets.

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "project-lint", version, about = "Lint a project tree")]
pub struct Cli {
    /// Path to the project root.
    #[arg(short, long, default_value = ".")]
    pub path: std::path::PathBuf,

    /// Output format: human, json, toon.
    #[arg(short, long, value_parser = ["human", "json", "toon"], default_value = "human")]
    pub format: String,

    #[command(subcommand)]
    pub command: Option<Command>,
}
```

### Enum-Based Subcommands

Model subcommands as an enum. Each variant carries its own argument struct. New
concerns add a variant without touching shared parsing.

```rust
#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Run all enabled rules.
    Check(CheckArgs),
    /// Explain a single rule by id.
    Explain { rule_id: String },
    /// Generate shell completions.
    Completions { shell: clap_complete::Shell },
}
```

### Argument Validators

Use `value_parser` for enum-like strings, path existence, and range checks.
Reject bad input at parse time so error messages are uniform and exit codes are
consistent.

```rust
#[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
pub parallelism: u8,
```

### Help Text Consistency

Write doc comments as the help text. Start each with an imperative verb. Keep
one line for short help; expand in `long_about` only when the flag has
non-obvious interactions. Never duplicate the about string across subcommands —
reference the parent.

### Shell Completions

Generate completions at install time with `clap_complete`. Wire a subcommand so
users can emit them on demand.

```rust
use clap::Command;
use clap_complete::{generate, Shell};

fn completions(shell: Shell, mut cmd: Command, mut out: impl std::io::Write) {
    generate(shell, &mut cmd, "project-lint", &mut out);
}
```

Ship the generated script in the release so package managers can install it
without invoking the binary.

## Related Concepts

- [CLI Tool Standards](cli-tool-standards.md) — cross-language CLI contract
- [Toml Config Validation](toml-config-validation.md) — config precedence with CLI args
- [Project Structure](project-structure.md) — cli module placement
