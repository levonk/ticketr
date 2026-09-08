---
type: Practice
title: Idempotent Database Migration
description: Ship schema changes as idempotent, additive migrations — CREATE TABLE IF NOT EXISTS, ADD COLUMN IF NOT EXISTS — wrapped in an advisory lock so concurrent startups do not race. The migration converges the database to the desired state on every run without destroying existing data. This is the database-layer expression of reversibility: additive migrations are always safe to re-run and always safe to roll back (by deploying the previous version).
tags: [architecture, database, migration, idempotent, advisory-lock, additive, reversibility, schema, postgresql, sqlite]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Idempotent Database Migration

## The General Rule

A database migration should be **idempotent** — running it multiple times
produces the same result as running it once — and **additive** — it adds
tables, columns, or indexes without dropping or renaming existing ones. The
migration converges the database to the desired schema state on every run,
whether the database is empty, partially migrated, or already up to date.

- **Use `IF NOT EXISTS` on every `CREATE` and `ADD`.** `CREATE TABLE IF NOT
  EXISTS` and `ADD COLUMN IF NOT EXISTS` are no-ops if the object already
  exists. The migration can be re-run safely on a database that is already
  migrated.
- **Wrap the migration in an advisory lock.** When multiple processes start
  simultaneously (e.g., a server scaling up), each may attempt to run the
  migration. An advisory lock ensures only one process runs the migration
  while the others wait.
- **Prefer additive over destructive.** Adding a column is reversible (drop
  the column). Dropping a column is not (the data is gone). Rename in three
  steps: add the new column, migrate data, drop the old column — each step
  is a separate migration.
- **The migration is the schema source of truth.** The combined migration
  file represents the final desired state. Individual numbered migrations
  are the history; the combined file is what runs on a fresh database.

## Why It Matters

A migration that is not idempotent cannot be safely re-run. If the migration
crashes halfway through (network blip, OOM, restart), the database is in a
partial state and re-running the migration fails on the steps that already
completed. The operator must manually inspect the database, determine which
steps completed, and construct a custom recovery — under pressure, at 3am.

An idempotent migration does not have this problem. Re-running it is always
safe: the steps that completed are no-ops, and the steps that did not
complete run to completion. The operator re-runs the migration and the
database converges.

This is the database-layer expression of
[Reversibility + Rollback-First](reversibility-rollback-first.md): an
additive, idempotent migration is always safe to roll back (deploy the
previous version — the added columns are harmless) and always safe to
re-apply (the `IF NOT EXISTS` guards make it a no-op on an already-migrated
database).

## The Combined Migration Pattern

For systems that ship frequent schema changes, maintain two artifacts:

1. **Numbered migrations** (`001_initial.sql`, `002_add_column.sql`, ...).
   These are the history — each one represents a single change. They are
   applied in order on a fresh database.
2. **A combined migration** (`000_combined.sql`). This is the final state —
  the schema as it exists after all numbered migrations are applied. It is
  idempotent and runs on every startup.

The combined migration is regenerated whenever a new numbered migration is
added. On startup, the system runs the combined migration inside an
advisory-lock transaction. If the database is empty, it creates the full
schema. If the database is partially migrated, it adds the missing pieces.
If the database is up to date, every statement is a no-op.

This eliminates the need for a migration tracking table — the schema itself
is the state, and `IF NOT EXISTS` makes the convergence safe.

## The Advisory Lock

When multiple processes start simultaneously, each may attempt to run the
migration. Without a lock, two processes can race: process A creates a
table, process B's `CREATE TABLE` fails because the table now exists (even
with `IF NOT EXISTS`, some operations are not idempotent under concurrency).

The advisory lock ensures only one process runs the migration:

```sql
BEGIN;
SELECT pg_advisory_xact_lock(<hash>);
-- ... migration statements ...
COMMIT;
```

The lock is held for the duration of the transaction. Other processes block
on `pg_advisory_xact_lock` until the first process commits. When they
acquire the lock, the migration statements are no-ops (the schema is already
up to date), so they commit immediately.

For SQLite, the `PRAGMA locking_mode = EXCLUSIVE` or a file-level lock
serves the same purpose — SQLite serializes writers, so the migration is
naturally single-writer.

## Additive vs Destructive

