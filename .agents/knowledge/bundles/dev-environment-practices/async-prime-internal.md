---
type: Practice
title: Async Prime Internal
description: prime_impl runs a sync git checkpoint (no push, follows pre-task-commit-checkpoint protocol) then kicks off cache-warming jobs (package downloads, build, recipe list, API doc generation) in parallel as fire-and-forget background tasks. Verification gates (typecheck/test/validate) stay synchronous. Triggered async by .envrc on directory entry, gated by direnv allow and DEVBOX_SHELL_ENABLED.
tags: [developer-experience, direnv, devbox, just, async, background-tasks, warmup, caching, git-checkpoint]
date:
  created: "2026-07-20"
  knowledge-basis: "2026-07-20"
  last-used: "2026-07-20"
sources:
  - id: levonk-base-boilerplate
    resource: internal-docs/adr/adr-20260131001-standard-developer-ux-flow.md
    title: levonk-base-boilerplate
  - id: skills-src-git-repository-management-skill
    resource: includes/pre-task-commit-checkpoint.md.tmpl
    title: skills-src `git-repository-management` skill
---

# Async Prime Internal

## Failure Mode

Without an async warmup phase, every developer and AI agent entering a project
pays the full cold-cache cost: package downloads, build compilation, doc
generation, and recipe discovery all happen on the first interactive command.
This adds seconds-to-minutes of latency to the first `just build`, `just test`,
or `just typecheck` — and worse, AI agents waste context window waiting for
serial warmup steps that could have overlapped.

A second failure mode: starting warmup jobs (especially `just build_impl`)
when the working tree has uncommitted changes means generated artifacts,
formatter rewrites, or compiler output could intermix with the user's
in-progress work. Without a checkpoint commit on HEAD, there's no clean
rollback point if warmup goes wrong.

The opposite failure mode is equally bad: running verification gates
(typecheck, test, validate) in the background means failures are silent and
the agent proceeds on stale assumptions. Verification **must** be synchronous
and blocking — only warmup belongs in the async phase.

## Practice

`prime_impl` has two phases:

1. **Phase 1 (sync): Git checkpoint** — commit any pending work as a single
   checkpoint commit so there's a safe rollback point before warmup jobs
   start. No push. Follows the `pre-task-commit-checkpoint` protocol from the
   `git-repository-management` skill. Skippable via `PRIME_SKIP_CHECKPOINT=1`.
2. **Phase 2 (async, fire-and-forget): Cache warmup** — kick off cache-warming
   jobs in parallel as `nohup ... &` background tasks, then return
   immediately. Verification gates (typecheck, test, validate) are NOT run in
   prime — they stay synchronous and blocking.

### Job Set

| Phase | Job | Why it's in prime | Sync/Async |
|-------|-----|-------------------|------------|
| 1 | Git checkpoint (`git add -A && git commit`) | Safe rollback point before warmup; follows pre-task-commit-checkpoint protocol | **Sync** (fast, must complete before warmup) |
| 2 | Download packages (`cargo fetch`, `pnpm install --frozen-lockfile`, `uv sync --frozen`) | Warms the package cache so the first `just build` doesn't stall on network | Async (network-bound; independent of build) |
| 2 | Build (`just build_impl`) | Warms the compiler/build cache so the first `just test` / `just typecheck` skips redundant compilation | Async (CPU-bound but independent of downloads) |
| 2 | List (`just --list`) | Discovers available recipes for AI agent context — the agent knows what targets exist without a round-trip | Async (trivial; instant) |
| 2 | Generate API doc (`cargo doc --no-deps`, `pnpm docs`, `uv run pdoc`) | Warms the doc cache so `just docs` is instant on demand; only runs if `has_docs` is set | Async (CPU-bound but independent of build) |

### Jobs NOT in prime (synchronous gates)

| Job | Why it stays synchronous |
|-----|-------------------------|
| Typecheck | Verification gate — failures must block the agent |
| Test | Verification gate — failures must block the agent |
| Validate | Verification gate — failures must block the agent |
| Lint | Verification gate — failures must block the agent |

The rule: **if a failure means the agent should stop and fix it, it's
synchronous. If a failure just means the cache didn't warm, it's async.**

### Phase 1: Git checkpoint (sync, no push)

Before kicking off any warmup jobs, `prime_impl` checks whether the working
tree is dirty (staged changes, unstaged changes, or untracked files). If dirty,
it commits everything as a single checkpoint commit. If clean, it skips.

This follows the `pre-task-commit-checkpoint` protocol from the
`git-repository-management` skill:

1. Check `git status --porcelain` — if empty, tree is clean, skip.
2. If dirty, commit: `git add -A && git commit -m "checkpoint: pre-prime warmup" -m "- Pre-prime checkpoint: commit pending work before async warmup jobs start"`
3. The checkpoint commit hash is on HEAD — roll back with `git reset HEAD~1` if
   the warmup jobs produce unwanted side effects.
   the warmup jobs produce unwanted side effects.

