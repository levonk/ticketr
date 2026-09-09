---
type: Practice
title: Diff Tooling Stack (difftastic + delta + gitattributes)
description: Three-layer diff stack — difftastic as an AST-based external diff driver routed per-filetype via gitattributes, delta as a unified-diff pager, and a context-aware wrapper that switches flags for human vs agent audiences. Documents when external diff drivers fire, the git-apply compatibility failure mode, and the format-patch / --no-ext-diff fixes.
tags: [developer-experience, git, diff, difftastic, delta, gitattributes, agent-aware, git-apply, pager]
date:
  created: "2026-08-30"
  knowledge-basis: "2026-08-30"
  last-used: "2026-08-30"
sources:
  - id: levonk-dotfiles-gitattributes
    resource: vendor/dotfiles/home/current/dot_config/git/gitattributes
    title: levonk/dotfiles gitattributes
  - id: levonk-dotfiles-diff-merge-gitconfig
    resource: vendor/dotfiles/home/current/dot_config/git/sub-config/diff-merge.gitconfig
    title: levonk/dotfiles diff-merge gitconfig
  - id: difftastic-github-issues
    resource: https://github.com/Wilfred/difftastic/issues/186
    title: difftastic unified-diff support (declined)
---

# Diff Tooling Stack (difftastic + delta + gitattributes)

## Failure Mode

An agent running a worktree-transfer workflow executes
`git diff > patch && git apply patch` to move uncommitted changes to a
linked worktree. The command fails — `git apply` reports "no changes" or
garbled hunk headers. The agent falls back to `cp` of individual files,
which is fragile, loses mode bits, and does not generalize.

The root cause is not a bug in the workflow: it is that the repository
has an **external diff driver** configured via `gitattributes`
(`*.rs diff=difftastic`), and the driver's output format is not unified
diff. Difftastic produces AST-based structural diff output (side-by-side
or inline) with no `@@ -a,b +c,d @@` hunk markers. `git apply` cannot
parse it.

A second, subtler failure mode: an agent or human disables difftastic
entirely to work around the patch problem, losing the structural diff
information that makes uncommitted changes far easier to understand
("this function moved" vs "delete 50 lines, add 50 lines"). The fix is
surgical (`--no-ext-diff` for the patch operation only), not a global
disable.

## Practice

Run a **three-layer diff stack**, each layer independent and configured
in a different place:

| Layer | Tool | Config Location | Role |
|-------|------|----------------|------|
| **Diff engine** | difftastic (`difft`) | `gitattributes`: `diff=difftastic` per filetype | Replaces git's built-in diff with AST-based structural diffing. Invoked as an external diff driver per-file. |
| **Pager** | delta | `gitconfig`: `core.pager=delta` | Pretty-prints unified diff output on a TTY. Display-only — never touches piped/redirected output. |
| **Wrapper** | `git-difftastic.bash` | `~/.local/bin/` (chezmoi-deployed) | Context-aware: detects human vs agent and passes different flags to `difft` (color, display mode, syntax highlighting). |

**Key insight**: delta and difftastic are independent. Delta is a pager
(display layer); difftastic is a diff engine (replaces git's diff
algorithm). They do not interact — delta formats git's unified diff
output, difftastic replaces the diff generation entirely. Delta is
inherently compatible with `git apply` because it never touches the data
stream that goes to files (git skips the pager when output is
redirected).

### Difftastic as External Diff Driver

**Configuration** (in `~/.config/git/gitattributes`):

```
*.json  diff=difftastic merge=mergiraf ident linguist-language=JSON
*.sh    diff=difftastic eol=lf merge=text ident
*.py    diff=difftastic merge=mergiraf text-std linguist-language=Python
*.rs    diff=difftastic merge=mergiraf text-std linguist-language=Rust
# ... 50+ file types mapped to difftastic
```

**How it works**: When git needs to diff a file with `diff=difftastic`
attribute, it invokes the command configured in
`[diff "difftastic"] command = git-difftastic.bash` with 7 positional
args (path, old-file, old-hex, old-mode, new-file, new-hex, new-mode).
The wrapper calls `difft` with those args.

**Difftastic's output formats**:

