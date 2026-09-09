---
type: Practice
title: Snapshot Testing in Rust
description: insta crate for inline and external snapshot tests. Review snapshots in CI, force-update only after human review, prefer snapshots over assertions for complex or structured output.
tags: [ci-cd, testing, rust, insta, snapshot]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Snapshot Testing in Rust

## Failure Mode

Hand-written assertions for complex output grow unwieldy. Developers paste
multi-line expected strings into tests, then stop updating them when output
changes shape. Tests rot, get deleted, or pass against wrong output because
the assertion was never precise.

## Practice

Use the **insta** crate. Capture output once, review the snapshot, and let
insta fail on any future drift.

### Inline Snapshots

For short output, embed the snapshot directly in the test source.

```rust
#[test]
fn formats_summary() {
    let report = format_summary(3, 12);
    insta::assert_snapshot!(report, @r###"
    3 passed
    12 failed
    "###);
}
```

insta rewrites the inline string on the first run and on intentional changes
via `cargo insta review`.

### External Snapshot Files

For long or binary-shaped output, use file snapshots. insta writes a `.snap`
file next to the test.

```rust
#[test]
fn cli_help_text_is_stable() {
    let help = render_help();
    insta::assert_snapshot!("cli_help", help);
}
```

The generated `snapshots/cli_help.snap` is committed and reviewed in PRs.

### Snapshot Review in CI

Never auto-accept snapshots in CI. Set `CARGO_INSTA_FORCE_UPDATE` only in
controlled update jobs, never on the main test job.

```bash
# CI test job — fail on new or changed snapshots
cargo nextest run --workspace

# Dedicated update job (manual trigger) — refresh snapshots
CARGO_INSTA_FORCE_UPDATE=1 cargo nextest run --workspace
cargo insta accept
```

Locally, run `cargo insta review` to walk pending snapshots one by one.

### Updating Snapshots Safely

1. Run the test. insta writes `.snap.new` files for mismatches.
2. Run `cargo insta review`. Accept, reject, or skip each diff.
3. Commit the accepted `.snap` files alongside the code change.
4. Never run `cargo insta accept --all` without reviewing. Blind acceptance
   defeats the purpose of snapshot tests.

### When Snapshots Beat Assertions

- **CLI output**: Help text, error messages, version strings.
- **Structured reports**: JSON or YAML with many fields.
- **Serialization round-trips**: Verify a struct serializes to the exact
  expected bytes.
- **Error rendering**: Multi-line diagnostic output.

Prefer plain assertions for single-value checks. Snapshots add review overhead
that single values do not justify.

### CI Integration

```yaml
- run: cargo nextest run --workspace
- run: |
    if git diff --name-only | grep -q '\.snap\.new$'; then
      echo "Unreviewed snapshot changes present"
      exit 1
    fi
```

## Related Concepts

- [Vitest Unified Runner](vitest-unified-runner.md) — TypeScript equivalent
  via Vitest snapshot matchers
- [Rust CI Tooling](rust-ci-tooling.md) — Run insta under cargo-nextest in CI