**No push**: the checkpoint is local-only. Pushing is a separate, explicit
step (`git push` or the `git-repository-management` skill's push entry point).

**Skippable**: set `PRIME_SKIP_CHECKPOINT=1` to skip the checkpoint step
entirely. Use this when you have work in progress that you don't want
auto-committed (e.g., debugging a tricky issue and cd-ing in and out of the
directory).

**Relationship to the `git-repository-management` skill**: the checkpoint step
in `prime_impl` uses plain git commands (no dependency on the skill's
script install path). When the `git-repository-management` skill is installed,
the AI agent can alternatively run the skill's checkpoint entry point
(`git-commit-batch.sh --slug pre-prime-checkpoint`) before invoking
`just prime` — this adds pre/post auto-tags for rollback safety and vertical
grouping validation. The inline git commands in `prime_impl` are the
fallback that works without the skill installed.

### Phase 2: Async warmup (fire-and-forget)

After the checkpoint completes (or is skipped), `prime_impl` kicks off
cache-warming jobs in parallel as `nohup ... &` background tasks:

```just
    # --- Download packages (language-dependent) ---
    @nohup cargo fetch >/dev/null 2>&1 &

    # --- Build (warm the build cache / compiler) ---
    @nohup just build_impl >/dev/null 2>&1 &

    # --- List (recipe inventory for AI agent context discovery) ---
    @nohup just --list >/dev/null 2>&1 &

    # --- Generate API doc (if enabled, warms doc cache) ---
    @nohup cargo doc --no-deps >/dev/null 2>&1 &

    @echo "✅ Prime: async warmup jobs kicked off (fire-and-forget)"
```

### Async .envrc trigger

`.envrc` kicks off `just prime_impl` in the background on every directory
entry. Async, quiet, non-blocking via `nohup ... > /dev/null 2>&1 &`. Gated by
two checks:

1. **`direnv allow`** — the user's explicit consent to run `.envrc` content.
   If direnv is not allowed, nothing happens.
2. **`DEVBOX_SHELL_ENABLED` check** — skip if already inside a devbox shell
   (set by `devbox shell`, `devbox run`, or `devbox shellenv --init-hook`).
   Prevents re-triggering prime when nesting shells.

```bash
# Async prime_impl trigger (per dev-environment-practices/async-prime-internal.md)
if [ -f devbox.json ] && command -v just >/dev/null 2>&1; then
  if [ "$DEVBOX_SHELL_ENABLED" != "1" ]; then
    nohup devbox run -- just prime_impl > /dev/null 2>&1 &
  fi
fi
```

When triggered by `.envrc`, the sync checkpoint phase still runs — if the tree
is dirty on directory entry, it will be checkpointed. This is intentional: it
ensures a safe starting point every time you enter the project. If you have
work in progress that you don't want auto-committed, set
`PRIME_SKIP_CHECKPOINT=1` in your `.envrc.local` or shell environment.

### Three entry paths, one target

All three paths that trigger prime go through `prime_impl`, which has the
same two-phase structure. No duplicated logic:

- `just prime` (manual) → `_devbox prime_impl` → (in devbox) `just prime_impl` → checkpoint + async jobs
- `.envrc` async trigger → `devbox run -- just prime_impl` → checkpoint + async jobs
- `just bootstrap` → `bootstrap_impl` → `prime_impl` → checkpoint + async jobs (if
  bootstrap calls prime; project-specific)

## Why fire-and-forget for warmup (not capture-and-surface)

Prime's warmup phase is **warmup**, not verification. A failed warmup job
means the first synchronous `just build` or `just typecheck` will pay the
cold-cache cost — it does not mean the project is broken. Capturing and
surfacing stale prime failures adds complexity (status files, staleness
checks on the status files themselves) for no benefit: the synchronous gates
will catch real failures when the agent runs them.

If the agent wants to know whether prime succeeded, it runs `just prime`
synchronously and observes. The async path is for the common case: enter
directory, warm caches in the background, proceed with work.

## Why the checkpoint is sync (not async)

The checkpoint commit must complete before any warmup jobs start. If
`just build_impl` runs in the background while `git add -A` is staging
files, the build might generate artifacts (in gitignored dirs, but still)
that intermix with the staging operation. By making the checkpoint sync and
the warmup async, we guarantee: checkpoint completes → tree is clean → warmup
jobs can run safely in the background.

## Why verification gates stay synchronous

Typecheck, test, and validate answer the question "is this code correct?" The
answer must be known before the agent proceeds. Running them in the background
would mean the agent acts on stale assumptions — it would assume the code is
correct until a background job finishes, which defeats the purpose of the gate.

The sync/async split is:

- **Sync (checkpoint)**: "commit pending work so there's a rollback point"
- **Async (warmup)**: "warm the caches so the first synchronous command is fast"
- **Sync (gates)**: "verify correctness before proceeding"

## Related Concepts

- [Standard Developer UX Flow](standard-developer-ux-flow.md) — The three-flow pattern; prime is part of the bootstrap → prime → work sequence
- [Auto-Detecting Devbox Targets](internal-vs-normal-targets.md) — `prime` (auto-detecting) delegates to `_devbox prime_impl` → `prime_impl` (implementation)
- [direnv Auto-Activation](direnv-auto-activation.md) — The `.envrc` async trigger rides on direnv's directory-entry hook
- `git-repository-management` skill — The `pre-task-commit-checkpoint` protocol that Phase 1 follows; the skill's checkpoint entry point (`git-commit-batch.sh --slug pre-prime-checkpoint`) is the preferred invocation when the skill is installed
