---
type: Practice
title: Tree-Sitter AST Queries
description: Use tree-sitter query syntax for linting, capture node metadata with @capture names, organize queries per language, cache compiled queries, and snapshot-test AST matches.
tags: [rust, tree-sitter, ast, linting, queries, parsing]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Tree-Sitter AST Queries

## Failure Mode

Hand-written AST walkers drift as grammars evolve. String-matching linters miss
nested constructs and produce false positives. Re-parsing files per rule wastes
CPU on large codebases. Uncached query compilation adds hundreds of milliseconds
per language per run.

## Practice

### Query Syntax for Linting

Tree-sitter queries combine sexp node patterns with field predicates and
captures. Match function calls, imports, and declarations directly against the
syntax tree.

```rust
// Detect bare println! calls in Rust source.
let query_src = r#"
(call_expression
  (macro_invocation
    (identifier) @macro.name
    (#eq? @macro.name "println"))
) @println.call
"#;

let query = Query::new(&language, query_src)?;
let mut cursor = QueryCursor::new();
for m in cursor.matches(&query, root_node, source_bytes) {
    for cap in m.captures {
        match cap.name.as_ref() {
            "println.call" => report(cap.node, "prefer tracing over println!"),
            _ => {}
        }
    }
}
```

Field names (`name:`, `body:`) anchor patterns to grammar slots. Predicates
(`#eq?`, `#match?`, `#not-match?`) filter captures without leaving the query
engine.

### Capture Naming

Use dot-namespaced captures so rule code can distinguish roles:

- `@function.name` — identifier of a function declaration
- `@import.path` — module path in a use statement
- `@rule.violation` — the node to report against

Consistent names let one matcher loop drive many rules.

### Multi-Language Organization

Keep one query file per language. Each grammar has different node types; a
unified query file becomes a forest of `#if`-style predicates. Load queries
lazily by language id.

```rust
fn load_query(lang: &str) -> Option<Query> {
    let src = include_str!(concat!("../queries/", lang, ".scm"));
    let grammar = grammar_for(lang)?;
    Query::new(grammar, src).ok()
}
```

Project-lint's AST analyzer follows this split: Rust, Python, JavaScript,
TypeScript, JSON, YAML, and TOML each ship a dedicated `.scm` file.

### Performance on Large Codebases

Parse once per file. Reuse the `Parser` and `Tree` across all rules. Compile
each `Query` once and store it in a `HashMap<&str, Query>` keyed by language.
Never call `Query::new` inside the per-file loop.

For incremental scans, feed `parser.parse(input, old_tree)` the previous tree so
edits re-parse only the changed ranges. This matters for watcher-driven linters
that re-check a single file on save.

### Snapshot Testing of AST Queries

Golden-file tests pin query output so grammar upgrades surface as reviewable
diffs. Store input source fixtures and expected capture lists side by side.

```rust
#[test]
fn detects_bare_println() {
    let src = r#"fn main() { println!("hi"); }"#;
    let captures = run_query("rust", "println.scm", src);
    insta::assert_yaml_snapshot!(captures);
}
```

Bump the tree-sitter grammar version in `Cargo.toml` and run the snapshot suite.
Any change in node shapes fails loudly with a diff to review.

## Related Concepts

- [Testing Strategy](testing-strategy.md) — snapshot tests for query output
- [Quality Gates](quality-gates.md) — CI gate on query snapshot drift
- [Cargo Configuration](cargo-configuration.md) — pinning tree-sitter 0.23
