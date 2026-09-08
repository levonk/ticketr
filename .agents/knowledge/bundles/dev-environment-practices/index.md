---
okf_version: "0.2"
---

# Dev Environment Practices

A compounding knowledge base documenting practices for reproducible developer
environments — the toolchain, workflow patterns, and configuration that make
"works on my machine" a thing of the past. Each concept captures a specific
failure mode and the practice that prevents it, sourced from real ADRs and
project migrations.

## Concepts

* [Overview](overview.md) - Synthesis of the full dev environment practice set and how the pieces fit together
* [Nix Flake Dev Shells](nix-flake-dev-shells.md) - Per-project reproducible tooling via flake.nix; superseded by devbox but documents the foundation
* [Devbox Over Raw Nix](devbox-over-raw-nix.md) - Why devbox.json replaces flake.nix for developer UX; simpler config, familiar CLI, Nix under the hood
* [direnv Auto-Activation](direnv-auto-activation.md) - Automatic environment loading on cd; watch_file for config changes; use_devbox pattern
* [Standard Developer UX Flow](standard-developer-ux-flow.md) - direnv → devbox → just (auto-detecting) → [build tool]; three flows for agents, novices, and power users
* [Just Over Makefiles](just-over-makefiles.md) - No .PHONY, simple syntax, better errors, command-runner focus; why just replaced Make
* [Auto-Detecting Devbox Targets](internal-vs-normal-targets.md) - Single-target auto-detection via DEVBOX_SHELL_ENABLED; _devbox helper + _impl targets; no more -internal thinking overhead
* [Async Prime Internal](async-prime-internal.md) - prime_impl kicks off cache-warming jobs (downloads, build, list, API docs) in parallel as fire-and-forget; verification gates stay synchronous
* [Index Staleness Check](index-staleness-check.md) - Staleness check inside prime_impl that wraps indexed AST tool invocations; reindexes when the index DB is missing or >1h old. The async .envrc trigger is owned by async-prime-internal.md.
* [Devbox Script Generation Bug](devbox-script-generation-bug.md) - Known v0.14.x regression; auto-detection pattern makes it largely irrelevant; workarounds still documented
* [Devbox Broken Override](devbox-broken-override.md) - When devbox cannot build the environment at all (nixpkgs pin missing a package on a platform), override devbox-wrapped commands with direct package-manager equivalents rather than blocking
* [Per-Platform Devbox Package Entries](devbox-per-platform-packages.md) - When a package builds on most platforms but fails on one (e.g., x86_64-darwin), use per-platform object entries in devbox.json — @latest with excluded_platforms for working platforms, pinned github:nixos/nixpkgs/<channel>#pkg with platforms for the broken one
* [Mandatory Testing Workflow](mandatory-testing-workflow.md) - TDD, regression tests for bug fixes, quality gates before completion; enforced via pre-commit and CI
* [Shell Scripting Best Practices](shell-scripting-best-practices.md) - Strict mode, PATH guards, git gates, dry-runs, logging, and shellcheck/shfmt/bats verification for safe shell scripts
* [Branch & Tag Hygiene](branch-tag-hygiene.md) - Archive stale branches and tags into a structured namespace; ownership exception for upstream repos; periodic pruning with retention windows
* [Just Recipes for Rust](just-recipes-for-rust.md) - Standard justfile recipes for Rust: build, test, clippy, fmt, doc, bench, feature flags, cross-compilation, workspace-aware
* [Devbox Rust Versions](devbox-rust-versions.md) - Pinning Rust toolchain in devbox.json, rustup integration, multi-version management, component selection
* [Direnv Rustup Toolchains](direnv-rustup-toolchains.md) - direnv integration with rustup, automatic toolchain switching, PATH management, loading speed
* [Multi-Language Devbox](multi-language-devbox.md) - Managing Python + Rust + Node in one devbox, PATH ordering, tree-sitter grammars, disk space
* [Diff Tooling Stack](diff-tooling-stack.md) - difftastic (AST diff engine via gitattributes) + delta (pager) + context-aware wrapper; agent vs human modes; git-apply compatibility failure and the --no-ext-diff / format-patch fix
