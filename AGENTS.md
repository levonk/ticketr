
# tkr

## Project Snapshot
- Type: Single-project repo (Rust CLI application)
- Category: Rust CLI ticket management system

## Project Overview

**tkr** is a modern Rust CLI ticket management system for developers who want a lightweight, git-friendly ticket system that integrates seamlessly with their workflow. It's a complete rewrite of the original `tk` bash script in Rust, providing type safety, better performance, and enhanced features. Tickets are stored as markdown files with YAML frontmatter, with support for dependency tracking, mono-repo organization, and multiple interfaces (CLI, TUI, Web).

## Install

Install tkr directly from GitHub using devbox:

```bash
# Add to existing devbox environment
devbox add github:levonk/tkr

# Or create new project with tkr
mkdir my-project && cd my-project
devbox init
devbox add github:levonk/tkr
devbox shell

# Verify installation
tkr --help
```

## JIT Index
- Developer Guide: [`.agents/knowledge/developer.md`](.agents/knowledge/developer.md) - Setup, build commands, environment tooling, workflows, repo structure, code style, boundaries, and PR checklist for developers working on this project

## Knowledge Bundles

Canonical practice bundles from [levonk/skills-releases](https://github.com/levonk/skills-releases/knowledge) are copied into [`.agents/knowledge/bundles/`](.agents/knowledge/bundles/) for offline agent access. Domain-specific bundles are URL-referenced and fetched on demand only when the task touches that domain.

### Installed (offline-ready)

| Bundle | Applies when |
|--------|--------------|
| [software-architecture-essentials](.agents/knowledge/bundles/software-architecture-essentials/) | Universal — any project |
| [dev-environment-practices](.agents/knowledge/bundles/dev-environment-practices/) | Universal — any project |
| [devsecops-codeguard](.agents/knowledge/bundles/devsecops-codeguard/) | Universal — any project |
| [cicd-testing-practices](.agents/knowledge/bundles/cicd-testing-practices/) | Universal — any project |
| [build-system-essentials](.agents/knowledge/bundles/build-system-essentials/) | Universal — any project |
| [rust-development-practices](.agents/knowledge/bundles/rust-development-practices/) | Rust detected |

### URL-referenced (fetched on demand)

| Bundle | Read when working on… |
|--------|-----------------------|
| [api-auth-payment-practices](https://github.com/levonk/skills-releases/knowledge/api-auth-payment-practices) | Auth, payments, billing |
| [infrastructure-networking-practices](https://github.com/levonk/skills-releases/knowledge/infrastructure-networking-practices) | Network topology, VPN, DNS |
| [secrets-egress-security](https://github.com/levonk/skills-releases/knowledge/secrets-egress-security) | Secrets management, egress firewalls |
| [cloud-provider-essentials](https://github.com/levonk/skills-releases/knowledge/cloud-provider-essentials) | AWS / Azure / GCP / OCI |
| [web-resource-catalog](https://github.com/levonk/skills-releases/knowledge/web-resource-catalog) | UI components, stock media, color palettes |
| [upstream-contribution-practices](https://github.com/levonk/skills-releases/knowledge/upstream-contribution-practices) | Contributing to upstream OSS |
| [ai-primitives](https://github.com/levonk/skills-releases/knowledge/ai-primitives) | AI tooling, prompt engineering |

**Installation**: Run `uv run --script scripts/install-knowledge-bundles.py <project-root>` (from the `project-adopter` skill) to populate `.agents/knowledge/bundles/`. Universal bundles install by default; pass `--bundles <name1>,<name2>` for stack-matched bundles.

## Out of Scope
For information about what this repo does NOT do, see [`internal-docs/oos/`](internal-docs/oos/).

## Improvements
For potential improvements to architecture, standards, and processes, see [`internal-docs/improvements/INDEX.md`](internal-docs/improvements/INDEX.md). These are suggestions to consider — not decisions yet. Check before proposing changes to avoid re-proposing already-evaluated improvements.

## Anti-Patterns
For things explicitly NOT to do (practices found harmful or inferior), see [`internal-docs/anti-patterns/INDEX.md`](internal-docs/anti-patterns/INDEX.md). These are negative findings — do NOT implement any approach listed there.

## Universal Contracts
- License: MIT — see `Cargo.toml` for the full license declaration
- Binary name is `tkr` (not `tk`) to avoid conflicts with the original bash script while maintaining recognizability; both tools can coexist during migration
- Ticket files are stored as markdown with YAML frontmatter — backward compatible with the original `tk` script format

## Agent Interaction Protocol

---
description: Shared ask-user protocol — anytime the AI has a question for the user, present the question, a recommendation, and the reasoning. Lightweight default for general project work; clarifying-questions.md escalates from this base for artifact generation.
---

### Ask the User (Question + Recommendation + Why)

Anytime you have a question for the user — mid-task, at a decision point, or
when ambiguity blocks progress — present it as **question + recommendation +
why**, in that order. Do not ask a bare question and wait. The user should be
able to reply with a single letter, a "yes/no", or "go ahead" without typing
out the reasoning himself.

#### Required Format

For each question, present:

1. **The question** — one sentence, plain language. Number it if there is
   more than one.
2. **Recommendation** — the option you would pick, labeled `(recommended)`.
   If you genuinely don't have a recommendation, say so and explain why
   (e.g. "no recommendation — both options are reasonable for your use
   case, depends on X").
3. **Why** — one or two sentences on the trade-off. Name what breaks, what
   is gained, or what is lost if the user picks the other option.

#### Example (single question)

```text
Q1. Should I add the new helper to the existing `utils.ts` or create a
    separate `helpers/` directory?

    Recommendation: B (separate `helpers/` directory) — recommended
    Why: `utils.ts` is already 600 lines and growing. Splitting now keeps
    each file under the 500-line guideline and makes the new helpers
    discoverable. The cost is one extra import path.
```

#### Example (multiple questions)

```text
Q1. Which auth flow should I implement first?
    A. Email + password (recommended) — fastest to ship, covers the
       happy path; can layer OAuth on top later.
    B. OAuth-only — better security posture upfront, but blocks the
       demo for users without a Google/GitHub account.

Q2. Should the audit log live in the same DB as the app data?
    A. Same DB (recommended) — simpler transactions, one connection
       pool; acceptable until write volume forces a split.
    B. Separate DB — cleaner isolation, but adds a second connection
       pool and a cross-store consistency problem.
```

#### When to Escalate

This is the **base** protocol — use it for ordinary mid-task decisions. For
high-stakes trade-offs (architecture, data model, destructive actions,
one-way doors) or before generating/updating an artifact, escalate to the
full **clarifying-questions** protocol (8-area gap analysis + Decision Brief
format) — see `clarifying-questions.md`.

#### When NOT to Ask

- The answer is already clear from the prompt, the codebase, or prior
  context — proceed and state your assumption.
- The decision is reversible and low-stakes — pick the default, note it,
  and move on. Only ask if the user would want to be consulted.
- You have already asked and the user answered — do not re-ask the same
  question.


This protocol applies to every interaction in this repo — mid-task decisions,
clarifications, and escalation points. For high-stakes trade-offs (architecture,
data model, destructive actions) or before generating/updating an artifact,
escalate to the full clarifying-questions protocol.

## Developer Guide
For workflows, repository structure, code style, boundaries, known gotchas, and the Definition of Done checklist, see [`.agents/knowledge/developer.md`](.agents/knowledge/developer.md).
