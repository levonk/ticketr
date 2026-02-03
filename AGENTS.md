# Agent Documentation: ticketr CLI Tool

## Overview

**ticketr** is a modern Rust CLI ticket management system designed for developers who want a lightweight, git-friendly ticket system that integrates seamlessly with their workflow. It's a complete rewrite of the original `tk` bash script in Rust, providing type safety, better performance, and enhanced features.

### What ticketr Does

- **Manages tickets as markdown files** with YAML frontmatter for human-readable storage
- **Tracks dependencies** between tickets to manage complex workflows
- **Supports mono-repo organization** with project and category tagging
- **Provides CLI interface** for all ticket operations (create, update, list, etc.)
- **Integrates with git** for version control and collaboration
- **Offers multiple interfaces** including CLI, TUI, and web UI

### Key Features

- **Markdown-based tickets** - Human-readable files stored in `.tickets/` directory
- **YAML frontmatter** - Structured metadata for easy querying and parsing
- **Dependency tracking** - Link tickets together with dependency relationships
- **Mono-repo support** - Tag tickets with project and category for organization
- **Partial ID matching** - Use short prefixes to reference tickets quickly
- **Status management** - Track ticket states (open, in_progress, closed, blocked, ready)
- **Note system** - Add timestamped notes to tickets
- **Multiple interfaces** - CLI, TUI (Terminal UI), and Web UI
- **Async support** - Built on Tokio for concurrent operations

## Quick Reference

- **Project Type**: Rust CLI application using clap for argument parsing
- **Build System**: Cargo with mise for environment management
- **Test Framework**: Built-in Rust testing with assert_cmd for CLI tests
- **Architecture**: Modular design with clear separation of concerns
- **File Format**: Markdown files with YAML frontmatter for ticket storage
- **Runtime**: Async with Tokio, supports TUI and Web interfaces

## Repository Structure

```
apps/active/cli/ticketr/
├── Cargo.toml              # Dependencies and project metadata
├── src/
│   ├── main.rs             # Entry point and application initialization
│   ├── cli.rs              # CLI argument parsing and command execution
│   ├── ticket.rs           # Core ticket management logic and data structures
│   ├── utils.rs            # Utility functions for path resolution
│   ├── tui.rs              # Terminal User Interface (TUI) implementation
│   └── web.rs              # Web API and UI server
├── tests/
│   └── cli_tests.rs        # Comprehensive CLI integration tests
├── web/                    # Web UI assets and templates
├── .tickets/               # Default ticket storage directory
├── README.md               # User-facing documentation
├── AGENTS.md               # This file - technical reference
├── Makefile                # Build and development commands
├── Dockerfile              # Container configuration
└── docker-compose.yml      # Container orchestration
```

## Core Technical Details

### Module System

The codebase uses Rust's module system for organization with async support:

- **main.rs**: Async entry point that initializes CLI and TicketManager using Tokio
- **cli.rs**: Contains `Cli` struct with clap-derived parsing and `Commands` enum
- **ticket.rs**: Core business logic with `TicketManager` and data structures
- **utils.rs**: Path resolution and directory discovery utilities
- **tui.rs**: Terminal User Interface using ratatui for interactive ticket management
- **web.rs**: Web API server using Warp framework for HTTP interface

### Key Data Structures

#### Ticket Structure
```rust
pub struct Ticket {
    pub id: String,                    // Unique identifier (e.g., "ja-1234")
    pub title: String,                 // Human-readable title
    pub status: String,                // Current status
    pub deps: Vec<String>,             // Dependency ticket IDs
    pub links: Vec<String>,            // Linked ticket IDs
    pub created: DateTime<Utc>,        // Creation timestamp
    pub issue_type: String,            // Type classification
    pub priority: i32,                 // Priority level (1-5)
    // Optional fields for extended metadata
    pub description: Option<String>,
    pub design: Option<String>,
    pub acceptance: Option<String>,
    pub assignee: Option<String>,
    pub external_ref: Option<String>,
    pub parent: Option<String>,
    pub project: Option<String>,       // Mono-repo project tag
    pub category: Option<String>,      // Mono-repo category tag
    pub notes: Option<Vec<Note>>,      // Timestamped notes
}
```

