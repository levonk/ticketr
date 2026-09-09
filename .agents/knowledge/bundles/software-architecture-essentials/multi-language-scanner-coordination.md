---
type: Practice
title: Multi-Language Scanner Coordination
description: Detect language by extension, shebang, and content sniffing; dispatch to language-specific scanners behind a shared Scanner trait; and run polyglot cross-language analysis on shared config — synthesized across the architecture and Rust bundles.
tags: [architecture, rust, linter, multi-language, scanner, polyglot, tree-sitter]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Multi-Language Scanner Coordination

## Failure Mode

A linter is built for one language and grows a second by copy-pasting the
whole binary. The two copies drift: a fix to copyright scanning lands in one
and not the other. A polyglot repo (Rust + Python + TOML) runs three
separate linters with three configs and three output formats, so the
developer writes a fourth tool to merge the reports. Cross-language rules —
"every Python entry point referenced in a Rust build script must exist" —
are impossible because no single process sees both languages.

## Practice

### Language detection heuristics

Detect language in three tiers, cheapest first:

1. **File extension.** `.rs` → Rust, `.py` → Python, `.toml` → TOML. This
   covers >95% of files and is O(1).
2. **Shebang line.** Extensionless scripts carry `#!/usr/bin/env python3`
   or `#!/bin/bash`. Read the first line only.
3. **Content sniffing.** For truly ambiguous files, sniff the first few
   hundred bytes for language-defining tokens. Use sparingly — it is the
   slowest tier and the most likely to misfire.

```rust
pub fn detect_language(path: &Path, content: &[u8]) -> Language {
    if let Some(lang) = Language::from_extension(path) {
        return lang;
    }
    if let Some(lang) = sniff_shebang(content) {
        return lang;
    }
    sniff_content(content).unwrap_or(Language::Unknown)
}
```

### Scanner registration and discovery

Register language-specific scanners in a map keyed by `Language`. The
coordinator detects the language of each file and dispatches to the
registered scanner. A file with `Language::Unknown` is skipped with a
debug log — never an error, because repos contain generated and vendored
files the linter should not touch.

### Shared scanner interfaces

All language scanners implement the same `Scanner` trait from
[Plugin Scanner Registration](plugin-scanner-registration.md). The trait is
language-agnostic; language-specific config rides on the `ScanContext`. A
scanner that needs a tree-sitter grammar receives the parsed tree through
the context, not by parsing itself — parsing is centralized so grammars are
loaded once per language, not once per scanner.

### Language-specific rule sets

Rules declare which languages they apply to. Three categories cover the
space:

- **Language-specific rules.** `unused-import` is Rust-only; `bare-except`
  is Python-only. These live in the language-specific scanner.
- **Shared rules.** `license-header`, `copyright-year`, and
  `no-trailing-whitespace` apply to every text file. These live in a
  language-agnostic scanner that runs for all languages.
- **Cross-language rules.** These run after per-language scans and see the
  union of findings — e.g., validating that a Python module referenced in a
  Rust `build.rs` actually exists on the path.

### Language dispatch

```rust
pub fn coordinate(files: &[ScanFile], ctx: &ScanContext) -> Report {
    let mut by_lang: HashMap<Language, Vec<&ScanFile>> = HashMap::new();
    for f in files {
        by_lang.entry(detect_language(&f.path, &f.content))
            .or_default().push(f);
    }
    let mut report = Report::default();
    for (lang, lang_files) in &by_lang {
        if let Some(scanner) = ctx.scanners.get(lang) {
            report.merge(scanner.scan_batch(lang_files, ctx));
        }
    }
    report.merge(cross_language_rules(&by_lang, ctx));
    report
}
```

### Polyglot project analysis

After per-language scans complete, run a cross-language pass that sees the
aggregated findings and the project graph. Examples:

- **Shared config validation.** A workspace's `Cargo.toml`, `pyproject.toml`,
  and `package.json` should agree on the project name and version. The
  cross-language pass reads all three and asserts consistency.
- **Cross-language dependencies.** A Rust build script that shells out to a
  Python tool should verify the Python tool is declared in
  `pyproject.toml`. This is only visible to a process that sees both
  languages.

For AST-level cross-language queries, use the shared tree-sitter query
pattern documented in
[tree-sitter-ast-queries](https://github.com/levonk/skills-releases/blob/main/knowledge/rust-development-practices/tree-sitter-ast-queries.md)
so every language scanner speaks the same query dialect.

## Related Concepts

- [Plugin Scanner Registration](plugin-scanner-registration.md) — the
  `Scanner` trait and registry that language-specific scanners plug into.
- [tree-sitter-ast-queries](https://github.com/levonk/skills-releases/blob/main/knowledge/rust-development-practices/tree-sitter-ast-queries.md)
  — shared AST query pattern used by every language scanner that walks a
  tree-sitter grammar.
