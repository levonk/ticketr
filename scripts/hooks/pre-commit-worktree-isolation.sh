#!/usr/bin/env bash
# pre-commit-worktree-isolation.sh — block commits to protected branches outside a worktree
#
# Bypass layers (in order):
#   1. SKILL_ALLOW_MAIN_WRITE=1 environment variable (user sets in shell)
#   2. .agents/config/script-guards.toml [worktree-isolation] allow_main_write=true
#   3. git commit --no-verify (emergency bypass, skips all hooks)
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "")"
if [[ -z "$repo_root" ]]; then
  exit 0
fi

if [[ "${SKILL_ALLOW_MAIN_WRITE:-0}" == "1" ]]; then
  exit 0
fi

cfg="$repo_root/.agents/config/script-guards.toml"
cfg_bypass=0
if [[ -f "$cfg" ]]; then
  section="$(awk '/^\[worktree-isolation\]/{f=1;next} /^\[/{f=0} f' "$cfg" 2>/dev/null || true)"
  if printf '%s' "$section" | grep -Eq 'allow_main_write[[:space:]]*=[[:space:]]*true'; then
    cfg_bypass=1
  fi
fi

if [[ "$cfg_bypass" -eq 1 ]]; then
  exit 0
fi

current_branch="$(git -C "$repo_root" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")"

protected_branches=("main" "master")
if [[ -f "$cfg" ]]; then
  section="$(awk '/^\[worktree-isolation\]/{f=1;next} /^\[/{f=0} f' "$cfg" 2>/dev/null || true)"
  pb_line="$(printf '%s' "$section" | grep -E 'protected_branches[[:space:]]*=' || true)"
  if [[ -n "$pb_line" ]]; then
    pb_values="$(printf '%s' "$pb_line" | sed 's/.*=\s*//; s/^\[//; s/\]$//; s/"//g; s/,/\n/g' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | grep -v '^$' || true)"
    if [[ -n "$pb_values" ]]; then
      # shellcheck disable=SC2206
      protected_branches=($pb_values)
    fi
  fi
fi

on_protected=0
for pb in "${protected_branches[@]}"; do
  if [[ "$current_branch" == "$pb" ]]; then
    on_protected=1
    break
  fi
done

if [[ "$on_protected" -eq 0 ]]; then
  exit 0
fi

abs_git_dir="$(git rev-parse --absolute-git-dir 2>/dev/null || echo "")"
if [[ "$abs_git_dir" == *"/.git/worktrees/"* ]]; then
  exit 0
fi

echo "" >&2
echo "PRE-COMMIT REJECTED: committing to '$current_branch' outside a worktree." >&2
echo "  Feature work must happen in a worktree on a feature branch." >&2
echo "  Use treehouse: devbox run -- treehouse get --lease" >&2
echo "  Or manual: git worktree add ../<name> -b feature/<scope>/<slug>" >&2
echo "  Bypass: SKILL_ALLOW_MAIN_WRITE=1 git commit ..." >&2
echo "  Bypass (emergency): git commit --no-verify" >&2
echo "" >&2
exit 1
