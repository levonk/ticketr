---
type: Practice
title: CLI Tool Standards
description: Cross-language CLI standards — standard args, config precedence, output discipline, color control, daemon mode, agent mode (AXI/TOON), skill emission (--gen-skill), coding-agent hook wiring, non-identifiable telemetry, shell completion, man pages, and structured logging.
tags: [cli, standards, axi, toon, agent-mode, daemon, cross-language, skill-emission, agent-hooks, telemetry]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-29"
  last-used: "2026-07-29"

sources:
  - id: levonk-base-boilerplate
    resource: "internal-docs/adr/adr-20260607001-cli-tool-standards.md"
    title: "levonk-base-boilerplate"
---

# CLI Tool Standards

## Failure Mode

CLI tools drift across projects and languages, causing inconsistent UX,
configuration handling, and operational posture. Traditional human-centric CLI
design is suboptimal for autonomous agent consumption.

## Practice

Adopt a unified cross-language standard for CLI program behavior with **agent
mode as the default**.

### Standard Arguments

All CLIs must support `--help`/`-h`, `--version`/`-v`, and `--usage`.

### Configuration Precedence

CLI args > env vars > local project config > user config (XDG) > system config >
hardcoded defaults. Prefer TOML for human-edited config.

### Output Discipline

- Results to stdout; logs/progress/errors to stderr
- `--json` output mode
- `--color=auto|always|never` with smart TTY detection
- Honor `NO_COLOR` environment variable

### Daemon Mode

For long-running tasks (>30s), provide `--daemon`/`--no-daemon` flags with
auto-spawning, `--list-jobs`, and `--cancel-job <id>`.

### Agent Mode (AXI)

- **TOON format** on stdout (~40% token savings over JSON)
- **Minimal default schemas**: 3-4 fields in list output, not 10
- **Content truncation**: truncated preview with total size and `--full` escape
- **Pre-computed aggregates**: include total count in list output
- **Definitive empty states**: "0 tasks found" not empty output
- **Structured errors on stdout**: actionable suggestions, no raw stack traces
- **No interactive prompts**: fail with clear error if required value missing
- **Content-first no-args**: show live state, not usage manual
- **Contextual disclosure**: suggest next steps after current output
- **Session integrations**: register hooks for Claude Code, Codex, OpenCode
- **Skill emission (`--gen-skill`)**: print an installable Agent Skill prompt to
  stdout that explains how to use the tool. Recommend integration paths in
  order: hooks (lowest token cost), CLI, MCP. Generate from the same content as
  the no-args home view so it never drifts; `--gen-skill --check` exits non-zero
  if the committed skill is stale (for CI). Output must be byte-identical across
  re-runs so CI can diff cleanly. Strip live state; rewrite command examples to
  non-interactive forms (`npx -y`, `uvx`, `pnpm dlx`).
- **Coding-agent hook wiring**: extend session integrations beyond Claude Code,
  Codex, and OpenCode to Devin, Cursor, Continue, Aider (opt-in). Ship
  `--install-agent-hooks` / `--uninstall-agent-hooks` that wire the tool into
  the user's chosen agents. Read per-agent preferences from
  `${XDG_CONFIG_HOME:-$HOME/.config}/{app-name}/config.toml` — an
  `[agent-hooks]` table with `enabled = [...]`, the hook event, and
  `recommend = ["hooks", "cli", "mcp"]`. Defaults: `enabled =
  ["claude-code", "codex", "opencode"]`, `recommend = ["hooks", "cli"]`.
  Install is idempotent and explicit-opt-in only; portable commands use a
  PATH-verified binary name with absolute-path fallback; path repair updates
  the executable path after reinstall.
- **Non-user-identifiable telemetry**: collect only how the tool is called
  (subcommand verb, coarse exit-code category, timing bucket) — never file
  paths, package names, project names, user identifiers, command arguments, or
  any user-supplied input. Toggle precedence (highest to lowest): (1) universal
  opt-out env vars `DO_NOT_TRACK=1` and `DISABLE_TELEMETRY=1` — either disables
  telemetry regardless of any other setting; (2) tool-specific
  `MYTOOL_TELEMETRY=off|on` env var; (3) `[telemetry] enabled` in config.toml;
  (4) hardcoded default (off). No network call without consent and a configured
  endpoint; `--telemetry-dry-run` prints the exact payload without transmitting
  so users and CI can verify no identifying data leaks. `--version`/`--help`
  state whether telemetry is on and document the universal opt-out env vars.

### Error Format

```
ERROR: <description> - <suggestion>
```

### File References

Use VSCode-compatible format: `file:///absolute/path/to/file:line:column`

## Related Concepts

- [Quality Gates](quality-gates.md) — Testing CLI behavior
- [Container Support](container-support.md) — Health check for containers
