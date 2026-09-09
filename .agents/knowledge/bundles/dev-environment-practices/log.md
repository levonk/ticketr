# Directory Update Log

## 2026-09-02

* **Add**: [devbox-per-platform-packages.md](devbox-per-platform-packages.md) —
  documents the per-platform devbox.json package entry pattern. When a package
  builds on most platforms but fails on one (e.g., x86_64-darwin / Intel Macs),
  use the object form with two entries per broken package: `pkg@latest` with
  `excluded_platforms: ["x86_64-darwin"]` for the working platforms, and
  `github:nixos/nixpkgs/<channel>#pkg` with `platforms: ["x86_64-darwin"]`
  pinned to a channel known to build on the broken platform. This keeps devbox
  functional on ALL platforms without bypassing it or pinning the entire
  nixpkgs to an older revision. Includes a migration guide from array form to
  per-platform object form, a decision table for per-platform pinning vs
  bypass (vs [Devbox Broken Override](devbox-broken-override.md)), and
  prevention guidance. Sourced from the skills-src devbox.json per-platform
  package entries.
* **Update**: Updated [index.md](index.md) with the new concept entry.
* **Update**: Updated [overview.md.tmpl](overview.md.tmpl) — added
  "Cross-Platform" row to the phase/practice table.
* **Update**: Updated [devbox-broken-override.md](devbox-broken-override.md)
  — added cross-reference to per-platform pinning as the preferred solution
  before escalating to the bypass.

## 2026-08-30

* **Add**: [diff-tooling-stack.md](diff-tooling-stack.md) — the
  three-layer diff stack (difftastic as AST-based external diff driver
  via gitattributes, delta as unified-diff pager, context-aware wrapper
  with human vs agent profiles). Documents when external diff drivers
  fire, the git-apply compatibility failure mode (no `@@` hunk markers
  in structural diff output), and the `format-patch` / `--no-ext-diff`
  / `stash show -p` fixes. Sourced from a real worktree-transfer
  failure where `git diff > patch && git apply patch` broke because of
  gitattributes-configured difftastic. Grounded against difftastic
  0.69.0 and the user's live dotfiles configuration on 2026-08-30.
* **Update**: Updated [index.md](index.md) with the new concept entry.
* **Update**: Updated [overview.md.tmpl](overview.md.tmpl) — added
  "Diff Tooling" row to the phase/practice table.

## 2026-08-09

* **Add**: [index-staleness-check.md](index-staleness-check.md) — staleness check inside prime_impl that wraps indexed AST tool invocations (reindex when DB missing or >1h old). References async-prime-internal.md for the trigger; does NOT duplicate it.
* **Update**: Updated [index.md](index.md) with the new concept entry.

## 2026-08-05

* **Ingest**: Authored 4 new Rust dev-environment concept pages sourced from
  the project-lint audit. Includes one TODO promotion (multi-language-devbox).
  All pages grounded against current tool versions on 2026-08-05.
  - [just-recipes-for-rust.md](just-recipes-for-rust.md) — standard justfile
    recipes for Rust: build, test, clippy, fmt, doc, bench, feature flags
  - [devbox-rust-versions.md](devbox-rust-versions.md) — pinning Rust
    toolchain in devbox.json, rustup integration, multi-version management
  - [direnv-rustup-toolchains.md](direnv-rustup-toolchains.md) — direnv +
    rustup integration, automatic toolchain switching, PATH management
  - [multi-language-devbox.md](multi-language-devbox.md) — managing Python +
    Rust + Node in one devbox (promoted from overview TODO)
* **Update**: Updated [index.md](index.md) with 4 new concept entries.
* **Update**: Updated [overview.md](overview.md.tmpl) — promoted
  `multi-language-devbox.md` from "Future concept candidates" TODO to a real
  page; annotated remaining 3 TODOs with deferral rationale.

## 2026-08-02

* **Ingest**: Created [branch-tag-hygiene.md](branch-tag-hygiene.md) —
  documents the practice for archiving stale branches and tags into a
  structured `archive/{branches,tags}/{type}/YYYY/MM/YYYYMMDD-{slug}` namespace.
  Covers the ownership exception (defer to repo conventions when upstream is
  not owned by the primary account), periodic pruning with retention windows,
  and the failure mode of unmanaged ref accumulation. Sourced from real-world
  accumulation in `levonk/dotfiles` (75 branches, 29 tags over 2 months) and
  the `git-repository-management` skill v1.10.0 `git-archive.sh` script. Added
  to [index.md](index.md).

## 2026-07-30

