---
type: Synthesis
title: Rust Development Practices Overview
description: Synthesis of Rust development practices — project structure, Cargo config, formatting/linting, testing, error handling, async, serialization, CLI standards, container support, security, and quality gates.
tags: [rust, development, cargo, cli, overview, synthesis]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md"
    title: "levonk-base-boilerplate"
  - id: levonk-base-boilerplate-adr-20260607001-cli-tool-standards
    resource: "internal-docs/adr/adr-20260607001-cli-tool-standards.md"
    title: "levonk-base-boilerplate"
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


# Rust Development Practices Overview

This bundle documents practices for creating production-ready Rust packages and
CLI tools. Each concept was extracted from real boilerplate requirements and CLI
standards ADRs — the rules that ensure Rust packages are well-structured,
properly tested, secure, and maintainable across the monorepo ecosystem.

## The Rust Development Lifecycle

```
project-structure → cargo-config → formatting → testing → error-handling → async → serde
                                                                              ↓
                            cli-standards ← container ← security ← quality-gates
```

| Phase | Practice | Prevents |
|-------|----------|----------|
| Structure | [Project Structure](project-structure.md) | Inconsistent layouts, missing modules, poor public API |
| Manifest | [Cargo Configuration](cargo-configuration.md) | Unpinned deps, missing metadata, unguarded features |
| Formatting | [Rustfmt and Clippy](rustfmt-clippy-config.md) | Style drift, complexity creep, linting gaps |
| Testing | [Testing Strategy](testing-strategy.md) | Untested code, missing benchmarks, no property tests |
| Errors | [Error Handling](error-handling.md) | Panics in libraries, unstructured errors, poor context |
| Async | [Async Patterns](async-patterns.md) | Wrong runtime, blocking in async, missing async tests |
| Serialization | [Serde Serialization](serde-serialization.md) | Missing serde derives, over-serialized optional fields |
| CLI | [CLI Tool Standards](cli-tool-standards.md) | Inconsistent UX, no agent mode, missing daemon support, no skill emission, no agent-hook wiring, unsafe telemetry |
| Container | [Container Support](container-support.md) | Bloated images, root containers, missing healthchecks |
| Security | [Security and Auditing](security-auditing.md) | Vulnerable deps, leaked secrets, unsafe FFI |
| Quality | [Quality Gates](quality-gates.md) | Unformatted commits, single-version testing, no CI audit |
| AST | [Tree-Sitter AST Queries](tree-sitter-ast-queries.md) | Brittle pattern matching, no multi-language query reuse, slow large-codebase scans |
| CLI | [Clap CLI Patterns](clap-cli-patterns.md) | Inconsistent argument parsing, missing shell completion, poor help text |
| Config | [TOML Config Validation](toml-config-validation.md) | Invalid configs accepted silently, no profile merging, no config migration |
| Logging | [Structured Logging with tracing](structured-logging-tracing.md) | Unstructured logs, no severity filtering, missing IDE hook event context |
| Watching | [File Watcher Patterns](file-watcher-patterns.md) | Watch limit exhaustion, FS race conditions, missed events on slow disks |
| Paths | [Cross-Platform Path Handling](cross-platform-path-handling.md) | Path display corruption, symlink loops, Windows UNC path failures |
| Errors | [Anyhow + Thiserror Combination](anyhow-thiserror-combination.md) | Library errors leaking into application context, untestable error variants |

## Scope

This bundle covers **Rust package and CLI development** — structure, tooling,
testing, security, and deployment. It does **not** cover:

- Devbox/Nix environment setup — see
  [dev-environment-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/dev-environment-practices/overview.md).
- Container runtime hardening — see
  [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md).
- Monorepo build orchestration — see
  [typescript-monorepo-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/typescript-monorepo-best-practices/overview.md).

## Sources

- `internal-docs/adr/adr-20260128001-rust-package-boilerplate-requirements.md` — boilerplate (756 lines)
- `internal-docs/adr/adr-20260607001-cli-tool-standards.md` — boilerplate (v5.0.0, 2026-07-29; 437 lines)

## Related Knowledge Bundles

- [dev-environment-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/dev-environment-practices/overview.md) —
  Environment management for Rust projects
- [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md) — Container
  patterns for Rust binaries
- [devsecops-codeguard](https://github.com/levonk/skills-releases/blob/main/knowledge/devsecops-codeguard/overview.md) — Security practices
  for Rust code

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

