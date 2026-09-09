use clap::{Parser, Subcommand};
use crate::db::{PortfolioDb, Portfolio, PortfolioSubcommand, ProjectSubcommand, AppSubcommand, RequirementSubcommand, Requirement, StorySubcommand, Story};
use crate::sync::SyncManager;
use crate::ticket::{TicketManager, CreateOptions};
use crate::utils::detect_github_info;

/// Subcommands for `tkr tag`.
#[derive(Subcommand)]
pub enum TagSubcommand {
    /// Add a tag to a task (writes to markdown frontmatter)
    Add {
        task_id: String,
        tag: String,
    },
    /// Remove a tag from a task (writes to markdown frontmatter)
    Remove {
        task_id: String,
        tag: String,
    },
    /// List all tags, or tags with their associated task IDs
    List {
        /// Show each tag with its associated task IDs
        #[arg(long)]
        tasks: bool,
    },
}

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
    /// Alias for 'list' command
    Ls {
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
    /// Display version and build information
    Version,
    /// Start web server with kanban board
    Web {
        #[arg(long = "host", default_value = "127.0.0.1")]
        host: String,
        #[arg(long = "port", default_value = "8080")]
        port: u16,
    },
    /// Start terminal user interface (TUI)
    Tui,
    /// Portfolio management
    Portfolio {
        #[command(subcommand)]
        command: PortfolioSubcommand,
    },
    /// Project management
    Project {
        #[command(subcommand)]
        command: ProjectSubcommand,
    },
    /// App management
    App {
        #[command(subcommand)]
        command: AppSubcommand,
    },
    /// Requirement management
    Requirement {
        #[command(subcommand)]
        command: RequirementSubcommand,
    },
    /// Story management
    Story {
        #[command(subcommand)]
        command: StorySubcommand,
    },
    /// Sync markdown tickets into the SQLite portfolio DB
    Sync {
        /// Sync from GitHub (stub — not yet implemented)
        #[arg(long)]
        github: bool,
        /// Show sync status instead of running a sync
        #[arg(long)]
        status: bool,
    },
    /// Tag management (cross-cutting task tags)
    Tag {
        #[command(subcommand)]
        command: TagSubcommand,
    },
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
                manager.start_ticket(&id)?;
            },
            Commands::Close { id } => {
                manager.close_ticket(&id)?;
            },
            Commands::Reopen { id } => {
                manager.reopen_ticket(&id)?;
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
            Commands::Ls {
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
                let tickets = manager.list_ready_tickets()?;
                if tickets.is_empty() {
                    println!("No ready tickets found");
                } else {
                    for ticket in tickets {
                        println!("{} - {} ({})", ticket.id, ticket.title, ticket.status);
                    }
                }
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
            Commands::Version => {
                println!("tkr {}", env!("CARGO_PKG_VERSION"));
                println!("A ticket management system with dependency tracking and mono-repo support");
            },
            Commands::Web { host, port } => {
                crate::web::start_web_server(manager, host, port).await?;
            },
            Commands::Tui => {
                crate::tui::run_tui(manager).await?;
            },
            Commands::Portfolio { command } => {
                let db_path = PortfolioDb::db_path()?;
                let db = PortfolioDb::open(&db_path)?;
                db.migrate()?;
                match command {
                    PortfolioSubcommand::Create { id, name, description } => {
                        db.create_portfolio(&id, &name, description.as_deref())?;
                        println!("Created portfolio: {} ({})", id, name);
                        println!("State: curated");
                    },
                    PortfolioSubcommand::List => {
                        let portfolios = db.list_portfolios(false)?;
                        if portfolios.is_empty() {
                            println!("No portfolios found");
                        } else {
                            println!(
                                "{:<14} {:<14} {:<9} Created",
                                "ID", "Name", "State"
                            );
                            for p in portfolios {
                                println!(
                                    "{:<14} {:<14} {:<9} {}",
                                    p.id, p.name, p.state, p.created
                                );
                            }
                        }
                    },
                    PortfolioSubcommand::Show { id } => {
                        let p: Portfolio = db.get_portfolio(&id)?;
                        println!("Portfolio: {}", p.id);
                        println!("Name: {}", p.name);
                        match p.description {
                            Some(d) => println!("Description: {}", d),
                            None => println!("Description: (none)"),
                        }
                        println!("State: {}", p.state);
                        println!("Created: {}", p.created);
                        println!("Position: {}", p.position);
                    },
                    PortfolioSubcommand::Dissolve { id } => {
                        db.dissolve_portfolio(&id)?;
                        let p = db.get_portfolio(&id)?;
                        println!("Dissolved portfolio: {} ({})", p.id, p.name);
                    },
                }
            },
            Commands::Project { command } => {
                let db_path = PortfolioDb::db_path()?;
                let db = PortfolioDb::open(&db_path)?;
                db.migrate()?;
                match command {
                    ProjectSubcommand::Register { path, portfolio, name } => {
                        // Validate the repo path exists
                        let repo_path = std::path::Path::new(&path);
                        if !repo_path.exists() {
                            anyhow::bail!(
                                "Path does not exist: {}",
                                repo_path.display()
                            );
                        }

                        // Validate the portfolio exists
                        db.get_portfolio(&portfolio)
                            .map_err(|_| anyhow::anyhow!("Portfolio not found: {}", portfolio))?;

                        // Canonicalize the repo path to an absolute path
                        let repo_path = repo_path.canonicalize()?;
                        let repo_path_str = repo_path.to_string_lossy().to_string();

                        // Auto-detect the project name from the directory name
                        let project_name = name.unwrap_or_else(|| {
                            repo_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unnamed".to_string())
                        });

                        // Auto-detect the .tickets directory
                        let tickets_dir = if repo_path.join(".tickets").exists() {
                            repo_path.join(".tickets")
                        } else {
                            // Default to <repo>/.tickets even if it doesn't exist yet
                            repo_path.join(".tickets")
                        };
                        let tickets_dir_str = tickets_dir.to_string_lossy().to_string();

                        // Auto-detect GitHub owner/repo from git remote
                        let (github_owner, github_repo) = match detect_github_info(&repo_path)? {
                            Some((owner, repo)) => (Some(owner), Some(repo)),
                            None => (None, None),
                        };

                        let id = db.register_project(
                            &portfolio,
                            &project_name,
                            &repo_path_str,
                            &tickets_dir_str,
                            github_owner.as_deref(),
                            github_repo.as_deref(),
                        )?;

                        // Auto-create a default app for the new project
                        let default_app_id = db.ensure_default_app(id)?;
                        if let Some(app_id) = default_app_id {
                            println!("Created default app (id: {}) for project {}", app_id, id);
                        }

                        let github_display = match (&github_owner, &github_repo) {
                            (Some(o), Some(r)) => format!("{}/{}", o, r),
                            _ => "(none)".to_string(),
                        };

                        println!("Registered project: {}", project_name);
                        println!("  Portfolio: {}", portfolio);
                        println!("  Repo: {}", repo_path_str);
                        println!("  GitHub: {}", github_display);
                        println!("  Tickets: {}", tickets_dir_str);
                        println!("  State: seeded");
                        println!("  ID: {}", id);
                    },
                    ProjectSubcommand::List { portfolio } => {
                        let projects = db.list_projects(portfolio.as_deref())?;
                        if projects.is_empty() {
                            println!("No projects found");
                        } else {
                            println!(
                                "{:<4} {:<16} {:<12} {:<34} {:<18} {:<8}",
                                "ID", "Name", "Portfolio", "Repo", "GitHub", "State"
                            );
                            for p in projects {
                                let github = match (&p.github_owner, &p.github_repo) {
                                    (Some(o), Some(r)) => format!("{}/{}", o, r),
                                    _ => "(none)".to_string(),
                                };
                                println!(
                                    "{:<4} {:<16} {:<12} {:<34} {:<18} {:<8}",
                                    p.id, p.name, p.portfolio_id, p.repo_path, github, p.state
                                );
                            }
                        }
                    },
                    ProjectSubcommand::Unregister { path } => {
                        let repo_path = std::path::Path::new(&path);
                        let repo_path_str = if repo_path.exists() {
                            repo_path.canonicalize()?.to_string_lossy().to_string()
                        } else {
                            path.clone()
                        };
                        db.unregister_project(&repo_path_str)?;
                        println!("Unregistered project: {}", repo_path_str);
                    },
                }
            },
            Commands::App { command } => {
                let db_path = PortfolioDb::db_path()?;
                let db = PortfolioDb::open(&db_path)?;
                db.migrate()?;
                match command {
                    AppSubcommand::Create { name, project, description, state } => {
                        let id = db.create_app(
                            project,
                            &name,
                            description.as_deref(),
                            &state,
                        )?;
                        println!("Created app \"{}\" (id: {}) for project {}", name, id, project);
                    },
                    AppSubcommand::List { project } => {
                        let apps = db.list_apps(project)?;
                        if apps.is_empty() {
                            println!("No apps found");
                        } else {
                            println!(
                                "{:<4} {:<12} {:<10} {:<10} Description",
                                "ID", "Name", "State", "Project"
                            );
                            for a in apps {
                                let desc = a.description.unwrap_or_else(|| "(none)".to_string());
                                println!(
                                    "{:<4} {:<12} {:<10} {:<10} {}",
                                    a.id, a.name, a.state, a.project_id, desc
                                );
                            }
                        }
                    },
                    AppSubcommand::Sunset { id } => {
                        let app = db.sunset_app(id)?;
                        println!("App {} (\"{}\") transitioned to sunset", app.id, app.name);
                    },
                }
            },
            Commands::Requirement { command } => {
                let db_path = PortfolioDb::db_path()?;
                let db = PortfolioDb::open(&db_path)?;
                db.migrate()?;
                match command {
                    RequirementSubcommand::Create { title, portfolio, description, state, target_date } => {
                        let id = db.create_requirement(
                            &portfolio,
                            &title,
                            description.as_deref(),
                            &state,
                            target_date.as_deref(),
                        )?;
                        println!("Created requirement \"{}\" (id: {}) in portfolio \"{}\"", title, id, portfolio);
                    },
                    RequirementSubcommand::List { portfolio } => {
                        let requirements = db.list_requirements(portfolio.as_deref())?;
                        if requirements.is_empty() {
                            println!("No requirements found");
                        } else {
                            println!(
                                "{:<14} {:<20} {:<10} {:<12} {:<12}",
                                "ID", "Title", "State", "Portfolio", "Target Date"
                            );
                            for r in &requirements {
                                let target = r.target_date.clone().unwrap_or_else(|| "—".to_string());
                                println!(
                                    "{:<14} {:<20} {:<10} {:<12} {:<12}",
                                    r.id, r.title, r.state, r.portfolio_id, target
                                );
                            }
                        }
                    },
                    RequirementSubcommand::Show { id } => {
                        let req: Requirement = db.get_requirement(&id)?;
                        let story_count = db.count_requirement_stories(&id).unwrap_or(0);
                        println!("Requirement: {}", req.id);
                        println!("Title:       {}", req.title);
                        println!("State:       {}", req.state);
                        println!("Portfolio:   {}", req.portfolio_id);
                        let target = req.target_date.unwrap_or_else(|| "—".to_string());
                        println!("Target Date: {}", target);
                        println!("Stories:     {}", story_count);
                        match req.description {
                            Some(d) => println!("\nDescription:\n{}", d),
                            None => println!("\nDescription: (none)"),
                        }
                    },
                    RequirementSubcommand::Supersede { id } => {
                        let (req, story_count) = db.supersede_requirement(&id)?;
                        println!(
                            "Requirement {} (\"{}\") transitioned to superseded",
                            req.id, req.title
                        );
                        if story_count > 0 {
                            eprintln!(
                                "Warning: {} {} still attached to this requirement",
                                story_count,
                                if story_count == 1 { "story is" } else { "stories are" }
                            );
                        }
                    },
                }
            },
            Commands::Story { command } => {
                let db_path = PortfolioDb::db_path()?;
                let db = PortfolioDb::open(&db_path)?;
                db.migrate()?;
                match command {
                    StorySubcommand::Create { title, requirement, app, description, state, target_date } => {
                        let id = db.create_story(
                            requirement.as_deref(),
                            app,
                            &title,
                            description.as_deref(),
                            &state,
                            target_date.as_deref(),
                        )?;
                        let req_display = requirement.as_deref().unwrap_or("—");
                        let app_display = app.map(|a| a.to_string()).unwrap_or_else(|| "—".to_string());
                        println!(
                            "Created story \"{}\" (id: {}) for requirement {}, app {}",
                            title, id, req_display, app_display
                        );
                    },
                    StorySubcommand::List { requirement, app, state } => {
                        let stories = db.list_stories(
                            requirement.as_deref(),
                            app,
                            state.as_deref(),
                        )?;
                        if stories.is_empty() {
                            println!("No stories found");
                        } else {
                            println!(
                                "{:<14} {:<20} {:<10} {:<14} {:<6}",
                                "ID", "Title", "State", "Requirement", "App"
                            );
                            for s in &stories {
                                let req = s.requirement_id.clone().unwrap_or_else(|| "—".to_string());
                                let a = s.app_id.map(|a| a.to_string()).unwrap_or_else(|| "—".to_string());
                                println!(
                                    "{:<14} {:<20} {:<10} {:<14} {:<6}",
                                    s.id, s.title, s.state, req, a
                                );
                            }
                        }
                    },
                    StorySubcommand::Show { id } => {
                        let story: Story = db.get_story(&id)?;
                        let task_count = db.count_story_tasks(&id).unwrap_or(0);
                        println!("Story:      {}", story.id);
                        println!("Title:       {}", story.title);
                        println!("State:       {}", story.state);
                        let req_display = story.requirement_id.unwrap_or_else(|| "—".to_string());
                        println!("Requirement: {}", req_display);
                        let app_display = story.app_id.map(|a| a.to_string()).unwrap_or_else(|| "—".to_string());
                        println!("App:         {}", app_display);
                        let target = story.target_date.unwrap_or_else(|| "—".to_string());
                        println!("Target Date: {}", target);
                        println!("Tasks:       {}", task_count);
                        match story.description {
                            Some(d) => println!("\nDescription:\n{}", d),
                            None => println!("\nDescription: (none)"),
                        }
                    },
                    StorySubcommand::Ship { id } => {
                        let story = db.ship_story(&id)?;
                        println!(
                            "Story {} (\"{}\") transitioned to shipped",
                            story.id, story.title
                        );
                    },
                    StorySubcommand::Link { task_id, story_id } => {
                        // Validate the story exists before touching markdown
                        if !db.story_exists(&story_id)? {
                            anyhow::bail!(
                                "story \"{}\" not found in portfolio DB",
                                story_id
                            );
                        }
                        manager.link_story(&task_id, &story_id)?;
                    },
                    StorySubcommand::Unlink { task_id } => {
                        manager.unlink_story(&task_id)?;
                    },
                }
            },
            Commands::Sync { github, status } => {
                let db_path = PortfolioDb::db_path()?;
                let db = PortfolioDb::open(&db_path)?;
                db.migrate()?;

                if github {
                    println!("GitHub sync not yet implemented");
                    return Ok(());
                }

                if status {
                    let sync_manager = SyncManager::new(&db);
                    let state = sync_manager.sync_status()?;
                    match state.last_sync_at {
                        Some(ts) => println!("Last sync: {}", ts),
                        None => println!("Last sync: (never)"),
                    }
                    println!("Projects: {}", state.project_count);
                    println!("Tasks:    {}", state.task_count);
                    return Ok(());
                }

                let sync_manager = SyncManager::new(&db);
                let projects = sync_manager.list_registered_projects()?;
                let project_word = if projects.len() == 1 { "project" } else { "projects" };
                println!("Syncing {} {}...", projects.len(), project_word);

                let report = sync_manager.sync_all()?;

                for pr in &report.projects {
                    println!(
                        "  Project \"{}\" (id: {}): {} tasks indexed, {} updated, {} removed",
                        pr.project_name, pr.project_id,
                        pr.tasks_indexed, pr.tasks_updated, pr.tasks_removed
                    );
                }

                let skipped_suffix = if report.files_skipped > 0 {
                    format!(" ({} file skipped)", report.files_skipped)
                } else {
                    String::new()
                };
                println!(
                    "Sync complete: {} tasks indexed, {} updated, {} removed{}",
                    report.tasks_indexed, report.tasks_updated, report.tasks_removed, skipped_suffix
                );
            },
            Commands::Tag { command } => {
                match command {
                    TagSubcommand::Add { task_id, tag } => {
                        manager.add_tag(&task_id, &tag)?;
                    },
                    TagSubcommand::Remove { task_id, tag } => {
                        manager.remove_tag(&task_id, &tag)?;
                    },
                    TagSubcommand::List { tasks } => {
                        let db_path = PortfolioDb::db_path()?;
                        let db = PortfolioDb::open(&db_path)?;
                        db.migrate()?;
                        if tasks {
                            let tags = db.list_tags_with_tasks()?;
                            if tags.is_empty() {
                                println!("No tags found");
                            } else {
                                for (name, task_ids) in tags {
                                    if task_ids.is_empty() {
                                        println!("{}:", name);
                                    } else {
                                        println!("{}: {}", name, task_ids.join(", "));
                                    }
                                }
                            }
                        } else {
                            let tags = db.list_tags()?;
                            if tags.is_empty() {
                                println!("No tags found");
                            } else {
                                for name in tags {
                                    println!("{}", name);
                                }
                            }
                        }
                    },
                }
            },
        }
        Ok(())
    }
}
