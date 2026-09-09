---
type: Practice
title: Root-Cause First — No Workarounds for Fixable Problems
description: Treat every failure as a symptom. Diagnose the underlying cause before applying any fix. Workarounds are last resort, never the default. When a workaround is unavoidable, document it and track its removal. When fixing one document, update all dependent documents — never leave the other side in a broken state.
tags: [root-cause, workarounds, maintenance, technical-debt, documentation, engineering-discipline]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Root-Cause First — No Workarounds for Fixable Problems

## Failure Mode

Reaching for a band-aid the moment a failure appears: commenting out a broken
template, copying a file manually to "make things work", adding a guard that
silences an error without understanding it, or updating one document while
leaving a contradictory sibling document in a broken state. Each workaround
compounds: the next person inherits the band-aid plus the original bug plus the
hidden coupling the band-aid introduced.

The related documentation failure: a request explains why a new approach is
better, the agent updates the local document to reflect the new approach, but
leaves the old document still promoting the rejected approach. The codebase
now has two sources of truth that contradict each other. The next reader has
to guess which one is correct.

## Symptoms

- "It works on my machine" because a manual file copy papers over a broken
  template include.
- A template is commented out instead of fixed; the underlying `includeTemplate`
  path is still wrong.
- Two documents disagree about the same standard (one says "use X", the other
  says "use Y") because only one was updated when the team switched.
- A guard clause silences an error without explaining what condition it
  protects against.
- Workarounds stack on top of workarounds; each retry papers over the previous
  failure instead of informing the next attempt.
- A credential is expired, so the agent attempts manual auth loops or state
  copying instead of surfacing the expiry.

## Practice

### Diagnose before fixing

Treat every failure as a symptom, not as the thing to fix.

1. **Reproduce reliably** — get a minimal failing case before touching anything.
2. **Trace the exact failing code path or configuration** — read the code,
   follow the dependency chain, find the line that actually breaks.
3. **Identify the root cause** — the smallest change that would make the
   symptom disappear *and* stay gone.
4. **Fix at the source** — script, config, dependency, or environment contract.
   Prefer durable, maintainable fixes at the origin.

### Workarounds are last resort, never the default

A workaround is acceptable only when **all** of these are true:

- The upstream/root fix is infeasible immediately.
- The workaround is safe, minimal, and scoped.
- The workaround is documented with rationale, scope, and a removal plan.
- The workaround is tracked for removal.

If a workaround is used, it must be recorded in:

- An issue link `internal-docs/issues/....md` in GitHub issue format.
- A checklist item in `internal-docs/todo/workarounds.md`.
- A `## TODO: ...` in-code comment summarizing the workaround and the
  contingency to remove it.

### Fix all dependent documents, not just the local one

When a request explains that a new approach is better than an existing one,
updating only the local document is a band-aid of the same form. The rejected
approach is still promoted elsewhere — the codebase now contradicts itself.

When changing a standard:

1. **Update the canonical source** (the include, the ADR, the tech-stack
   table).
2. **Update every document that restates the old standard** — search for the
   old phrasing, the old tool name, the old pattern. If the standard was
   included via a shared include, the include update propagates; if documents
   restated it inline, each one must be edited.
3. **Remove the old standard from places that contradict the new one** — do
   not leave both. A reader who finds the contradiction has to guess which is
   correct.
4. **Verify with search** — `rg "<old phrasing>"` should return zero hits in
   authoritative content after the change.

If two documents genuinely conflict and the conflict is unresolved (no clear
winner), document the conflict explicitly in both places with a link to the
issue that will resolve it. **Ambiguity is not a license to leave one side
broken.**

### Failing early beats working around

- If a deployment fails, investigate the actual cause before retrying or
  trying alternatives.
- If a credential is expired, say so and tell the user where to update it —
  do not attempt manual authentication loops, state copying, or other
  band-aids.
- If a container keeps restarting, find out why (restart count, logs, exit
  codes) before redeploying.
- If an existing resource conflicts with a new one (e.g., a node already
  exists in Tailscale), stop and surface the conflict — do not proceed with a
  renamed variant without permission.

## Anti-Patterns

- ❌ Manual file copying as a workaround for a broken template include.
- ❌ Commenting out a template instead of fixing the path.
- ❌ Applying partial fixes that don't address the root cause.
- ❌ Ignoring template errors to "make things work".
- ❌ Updating one document to a new standard while leaving a sibling document
  promoting the rejected standard.
- ❌ Chaining workaround on top of workaround — each failed attempt should
  inform the next, not paper over the previous failure.
- ❌ Silencing an error with a guard clause without documenting what condition
  it protects against.

## Required Approach

- ✅ Complete audit of all dependencies in the failing chain.
- ✅ Fix every broken reference, not just the one that surfaced.
- ✅ Ensure all source files exist and are accessible.
- ✅ Verify the system works end-to-end after the fix.
- ✅ When changing a standard, update every document that restates the old
  standard — search for the old phrasing and remove it.

## Related Concepts

- [Architecture Philosophy](philosophy.md) — Modular architecture is the
  structural complement: clear ownership makes root causes easier to locate.
- [Adding New Tools](adding-tools.md) — The procedure for wiring new tools in
  cleanly, rather than bolting them on as workarounds.
- [Tool Detection Architecture](tool-detection.md) — Guard clauses done
  right: detect missing tools with clear errors, not silent fallbacks.

## Sources

- `dotfiles/.devin/rules/testing.md` — Root-Cause First Policy section
  (originally authored for the dotfiles project's testing guide).
- `infrahub/shared/active/02-config/ansible/AGENTS.md` — "Root Cause First -
  No Workarounds" section (Ansible deployment context).
- `dotfiles/AGENTS.md` — "CRITICAL: Root Cause Analysis Required" section
  (chezmoi template failure context).
