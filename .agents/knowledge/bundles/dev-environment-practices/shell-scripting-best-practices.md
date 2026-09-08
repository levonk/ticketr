---
type: Practice
title: Shell Scripting Best Practices
description: Critical rules to write, change, and run shell scripts safely — strict mode, PATH guards, git gates, dry-runs, logging, testing, and verification workflow.
tags: [shell, bash, scripting, safety, testing, shellcheck, shfmt, bats, dry-run, git]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-18"
  last-used: "2026-07-18"
---

# Shell Scripting Best Practices

Critical rules to write, change, and run shell scripts safely. Derived from
lessons in `scripts/danger/danger-scratch-apply.sh`.

Use when: editing or creating shell scripts, CI jobs, or local helpers.

## Project Integration & Structure

- Respect repository structure. Shared, shell-agnostic logic belongs in a
  `shared/{util,prompt,env}` directory. Keep per-shell hooks/activation in
  `.../zsh/{util,prompt,env}` or `.../bash/{util,prompt,env}`.
- Do not refactor across shells unless explicitly asked. Keep compatibility.
- Do not change dependencies/configs unless explicitly tasked.

## Authoring Standards

- Use bash with strict mode at the top of executable scripts:
  ```sh
  #!/usr/bin/env bash
  set -euo pipefail
  ```
- Prefer portability; avoid non-POSIX features unless bash-only is declared.
- When a script's final action is to run another command (e.g., a dev server),
  use `exec` to replace the shell process so signals and exit codes propagate
  correctly:
  ```sh
  exec npm run dev
  ```

## PATH Guards and Tool Detection

- Provide small helpers like
  `command_exists() { command -v "$1" >/dev/null 2>&1; }`.
- Guard PATH additions; avoid duplicates:
  ```sh
  case ":$PATH:" in *":$XDG_BIN_HOME:"*) : ;; *) export PATH="$XDG_BIN_HOME:$PATH" ;; esac
  ```
- Feature-detect optional tools (`strace`, `lsof`, `fuser`).

### Required Preflights (Fail Fast)

Before any destructive or stateful action, confirm tools and environment:

- Validate required tools; exit with a clear message if missing:
  - Required: `git`, `timeout`.
  - Optional (log presence): `strace`, `lsof`, `fuser`.
- Print versions for traceability:
  - `git --version`, `timeout --version`.
- Detect and report running processes and lock holders when relevant:
  - Use `pgrep`, `lsof -F`, and `fuser -v` when available.
- Provide an interactive pause only when a TTY exists.

### Example Snippets

- `command_exists` helper:
  ```sh
  command_exists() { command -v "$1" >/dev/null 2>&1; }
  ```
- PATH guard:
  ```sh
  case ":$PATH:" in *":$XDG_BIN_HOME:"*) : ;; *) export PATH="$XDG_BIN_HOME:$PATH" ;; esac
  ```

## Git Cleanliness Gate (Do Not Proceed if Dirty)

- Refuse to proceed when any of the following are present:
  - Untracked files: `git ls-files --others --exclude-standard`.
  - Staged changes: `git diff --cached --name-status`.
  - Unstaged modifications: `git diff --name-status`.
  - Unpushed commits: check upstream vs HEAD with
    `git rev-list --left-right --count @{u}...HEAD`.
- Allow an explicit override only if configured (e.g.,
  `DANGER_SKIP_GIT_PREFLIGHT=1` or a `--no-git-checks` flag). Always log the
  override.

### Git Cleanliness Gate Commands

```sh
# Untracked
U=$(git ls-files --others --exclude-standard)
# Staged
S=$(git diff --cached --name-status)
# Modified
M=$(git diff --name-status)

if [ -n "$U$S$M" ]; then
  echo "[error] repo not clean: untracked/staged/modified present" >&2
  git status -s -uall
  exit 2
fi
```

## Dry-Run Patterns and Safety Guards

### Dry-Run First, Always

- For each destructive step, run a dry-run and only proceed on success.
- If a dry-run fails:
  1) Log a concise failure line.
  2) Re-run the same dry-run with full output tee’d to the log.
  3) Abort the pipeline with a distinct exit code.
- If `--dry-run` is unsupported by the tool, fall back to safe diagnostics
  (e.g., `doctor`, `status`) without failing.
- Example sequence (pseudocode):
  ```sh
  if ! dryrun_step; then exit 10; fi
  real_step
  ```
- Dry-run gate pattern:
  ```sh
  if ! tool --dry-run; then
    echo "dry-run failed; rerunning with full output" >&2
    tool --dry-run || true
    exit 12
  fi
  tool
  ```

### Safety Guards for State/Workspace

- Refuse destructive operations when the target/source resolves to the current
  repo working tree (e.g., refuse to purge when the source path equals CWD).
- Decouple destructive commands from the repo CWD when possible (e.g., run purge
  from `$HOME`).

## Logging and Diagnostics

- Log clearly to stderr or a log file for long-running flows.
- Use bounded timeouts for dry-runs and real apply (`timeout 90s` or
  configurable via env).
- Optionally enable `strace` for real apply; write traces to a file and include
  timestamps.