#### Note Structure
```rust
pub struct Note {
    pub timestamp: DateTime<Utc>,       // When note was added
    pub content: String,                // Note content
}
```

### ID Generation Algorithm

Ticket IDs are generated using:
1. Directory name analysis to create prefix
2. Timestamp (milliseconds since Unix epoch)
3. UUID suffix for uniqueness

Format: `{prefix}-{timestamp_suffix}{uuid_suffix}`

Example: `ja-6b9a0dc` where `ja` is derived from directory name.

### File Storage Format

Tickets are stored as markdown files with YAML frontmatter:

```markdown
---
id: ja-1234
title: Example Ticket
status: open
deps: []
links: []
created: 2023-01-01T12:00:00Z
type: task
priority: 2
project: backend
category: api
---

# Example Ticket

Ticket description goes here.

## Notes

**2023-01-01 12:30:00**: Initial investigation
**2023-01-01 14:15:00**: Implementation complete
```

## Development Guidelines

### AI Agent Workflow Loop

This section describes the systematic workflow for AI agents working with ticketr to ensure consistent, high-quality development cycles.

#### The AI Development Loop

##### Core Philosophy

The AI Development Loop is designed to:

- **Maintain Context**: Always know what you're working on and why
- **Ensure Quality**: Verify work before marking it complete
- **Track Progress**: Clear ticket states prevent work from falling through cracks
- **Enable Collaboration**: Proper documentation and notes help other agents understand your work

##### The 7-Step Loop

```bash
# 0. Assure work is starting from clean foundation
- Assure the typechecks, lint, build, test, and security checks pass
- Assure the repository is clean, and everything is committed
- Assure you're on the latest version of the repository
- Assure you're on a working branch

# 1. Grab the next ticket
tkr ready                    # Get next actionable ticket
# OR
tkr list --status=open       # See all open tickets and choose one

# 2. Start work on the ticket
tkr start <ticket-id>         # Mark as in_progress

# 3. Do the work
# - Add tests as needed
# - Implement the required changes
# - Follow coding standards and best practices
# - Update documentation

# 4. Verify the work
just test                     # Run tests
just lint                     # Run linting
just typecheck                # Run type checking
# OR manual testing of the feature

# 5. Complete the ticket
tkr close <ticket-id>         # Mark as completed
# OR
tkr ready <ticket-id>         # If ready for review/merge

# 6. Commit the changes

# 7. Loop again - grab the next ticket
tkr ready                    # Back to step 1
```

##### Workflow Principles

**1. Atomic Operations**

- Update ticket status immediately when starting work
- Update ticket status immediately when completing work
- Never leave tickets in ambiguous states

**2. Verification First**

- Always run tests before marking a ticket complete
- Ensure all linting passes
- Verify the implementation meets acceptance criteria

**3. Documentation Updates**

- Update relevant documentation as part of the work
- Add notes to tickets explaining what was done
- Keep README.md and AGENTS.md in sync

**4. Quality Gates**

- All tests must pass before completion
- Code must follow established patterns
- Dependencies must be properly resolved

##### Ticket Status Flow

```
open → in_progress → ready → closed
  ↑         ↓           ↓
  └─────── ready ←─────┘
```

- **open**: Ready to start work
- **in_progress**: Currently being worked on
- **ready**: Work complete, ready for review
- **closed**: Fully completed and verified

##### Best Practices

**Starting Work:**

```bash
# Always start with a ready ticket
tkr ready                    # Get next available ticket
tkr start <ticket-id>         # Begin work immediately

# Add a note about what you're doing
tkr add-note <ticket-id> "Starting implementation of feature X"
```

**Completing Work:**

```bash
# Verify everything works
just test && just lint        # Quality gates

# Add completion notes
tkr add-note <ticket-id> "Implementation complete. Tests passing."

# Mark as ready for review
tkr ready <ticket-id>         # Or close if fully verified
```

**Handling Dependencies:**

```bash
# Check if dependencies are resolved
tkr show <ticket-id>         # Review dependencies
tkr ready                    # See if blocked tickets are ready

# Add dependencies if needed
tkr dep <ticket-id> <dep-id>  # Add dependency relationship
```

##### Continuous Improvement

The AI agent should:

1. **Reflect** on each completed cycle
2. **Learn** from any issues encountered
3. **Improve** the process for next iteration
4. **Document** any new patterns or decisions

