---
type: Synthesis
title: Build System Essentials Overview
description: Synthesis of build system principles — Makefile orchestration, standard target conventions, centralized scripting, and modular build structure across projects.
tags: [build-system, makefile, make, just, build-orchestration, overview, synthesis]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
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


# Build System Essentials Overview

This bundle documents practices for build system orchestration — the
Makefile structure, standard target conventions, and principles that keep
build automation maintainable, readable, and consistent across all projects.
Each concept was extracted from real project guidelines and boilerplate
Makefiles.

## The Build System Stack

```
makefile-orchestration → standard-targets → centralized-scripting → modular-structure
```

Each layer has practices that prevent specific failure modes:

| Layer | Practice | Prevents |
|-------|----------|----------|
| Orchestration | [Makefile Essentials](makefile-essentials.md) | Complex shell scripting in Makefiles, unclear dependency ordering |
| Targets | [Build Target Conventions](build-target-conventions.md) | Inconsistent commands across projects, missing quality targets |
| Scripting | Centralized `/bin` scripts | Logic scattered across Makefiles, duplicated shell code |
| Structure | Modular Makefiles down to module level | Monolithic builds, no incremental or per-module builds |

## Scope

This bundle covers **build system orchestration and Makefile conventions** —
the targets, structure, and scripting patterns that standardize how projects
are built, tested, and deployed. It does **not** cover:

- Dev environment setup — see
  [dev-environment-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/dev-environment-practices/overview.md).
- CI/CD pipeline configuration — see
  [cicd-testing-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/cicd-testing-practices/overview.md).
- Container build environments — see
  [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md).

## Build System Principles

### Make as Orchestrator

The Makefile's role is dependency management and execution ordering — not
complex shell scripting. All non-`.PHONY` executable logic resides in scripts
within a designated `/bin` directory. The Makefile orchestrates these scripts,
managing dependencies and sequencing.

### Standardized Targets

Every Makefile across all projects implements a common set of targets —
`clean`, `test`, `lint`, `format`, `check`, `coverage`, `help`, and more.
This ensures that any developer or AI agent can interact with any project
using the same command vocabulary.

### Documentation-Driven

Every target in a Makefile is documented in the project's `README.md` file
and the auto-generated `help` target. The `help` target extracts descriptions
from `##` comments alongside target definitions.

### Modular Builds

Makefiles exist at every level of the project tree down to the module level.
Calling `make` at the module level operates solely on that module; calling
`make` at the library level operates on the library and all its children;
calling `make` at the application level operates on the app and all same-repo
dependencies.

### Just as a Modern Alternative

While Make remains the standard for many projects, `just` has emerged as a
modern task runner that avoids `.PHONY` confusion, file-name collisions, and
complex variable syntax. The orchestration principles — centralized scripting,
standardized targets, documentation-driven help — apply equally to both.

## Related Knowledge Bundles

- [dev-environment-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/dev-environment-practices/overview.md) — Dev
  environment setup that build systems run inside of.
- [cicd-testing-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/cicd-testing-practices/overview.md) — CI/CD
  integration that invokes build system targets in pipelines.

## Sources

- `src/current/rules/software-dev/platforms/build-sys/makefile-essentials.md` — Makefile guidelines and best practices

## Compounding

New lessons from future build system work — new build tools, cross-platform
build issues, performance optimization — should be filed as new concept pages.
Append to `log.md` when adding.

Future concept candidates (not yet in the bundle):

- `just-task-runner-essentials.md` — Justfile conventions as a Make alternative
- `build-caching-strategies.md` — Incremental builds, sentinel files, cache invalidation
- `cross-platform-make.md` — Handling platform differences in Makefiles

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

