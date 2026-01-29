use clap::{Parser, Subcommand};
use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
mod ticket;
mod cli;
mod utils;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use ticket::TicketManager;
use cli::{Cli, Commands};
use utils::find_tickets_dir;

#[derive(Parser)]
#[command(name = "tk", version, about = "A ticket management system with dependency tracking and mono-repo support")]
#[command(author = "Levon K")]
struct Cli {
    /// Path to the .tickets directory (default: .tickets)
    #[arg(long = "tickets-dir", env = "TICKETS_DIR")]
    tickets_dir: Option<String>,

    /// Path to repository root (for finding .tickets/ if not specified)
    #[arg(long = "repo-root", env = "REPO_ROOT")]
    repo_root: Option<PathBuf>,

    /// Project tag for mono-repo organization
    #[arg(long = "project", env = "TICKET_PROJECT")]
    project: Option<String>,

    /// Category tag for mono-repo organization
    #[arg(long = "category", env = "TICKET_CATEGORY")]
    category: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new ticket
    Create {
        /// Ticket title
        title: String,
        /// Description
        #[arg(long = "description", short = 'd')]
        description: Option<String>,
        /// Design notes
        #[arg(long = "design")]
        design: Option<String>,
        /// Acceptance criteria
        #[arg(long = "acceptance")]
        acceptance: Option<String>,
        /// Issue type
        #[arg(long = "type", short = 't', default_value = "task")]
        issue_type: String,
        /// Priority (0-4, 0=highest)
        #[arg(long = "priority", short = 'p', default_value = "2")]
        priority: u8,
        /// Assignee
        #[arg(long = "assignee", short = 'a')]
        assignee: Option<String>,
        /// External reference
        #[arg(long = "external-ref")]
        external_ref: Option<String>,
        /// Parent ticket ID
        #[arg(long = "parent")]
        parent: Option<String>,
    },
    /// Set ticket status to in_progress
    Start { id: String },
    /// Set ticket status to closed
    Close { id: String },
    /// Set ticket status to open
    Reopen { id: String },
    /// Update ticket status
    Status { id: String, status: String },
    /// Add dependency
    Dep { id: String, dep_id: String },
    /// Show dependency tree
    DepTree {
        id: String,
        #[arg(long)]
        full: bool,
    },
    /// Remove dependency
    Undep { id: String, dep_id: String },
    /// Link tickets together
    Link { ids: Vec<String> },
    /// Remove link between tickets
    Unlink { id: String, target_id: String },
    /// List tickets
    List {
        #[arg(long = "status")]
        status: Option<String>,
    },
    /// List ready tickets (no open dependencies)
    Ready,
    /// List blocked tickets
    Blocked,
    /// List recently closed tickets
    Closed {
        #[arg(long = "limit", default_value = "20")]
        limit: usize,
    },
    /// Show ticket details
    Show { id: String },
    /// Edit ticket in $EDITOR
    Edit { id: String },
    /// Add note to ticket
    AddNote {
        id: String,
        #[arg(trailing_var_arg = true)]
        note: Vec<String>,
    },
    /// Query tickets as JSON
    Query {
        #[arg(default_value = ".")]
        filter: String,
    },
    /// Import from beads format
    MigrateBeads,
}

impl Commands {
    fn execute(&self, manager: &mut TicketManager) -> Result<()> {
        match self {
            Commands::Create { title, description, design, acceptance, issue_type, priority, assignee, external_ref, parent } => {
                let options = CreateOptions {
                    description: description.clone(),
                    design: design.clone(),
                    acceptance: acceptance.clone(),
                    issue_type: issue_type.clone(),
                    priority: *priority,
                    assignee: assignee.clone(),
                    external_ref: external_ref.clone(),
                    parent: parent.clone(),
                };
                manager.create_ticket(title.clone(), options)?;
            },
            Commands::Start { id } => {
                manager.update_status(id, "in_progress")?;
            },
            Commands::Close { id } => {
                manager.update_status(id, "closed")?;
            },
            Commands::Reopen { id } => {
                manager.update_status(id, "open")?;
            },
            Commands::Status { id, status } => {
                manager.update_status(id, status)?;
            },
            Commands::Dep { id, dep_id } => {
                manager.add_dependency(id, dep_id)?;
            },
            Commands::DepTree { id, full: _ } => {
                eprintln!("Dependency tree visualization not yet implemented");
            },
            Commands::Undep { id, dep_id } => {
                manager.remove_dependency(id, dep_id)?;
            },
            Commands::Link { ids } => {
                eprintln!("Link command not yet implemented");
            },
            Commands::Unlink { id, target_id } => {
                eprintln!("Unlink command not yet implemented");
            },
            Commands::List { status } => {
                manager.list_tickets(status.as_deref())?;
            },
            Commands::Ready => {
                eprintln!("Ready command not yet implemented");
            },
            Commands::Blocked => {
                eprintln!("Blocked command not yet implemented");
            },
            Commands::Closed { limit: _ } => {
                eprintln!("Closed command not yet implemented");
            },
            Commands::Show { id } => {
                manager.show_ticket(id)?;
            },
            Commands::Edit { id } => {
                eprintln!("Edit command not yet implemented");
            },
            Commands::AddNote { id, note } => {
                let note_content = if note.is_empty() {
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer)?;
                    buffer
                } else {
                    note.join(" ")
                };
                manager.add_note(id, note_content)?;
            },
            Commands::Query { filter: _ } => {
                eprintln!("Query command not yet implemented");
            },
            Commands::MigrateBeads => {
                eprintln!("Migrate beads command not yet implemented");
            },
        }
        Ok(())
    }
}

#[derive(Debug)]
struct CreateOptions {
    description: Option<String>,
    design: Option<String>,
    acceptance: Option<String>,
    issue_type: String,
    priority: u8,
    assignee: Option<String>,
    external_ref: Option<String>,
    parent: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine tickets directory
    let tickets_dir = if let Some(dir) = cli.tickets_dir {
        PathBuf::from(dir)
    } else {
        find_tickets_dir(cli.repo_root)?
    };

    // Create ticket manager
    let mut manager = TicketManager::new(
        tickets_dir,
        cli.project.clone(),
        cli.category.clone(),
    );

    // Execute command
    cli.command.execute(&mut manager)?;

    Ok(())
}
  reopen <id>                 Set status to open
  status <id> <status>        Update status (open|in_progress|closed)
  dep <id> <dep-id>           Add dependency (id depends on dep-id)
  dep-tree <id>               Show dependency tree
  undep <id> <dep-id>         Remove dependency
  link <id> <id> [id...]      Link tickets together (symmetric)
  unlink <id> <target-id>     Remove link between tickets
  ls [--status=X]             List tickets
  ready                       List open/in-progress tickets with deps resolved
  blocked                     List open/in-progress tickets with unresolved deps
  closed [--limit=N]          List recently closed tickets (default 20)
  show <id>                   Display ticket
  edit <id>                   Open ticket in $EDITOR
  add-note <id> [text]        Append timestamped note (or pipe via stdin)
  query [jq-filter]           Output tickets as JSON, optionally filtered
  migrate-beads               Import tickets from .beads/issues.jsonl
  help                        Show this help message

Tickets stored as markdown files in .tickets/
Supports partial ID matching (e.g., 'tk show 5c4' matches 'nw-5c46')
");
}
