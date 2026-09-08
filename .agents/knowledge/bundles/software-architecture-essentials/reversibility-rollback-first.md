---
type: Practice
title: Reversibility + Rollback-First Thinking
description: Keep changes easy to revert — small scope, clear blast radius. For risky changes, define the rollback path before merging. Avoid mixed mega-patches that block safe rollback. Reversibility is a design property: every change should be evaluable for "can I undo this?" before it is applied.
tags: [architecture, reversibility, rollback, blast-radius, change-management, safety, deployment]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Reversibility + Rollback-First Thinking

## The General Rule

Every change should be reversible. Before applying a change, ask: "If this
goes wrong, how do I undo it?" If the answer is "I cannot," the change is too
risky — restructure it until the answer is "I revert this one commit / roll
back this one deployment / delete this one feature flag."

- **Keep changes easy to revert: small scope, clear blast radius.** A change
  that touches one module is easy to revert. A change that touches ten
  modules in one commit is hard to revert — you must untangle the parts that
  worked from the parts that did not.
- **For risky changes, define the rollback path before merging.** The
  rollback path is a concrete set of steps (revert commit, run migration
  down, toggle flag, re-deploy previous image). If you cannot write the
  rollback steps before merging, you cannot execute them under pressure
  after merging.
- **Avoid mixed mega-patches that block safe rollback.** A PR that combines
  a refactor, a feature, and a config change cannot be reverted by reverting
  the PR — the refactor may be fine, the feature may be broken, and the
  config change may be required by the refactor. Split them into separate
  PRs so each can be reverted independently.

## Why It Matters

The probability that a change introduces a bug is never zero. The cost of a
bug is the product of its probability and its blast radius. Reversibility
shrinks the blast radius: a reversible change can be undone in minutes; an
irreversible change requires a forward fix, which takes longer and carries
its own risk.

This is not just a deployment concern. It applies to every change: a schema
migration, a config update, a refactor, a dependency upgrade. Each should be
evaluated for reversibility before it is applied.

## Reversibility At Each Layer

| Layer | Reversible change | Irreversible change |
|-------|-------------------|---------------------|
| **Code** | One commit per concern; revert the commit | Mega-patch mixing refactor + feature + config |
| **Database** | Additive migration (add column, add table); `IF NOT EXISTS` | Destructive migration (drop column, rename table) without a down path |
| **Config** | New key with a default; old key still works | Renaming a key without a compatibility period |
| **Deployment** | Blue-green or canary; roll back to previous image | Force-push a migration that mutates existing rows |
| **API** | Add a field; old clients still work | Remove a field or change a type without a deprecation period |

The pattern: **additive changes are reversible; destructive changes are
not.** Prefer additive. When destructive is necessary, stage it: add the new
thing, migrate callers to the new thing, then remove the old thing in a
separate change.

## The Rollback-First Checklist

Before merging a risky change, answer:

1. **What is the blast radius?** Which modules, services, users, or data are
   affected?
2. **What is the rollback path?** The exact steps to undo the change.
3. **How long does rollback take?** Minutes (revert + redeploy) or hours
   (manual data repair)?
4. **Is the rollback path tested?** Have you actually run it, or is it a
   theory?
5. **Can the change be split?** If the rollback path is complex, the change
   is probably a mega-patch — split it.

If any answer is "I don't know," the change is not ready to merge.

## Anti-Patterns

- **The mega-patch.** A single PR that refactors the data layer, adds a
  feature, and changes the config format. If the feature has a bug, you
  cannot revert the PR without losing the refactor and the config change.
- **The destructive migration without a down path.** `ALTER TABLE users
  DROP COLUMN legacy_field` in a deployment. If the deployment has a bug
  that depends on `legacy_field`, the column is gone — you must re-add it
  and backfill, which may be impossible if the data was not backed up.
- **The untested rollback path.** "We can revert the commit" — but the
  commit includes a migration that added a column, and the revert does not
  drop the column. The rollback is a theory, not a tested procedure.
- **The flagless deployment.** A direct deployment with no feature flag, no
  canary, no blue-green. If the deployment is broken, the only rollback is
  to re-deploy the previous image — which requires the previous image to be
  available and the deployment process to support it.

## Concrete Instances

- **Archon (Bun + TypeScript).** The Engineering Principles state: "Keep
  changes easy to revert: small scope, clear blast radius; for risky
  changes, define the rollback path before merging; avoid mixed mega-patches
  that block safe rollback." The PR template includes a dedicated "Rollback
  Plan" section that must be filled before merge. The database migration is
  idempotent and additive (`CREATE TABLE IF NOT EXISTS`, `ADD COLUMN IF NOT
  EXISTS`) so re-running it never destroys data — see
  [Idempotent Database Migration](idempotent-database-migration.md).
- **Stripe API versioning.** Stripe never removes or renames a field in a
  given API version. Changes are additive (new fields, new endpoints). Old
  versions continue to work. A client that breaks on a new version can pin
  to the old version — the rollback path is the API version header. This is
  reversibility at the API design level.
- **Kubernetes rolling updates.** The deployment controller keeps the
  previous ReplicaSet alive for `revisionHistoryLimit` revisions. A
  `kubectl rollout undo` reverts to the previous revision in seconds. The
  rollback path is built into the platform — it is not a manual procedure.

## See Also

- [Idempotent Database Migration](idempotent-database-migration.md) —
  additive, idempotent migrations are the database-layer expression of
  reversibility.
- [Fail Fast + Explicit Errors](fail-fast-explicit-errors.md) — fail early
  so the rollback path is triggered before the damage spreads.
- [Process-Boundary State Ownership](process-boundary-state-ownership.md) —
  do not autonomously mutate state you do not own; the rollback for an
  autonomous mutation is unknowable.
- [Root-Cause First](root-cause-first.md) — a rollback is not a fix; it is a
  safety net. After rolling back, find the root cause.