- For long operations, arm an automatic `SIGQUIT` just before the timeout to
  capture goroutine stacks (for Go-based tools).
- After success/failure, run a status command and log what remains to be
  applied.
- Keep logs clear and store them under a predictable path (e.g.,
  `/tmp/your-script.log`).

## CLI Flags and Argument Handling

- Support explicit flags to skip gates (with warnings):
  - `--no-git-checks` → sets `DANGER_SKIP_GIT_PREFLIGHT=1`.
  - `--no-dry-run` (optional) → sets `DANGER_SKIP_DRYRUN=1`.
- Communicate clearly in logs when these flags are used.

## Testing and Validation (Do Not Claim Done if Failing)

- Run `shellcheck` on all relevant scripts. Treat warnings and errors as issues
  to resolve or explicitly explain/reason about exceptions.
  ```sh
  FILES=$(git ls-files | grep -E '(\.sh$|/bin/|/util/|/env/|/aliases/)')
  [ -n "$FILES" ] && shellcheck -x $FILES || true
  ```
- Run `shfmt -d` to enforce formatting where applicable.
- Run `bats -r scripts/tests` and ensure tests pass in the project’s harness. If
  tests fail or scripts error, do not claim completion; fix or report clearly.
- Prefer adding a minimal stub or guard when tests expect optional utilities.

## Verification Workflow Checklist

A practical checklist and command set to verify shell changes before saying
"done." Use this anytime shell scripts are added or modified.

### Quick Checklist

- [ ] Required tools present: `git`, `shellcheck`, `shfmt`, `bats` (if tests
  exist).
- [ ] Repository is clean: no untracked, staged, or unstaged files left behind.
- [ ] Lint clean: `shellcheck` runs without errors; warnings reviewed or
  annotated.
- [ ] Format clean: `shfmt -d` shows no diffs.
- [ ] Unit tests pass with `bats -r tests`.
- [ ] If scripts orchestrate external tools, include safe dry-runs and clear
  logs.
- [ ] Commit(s) grouped by functionality; clean status after commit.

### Commands

Run from the repository root.

#### 1) Status sanity

```sh
# Porcelain status (all files, including untracked)
git status --untracked-files=all --porcelain
```

#### 2) Lint with shellcheck

```sh
# Collect typical shell script locations; ignore if no files found
FILES=$(git ls-files | grep -E '(\\.sh$|/bin/|/util/|/env/|/aliases/|/scripts/)')
[ -n "$FILES" ] && shellcheck -x $FILES || echo "[info] no shell files to lint"
```

Notes:
- Use `# shellcheck disable=SCxxxx` sparingly, with rationale.
- Prefer fixing warnings unless false positives or intentional patterns.

#### 3) Format with shfmt (diff mode)

```sh
[ -n "$FILES" ] && shfmt -d $FILES || echo "[info] no shell files to format"
```

To autoformat in-place (optional):
```sh
[ -n "$FILES" ] && shfmt -w $FILES
```

#### 4) Run bats tests

```sh
# Only if tests directory exists
[ -d tests ] && bats -r tests || echo "[info] bats not available or tests missing"
```

If tests fail:
- Do not claim completion.
- Fix the failure or add a minimal guard/stub under the test harness path as
  appropriate.

#### 5) Optional environment checks (if applicable)

```sh
# Repo health check
scripts/repo-health.sh --quick || true
```

#### 6) Git cleanliness gate

```sh
# Untracked
U=$(git ls-files --others --exclude-standard)
# Staged
S=$(git diff --cached --name-status)
# Modified
M=$(git diff --name-status)

if [ -n "$U$S$M" ]; then
  echo "[error] repo not clean: untracked/staged/modified present" >&2
  git status -s -uall
  exit 2
fi
```

#### 7) Commit and summarize

```sh
# Example commit (edit title/body)
git commit -m "fix-shell: harden preflights and dry-run gates" \
  -m "Add validation for required tools; gate destructive steps behind dry-runs; update logs."

git status --untracked-files=all --porcelain

git log -n 3 --oneline --decorate --stat
```

## Commit Hygiene

- Group changes by user-facing functionality; use imperative, concise titles.
- If signing is configured, sign commits; otherwise don’t force it.
- After committing, verify clean status and provide a short `git log --stat`
  summary.

## Guidance & Best Practices

- Use `#!/usr/bin/env bash` and `set -euo pipefail` for executable scripts.
- Guard `PATH` additions; avoid duplicates.
- Provide `command_exists()` helper and feature-detect optional tools
  (`strace`, `lsof`, `fuser`).
- Dry-run before destructive steps; on failure, re-run and tee full output;
  then abort with a distinct exit code.
- For long-running operations, prefer bounded `timeout` and capture diagnostics
  (e.g., SIGQUIT for Go tools, `strace` when available).
- Keep logs clear and store them under a predictable path (e.g.,
  `/tmp/your-script.log`).
- Respect repository conventions and avoid unsolicited refactors across shells.

## Sources

- Migrated from src/current/rules/software-dev/platforms/shell/shell-essentials.md
- Migrated from src/current/rules/software-dev/platforms/shell/shell-verify.md
