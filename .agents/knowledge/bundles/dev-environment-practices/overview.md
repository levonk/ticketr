---
type: Synthesis
title: Dev Environment Practices Overview
description: Synthesis of dev environment practices — Nix foundations, devbox migration, direnv auto-activation, just task runner, standard UX flow, script generation bugs, and mandatory testing workflow.
tags: [dev-environment, nix, devbox, direnv, just, developer-experience, overview, synthesis]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: adr-20251219001-nix-direnv-dev-environment
    resource: internal-docs/adr/adr-20251219001-nix-direnv-dev-environment.md
    title: levonk-base-boilerplate
  - id: adr-20251226001-devbox-direnv-dev-environment
    resource: internal-docs/adr/adr-20251226001-devbox-direnv-dev-environment.md
    title: levonk-base-boilerplate
  - id: adr-20260131001-standard-developer-ux-flow
    resource: internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md
    title: levonk-base-boilerplate
---

---
description: STE100-inspired Simplified Technical English guidelines for technical prose output — active voice, short sentences, one-word-one-meaning, imperative for instructions
---

### Simplified Technical English (STE100-Inspired)

This artifact produces technical English (instructions, procedures, descriptions,
reference documentation). Apply these STE100-inspired guidelines to all technical
prose output so the result is unambiguous, translatable, and easy to read for
non-native speakers and for AI agents that must execute the steps precisely.

These are **STE-inspired guidelines**, not the full ASD-STE100 vocabulary
restriction. Domain terms (Dockerfile, pnpm, devbox, Nx, etc.) are permitted
when they are the correct technical term — STE100's 1000-word approved
vocabulary is too narrow for this domain. The goal is the *clarity discipline*
of STE100, not its word list.

