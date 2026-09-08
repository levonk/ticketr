# Directory Update Log

## 2026-08-20

* **Update**: Updated [security-auditing.md](security-auditing.md) with
  cargo-outdated drift detection, shared devbox package provisioning
  (`cargo-audit` + `cargo-outdated` via `_shared/partials/devbox-partials/
  devbox-packages-rust.jinja`), and the 6-step validation pipeline
  (fmt → clippy → test → doc → audit → outdated).
* **Update**: Updated [quality-gates.md](quality-gates.md) with the full
  `just validate` pipeline, shared CI workflow (5 jobs: test, lint, build,
  security, outdated-check), and weekly dependency update workflow
  (cargo update + audit + build + test → PR via peter-evans/create-pull-request).
* **Source**: Added cargo-audit-wiring feature PRD
  (`internal-docs/feature/todo/cargo-audit-wiring/feat-202608201814-cargo-audit-wiring.md`)
  as a source on security-auditing and quality-gates.

## 2026-08-05

* **Ingest**: Authored 7 new concept pages sourced from the project-lint
  audit (Rust CLI linter tooling gaps). All pages grounded against
  project-lint/Cargo.toml versions on 2026-08-05.
  - [tree-sitter-ast-queries.md](tree-sitter-ast-queries.md) — query syntax
    for linting, multi-language query organization, performance, snapshot
    testing (tree-sitter 0.23)
  - [clap-cli-patterns.md](clap-cli-patterns.md) — derive vs builder,
    subcommand organization, argument validators, shell completion (clap 4.4)
  - [toml-config-validation.md](toml-config-validation.md) — serde + toml
    schema validation, profile merging, config migration, hot-reload
    (toml 0.8, serde 1.0)
  - [structured-logging-tracing.md](structured-logging-tracing.md) — tracing
    vs log, structured field conventions, subscriber config, RUST_LOG
    (tracing 0.1, tracing-subscriber 0.3)
  - [file-watcher-patterns.md](file-watcher-patterns.md) — notify event
    filtering, cross-platform watch limits, FS race conditions, async
    integration (notify 6.1)
  - [cross-platform-path-handling.md](cross-platform-path-handling.md) —
    std::path vs camino, normalization, symlinks, Windows UNC paths
  - [anyhow-thiserror-combination.md](anyhow-thiserror-combination.md) —
    layered error handling: thiserror in libs, anyhow at application layer
    (anyhow 1.0, thiserror 1.0)
* **Update**: Updated [index.md](index.md) with 7 new concept entries.

## 2026-07-29
* **Update**: Extended [cli-tool-standards.md](cli-tool-standards.md) with
  three new agent-mode requirements from ADR-20260607001 v5.0.0:
  - `--gen-skill` skill emission (print installable Agent Skill prompt to
    stdout, recommend hooks > CLI > MCP, `--check` for CI staleness)
  - Coding-agent hook wiring (Devin, Cursor, Continue, Aider opt-in; per-agent
    `[agent-hooks]` table in `${XDG_CONFIG_HOME:-$HOME/.config}/{app-name}/config.toml`;
    `--install-agent-hooks` / `--uninstall-agent-hooks`)
  - Non-user-identifiable telemetry (no file paths, no identifiers, no command
    args; `MYTOOL_TELEMETRY` env var + `[telemetry]` config; `--telemetry-dry-run`)
* **Source**: ADR-20260607001 bumped to v5.0.0 (date-updated 2026-07-29) in
  levonk-base-boilerplate.

## 2026-07-26
* **Migration**: Migrated `## Citations` body sections to `sources` frontmatter with stable `id` attributes per OKF v0.2 §13.1.
* **Migration**: Migrated bundle from OKF v0.1 to OKF v0.2 — bumped `okf_version` in index.md. No `# Citations` sections or `timestamp` fields to migrate.

## 2026-07-17

* **Initialization**: Created the `rust-development-practices` knowledge bundle to consolidate Rust package and CLI development practices from two ADRs in levonk-base-boilerplate.
* **Creation**: Authored 11 concept pages covering the full Rust development lifecycle.
  - [project-structure.md](project-structure.md) — standard directory layout, module organization
  - [cargo-configuration.md](cargo-configuration.md) — Cargo.toml metadata, dependency pinning, workspace config
  - [rustfmt-clippy-config.md](rustfmt-clippy-config.md) — formatting and linting standards
  - [testing-strategy.md](testing-strategy.md) — unit, integration, doc tests, benchmarks, proptest
  - [error-handling.md](error-handling.md) — thiserror, anyhow, no panic in libraries
  - [async-patterns.md](async-patterns.md) — tokio runtime, async tests, stream implementation
  - [serde-serialization.md](serde-serialization.md) — optional serde, multiple formats, skip patterns
  - [cli-tool-standards.md](cli-tool-standards.md) — cross-language CLI standards, AXI agent mode, TOON
  - [container-support.md](container-support.md) — multi-stage Dockerfile, non-root, healthcheck
  - [security-auditing.md](security-auditing.md) — cargo audit, secrecy, zeroize, safe FFI
  - [quality-gates.md](quality-gates.md) — pre-commit hooks, multi-version CI, validation criteria
* **Creation**: Established [overview.md](overview.md) synthesis and [index.md](index.md) directory listing.
* **Note**: Concepts extracted from ADR-20260128001 (Rust package boilerplate requirements, 756 lines) and ADR-20260607001 (CLI tool standards, 401 lines) in levonk-base-boilerplate.
