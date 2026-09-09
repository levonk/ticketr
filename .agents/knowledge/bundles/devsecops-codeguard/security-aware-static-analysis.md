---
type: Practice
title: Security-Aware Static Analysis — Safe Handling of Untrusted Code and Tool Output
description: Treat all analyzed code as hostile, never execute it, redact source content from error messages and logs, isolate tree-sitter grammar parsing, audit the analysis tools themselves, and minimize telemetry to aggregate metrics.
tags: [security, devsecops, rust, static-analysis, tree-sitter, sandboxing, supply-chain, telemetry]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Security-Aware Static Analysis

## Failure Mode

A static analysis tool reads untrusted source code and emits findings. If the
tool executes the code, an attacker can compromise the analysis host. If the
tool prints source snippets in errors or logs, it leaks secrets that the code
contained. If the tool's own dependencies are unaudited, the linter becomes
the attack vector. If telemetry sends source-level data, the CI pipeline
becomes a secret exfiltration channel. These risks apply to every static
analysis tool — linters, SAST engines, dependency scanners — not only to one
project.

## Practice

### Never Execute Analyzed Code

Treat every input file as hostile. Parse it; do not run it. Build directives
in `Cargo.toml` build scripts, `package.json` lifecycle scripts, and
`Makefile` rules are code, not data. Read them as text. Never invoke a build
step to "understand" the project.

```rust
// Bad: runs the project's build, which can execute arbitrary code
Command::new("cargo").arg("build").output()?;

// Good: parse the manifest as data, never execute it
let manifest: toml::Value = std::fs::read_to_string(manifest_path)?
    .parse()?;
let deps = manifest["dependencies"].as_table().unwrap_or_default();
```

### Redact Source Content from Errors and Logs

CI output is shared, archived, and often public. Print rule IDs and line
numbers — not source snippets. A SARIF report must omit `region.snippet.text`
for any rule that can match a secret.

```rust
// Bad: the snippet may contain a leaked credential
eprintln!("SEC001 at {}:{}: {}", path, line, source_line);

// Good: rule ID and location only
eprintln!("SEC001 hardcoded-credential at {}:{}", path, line);
```

Apply the same rule to debug logs. A `tracing` span must not capture source
content as a field.

### Isolate tree-sitter Grammar Parsing

tree-sitter 0.23 compiles C grammars into the Rust process. A grammar bug
can corrupt memory. For grammars from untrusted sources, parse in a sandboxed
child process with resource limits so a crash cannot take down the host.

```rust
// Trusted vendored grammar: in-process parse
let tree = parser.parse(source, None);

// Untrusted grammar: child process with rlimit and timeout
let report = sandbox::parse_in_child(&grammar, source, Duration::from_secs(10))?;
```

### Audit the Analysis Tools Themselves

The linter is a dependency. It runs against every project and its output
drives CI decisions. Audit the linter's own supply chain with the same rigor
as any production dependency.

```bash
# Audit the linter's dependencies before shipping it
cargo audit            # RustSec advisories
cargo deny check       # advisories, licenses, bans
```

Pin the linter's `Cargo.lock` and review changes to it in CI. See
[Dependency Supply Chain](dependency-supply-chain.md) for the full practice.

### Minimize Telemetry

Send aggregate metrics only. Never send file paths, source content, or
identifiers. A telemetry event carries counts and rule IDs — nothing that
can identify a secret or a project.

```rust
// Good: aggregate counts, no source-level data
telemetry::emit("analysis_run", json!({
    "rules_run": rule_count,
    "findings": findings_by_rule, // { "SEC001": 3, "SEC002": 1 }
    "duration_ms": elapsed.as_millis() as u64,
}));
```

## Related Concepts

- [Linter Security Patterns](linter-security-patterns.md) — the linter-specific application of these principles.
- [Hardcoded Credentials Detection](hardcoded-credentials-detection.md) — recognition patterns that analysis rules match against.
- [Dependency Supply Chain](dependency-supply-chain.md) — auditing the analysis tool's own dependencies.
- [secrets-egress-security overview](https://github.com/levonk/skills-releases/blob/main/knowledge/secrets-egress-security/overview.md) — egress controls that contain analysis tool network access.
- [rust-development-practices overview](https://github.com/levonk/skills-releases/blob/main/knowledge/rust-development-practices/overview.md) — Rust toolchain conventions for building analysis tools safely.
