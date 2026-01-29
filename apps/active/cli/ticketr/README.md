# ticketr - Rust CLI Ticket Management System

A modern Rust implementation of a ticket management system with dependency tracking and mono-repo support, ported from the original `tk` bash script.

## Purpose

`ticketr` is a command-line tool for managing tickets stored as markdown files with YAML frontmatter. It's designed for developers who want a lightweight, git-friendly ticket system that integrates seamlessly with their workflow.

### Key Features

- **Markdown-based tickets** - Human-readable files stored in `.tickets/` directory
- **YAML frontmatter** - Structured metadata for easy querying and parsing
- **Dependency tracking** - Link tickets together with dependency relationships
- **Mono-repo support** - Tag tickets with project and category for organization
- **Partial ID matching** - Use short prefixes to reference tickets quickly
- **Status management** - Track ticket states (open, in_progress, closed, blocked, ready)
- **Note system** - Add timestamped notes to tickets
- **CLI-driven** - Full command-line interface with comprehensive options

## Architecture

### Module Structure

The codebase is organized into clear, focused modules:

```
src/
├── main.rs      # Entry point and application initialization
├── cli.rs       # CLI argument parsing and command execution
├── ticket.rs    # Core ticket management logic and data structures
└── utils.rs     # Utility functions for path resolution
```

### Core Components

#### TicketManager (`src/ticket.rs`)
The heart of the system responsible for:
- Ticket creation, loading, and saving
- ID generation with unique timestamps
- Dependency management
- Note handling
- Directory management

#### CLI Interface (`src/cli.rs`)
Command-line interface built with `clap`:
- Argument parsing and validation
- Command routing and execution
- Help system
- Environment variable support

#### Data Structures

```rust
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: String,
    pub deps: Vec<String>,
    pub links: Vec<String>,
    pub created: DateTime<Utc>,
    pub issue_type: String,
    pub priority: i32,
    pub description: Option<String>,
    pub design: Option<String>,
    pub acceptance: Option<String>,
    pub assignee: Option<String>,
    pub external_ref: Option<String>,
    pub parent: Option<String>,
    pub project: Option<String>,
    pub category: Option<String>,
    pub notes: Option<Vec<Note>>,
}
```

## Build Process

### Prerequisites

- Rust 1.70+ (for modern Rust features)
- `mise` for environment management
- Git repository (for ticket ID generation)

### Building

```bash
# Using mise for environment management
mise install
mise exec -- cargo build

# Or directly with cargo
cargo build --release
```

### Testing

```bash
# Run all tests
mise exec -- cargo test

# Run specific test
mise exec -- cargo test test_name

# Run with output
mise exec -- cargo test -- --nocapture
```

### Installation

```bash
# Install to local bin
cargo install --path .

# The binary will be named `tk` (matching the original script)
```

## Usage

### Basic Commands

```bash
# Create a ticket
tk create "Fix login bug" --description="Users cannot login with SSO"

# List all tickets
tk list

# Update ticket status
tk start ja-1234
tk close ja-1234

# Add dependencies
tk dep ja-1235 ja-1234

# Add notes
tk add-note ja-1234 "Fixed the authentication flow"

# Show ticket details
tk show ja-1234
```

### Mono-repo Features

```bash
# Create ticket with project and category tags
tk create "Add API endpoint" --project=backend --category=api

# List tickets for specific project
tk list --project=backend

# Use environment variables for defaults
export TICKET_PROJECT=backend
export TICKET_CATEGORY=api
tk create "Update database schema"
```

### Advanced Usage

```bash
# Custom tickets directory
tk --tickets-dir /path/to/custom/tickets list

# Custom repository root
tk --repo-root /path/to/repo create "New feature"

# Combined usage
tk --project=frontend --category=ui --tickets-dir ./tickets create "Fix button"
```

## Configuration

### Environment Variables

- `TICKETS_DIR` - Path to tickets directory (default: `.tickets`)
- `REPO_ROOT` - Path to repository root (for auto-discovery)
- `TICKET_PROJECT` - Default project tag
- `TICKET_CATEGORY` - Default category tag

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
description: Users cannot login with SSO
project: backend
category: auth
---

# Fix login bug

Users cannot login with SSO due to token validation issue.

## Notes

**2023-01-01 12:30:00**: Investigating the token validation flow
**2023-01-01 14:15:00**: Found the issue in the middleware
```

## Development

### Adding New Commands

1. Add the command variant to `Commands` enum in `src/cli.rs`
2. Implement the command logic in the `execute` method
3. Add corresponding methods to `TicketManager` if needed
4. Write tests in `tests/cli_tests.rs`

### Testing Strategy

- Unit tests for core functionality in `tests/cli_tests.rs`
- Integration tests using temporary directories
- CLI testing with `assert_cmd` crate
- File system validation with `tempfile` crate

### Code Style

- Modular architecture with clear separation of concerns
- Comprehensive error handling with `anyhow`
- Type-safe data structures with `serde`
- Modern Rust patterns and idioms

## Migration from Original `tk`

This Rust implementation maintains compatibility with the original `tk` bash script while adding:

- **Type safety** - Compile-time error checking
- **Performance** - Faster execution and lower overhead
- **Maintainability** - Modular, testable codebase
- **Extensibility** - Easy to add new features
- **Cross-platform** - Works on Windows, macOS, and Linux

The file format and basic commands remain compatible, allowing seamless migration.

## Dependencies

### Core Dependencies

- `clap` - Command-line argument parsing
- `serde` + `serde_yaml` - Serialization/deserialization
- `chrono` - Date/time handling
- `regex` - Pattern matching for ID generation
- `uuid` - Unique identifier generation
- `anyhow` - Error handling

### Development Dependencies

- `assert_cmd` - CLI testing
- `predicates` - Test assertions
- `tempfile` - Temporary file/directory creation
- `tokio-test` - Async testing support

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the GNU AGPL-3.0 License.
