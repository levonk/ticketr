use clap::{Parser, Subcommand};
use crate::ticket::{TicketManager, CreateOptions};

#[derive(Parser)]
#[command(name = "tkr")]
#[command(about = "A ticket management system with dependency tracking and mono-repo support")]
pub struct Cli {
    #[arg(long = "tickets-dir", env = "TICKETS_DIR")]
    pub tickets_dir: Option<String>,

    #[arg(long = "repo-root", env = "REPO_ROOT")]
    pub repo_root: Option<String>,

    #[arg(long = "project", env = "TICKET_PROJECT")]
    pub project: Option<String>,

    #[arg(long = "category", env = "TICKET_CATEGORY")]
    pub category: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new ticket
    Create {
        title: String,
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        #[arg(long = "design")]
        design: Option<String>,
        #[arg(long = "acceptance")]
        acceptance: Option<String>,
        #[arg(short = 't', long = "type", default_value = "task")]
        issue_type: String,
        #[arg(short = 'p', long = "priority", default_value = "2")]
        priority: i32,
        #[arg(short = 'a', long = "assignee")]
        assignee: Option<String>,
        #[arg(long = "external-ref")]
        external_ref: Option<String>,
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
        #[arg(long, default_value = "false")]
        full: bool
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
        #[arg(long = "type")]
        issue_type: Option<String>,
        #[arg(long = "project")]
        project: Option<String>,
        #[arg(long = "category")]
        category: Option<String>,
    },
    /// List ready tickets (no open dependencies)
    Ready,
    /// List blocked tickets
    Blocked,
    /// List recently closed tickets
    Closed,
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
    /// Migrate from beads or bash tk format
    Migrate {
        #[arg(long, default_value = "auto")]
        from: String,
    },
    /// Start web server with kanban board
    Web {
        #[arg(long = "host", default_value = "127.0.0.1")]
        host: String,
        #[arg(long = "port", default_value = "8080")]
        port: u16,
    },
    /// Start terminal user interface (TUI)
    Tui,
}

impl Commands {
    pub async fn execute(self, manager: &mut TicketManager) -> anyhow::Result<()> {
        match self {
            Commands::Create {
                title,
                description,
                design,
                acceptance,
                issue_type,
                priority,
                assignee,
                external_ref,
                parent
            } => {
                let options = CreateOptions {
                    issue_type,
                    priority,
                    description,
                    design,
                    acceptance,
                    assignee,
                    external_ref,
                    parent,
                };
                manager.create_ticket(title, options)?;
            },
            Commands::Start { id } => {
                manager.update_status(&id, "in_progress")?;
            },
            Commands::Close { id } => {
                manager.update_status(&id, "closed")?;
            },
            Commands::Reopen { id } => {
                manager.update_status(&id, "open")?;
            },
            Commands::Status { id, status } => {
                manager.update_status(&id, &status)?;
            },
            Commands::Dep { id, dep_id } => {
                manager.add_dependency(&id, &dep_id)?;
            },
            Commands::DepTree { id: _, full: _ } => {
                eprintln!("Dependency tree command not yet implemented");
            },
            Commands::Undep { id, dep_id } => {
                manager.remove_dependency(&id, &dep_id)?;
            },
            Commands::Link { ids: _ } => {
                eprintln!("Link command not yet implemented");
            },
            Commands::Unlink { id: _, target_id: _ } => {
                eprintln!("Unlink command not yet implemented");
            },
            Commands::List {
                status: _,
                issue_type: _,
                project: _,
                category: _
            } => {
                let tickets = manager.list_tickets()?;
                if tickets.is_empty() {
                    println!("No tickets found");
                } else {
                    for ticket in tickets {
                        println!("{} - {} ({})", ticket.id, ticket.title, ticket.status);
                    }
                }
            },
            Commands::Ready => {
                eprintln!("Ready command not yet implemented");
            },
            Commands::Blocked => {
                eprintln!("Blocked command not yet implemented");
            },
            Commands::Closed => {
                eprintln!("Closed command not yet implemented");
            },
            Commands::Show { id } => {
                manager.show_ticket(&id)?;
            },
            Commands::Edit { id: _ } => {
                eprintln!("Edit command not yet implemented");
            },
            Commands::AddNote { id, note } => {
                let note_content = if note.is_empty() {
                    // Read from stdin
                    use std::io::Read;
                    let mut input = String::new();
                    std::io::stdin().read_to_string(&mut input)?;
                    input.trim().to_string()
                } else {
                    note.join(" ")
                };
                manager.add_note(&id, &note_content)?;
            },
            Commands::Query { filter: _ } => {
                eprintln!("Query command not yet implemented");
            },
            Commands::Migrate { from } => {
                manager.migrate_tickets(&from)?;
            },
            Commands::Web { host, port } => {
                crate::web::start_web_server(manager, host, port).await?;
            },
            Commands::Tui => {
                crate::tui::run_tui(manager).await?;
            },
        }
        Ok(())
    }
}