| Operation | Additive? | Reversible? | Safe to re-run? |
|-----------|-----------|-------------|-----------------|
| `CREATE TABLE IF NOT EXISTS` | Yes | Yes (drop table) | Yes |
| `ADD COLUMN IF NOT EXISTS` | Yes | Yes (drop column) | Yes |
| `CREATE INDEX IF NOT EXISTS` | Yes | Yes (drop index) | Yes |
| `DROP TABLE` | No | No | No |
| `DROP COLUMN` | No | No | No |
| `RENAME TABLE` | No | No | No |
| `ALTER COLUMN TYPE` | No | No | No |

The rule: **if the operation removes or transforms existing data, it is
destructive.** Destructive operations must be staged as a sequence of
additive migrations with a data-migration step in between:

1. **Add** the new column/table (additive, idempotent).
2. **Migrate** data from the old to the new (a separate, one-time script).
3. **Switch** reads and writes to the new column/table (code change).
4. **Drop** the old column/table (destructive, in a later migration, after
   the code change is confirmed safe).

Each step is independently deployable and independently reversible (except
step 4, which is only run after step 3 is confirmed).

## Anti-Patterns

- **The non-idempotent migration.** `CREATE TABLE remote_agent_codebases`
  without `IF NOT EXISTS`. Re-running the migration fails because the table
  exists. The operator must manually drop the table (losing data) or skip
  the migration (leaving the schema in an unknown state).
- **The destructive migration without a down path.** `ALTER TABLE users
  DROP COLUMN legacy_field` in a single migration. If the deployment has a
  bug that depends on `legacy_field`, the column is gone and the data is
  lost. Stage it: add the new field, migrate data, switch code, then drop.
- **The migration without a lock.** Two server processes start
  simultaneously, both run the migration, and one fails on a race condition.
  The failed process crashes on startup; the operator must restart it.
- **The migration tracking table that drifts from the schema.** A
  `schema_migrations` table says "migration 010 is applied" but the
  database is missing a column that migration 010 was supposed to add
  (because migration 010 crashed halfway). The tracking table lies. The
  combined-migration pattern avoids this: the schema is the state, and
  `IF NOT EXISTS` makes convergence self-healing.

## Concrete Instances

- **Archon (Bun + TypeScript).** The database layer ships a combined
  migration (`migrations/000_combined.sql`) that is idempotent — every
  `CREATE TABLE` and `ADD COLUMN` uses `IF NOT EXISTS`. The Postgres adapter
  runs the combined migration inside an advisory-lock transaction on first
  connection, so upgrades that add tables or columns converge automatically
  without manual `psql`. The SQLite adapter applies the same schema on
  initialization. The combined migration is regenerated from the numbered
  migrations via `bun run generate:bundled-schema`, and CI fails if the
  combined file is stale.
- **Go `golang-migrate`.** The popular migration tool supports both
  up/down migrations and idempotent migrations. For additive changes, the
  up migration uses `IF NOT EXISTS` and the down migration is a no-op or a
  safe `DROP IF EXISTS`. The tool's `force` command can mark a migration as
  applied without running it — useful for recovering from a crashed
  migration, but unnecessary with the combined-migration pattern.
- **Rails `ActiveRecord::Migration`.** Rails migrations are versioned and
  tracked in a `schema_migrations` table. The `change` method supports
  reversible migrations (additive operations auto-generate a down path).
  Destructive operations require explicit `up`/`down` methods. Rails also
  generates a `schema.rb` (the combined schema) that can be loaded directly
  on a fresh database — the same pattern as the combined migration file.

## See Also

- [Reversibility + Rollback-First](reversibility-rollback-first.md) —
  idempotent additive migrations are the database-layer expression of
  reversibility.
- [Fail Fast + Explicit Errors](fail-fast-explicit-errors.md) — if the
  migration fails, fail with a clear error; do not leave the database in a
  partial state.
- [Type-Safe Data Interchange](https://github.com/levonk/skills-releases/blob/main/knowledge/typescript-monorepo-best-practices/type-safe-data-interchange.md)
  — the row schema (one-file-per-shape) is the application-layer mirror of
  the database schema; both must stay in sync with the combined migration.
- [Database Scaling](database-scaling.md) — migration strategy interacts
  with replication (additive migrations replicate safely; destructive
  migrations can break replicas).