This workflow ensures consistent, high-quality development while maintaining clear ticket state tracking throughout the process.

### Building and Testing

```bash
# Build with justfile (recommended)
just build

# Run tests
just test

# Run specific test with output
just test-inner test_name -- --nocapture

# Build debug version
just debug

# Build release version
just build
```

### Adding New Commands

1. **Update CLI enum** in `src/cli.rs`:
```rust
#[derive(Subcommand)]
pub enum Commands {
    // Existing commands...
    /// Your new command description
    NewCommand {
        arg1: String,
        #[arg(long)]
        optional_arg: Option<String>,
    },
}
```

2. **Implement command logic** in `execute` method:
```rust
Commands::NewCommand { arg1, optional_arg } => {
    // Your implementation
    manager.your_method(arg1, optional_arg)?;
},
```

3. **Add TicketManager method** if needed in `src/ticket.rs`:
```rust
impl TicketManager {
    pub fn your_method(&self, arg1: String, arg2: Option<String>) -> Result<()> {
        // Implementation
    }
}
```

4. **Write tests** in `tests/cli_tests.rs`:
```rust
#[test]
fn test_new_command() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tk").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("new-command")
        .arg("test-arg")
        .assert()
        .success();
}
```

### Error Handling

- Use `anyhow::Result<T>` for error propagation
- Provide context with `anyhow::anyhow!()` macro
- Chain errors with `.context()` method
- Handle file I/O errors gracefully

### Testing Patterns

#### CLI Testing Pattern
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_command_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tk").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("command")
        .arg("argument")
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

#### File Creation Testing
```rust
// Create test ticket file
let ticket_content = r#"---
id: test-123
title: Test Ticket
status: open
---
# Test Ticket
Description
"#;
fs::write(tickets_dir.join("test-123.md"), ticket_content).unwrap();
```

## Common Operations

### Ticket Creation Flow
1. Parse CLI arguments
2. Generate unique ID using `generate_id()`
3. Create `Ticket` struct with defaults
4. Apply CLI options (description, project, category, etc.)
5. Save using `save_ticket()` which formats as markdown with YAML frontmatter

### Ticket Loading Flow
1. Resolve ticket ID using `ticket_path()` (supports partial matching)
2. Read file content
3. Split YAML frontmatter from markdown content
4. Deserialize YAML into `Ticket` struct
5. Return loaded ticket

### Dependency Management
- Dependencies stored as `Vec<String>` of ticket IDs
- No validation of dependency existence (by design for flexibility)
- Add/remove operations modify the dependency list directly

## Configuration and Environment

### Environment Variables
- `TICKETS_DIR`: Override default tickets directory path
- `REPO_ROOT`: Specify repository root for auto-discovery
- `TICKET_PROJECT`: Default project tag for new tickets
- `TICKET_CATEGORY`: Default category tag for new tickets

### Path Resolution Priority
1. `--tickets-dir` CLI argument
2. `TICKETS_DIR` environment variable
3. Auto-discovery from current directory up to git root
4. Fallback to `.tickets` in current directory

## Performance Considerations

- File I/O is synchronous for simplicity
- Ticket listing loads all tickets into memory
- ID generation uses system time and UUID for uniqueness
- YAML parsing happens on every ticket load

## Security Considerations

- No external network calls
- File access limited to tickets directory
- No privilege escalation
- Input validation through clap's type system

## Dependencies Architecture

### Core Dependencies
- `clap ^4.4`: CLI argument parsing with derive macros and environment variable support
- `serde ^1.0` + `serde_yaml ^0.9`: Serialization/deserialization
- `chrono ^0.4`: Date/time handling with UTC support and serde integration
- `anyhow ^1.0`: Error handling with context
- `thiserror ^1.0`: Custom error type definitions
- `regex ^1.0`: Pattern matching for ID generation
- `uuid ^1.0`: Unique identifier generation with v4 support
- `tokio ^1.0`: Async runtime with full features
- `warp ^0.3`: Web framework for HTTP API server
- `ratatui ^0.24`: Terminal User Interface framework
- `crossterm ^0.27`: Cross-platform terminal manipulation
- `directories ^5.0`: Project directory discovery
- `ctrlc ^3.4`: Signal handling for graceful shutdown
- `indicatif ^0.17`: Progress bars and spinners
- `glob ^0.3`: File pattern matching
- `atty ^0.2`: Terminal detection
- `serde_json ^1.0`: JSON serialization for web API
- `shell-escape ^0.1`: Shell command escaping
- `url ^2.0`: URL parsing and manipulation

