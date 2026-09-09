---
type: Practice
title: Destructive Git Prohibition
description: Never use commands that permanently delete untracked or uncommitted work — git clean -fd, git checkout ., git reset --hard — without explicit user confirmation. Untracked files are invisible to version control and cannot be recovered once destroyed.
tags: [git, destructive-commands, safety, untracked-files, data-loss, worktree]
date:
  created: "2026-08-10"
  knowledge-basis: "2026-08-10"
  last-used: "2026-08-10"
---

# Destructive Git Prohibition

## Failure Mode

An agent or developer runs `git clean -fd` to "tidy up" a working
directory. This permanently deletes every untracked file and directory —
including `.env`, local config, generated artifacts that took minutes to
build, and files the developer never intended to remove. There is no
reflog, no trash, no undo. The data is gone.

The same applies to `git checkout .` and `git reset --hard`: they
discard uncommitted changes to tracked files, which may represent hours
of unsaved work. In a worktree context, the damage is compounded because
the worktree may have been created by an automated tool that copied
config files into it — those copies are untracked and vanish with a
single `git clean`.

## Practice

**Never run destructive git commands without explicit user
confirmation.** Destructive commands are those that delete or discard
work that is not recoverable through git's normal history mechanisms
(reflog, `git revert`, `git reset --soft`).

### Prohibited Commands

| Command | What it destroys | Why it is dangerous |
|---------|-----------------|---------------------|
| `git clean -fd` | All untracked files and directories | No reflog, no recovery — untracked files are invisible to git history |
| `git clean -fdx` | All untracked + ignored files | Also deletes `.env`, build caches, downloaded dependencies |
| `git checkout .` | Uncommitted changes to all tracked files | Discards work that has not been committed; no undo |
| `git reset --hard` | Uncommitted changes + moves HEAD | Discards work and rewrites branch pointer; reflog may help but changes are gone |
| `git checkout -- <file>` | Uncommitted changes to one file | Silent — no confirmation prompt, no output on success |

### Safe Alternatives

| Instead of | Use | Why |
|------------|-----|-----|
| `git clean -fd` | `git clean -nd` (dry-run) | Shows what would be deleted without deleting |
| `git checkout .` | `git stash` | Changes are saved and recoverable via `git stash pop` |
| `git reset --hard` | `git reset --soft` | Moves HEAD but preserves changes in the working tree |
| `git checkout -- <file>` | `git stash push <file>` | Saves the change before discarding it |

### Core Principles

1. **Untracked files are sacred**: Git does not track them, so git
   cannot recover them. Treat every untracked file as potentially
   irreplaceable.
2. **Dry-run before execute**: Always run `git clean -nd` (or
   equivalent) before any clean operation. Show the user exactly what
   will be deleted and require confirmation.
3. **Prefer reversible operations**: `git stash` is reversible. `git
   reset --soft` is reversible. `git clean -fd` is not. Choose the
   reversible option.
4. **Trust git's guardrails**: Git refuses to remove a worktree with
   uncommitted changes. This is a safety feature — do not work around it
   with `--force` unless the user explicitly confirms.
5. **Surface, do not decide**: When encountering a dirty state that
   blocks an operation, surface the situation to the user. Do not
   autonomously clean it up. The user may have uncommitted work they
   intend to keep.

## Concrete Instances

### Git (built-in)

```bash
# WRONG — permanently deletes untracked files
git clean -fd

# RIGHT — show what would be deleted, then confirm
git clean -nd
# Output: "Would remove .env.local"
# Output: "Would remove tmp/debug.log"
# User confirms → git clean -f (without -d unless directories are confirmed)
```

```bash
# WRONG — discards uncommitted changes silently
git checkout .

# RIGHT — stash so changes are recoverable
git stash
# ... do the operation ...
git stash pop  # restore if needed
```

### Jujutsu (jj)

```bash
# jj does not have a direct equivalent of git clean, but:
jj abandon   # abandons the current commit's changes
jj restore   # restores files to a previous state

# These are less destructive than git clean (they operate on commits,
# not untracked files), but still discard work. Always confirm first.
```

Jujutsu's model is less prone to accidental destruction because the
working copy is a commit — changes are automatically tracked. However,
`jj abandon` and `jj restore` still discard work and require the same
caution.

### Automated agent guardrails

```
# Agent runtime pseudo-code
function cleanWorktree(path):
    status = gitStatus(path)
    if status.hasUntrackedFiles:
        log.warn("untracked files present", files=status.untracked)
        return Error("refusing to clean: untracked files present")
    if status.hasUncommittedChanges:
        log.warn("uncommitted changes present")
        return Error("refusing to clean: uncommitted changes present")
    # Only proceed if the worktree is clean
    gitWorktreeRemove(path)
```

Agent runtimes must never autonomously run destructive git commands.
When a worktree has untracked files or uncommitted changes that block
cleanup, the runtime surfaces the situation and lets the user decide.
This is a specific instance of the broader principle: **no autonomous
destructive mutation across process boundaries**.

## Prevention

1. **Document the prohibition in AGENTS.md** — state explicitly:
   "NEVER run `git clean -fd`." Make it a universal contract, not a
   suggestion.
2. **Use `--dry-run` by default** — any cleanup script should default to
   dry-run mode and require an explicit flag to execute destructive
   operations. See [Shell Scripting Best Practices](shell-scripting-best-practices.md)
   for the dry-run pattern.
3. **Check before clean** — before any clean operation, check
   `git status --porcelain` for untracked files. If any exist, abort and
   surface the list to the user.
4. **Prefer `git stash` over `git checkout .`** — when a clean working
   tree is needed for an operation, stash rather than discard. The stash
   can be popped later if the changes are still needed.
5. **Never use `--force` to bypass git's safety checks** — git refuses
   to remove worktrees with uncommitted changes for a reason. If
   `--force` is needed, the user must explicitly confirm.

## Related Concepts

- [Git Worktree Isolation](git-worktree-isolation.md) — Worktree cleanup
  must respect the destructive-git prohibition; uncommitted changes block
  removal by design
- [Shell Scripting Best Practices](shell-scripting-best-practices.md) —
  Dry-run patterns, strict mode, and confirmation prompts for safe scripts
- [Branch & Tag Hygiene](branch-tag-hygiene.md) — Branch archiving is
  non-destructive (refs are moved, not deleted); pruning requires explicit
  confirmation and `--dry-run` first
