---
type: Practice
title: Index Staleness Check
description: Staleness check inside prime_impl that wraps indexed AST tool invocations (CodeGraph/Graphify/GitNexus). Reindexes when the index DB is missing or >1h old. The async .envrc trigger that kicks off prime_impl is owned by async-prime-internal.md — this page covers only the check.
tags: [developer-experience, just, indexing, indexed-ast-tools, staleness, codegraph, graphify, gitnexus]
date:
  created: "2026-08-09"
  knowledge-basis: "2026-08-09"
  last-used: "2026-08-09"
sources:
  - id: levonk-base-boilerplate
    resource: internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md
    title: levonk-base-boilerplate
---

# Index Staleness Check

## Failure Mode

Indexed AST tools (CodeGraph/Graphify/GitNexus) produce incorrect or incomplete
results when their index DB is stale — after big code changes, branch switches,
or pulls. Developers forget to re-run `just prime` after pulling changes. Stale
indexes erode trust in the tooling: the agent queries the index, gets outdated
edges, and makes decisions on wrong assumptions.

## Practice

The staleness check lives INSIDE `prime_impl`, wrapping the indexed AST tool
invocation. The check is: is the index DB missing OR older than 1 hour? If
either, reindex.

```just
# Inside prime_impl (added by dev-env-upsert add-prime-steps)
INDEX_DB=".codegraph/codegraph.db"
if [ ! -f "$INDEX_DB" ] || [ $(find "$INDEX_DB" -mmin +60 2>/dev/null | wc -l) -gt 0 ]; then
    codegraph index .
fi
```

## Why inside prime_impl

All three paths that trigger prime go through `prime_impl` (manual `just prime`,
bootstrap, `.envrc` async trigger — see `async-prime-internal.md`). Putting the
check inside `prime_impl` means every path gets it for free. No duplicated
logic. If the check lived in `.envrc`, it would only apply to the `.envrc`
path, not to manual `just prime` or bootstrap.

## Relationship to async-prime-internal.md

`async-prime-internal.md` owns the async `.envrc` trigger that kicks off
`prime_impl` on directory entry (gated by `direnv allow` +
`DEVBOX_SHELL_ENABLED`). This page owns the staleness check that runs INSIDE
`prime_impl` for index freshness specifically. The two are complementary — do
NOT duplicate the `.envrc` trigger block here; reference
`async-prime-internal.md`.

## Why 1 hour

1 hour balances freshness against CPU cost. Reindexing on every directory entry
(the `.envrc` trigger frequency) would burn CPU for no benefit on unchanged
code. 1 hour catches "I pulled changes an hour ago and forgot to reindex" while
not re-indexing on every `cd`. The threshold is configurable per-project —
change `+60` to `+30` for faster-moving projects, `+240` for slow-moving ones.

## Integration with dev-env-upsert

The `dev-env-upsert` skill's `add-prime-steps` (or `setup`) command adds the
staleness check block to `prime_impl` idempotently. The `update-envrc`
operation appends the async trigger block to `.envrc` idempotently (the trigger
block itself is owned by `async-prime-internal.md`).

## Related Concepts

- [Async Prime Internal](async-prime-internal.md) — the async trigger that kicks off prime_impl on directory entry
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — the `prime` target
- [Auto-Detecting Devbox Targets](internal-vs-normal-targets.md) — the `*_impl` convention

## Citations

- ADR-20260131001 — `internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md`
  (levonk-base-boilerplate) as the source of the `prime` target pattern. This is
  a citation (attribution), not a runtime reference — the concept page stands on
  its own.