For the full writing rules, before/after examples, and the approved-words
reference, see the
[Simplified Technical English](https://github.com/levonk/skills-releases/blob/main/knowledge/simplified-technical-english/simplified-technical-english.md)
and
[Detailed Guide](https://github.com/levonk/skills-releases/blob/main/knowledge/simplified-technical-english/detailed-guide.md)
concept pages in the `simplified-technical-english` knowledge bundle. The
bundle is the canonical, publicly-reachable home for these guidelines; this
include is the build-time gist that gets inlined into skills and other
bundles.

#### Core Principles

1. **One word, one meaning.** Pick one term for each concept and use it
   everywhere. Do not alternate between "image" and "container image" and
   "docker image" for the same thing. Pick one, define it once, reuse it.

2. **Short sentences.** Keep procedural sentences under 20 words. Keep
   descriptive sentences under 25 words. Split long sentences into two.

3. **Active voice.** Write "The build copies the file" not "The file is copied
   by the build." The actor does the action. Passive voice hides who does what
   and is the single largest source of ambiguity in technical prose.

4. **Imperative mood for instructions.** Write "Run the tests" not "You should
   run the tests" or "The tests should be run." Instructions tell the reader
   (or agent) what to do, directly.

5. **One topic per sentence.** One idea per sentence. Do not chain unrelated
   clauses with "and" or "while." If a sentence has two ideas, split it.

6. **Consistent verb forms.** Use the same verb for the same action across the
   document. If you "run" a script in section 1, do not "execute" it in
   section 2. Pick one verb per action and keep it.

7. **Approved modifiers only.** Avoid decorative adjectives and adverbs
   ("very", "extremely", "simply", "just"). Keep modifiers that carry
   information ("non-root", "read-only", "idempotent"). Drop modifiers that
   carry only emphasis.

8. **Define every acronym on first use.** Write "Continuous Integration (CI)"
   on first use, then "CI" thereafter. Never assume the reader knows the
   acronym.

#### What Counts as Technical English

Apply these guidelines to:

- Procedural instructions ("Run `just build`", "Add the user to the group")
- Descriptions of failure modes, symptoms, and practices
- Reference documentation and concept pages
- Checklists and review guidance
- Synthesis and overview prose in knowledge bundles

Do **not** apply these guidelines to:

- Code, commands, and file paths (those have their own syntax)
- Frontmatter and structured data (YAML, JSON)
- Diagrams and their source syntax (Mermaid, PlantUML)
- Business communication, marketing copy, or creative content
- Log entries and change logs (those are append-only records)

#### Quick Self-Check

Before finishing a piece of technical prose, run this checklist:

- [ ] Is every sentence under 25 words? (Procedural: under 20.)
- [ ] Is every sentence active voice? (Or is the passive voice intentional and
      necessary?)
- [ ] Are instructions in imperative mood?
- [ ] Does each technical term have one and only one form in this document?
- [ ] Is every acronym defined on first use?
- [ ] Are decorative modifiers removed?
- [ ] Does each sentence carry one topic?

If any answer is "no," revise before publishing.


# Dev Environment Practices Overview

This bundle documents practices for creating reproducible, low-friction developer
environments. Each concept was extracted from real ADRs and project migrations —
the evolution from raw Nix flakes to devbox, the standardization on just over
Makefiles, the three-flow developer UX pattern, and the mandatory testing
workflow that gates all changes.

## The Dev Environment Evolution

```
nix-flake → devbox → direnv → just → standard-ux-flow → testing-gates
```

Each phase has practices that prevent specific failure modes:

| Phase | Practice | Prevents |
|-------|----------|----------|
| Foundation | [Nix Flake Dev Shells](nix-flake-dev-shells.md) | Missing tools, inconsistent local setups, "works on my machine" |
| Migration | [Devbox Over Raw Nix](devbox-over-raw-nix.md) | Steep Nix learning curve, verbose flake.nix, poor developer UX |
| Activation | [direnv Auto-Activation](direnv-auto-activation.md) | Manual environment sourcing, forgotten activation, stale shells |
| Task Runner | [Just Over Makefiles](just-over-makefiles.md) | .PHONY confusion, file-name collisions, complex variable syntax |
| Workflow | [Standard Developer UX Flow](standard-developer-ux-flow.md) | Inconsistent commands across projects, AI agent environment drift |
| Targets | [Auto-Detecting Devbox Targets](internal-vs-normal-targets.md) | Double-wrapping devbox run, -internal thinking overhead, unclear target responsibilities |
| Warmup | [Async Prime Internal](async-prime-internal.md) | Cold-cache latency on first command, serial warmup steps that could overlap |
| Reliability | [Devbox Script Generation Bug](devbox-script-generation-bug.md) | Silent script failures, "command not found" in CI |
| Fallback | [Devbox Broken Override](devbox-broken-override.md) | Devbox cannot build environment at all (nixpkgs pin missing a package); subagents block instead of using direct package-manager equivalents |
| Cross-Platform | [Per-Platform Devbox Package Entries](devbox-per-platform-packages.md) | Package builds on most platforms but fails on one (e.g., x86_64-darwin); per-platform object entries keep devbox functional everywhere without pinning the entire nixpkgs to an older revision |
| Quality | [Mandatory Testing Workflow](mandatory-testing-workflow.md) | Untested changes, regressions, missing quality gates |
| Scripts | [Shell Scripting Best Practices](shell-scripting-best-practices.md) | Unsafe shell scripts, missing dry-runs, untested scripts, dirty repo state |
| Rust Recipes | [Just Recipes for Rust](just-recipes-for-rust.md) | Inconsistent justfile recipes across Rust projects, missing workspace-aware targets |
| Rust Toolchain | [Devbox Rust Versions](devbox-rust-versions.md) | Unpinned Rust toolchain, drift across developers, missing components |
| Rust direnv | [Direnv Rustup Toolchains](direnv-rustup-toolchains.md) | Manual toolchain switching, stale CARGO_HOME, slow direnv loading |
| Multi-Language | [Multi-Language Devbox](multi-language-devbox.md) | PATH conflicts between Rust/Node/Python, missing tree-sitter grammar support |
| Diff Tooling | [Diff Tooling Stack](diff-tooling-stack.md) | `git apply` failure from gitattributes-configured external diff drivers; losing structural diff by disabling difftastic globally |

## Scope

This bundle covers **developer environment management and workflow patterns** —
the tools, configuration, and processes that ensure every developer and AI agent
works in the same reproducible environment. It does **not** cover:

- Build system orchestration (Nx, Turborepo) — see
  [typescript-monorepo-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/typescript-monorepo-best-practices/overview.md).
- Container build environments — see
  [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md).
- CI/CD pipeline configuration — separate bundle.
- IDE-specific configuration (editor settings, extensions) — separate bundle.

## Relationship to ADRs

The concepts in this bundle were extracted from three ADRs in the
levonk-base-boilerplate repository:

- ADR-20251219001: Nix flake + direnv (superseded)
- ADR-20251226001: Devbox + direnv (accepted, supersedes 20251219001)
- ADR-20260131001: Standard Developer UX Flow (proposed, latest)

The ADRs document the decision-making process; this bundle extracts the
**generalizable practices** that can be applied to any project.

## Sources

- `internal-docs/adr/adr-20251219001-nix-direnv-dev-environment.md` — boilerplate
- `internal-docs/adr/adr-20251226001-devbox-direnv-dev-environment.md` — boilerplate
- `internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md` — boilerplate

## Compounding

New lessons from future environment work — new tool integrations, performance
tuning, cross-platform issues, devbox version regressions — should be filed as
new concept pages. Append to `log.md` when adding.

Future concept candidates (not yet in the bundle):

- `remote-dev-environments.md` — devcontainer, GitHub Codespaces, remote Nix
  — deferred: out of scope for current cycle (no remote dev usage yet)
- `devbox-caching.md` — Nix store caching, binary cache configuration —
  deferred: out of scope for current cycle (caching is adequate on local NVMe)
- `shell-startup-performance.md` — measuring and optimizing direnv activation
  time — deferred: out of scope for current cycle (no measured latency issue)

Promoted from TODO on 2026-08-05:

- `multi-language-devbox.md` — promoted to a real page; see
  [Multi-Language Devbox](multi-language-devbox.md)

The shell scripting practices — strict mode, PATH guards, git cleanliness gates,
dry-run patterns, and shellcheck/shfmt/bats verification — are captured in
[Shell Scripting Best Practices](shell-scripting-best-practices.md), migrated
from the platform shell rules.

## Related Knowledge Bundles

- [typescript-monorepo-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/typescript-monorepo-best-practices/overview.md)
  — Monorepo build orchestration that runs inside the dev environment.
- [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md) — Container
  build environments that complement local dev environments.
- [upstream-contribution-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/upstream-contribution-practices/overview.md)
  — Contribution workflow that depends on consistent local environments.

---

## Content Ordering

This artifact is optimized for machine consumption. Generic framework content
(shared includes, knowledge bundles) appears before the skill-specific body.
This ordering maximizes cross-skill prefix caching: skills that share the same
includes produce identical byte prefixes, so an LLM context cache warmed by one
skill serves all skills that share the same preamble.

This is sub-optimal for human reading — the skill-specific content starts deep
in the file, after the generic preamble. Human readers can jump to the
skill-specific body by searching for the first `# ` heading that follows the
generic sections. Each section is self-contained and documented with its own
heading hierarchy.

