---
type: Practice
title: Linter Security Patterns — Safe Analysis of Untrusted Source Code
description: Redact secrets from linter output, treat analyzed source as hostile, isolate tree-sitter grammar parsing, sign and verify plugins, and minimize telemetry to aggregate metrics only.
tags: [security, devsecops, rust, linter, tree-sitter, sandboxing, telemetry, plugins]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Linter Security Patterns

## Failure Mode

A linter reads untrusted source files and emits diagnostics. If the linter
echoes file contents, paths, or identifiers in its output, it leaks secrets
that the source code contained. If the linter executes code or trusts file
metadata, an attacker can compromise the analysis host. If plugins run with
full process privileges, a malicious plugin can exfiltrate data or persist
backdoors. If telemetry sends source-level data, the CI pipeline becomes a
secret exfiltration channel.

## Practice

### Redact File Contents in Error Output

Never print raw source lines in diagnostics. Print the rule ID, the line
number, and a redacted hint — not the offending text.

```rust
// Bad: leaks the secret value into CI logs
eprintln!("hardcoded token at {}: {}", path.display(), line_text);

// Good: redact the content, keep the location and rule
eprintln!("SEC001 hardcoded-credential at {}:{} (redacted)", path.display(), line);
```

Apply the same redaction to structured output (JSON/SARIF). A SARIF result
carries `ruleId` and `region.startLine` — never `region.snippet.text` when
the region may contain a secret.

### Treat Analyzed Source as Hostile

The linter reads untrusted source files. Do not execute the code. Do not
trust `Cargo.toml` metadata, `package.json` scripts, or build directives —
parse them, never run them. Reject files above a size limit and cap node
counts to prevent resource exhaustion.

```rust
const MAX_FILE_BYTES: usize = 1 << 20; // 1 MiB
const MAX_PARSE_NODES: usize = 1_000_000;

let bytes = std::fs::read(&path)?;
if bytes.len() > MAX_FILE_BYTES { return Err(AnalysisError::FileTooLarge); }
let tree = parser.parse(&bytes, None)?;
if tree.root_node().descendant_count() > MAX_PARSE_NODES {
    return Err(AnalysisError::NodeLimitExceeded);
}
```

### Sandbox AST Parsing

tree-sitter 0.23 compiles C grammars into the Rust process. A grammar bug
or a malicious grammar can corrupt memory. For grammars from untrusted
sources, parse in a sandboxed child process so a crash cannot take down the
linter host.

```rust
let tree = parser.parse(source, None);              // trusted vendored grammar
let report = sandbox::parse_in_child(&grammar_path, source)?; // untrusted
```

### Sign and Verify Plugins

Plugins run with full process privileges — they can read the filesystem,
open sockets, and write build artifacts. Require a signature on every plugin,
verify it before load, and reject unsigned plugins in CI.

```toml
# project-lint.toml
[[plugins]]
name = "ban-weak-crypto"
version = "1.0.0"
signature = "sha256:9f86d081884c7d65..."
```

```rust
fn load_plugin(path: &Path) -> Result<Plugin> {
    let sig = read_signature(path)?;
    verify_signature(path, &sig)?; // fail closed
    Ok(Plugin::load(path)?)
}
```

### Minimize Telemetry

Send aggregate metrics only. Never send file paths, source content, or
identifiers. A telemetry event carries counts and rule IDs — nothing that
can identify a secret or a project.

```rust
// Bad: leaks the path and the matched text
telemetry::emit("finding", json!({ "path": path.display().to_string(), "snippet": matched_text }));

// Good: aggregate counts only
telemetry::emit("findings", json!({ "rule": rule_id, "count": findings_for_rule }));
```

## Related Concepts

- [Security Audit Playbook](security-audit-playbook.md) — final validation that the linter runs in CI and fails the build on critical findings.
- [Hardcoded Credentials Detection](hardcoded-credentials-detection.md) — recognition patterns the linter rules match against.
- [Security-Aware Static Analysis](security-aware-static-analysis.md) — cross-bundle synthesis of these patterns for any static analysis tool.
