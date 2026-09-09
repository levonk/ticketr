---
type: Practice
title: Rule Engine Design
description: Order rules by dependency, activate them by profile, resolve conflicts by severity hierarchy, and re-run only rules affected by a changed file.
tags: [architecture, rust, linter, rule-engine, incremental]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Rule Engine Design

## Failure Mode

A rule engine runs every rule against every file on every invocation. Rules
contradict each other with no documented winner, so developers silence the
whole engine rather than debug the conflict. Adding a rule that depends on
another rule's output is impossible because evaluation order is undefined, so
dependent rules either race or are hand-wired into a fragile pipeline.

## Practice

### Rule struct

A rule is data, not a free-floating function. Keeping the metadata next to the
evaluator lets the engine reason about ordering, profiles, and conflicts
without `match` ladders.

```rust
pub struct Rule {
    pub id: &'static str,
    pub severity: Severity,
    pub profiles: &'static [&'static str],
    pub triggers: &'static [&'static str],
    pub depends_on: &'static [&'static str],
    pub evaluate: fn(&ScanFile, &ScanContext) -> Vec<Finding>,
}

pub enum Severity { Error, Warn, Info }
```

### Evaluation order and dependencies

Build a dependency graph from `depends_on` and topologically sort it before
each run. Rules with no dependency between them run in parallel across files.
A cycle is a hard error at load time — never a silent mis-ordering.

```rust
fn order_rules(rules: &[Rule]) -> Result<Vec<&Rule>, CycleError> {
    let mut graph = petgraph::Graph::new();
    // ...add nodes and edges from depends_on...
    let idx = petgraph::algo::toposort(&graph, None)
        .map_err(|_| CycleError::new(rules))?;
    Ok(idx.into_iter().map(|i| &rules[i.index()]).collect())
}
```

### Rule activation profiles

Rules declare which profiles they belong to (`profiles: ["rust", "strict"]`).
The engine enables a rule only when the active profile is in that list. This
keeps a single rule set shippable across project types — a library, a CLI,
and a workspace can all share the binary and differ only in
`--profile rust-strict`.

### Conflict resolution

Two rules will eventually flag the same line with opposing fixes. Resolve by
a fixed hierarchy, not by insertion order:

1. **Severity hierarchy.** An `Error` finding suppresses a conflicting `Warn`
   on the same span.
2. **Explicit overrides.** A `[overrides]` table in config can pin a winner
   by rule id for a known conflict. Document the conflict in the override so
   the next reader knows why it exists.

Never resolve conflicts by "last rule wins" — that makes the engine's output
depend on hash-map iteration order.

### Performance optimization

- **Compile regex patterns once.** Build `regex::Regex` (or
  `regex::RegexSet`) in `init`, store on the rule, and reuse across files.
  Compiling per file is the single most common linter perf bug.
- **Cache file metadata.** Skip re-reading a file when its mtime and size are
  unchanged since the last run. Store the cache under the build directory so
  it is cleaned by `just clean`.
- **Incremental evaluation.** Only re-run rules whose `triggers` intersect
  the set of changed files (see below).

### Incremental rule evaluation

On a file change, compute the affected rule set: a rule is affected if the
changed file's language or path matches one of its `triggers`, or if the rule
`depends_on` another affected rule. Re-run only that subset. The unaffected
findings are reused from the previous run's cached report. This turns a
full-repo scan into a per-file scan for the common "edit one file" case.

## Related Concepts

- [Plugin Scanner Registration](plugin-scanner-registration.md) — scanners
  host rules; the rule engine decides which run and in what order.
- [Configuration System](configuration-system.md) — profiles and overrides
  are config-driven; the engine reads them through the config layer.
