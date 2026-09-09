---
okf_version: "0.2"
---

# Rust Development Practices

A compounding knowledge base documenting practices for Rust package and CLI
development — project structure, tooling, testing, error handling, async
patterns, serialization, container support, and CLI standards. Each concept
captures specific requirements and patterns sourced from real boilerplate ADRs.

## Concepts

* [Overview](overview.md) - Synthesis of the full Rust development practice set
* [Project Structure](project-structure.md) - Standard directory layout, module organization, re-exports
* [Cargo Configuration](cargo-configuration.md) - Required metadata, dependency pinning, feature flags, workspace config
* [Rustfmt and Clippy](rustfmt-clippy-config.md) - Formatting and linting configuration standards
* [Testing Strategy](testing-strategy.md) - Unit, integration, doc tests, benchmarks, property-based testing
* [Error Handling](error-handling.md) - thiserror for structured errors, anyhow for context, no panic in libraries
* [Async Patterns](async-patterns.md) - tokio runtime, async test macro, stream implementation
* [Serde Serialization](serde-serialization.md) - Optional serde integration, multiple format support, skip patterns
* [CLI Tool Standards](cli-tool-standards.md) - Cross-language CLI standards: args, config, output, daemon, agent mode (AXI)
* [Container Support](container-support.md) - Multi-stage Dockerfile, non-root user, healthcheck, docker-compose
* [Security and Auditing](security-auditing.md) - cargo audit, secrecy crate, zeroize, input validation
* [Quality Gates](quality-gates.md) - Pre-commit hooks, CI/CD multi-version testing, cross-platform validation
* [Tree-Sitter AST Queries](tree-sitter-ast-queries.md) - Query syntax for linting, multi-language query organization, performance for large codebases
* [Clap CLI Patterns](clap-cli-patterns.md) - Derive vs builder, subcommand organization, argument validators, shell completion
* [TOML Config Validation](toml-config-validation.md) - serde + toml schema validation, profile merging, config migration, hot-reload
* [Structured Logging with tracing](structured-logging-tracing.md) - tracing vs log, structured field conventions, subscriber config, RUST_LOG
* [File Watcher Patterns](file-watcher-patterns.md) - notify crate event filtering, cross-platform watch limits, FS race conditions, async integration
* [Cross-Platform Path Handling](cross-platform-path-handling.md) - std::path vs camino, normalization, symlinks, Windows UNC paths
* [Anyhow + Thiserror Combination](anyhow-thiserror-combination.md) - Layered error handling: thiserror in libs, anyhow at application layer
