# Agent Documentation: ticketr CLI Tool

This document provides detailed technical information for automated code assistants and developers working with the ticketr Rust CLI application.

## Quick Reference

- **Project Type**: Rust CLI application using clap for argument parsing
- **Build System**: Cargo with mise for environment management
- **Test Framework**: Built-in Rust testing with assert_cmd for CLI tests
- **Architecture**: Modular design with clear separation of concerns
- **File Format**: Markdown files with YAML frontmatter for ticket storage

## Repository Structure

```
apps/active/cli/ticketr/core/
├── Cargo.toml              # Dependencies and project metadata
├── src/
│   ├── main.rs             # Entry point and application initialization
│   ├── cli.rs              # CLI argument parsing and command execution
│   ├── ticket.rs           # Core ticket management logic and data structures
│   └── utils.rs            # Utility functions for path resolution
├── tests/
│   └── cli_tests.rs        # Comprehensive CLI integration tests
├── README.md               # User-facing documentation
└── AGENTS.md               # This file - technical reference
```

## Core Technical Details

### Module System

The codebase uses Rust's module system for organization:

- **main.rs**: Minimal entry point that initializes CLI and TicketManager
- **cli.rs**: Contains `Cli` struct with clap-derived parsing and `Commands` enum
- **ticket.rs**: Core business logic with `TicketManager` and data structures
- **utils.rs**: Path resolution and directory discovery utilities

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

### Building and Testing

```bash
# Build with mise (recommended)
mise exec -- cargo build

# Run tests
mise exec -- cargo test

# Run specific test with output
mise exec -- cargo test test_name -- --nocapture

# Build release version
mise exec -- cargo build --release
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
- `clap ^4.0`: CLI argument parsing with derive macros
- `serde ^1.0` + `serde_yaml ^0.9`: Serialization/deserialization
- `chrono ^0.4`: Date/time handling with UTC support
- `anyhow ^1.0`: Error handling with context
- `regex ^1.0`: Pattern matching for ID generation
- `uuid ^1.0`: Unique identifier generation

### Development Dependencies
- `assert_cmd ^2.0`: CLI testing framework
- `predicates ^3.0`: Test assertion predicates
- `tempfile ^3.0`: Temporary directory creation for tests

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

## Future Extensibility

The modular architecture supports easy addition of:
- New commands and subcommands
- Additional metadata fields
- Alternative storage backends
- Plugin system for custom workflows
- Integration with external ticket systems