- `side-by-side` — two columns, line numbers aligned (default for humans)
- `inline` — single column, closer to traditional diff (used for agents)
- `json` — machine-readable JSON array
- **No unified diff format** — difftastic does NOT produce
  `@@ -a,b +c,d @@` hunk markers. This is by design (AST-based diff is
  fundamentally different from line-based diff).

Difftastic's author has explicitly declined to add unified diff output
(GitHub issues #186, #532, #820): structural diff and line-based diff
are different problem domains; the tool focuses on the former.

### Delta as Pager

**Configuration** (in `~/.config/git/sub-config/diff-merge.gitconfig`):

```ini
[delta]
    syntax-theme = TwoDark
    line-numbers = true
    side-by-side = true
    dark = true
    navigate = true
    hyperlinks = true
```

Delta is a **pager** (`core.pager=delta`), not a diff driver. It
receives git's unified diff output on stdin and pretty-prints it on the
terminal. When output is redirected/piped to a file or another program,
git skips the pager entirely and writes unified diff directly. **Delta
is inherently compatible with `git apply`** — it never touches the data
stream that goes to files.

### Context-Aware Wrapper (`git-difftastic.bash`)

The wrapper detects whether the audience is a human or an AI agent and
dispatches to `difft` with different flags:

**Human profile**: `--color=always --syntax-highlight=on --display=side-by-side --tab-width=4`
**Agent profile**: `--color=never --syntax-highlight=off --display=inline --tab-width=2 --width=120 --skip-unchanged`

**Detection** (in priority order):

1. `GIT_DIFFTASTIC_PROFILE=human|agent` — explicit override
2. `DOTFILES_IS_AGENT_SHELL=1` — shell framework signal
3. `NO_COLOR` set — de facto "no color" standard
4. `TERM=dumb` or empty — non-interactive / editor / CI
5. stdout not a TTY — output is being captured/piped

**Important limitation**: The agent profile switches to
`--display=inline`, which is still difftastic's own format (not unified
diff). The `--- JSON` header with old/new columns is still produced.
`git apply` still cannot parse it. The wrapper makes the output *more
readable* for agents but does not make it *machine-parseable* as a
patch.

## Concrete Instances

### When External Diff Drivers Fire

External diff drivers (via gitattributes `diff=<driver>`) are invoked
ONLY when the working tree is one side of the diff:

| Command | Invokes external driver? | Output format |
|---------|--------------------------|---------------|
| `git diff` (working tree vs index) | **Yes** | difftastic format |
| `git diff HEAD` (working tree vs HEAD) | **Yes** | difftastic format |
| `git diff <c1>` (commit vs working tree) | **Yes** | difftastic format |
| `git diff <c1> <c2>` (commit vs commit) | No | unified diff |
| `git diff --cached` (staged vs HEAD) | No | unified diff |
| `git log -p` | No | unified diff |
| `git show <commit>` | No | unified diff |
| `git format-patch` | No | unified diff (mail format) |
| `git diff --no-ext-diff` | No | unified diff (with configured `diff.algorithm`) |

**Implication for agents**: An agent reading committed history
(`git log -p`, `git show`, `git diff main..HEAD`) always gets unified
diff — no problem. An agent reading uncommitted working-tree changes
(`git diff`) gets difftastic's structural diff — which is actually
*more useful* for understanding changes ("this function moved" vs
"delete 50 lines, add 50 lines"). The only scenario that needs unified
diff is explicit patch generation for `git apply`.

### Git-Apply Compatibility (the fix)

**The problem**: `git diff > patch && git apply patch` fails when
external diff drivers are configured via gitattributes, because the
driver output has no `@@ -a,b +c,d @@` hunk markers.

**Solutions** (in order of preference):

1. **`git format-patch` + `git am`** — purpose-built for transferring
   commits as patches; bypasses external diff drivers by design;
   preserves commit metadata. Requires changes to be committed first
   (stage or stash → commit → format-patch → am).
2. **`git diff --no-ext-diff` + `git apply`** — tells git to use its
   built-in diff (with configured `diff.algorithm`, e.g., patience)
   instead of any external driver. Correct flag for "I need
   machine-parseable diff output." Works on uncommitted changes
   directly.
