---
type: Practice
title: Trait-Based Extensibility
description: Build linter rule systems on a base Rule trait with default impls, choose dyn Rule vs generics deliberately, compose rules with combinators, and version traits with semver and default methods.
tags: [architecture, rust, linter, trait, extensibility, semver]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Trait-Based Extensibility

## Failure Mode

A linter defines rules as an enum and adds a variant per check. Every new
check touches the enum, the match in the evaluator, and the match in the
reporter. Default behavior is copy-pasted across variants because enums
cannot share impls, so a fix to the default path must be applied N times.
After twenty rules the enum is the file, and nobody refactors it.

## Practice

### Base Rule trait with default impls

Define one base trait that captures the universal rule contract. Provide
sensible defaults for every method that is not the core evaluator, so a new
rule is one method plus metadata.

```rust
pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn severity(&self) -> Severity { Severity::Warn }
    fn applies_to(&self, _file: &ScanFile) -> bool { true }
    fn evaluate(&self, file: &ScanFile, ctx: &ScanContext) -> Vec<Finding>;
    fn finalize(&self, _findings: &mut Vec<Finding>) {}
}
```

A new rule implements `id` and `evaluate`; everything else inherits the
default. Override a default only when the rule genuinely differs.

### Specialized traits for rule categories

Categories with shared shape (e.g., AST-walking rules vs. line-based rules)
get a specialized trait that extends the base. The specialized trait adds
the category-specific hook and a blanket impl of the base trait so a
category rule is automatically a `Rule`.

```rust
pub trait AstRule: Rule {
    fn visit(&self, tree: &tree_sitter::Tree, src: &[u8], ctx: &ScanContext)
        -> Vec<Finding>;
}
```

### dyn Rule vs `<R: Rule>`

The two dispatch styles have different costs; pick per call site.

- **`Box<dyn Rule>` (dynamic dispatch).** Store heterogeneous rules in one
  collection, iterate, call `evaluate`. One vtable lookup per call —
  negligible next to file I/O and regex matching. Use this for the rule
  registry and the main evaluation loop.
- **`<R: Rule>` (static dispatch, monomorphization).** The compiler generates
  a fresh copy of the surrounding code per concrete rule. Faster in tight
  inner loops, but binary size and compile times grow with the rule count.
  Use this for hot, single-rule paths — e.g., a combinator that wraps one
  inner rule (below).

Default to `dyn Rule` for collections; reach for generics only when a
profile shows a dispatch-bound hot loop.

### Combinator patterns

Compose rules with `AndRule`, `OrRule`, and `NotRule` rather than writing
one-off combined rules. Combinators are generic over the inner rule(s), so
they monomorphize and stay zero-cost.

```rust
pub struct AndRule<A: Rule, B: Rule>(pub A, pub B);

impl<A: Rule, B: Rule> Rule for AndRule<A, B> {
    fn id(&self) -> &'static str { "and" }
    fn evaluate(&self, file: &ScanFile, ctx: &ScanContext) -> Vec<Finding> {
        let mut out = self.0.evaluate(file, ctx);
        out.extend(self.1.evaluate(file, ctx));
        out
    }
}
```

Combinators let users express "warn only if both A and B fire" in config
without a new rule type.

### Trait versioning for backward compatibility

Traits are public API. Version them accordingly.

- **Semver for traits.** Adding a method with a default impl is a minor
  bump (existing impls still compile). Adding a method without a default, or
  changing a signature, is a major bump. Document this in the trait's
  doc-comment so downstream plugin authors know what to expect.
- **Default methods for new requirements.** When a new requirement appears,
  add it as a method with a default impl that preserves the old behavior.
  Rules that need the new behavior override it; the rest keep working
  unchanged. Never remove a method in a minor release — deprecate first,
  remove at the next major.

## Related Concepts

- [Plugin Scanner Registration](plugin-scanner-registration.md) — scanners
  are the host; traits are how rules plug into them.
- [Rule Engine Design](rule-engine-design.md) — the engine that evaluates
  trait-object rules and orders them.
