---
type: Practice
title: Security and Auditing
description: Regular cargo audit for dependency vulnerabilities, cargo outdated for dependency drift detection, secrecy crate for secret handling, zeroize for secure memory clearing, input validation, and safe FFI practices.
tags: [rust, security, cargo-audit, cargo-outdated, secrecy, zeroize, ffi, input-validation]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-08-20"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md"
    title: "levonk-base-boilerplate"
  - id: levonk-base-boilerplate-cargo-audit-wiring
    resource: "internal-docs/feature/todo/cargo-audit-wiring/feat-202608201814-cargo-audit-wiring.md"
    title: "levonk-base-boilerplate"
---

# Security and Auditing

## Failure Mode

Vulnerable dependencies ship to production without detection. Secrets leak
through logging. Unsafe FFI causes undefined behavior. Missing input validation
allows injection attacks. Dependency drift accumulates silently — transitive
deps fall behind security patches, and no one notices until a CVE drops.

## Practice

### Security Checklist

- **Input validation**: Validate all external inputs
- **Memory safety**: Leverage Rust's memory safety guarantees
- **Dependency auditing**: Regular security audits with `cargo audit`
- **Dependency drift detection**: Regular outdated checks with `cargo outdated`
- **Safe FFI**: Proper error handling for foreign function interfaces
- **Secrets management**: Never commit secrets, use environment variables

### Security Dependencies

```toml
[dependencies]
secrecy = "0.8"   # For secret handling
zeroize = "1.7"   # For secure memory clearing
```

### Devbox Package Provisioning

Both `cargo-audit` and `cargo-outdated` are provisioned via the shared devbox
package partial at `_shared/partials/devbox-partials/devbox-packages-rust.jinja`,
so every Rust project (CLI and package) gets them automatically:

```json
"cargo-audit": "",
"cargo-outdated": ""
```

This eliminates the "tool not available" fallback path — the tools are always
present in the devbox environment.

### CI/CD Security

```bash
cargo audit    # Check for known vulnerabilities (blocking)
cargo outdated # Check for outdated dependencies (non-blocking in validate)
```

### Validation Pipeline

`just validate` runs both checks as part of the standard validation pipeline:

1. `cargo fmt --check` (blocking)
2. `cargo clippy -D warnings` (blocking)
3. `cargo test` (blocking)
4. `cargo doc` (blocking)
5. `cargo audit` (blocking — vulnerable deps fail validation)
6. `cargo outdated --exit-code 0` (non-blocking — reports drift, warns on
   failure, never aborts validate)

The outdated check is non-blocking because drift is informational — it
doesn't mean the project is broken, just that updates are available. The
weekly CI cron job (see [Quality Gates](quality-gates.md)) is the blocking
version that actually applies updates and creates a PR.

## Related Concepts

- [Cargo Configuration](cargo-configuration.md) — Dependency management
- [Quality Gates](quality-gates.md) — CI runs cargo audit + weekly outdated PR
- [Container Support](container-support.md) — Non-root container
