---
type: Practice
title: Plugin Scanner Registration
description: Trait-based plugin registration for linters — define a Scanner trait, register implementations, and manage the init → scan → report → cleanup lifecycle with explicit dependency injection.
tags: [architecture, rust, linter, plugin, trait, scanner]
date:
  created: "2026-08-05"
  knowledge-basis: "2026-08-05"
  last-used: "2026-08-05"

sources:
  - id: project-lint-audit
    resource: "project-lint/Cargo.toml"
    title: "project-lint"
---

# Plugin Scanner Registration

## Failure Mode

A linter hard-codes every check in a single match statement or a giant
`fn run_all` dispatcher. Adding a check means editing the dispatcher, the
config parser, and the report aggregator — three touch points that drift out
of sync under pressure. New contributors cannot ship a scanner without
understanding the whole binary, so checks either land unreviewed or never
land at all.

## Practice

### Define a Scanner trait

Every scanner implements one trait. The trait is the only contract the host
binary depends on; everything behind it can evolve independently.

```rust
pub trait Scanner: Send + Sync {
    /// Stable identifier used in config and reports.
    fn id(&self) -> &'static str;

    /// One-time setup: compile regexes, load baseline data.
    fn init(&mut self, ctx: &ScanContext) -> Result<(), ScanError>;

    /// Run against a single file (or buffer). Emits zero or more findings.
    fn scan(&self, file: &ScanFile, ctx: &ScanContext) -> Vec<Finding>;

    /// Aggregate per-file findings into a report section.
    fn report(&self, findings: &[Finding]) -> ReportSection;

    /// Release resources before drop (close file handles, flush caches).
    fn cleanup(&mut self) {}
}

pub struct ScanContext {
    pub config: serde_json::Value,
    pub workdir: std::path::PathBuf,
    pub baseline: Option<Baseline>,
}
```

### Register implementations

Two registration styles cover most linters. Pick by team size and release
cadence, not by performance.

**Compile-time registration (static).** Scanners are linked into the binary
and registered in a single `inventory`-style or explicit list. Zero runtime
cost, zero ABI surface, zero unsafe loading.

```rust
pub fn registered_scanners() -> Vec<Box<dyn Scanner>> {
    vec![
        Box::new(CopyrightScanner::default()),
        Box::new(LicenseHeaderScanner::default()),
        Box::new(ImportOrderScanner::default()),
    ]
}
```

**Dynamic loading (via `libloading`).** Scanners ship as `.so` / `.dylib` /
`.dll` and are loaded at startup. Use this only when scanners must be updated
without recompiling the host — e.g., a plugin marketplace. The cost is an
`unsafe` extern "C" boundary, ABI pinning, and a harder debugging story.

```rust
unsafe fn load_plugin(path: &Path) -> Result<Box<dyn Scanner>, LoadError> {
    let lib = libloading::Library::new(path)?;
    let ctor: libloading::Symbol<unsafe fn() -> *mut dyn Scanner> =
        lib.get(b"_create_scanner")?;
    let raw = ctor();
    Ok(Box::from_raw(raw))
}
```

Prefer static registration until you have a concrete reason to ship plugins
out-of-band. Most linters never reach that bar.

### Plugin discovery

- **Convention-based.** Scan a `scanners/` directory; any file matching
  `scanner-*.toml` or `*.wasm` is a candidate. Fast to author, but a typo
  silently registers nothing.
- **Explicit config.** A `[scanners]` table in the project config lists
  scanner ids and their per-scanner config. Verbose, but a missing scanner
  fails loudly with a "unknown scanner" error.

Use explicit config for anything shipped to users; convention-based only for
local developer tooling where the failure is visible immediately.

### Lifecycle management

Every scanner moves through `init → scan → report → cleanup` in that order.
The host drives the lifecycle; scanners never call back into the host. This
keeps scanners testable in isolation — construct one, call `init` with a
fake `ScanContext`, assert on `scan` output.

### Dependency injection

Pass config and context to scanners; do not let scanners reach into global
state. `ScanContext` carries the workdir, the parsed config slice for that
scanner, and any baseline data. A scanner that reads `std::env::var` or
walks the filesystem outside its assigned files is a scanner that cannot be
unit-tested without polluting the test process.

## Related Concepts

- [Tool Detection](tool-detection.md) — how the host discovers the external
  tools that some scanners wrap.
- [Technology Selection Pattern](technology-selection-pattern.md) — the
  decision format for choosing static vs. dynamic registration.