### Development Dependencies
- `assert_cmd ^2.0`: CLI testing framework
- `predicates ^3.0`: Test assertion predicates
- `tempfile ^3.8`: Temporary directory creation for tests

## Migration Notes

### From Original `tk` Script
- Compatible file format and basic commands
- Enhanced with type safety and better error handling
- Added mono-repo support with project/category tags
- Improved ID generation to prevent collisions
- Better cross-platform compatibility

### Breaking Changes
- None - maintains compatibility with existing ticket files
- CLI arguments are backward compatible
- Environment variables follow same naming convention

## Troubleshooting

### Common Issues

1. **Ticket ID Collisions**: Fixed with improved ID generation using timestamp + UUID
2. **Test Failures**: Ensure proper temporary directory setup and cleanup
3. **Path Resolution**: Check environment variables and directory permissions
4. **YAML Parsing**: Verify frontmatter format and proper indentation
5. **Clippy Warnings**: All lint issues resolved with `#[allow(dead_code)]` attributes and modern Rust patterns

### Recent Fixes

- **Dead Code**: Added `#[allow(dead_code)]` to `get_git_user()` and `get_repo_root()` functions
- **Clippy Lints**: Fixed needless borrows and unnecessary map_or warnings
- **Code Quality**: Clean, lint-free codebase that passes strict clippy checks

### Debug Mode
Set `RUST_LOG=debug` environment variable for detailed logging output.

## Interface Modes

### CLI Mode (Default)
Traditional command-line interface for scripting and automation:
```bash
tkr create "Fix login bug"
tkr list --project=backend
tkr start ja-1234
```

### TUI Mode (Terminal UI)
Interactive terminal interface using ratatui:
```bash
tkr tui
```
Features:
- Interactive ticket browsing and editing
- Real-time filtering and search
- Keyboard navigation
- Status updates with visual feedback

### Web Mode
HTTP API server with web interface:
```bash
tkr web --port=8080
```
Features:
- RESTful API endpoints
- Web-based ticket management
- Real-time updates
- Mobile-friendly interface

## Container Support

### Docker Configuration
The application includes Docker support for consistent deployment:

```dockerfile
# Minimal Dockerfile for containerized deployment
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tkr /usr/local/bin/
CMD ["tkr"]
```

### Docker Compose
```yaml
version: '3.8'
services:
  ticketr:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./tickets:/app/.tickets
    environment:
      - TICKETS_DIR=/app/.tickets
```

## Performance Considerations

- **Async Operations**: Built on Tokio for concurrent file I/O and network operations
- **Memory Usage**: Ticket listing loads all tickets into memory (suitable for <10k tickets)
- **File I/O**: Synchronous for simplicity, but can be made async for large datasets
- **ID Generation**: Uses system time and UUID for uniqueness without collisions
- **YAML Parsing**: Happens on every ticket load, cached in memory during session

## Security Considerations

- **No External Network Calls**: All operations are local filesystem-based
- **File Access**: Limited to configured tickets directory and subdirectories
- **No Privilege Escalation**: Runs with current user permissions
- **Input Validation**: Through clap's type system and serde validation
- **Path Traversal Protection**: Validates all file paths are within tickets directory

## Future Extensibility

The modular architecture supports easy addition of:
- **New Commands**: CLI subcommands for specialized workflows
- **Additional Metadata**: Custom fields for domain-specific requirements
- **Alternative Storage**: Database backends (SQLite, PostgreSQL) for team collaboration
- **Plugin System**: Custom workflows and integrations
- **External Integrations**: GitHub Issues, Jira, Linear, etc.
- **Real-time Collaboration**: WebSocket-based multi-user editing
- **Advanced Search**: Full-text search with indexing
- **Reporting**: Analytics and burndown charts

## Binary Name

The compiled binary is named `tkr` (not `tk`) to avoid conflicts with the original bash script while maintaining recognizability. This allows both tools to coexist during migration.