* **Ingest**: Created [devbox-broken-override.md](devbox-broken-override.md) —
  documents the practice for when devbox cannot build the environment at all
  (nixpkgs pin missing a package on a specific platform, e.g., the `jujitsu`
  package missing on Intel Macs). Distinct from the script generation bug
  (which is a v0.14.x regression where `devbox run <script>` fails but
  `devbox run -- <cmd>` works). The override practice: replace
  devbox-wrapped commands with direct package-manager equivalents
  (`pnpm exec vitest run` instead of `devbox run -- just test-internal`), and
  document the override in the project's execution workflow file so subagents
  use the direct form without re-discovering the breakage on every story.
  Sourced from the seodata-execute workflow (commit 2c23734). Added to
  [index.md](index.md) and [overview.md](overview.md.tmpl) table.

## 2026-07-28

* **Update**: Renamed `internal-vs-normal-targets.md` concept to "Auto-Detecting Devbox Targets" — replaced the two-tier `just build` / `just build-internal` / `devbox run build` pattern with a single-target auto-detection pattern using a DRY `_devbox` helper recipe. Implementation targets renamed from `*-internal` to `*_impl` (underscore-prefixed to hide from `just --list`). `doctor` is special-cased to run directly (it's the fallback when devbox is missing).
* **Update**: Updated [standard-developer-ux-flow.md](standard-developer-ux-flow.md) — all three flows now use `just build` (auto-detecting). No more `devbox run -- just build-internal` in Flow 1. Added `_devbox` helper documentation.
* **Update**: Updated [devbox-script-generation-bug.md](devbox-script-generation-bug.md) — added section explaining why auto-detection makes the script generation bug largely irrelevant (`just build` uses `devbox run --` raw form, not `devbox run <script>`).
* **Update**: Updated [async-prime-internal.md](async-prime-internal.md) — renamed `prime-internal` → `prime_impl`, `build-internal` → `build_impl`, `bootstrap-internal` → `bootstrap_impl` throughout. Updated `.envrc` trigger and three-entry-paths section.
* **Update**: Updated [just-over-makefiles.md](just-over-makefiles.md), [mandatory-testing-workflow.md](mandatory-testing-workflow.md), [devbox-over-raw-nix.md](devbox-over-raw-nix.md) — updated code examples to use `*_impl` targets and `just _devbox` pattern.

## 2026-07-26
* **Migration**: Migrated `## Citations` body sections to `sources` frontmatter with stable `id` attributes per OKF v0.2 §13.1.
* **Migration**: Migrated bundle from OKF v0.1 to OKF v0.2 — bumped `okf_version` in index.md. No `# Citations` sections or `timestamp` fields to migrate.

## 2026-07-20

* **Creation**: Authored [async-prime-internal.md](async-prime-internal.md) — documents the two-phase prime pattern: Phase 1 (sync) git checkpoint commit (no push, follows pre-task-commit-checkpoint protocol from git-repository-management skill, skippable via PRIME_SKIP_CHECKPOINT=1); Phase 2 (async, fire-and-forget) cache-warming jobs (package downloads, build, recipe list, API doc generation) in parallel. Verification gates (typecheck/test/validate) stay synchronous and blocking. Includes the `.envrc` async trigger (gated by direnv allow + `DEVBOX_SHELL_ENABLED` check) and the sync/async split rule.
* **Update**: Updated [standard-developer-ux-flow.md](standard-developer-ux-flow.md) — added Prime Flow section documenting the two-phase pattern (sync checkpoint + async warmup) and the rule that verification gates stay synchronous.
* **Update**: Updated [internal-vs-normal-targets.md](internal-vs-normal-targets.md) — clarified `prime`/`prime-internal` as sync checkpoint + async warmup (not just "code indexing") with cross-link to async-prime-internal.md.

## 2026-07-17

* **Initialization**: Created the `dev-environment-practices` knowledge bundle to consolidate developer environment practices from three ADRs in levonk-base-boilerplate.
* **Creation**: Authored 8 concept pages covering the dev environment evolution from Nix flakes to devbox to the standard UX flow.
  - [nix-flake-dev-shells.md](nix-flake-dev-shells.md) — original Nix flake approach (superseded)
  - [devbox-over-raw-nix.md](devbox-over-raw-nix.md) — devbox migration from raw Nix
  - [direnv-auto-activation.md](direnv-auto-activation.md) — automatic environment activation
  - [standard-developer-ux-flow.md](standard-developer-ux-flow.md) — three-flow pattern for agents, novices, power users
  - [just-over-makefiles.md](just-over-makefiles.md) — just as task runner replacement for Make
  - [internal-vs-normal-targets.md](internal-vs-normal-targets.md) — *-internal naming convention
  - [devbox-script-generation-bug.md](devbox-script-generation-bug.md) — known v0.14.x regression and workarounds
  - [mandatory-testing-workflow.md](mandatory-testing-workflow.md) — TDD, regression tests, quality gates
* **Creation**: Established [overview.md](overview.md) synthesis and [index.md](index.md) directory listing.
* **Note**: Concepts extracted from ADR-20251219001 (Nix flake, superseded), ADR-20251226001 (devbox+direnv, accepted), and ADR-20260131001 (standard UX flow, proposed) in levonk-base-boilerplate.
