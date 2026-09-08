# tkr

[![Nix Flake](https://img.shields.io/badge/Nix-flake-blue.svg)](https://github.com/levonk/tkr)
[![Devbox](https://img.shields.io/badge/Devbox-ready-green.svg)](https://github.com/levonk/tkr)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern Rust CLI ticket management system with dependency tracking and mono-repo support, ported from the original `tk` bash script. Tickets are stored as markdown files with YAML frontmatter, making them human-readable and git-friendly.

## Quick Install

```bash
# Nix (recommended)
nix profile install github:levonk/tkr

# Devbox
devbox add github:levonk/tkr

# Run immediately
nix run github:levonk/tkr -- --help
```

## Quick Start

```bash
# Clone and enter project
git clone https://github.com/levonk/tkr.git
cd tkr

# direnv auto-activates devbox environment
# If not auto-activated, run:
direnv allow
source .envrc

# Bootstrap environment
just bootstrap

# Build
just build

# Run tests
just test
```

## Build and Test Commands

This project uses **devbox + direnv + just** following the Standard Developer UX Flow (ADR-20260131001).

```bash
just build       # Build the project (release mode)
just test        # Run all tests
just lint        # Run clippy lints
just typecheck   # Run cargo check
just dev         # Run in development mode
just doctor      # Check environment health
just quality     # Run lint + test + typecheck
just clean       # Clean build artifacts
```

**Note**: All `just` targets auto-detect the devbox environment. AI agents in automated contexts can use `devbox run just build_impl` directly.

## Project Structure

```
.
├── src/
│   ├── main.rs        # Entry point (async, Tokio)
│   ├── cli.rs          # CLI argument parsing (clap) and command execution
│   ├── ticket.rs       # Core ticket management logic and data structures
│   ├── utils.rs        # Path resolution and directory discovery
│   ├── tui.rs          # Terminal User Interface (ratatui)
│   └── web.rs          # Web API server (Warp)
├── tests/
│   └── cli_tests.rs    # CLI integration tests (assert_cmd)
├── web/                # Web UI assets and templates
├── .tickets/           # Default ticket storage directory
├── justfile            # Command runner recipes
├── devbox.json         # Devbox environment config
├── Cargo.toml          # Rust project dependencies and metadata
├── flake.nix           # Nix flake for reproducible builds
├── Dockerfile          # Container configuration
└── docker-compose.yml  # Container orchestration
```

## Usage

### Basic Commands

```bash
# Create a ticket
tkr create "Fix login bug" --description="Users cannot login with SSO"

# List all tickets
tkr list

# Update ticket status
tkr start ja-1234
tkr close ja-1234

# Add dependencies
tkr dep ja-1235 ja-1234

# Add notes
tkr add-note ja-1234 "Fixed the authentication flow"

# Show ticket details
tkr show ja-1234
```

### Mono-repo Features

```bash
# Create ticket with project and category tags
tkr create "Add API endpoint" --project=backend --category=api

# List tickets for specific project
tkr list --project=backend

# Use environment variables for defaults
export TICKET_PROJECT=backend
export TICKET_CATEGORY=api
tkr create "Update database schema"
```

### Interface Modes

```bash
# CLI mode (default)
tkr create "New feature"

# TUI mode (interactive terminal interface)
tkr tui

# Web mode (HTTP API server)
tkr web --port=8080
```

## Configuration

### Environment Variables

- `TICKETS_DIR` - Path to tickets directory (default: `.tickets`)
- `REPO_ROOT` - Path to repository root (for auto-discovery)
- `TICKET_PROJECT` - Default project tag for new tickets
- `TICKET_CATEGORY` - Default category tag for new tickets

### Ticket File Format

Tickets are stored as markdown files with YAML frontmatter:

```markdown
---
id: ja-1234
title: Fix login bug
status: in_progress
deps: []
links: []
created: 2023-01-01T12:00:00Z
type: task
priority: 2
project: backend
category: auth
---

# Fix login bug

Users cannot login with SSO due to token validation issue.

## Notes

**2023-01-01 12:30:00**: Investigating the token validation flow
```

## AI Agent Documentation

For AI assistants working on this project, see [AGENTS.md](AGENTS.md) for comprehensive agent-specific workflows, guidelines, and the developer guide.

For information about what this project does NOT do, see [internal-docs/oos/](internal-docs/oos/).

## Installation Options

### Nix (Recommended)

```bash
# Install from the flake registry
nix profile install github:levonk/tkr

# Or build and run directly
nix run github:levonk/tkr -- --help

# For development environments
nix develop github:levonk/tkr
```

### Devbox

```bash
# Add tkr to your existing devbox environment
devbox add github:levonk/tkr

# Or create a new project with tkr
mkdir my-project && cd my-project
devbox init
devbox add github:levonk/tkr
devbox shell

# Verify installation
tkr --help
```

### Cargo

```bash
# Install to local bin
cargo install --path .

# The binary will be named tkr
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality (TDD approach)
4. Ensure all tests pass: `just test`
5. Ensure linting passes: `just lint`
6. Submit a pull request

## License

This project is licensed under the MIT License — see `Cargo.toml` for the full license declaration.
