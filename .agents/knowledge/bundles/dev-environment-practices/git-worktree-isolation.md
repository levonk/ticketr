---
type: Practice
title: Git Worktree Isolation
description: Parallel development without branch conflicts by using linked worktrees — each task gets its own working directory on its own branch, sharing a single object database, with deterministic path layout and base-branch precedence.
tags: [worktree, parallel-development, isolation, git, branch-management, developer-experience]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Git Worktree Isolation

## Failure Mode

A developer is working on a feature branch and needs to review a PR or
fix an urgent bug. They must either stash their in-progress work, switch
branches (losing their mental context and editor state), or clone a
second copy of the repository (wasting disk space and time on a full
checkout). AI agents face the same problem: an automated workflow that
needs to work on a task cannot run in the developer's active checkout
without risking uncommitted changes.

The result: context switching, stash management overhead, accidental
commits to the wrong branch, and parallel work that is anything but
parallel.

## Practice

Use **linked worktrees** to give each task its own working directory on
its own branch, all sharing a single git object database. No stashing,
no cloning, no branch switching. Each worktree is an independent
checkout that can be opened in a separate editor, run by a separate
agent, and cleaned up independently.

### Core Principles

1. **One worktree per task**: Each parallel task (feature, PR review,
   bug fix, agent workflow) gets its own worktree. The main checkout
   stays clean for ad-hoc work.
2. **Shared object database**: All worktrees share one `.git` directory.
   A commit in one worktree is immediately visible in all others — no
   `git fetch` needed. Disk usage is one object store plus N working
   directories.
3. **Deterministic path layout**: Worktree paths follow a predictable
   structure (e.g., `~/.workspaces/owner/repo/worktrees/<branch>`) so
   tools can discover them programmatically.
4. **Base-branch precedence**: New worktrees branch from the project's
   base branch (main, dev, or auto-detected default), not from the
   current HEAD of another worktree. This prevents cascading divergence.
5. **Sync before create**: The base branch is fetched from origin before
   a new worktree is created, ensuring the worktree starts from the
   latest upstream state.
6. **Trust git's guardrails**: Git refuses to remove a worktree with
   uncommitted changes. Do not work around this — surface the error to
   the user and let them decide.

### Worktree Lifecycle

```
create → use (develop, test, run) → merge/abandon → cleanup
```

1. **Create**: Sync the base branch, create the worktree on a new branch
   from that base, copy any required config files (e.g., `.env`,
   project-specific settings) into the worktree.
2. **Use**: Work in the worktree as if it were a normal checkout. Run
   dev servers (with [deterministic port allocation](worktree-port-allocation.md)),
   run tests, make commits.
3. **Cleanup**: After the branch is merged or abandoned, remove the
   worktree and delete the branch. Git refuses removal if there are
   uncommitted changes — this is a safety feature, not an obstacle.

### Adoption Pattern

Sometimes a worktree already exists (created by a human or another tool).
Rather than creating a duplicate, the system should **adopt** it: detect
that the expected path or branch already has a valid worktree, and reuse
it. This avoids "worktree already exists" errors when multiple tools
operate on the same repository.

### Limits and Cleanup

Without limits, worktrees accumulate indefinitely — merged branches,
abandoned experiments, stale agent runs. A cleanup strategy should:

- Remove worktrees whose branches have been merged into the base branch.
- Remove worktrees older than a staleness threshold (e.g., 14 days with
  no commits).
- Skip removal if the worktree has uncommitted changes or active
  references (e.g., open PRs, running workflows).
- Enforce a maximum concurrent worktree count (e.g., 25) to prevent
  runaway disk usage.

## Concrete Instances

### Git worktree (built-in)

```bash
# Create a worktree for a feature branch
git fetch origin main
git worktree add ~/.workspaces/myrepo/worktrees/feature-auth \
  -b feature-auth origin/main

# Work in it
cd ~/.workspaces/myrepo/worktrees/feature-auth
# ... develop, test, commit ...

# Clean up after merge
git worktree remove ~/.workspaces/myrepo/worktrees/feature-auth
git branch -d feature-auth
```

Git's built-in `git worktree` command creates linked worktrees that
share the object database. The path layout is determined by the caller;
project-scoped layouts (under a workspace directory per repo) keep
worktrees organized and discoverable.

### Jujutsu (jj)

```bash
# Jujutsu's working copy is inherently multi-headed
jj new main  # create a new working-copy commit on top of main
# ... work ...
jj new main  # create another parallel commit
# Both exist simultaneously without explicit worktree commands
```

Jujutsu models the working copy as a commit and supports multiple
overlapping working copies natively. There is no separate "worktree"
concept — each `jj new` creates an independent change that can be
worked on in parallel via `jj workspace add` for separate directories.
The isolation principle is the same: parallel work on separate branches
without stashing or cloning.

### Automated agent isolation

```bash
# An agent runtime creates a worktree per task
WORKTREE_PATH=$(git worktree add --detach /tmp/agent-task-123 2>/dev/null)
cd "$WORKTREE_PATH"
git checkout -b task-123 origin/main
# ... agent runs tests, makes commits ...
# On completion: merge or abandon, then git worktree remove
```

Agent runtimes use worktrees to isolate automated tasks from the
developer's active checkout. Each task gets a fresh branch from the
base, runs to completion, and is cleaned up — without ever touching the
developer's working directory.

## Prevention

1. **Always sync before create** — `git fetch origin <base>` before
   creating a worktree ensures it starts from the latest upstream state,
   not a stale local branch.
2. **Copy config files** — worktrees do not inherit untracked files
   (`.env`, local configs). Copy required files into the worktree after
   creation, with path-traversal validation for security.
3. **Validate paths** — when generating worktree paths from user input
   (branch names, task IDs), validate that the resulting path is within
   the workspace root. Prevent path traversal via `..` or absolute paths.
4. **Time out git operations** — `git fetch` and `git worktree add` can
   hang on network issues. Set a timeout (e.g., 60 seconds) and surface
   the error rather than blocking indefinitely.
5. **Clean up on failure** — if worktree creation fails partway, remove
   the orphaned directory before re-throwing the error. Do not leave
   partial state for the next caller to discover.

## Related Concepts

- [Worktree Port Allocation](worktree-port-allocation.md) — Deterministic
  ports so parallel worktree dev servers do not conflict
- [Destructive Git Prohibition](destructive-git-prohibition.md) — Never
  use `git clean -fd` in a worktree; it destroys untracked files
- [Branch & Tag Hygiene](branch-tag-hygiene.md) — Archiving stale branches
  from completed worktrees
- [Standard Developer UX Flow](standard-developer-ux-flow.md) — The
  overall workflow that worktree-based parallelism fits into