3. **`git stash show -p`** — stash show does NOT invoke external diff
   drivers; produces unified diff of the stash. Works for stashed
   changes.

**What does NOT work**:

- `git diff > patch` (without `--no-ext-diff`) — invokes external
  driver, output is not unified diff
- `git -c diff.external= diff` — `diff.external` is not the issue; the
  drivers are set via gitattributes, not `diff.external`
- `git -c core.pager=cat diff` — the pager is not the issue; the
  external diff driver intercepts before the pager

### Adding the Tools to devbox.json

Both `difftastic` and `delta` should be available in every project's
devbox environment for consistent diff tooling. They are added to
`devbox.json` packages:

```json
{
  "packages": {
    "difftastic": "",
    "delta": ""
  }
}
```

The user's dotfiles configure the gitattributes and gitconfig globally
(`~/.config/git/gitattributes` and
`~/.config/git/sub-config/diff-merge.gitconfig`), so the tools just need
to be on PATH. Adding them to devbox ensures they are available in every
project environment without relying on global installation. See
[Devbox Over Raw Nix](devbox-over-raw-nix.md) for the devbox package
convention.

### Agent vs Human Use

**For humans**: Difftastic's structural diff is the primary value —
"this function was renamed and moved" is far more readable than 200
lines of add/remove. Side-by-side display with syntax highlighting and
color is the best visual experience.

**For agents**: Difftastic's structural diff is ALSO valuable — an
agent understanding changes benefits from semantic diff ("function
moved" vs "delete + add") just as much as a human. The agent profile
(`--display=inline --color=never --syntax-highlight=off`) makes the
output parseable as text without ANSI codes, but preserves the
structural diff information.

**The one scenario where agents need unified diff instead**: patch
generation for `git apply`. This is rare and explicit — an agent does
not accidentally need to generate a patch; it is always a deliberate
operation (transferring to a worktree, creating a patch file). For this
case, `--no-ext-diff` or `format-patch` is the surgical fix that does
not lose difftastic's structural diff everywhere else.

## Prevention

1. **Never disable difftastic globally to fix a patch problem** — use
   `--no-ext-diff` for the single patch-generation command. The
   structural diff is valuable for every other `git diff` invocation.
2. **Prefer `git format-patch` + `git am` for worktree transfers** —
   they are purpose-built for transferring commits as patches and
   bypass external diff drivers by design. They also preserve commit
   metadata, which `git diff | git apply` does not.
3. **Add `difftastic` and `delta` to every project's devbox.json** —
   consistent diff tooling across environments prevents "the diff looks
   different on my machine" confusion. See
   [Multi-Language Devbox](multi-language-devbox.md) for managing
   multiple tool packages in one devbox.
4. **Set `GIT_DIFFTASTIC_PROFILE=agent` in agent shells** — the wrapper
   detects this explicitly and produces agent-appropriate output
   (inline, no color, no syntax highlighting). Do not rely on TTY
   detection alone; the explicit env var is the highest-priority signal.
5. **Document the git-apply failure mode in any workflow that transfers
   uncommitted changes** — the failure is silent (the patch file is
   created but `git apply` rejects it), so a workflow that does not
   anticipate it will produce a broken patch and fall back to fragile
   `cp` operations.

## Related Concepts

- [Git Worktree Isolation](git-worktree-isolation.md) — Linked worktrees
  are the most common trigger for the git-apply failure mode (transferring
  uncommitted changes to a worktree). The `format-patch` / `--no-ext-diff`
  fix applies directly to worktree-transfer workflows.
- [Shell Scripting Best Practices](shell-scripting-best-practices.md) —
  The context-aware wrapper follows the strict-mode and PATH-guard
  conventions documented here.
- [Devbox Over Raw Nix](devbox-over-raw-nix.md) — The devbox.json
  package convention for adding `difftastic` and `delta` to every
  project environment.
- [Destructive Git Prohibition](destructive-git-prohibition.md) —
  Disabling difftastic globally to work around a patch problem is a
  destructive workaround; prefer the surgical `--no-ext-diff` fix.
