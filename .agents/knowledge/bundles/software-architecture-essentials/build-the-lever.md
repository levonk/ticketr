---
type: Practice
title: Build the Lever
description: For any non-trivial work, build the tool — a codemod, a script, a generator — that does or proves it, rather than doing it by hand. The tool is the artifact a reviewer reruns. Manual work is unverifiable; tool-based work is repeatable. This generalizes the practice of extracting deterministic phases into scripts.
tags: [architecture, tooling, codemods, automation, verifiability, repeatability, levers]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
sources:
  - id: open-pstack-build-the-lever
    resource: "https://github.com/ericlitman/open-pstack"
    title: "open-pstack poteto-mode — Build the Lever principle"
---

# Build the Lever

## The General Rule

For any non-trivial work — a migration, a refactoring pattern applied
across many files, a verification check — **build the tool that does
or proves it, rather than doing it by hand.** The tool is the artifact
a reviewer reruns. Manual work is unverifiable (the reviewer must trust
that you did it correctly); tool-based work is repeatable (the reviewer
runs the tool and sees the same result).

### What Counts as a Lever

A lever is any tool that amplifies a single effort into repeated
results:

- **Codemods** — automated code transformations that apply a pattern
  across many files. The codemod is the lever; the manual edit is not.
- **Scripts** — deterministic checks that verify a property (build
  passes, tests pass, no leaked secrets). The script is the lever; the
  manual check is not.
- **Generators** — code generators that produce boilerplate from a
  spec. The generator is the lever; the hand-written boilerplate is
  not.
- **Lint rules** — structural enforcement of a convention. The lint
  rule is the lever; the text instruction is not.

### The Tool Is the Artifact

When you build a lever, the tool itself is the deliverable — not just
the result it produces. A reviewer reruns the tool to verify the
result. A future developer reruns the tool to apply the same pattern
to new code. The tool compounds in a way that manual work does not.

This is why extracting deterministic phases into scripts is a core
practice: the script is the lever that makes the phase verifiable and
repeatable. A phase done by hand in the conversation is neither.

### When to Build a Lever vs Do It by Hand

Build a lever when:

- **The work is repeated** — the same pattern applies to many files or
  many cases. A tool amortizes the effort.
- **The work must be verifiable** — a reviewer needs to confirm the
  result. A tool produces a repeatable check.
- **The work will recur** — the same pattern will be needed again in
  the future. A tool is reusable; manual work is not.

Do it by hand when:

- **The work is one-off** — it happens once and will never recur. The
  tool would take longer to build than the manual work.
- **The work is trivial** — the manual effort is smaller than the tool
  effort and the result is obvious by inspection.

## Concrete Instances

### Codemod for a Migration

Migrating from one API to another across 100 files: build a codemod
that applies the transformation, run it, and commit the result. The
codemod is the artifact a reviewer reruns to verify the migration is
complete. A manual edit of 100 files is unverifiable.

### Script for a Verification Check

Verifying that no secrets are committed: build a script that scans
staged files for secret patterns. The script is the artifact that runs
in CI and on every commit. A manual review of every diff is
unverifiable.

### Lint Rule for a Convention

Enforcing that all functions have docstrings: build a lint rule that
fails on missing docstrings. The lint rule is the artifact that runs
on every commit. A text instruction in a style guide is unverifiable
(text instructions get skipped).

## See Also

- [Encode Lessons in Structure](https://github.com/levonk/skills-releases/blob/main/knowledge/agent-orchestration-practices/encode-lessons-in-structure.md)
  — the orchestration-oriented companion: when you write the same
  instruction twice, encode it as a lint rule, script, or runtime check
  instead of more text.
- [Config-Driven Tool Design](config-driven-tool-design.md) — levers
  that are configurable rather than hardcoded.
