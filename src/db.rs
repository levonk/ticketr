use anyhow::Result;
use clap::Subcommand;
use rusqlite::Connection;
use serde::Serialize;
use std::path::PathBuf;

/// Current schema version. Increment every time a new migration step is added
/// to [`PortfolioDb::migrate`].
pub const SCHEMA_VERSION: u32 = 1;

/// A portfolio record — the top level of the 7-level hierarchy.
#[derive(Debug, Serialize)]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub created: String,
    pub position: i64,
}

/// Subcommands for `tkr portfolio`.
#[derive(Subcommand)]
pub enum PortfolioSubcommand {
    /// Create a new portfolio
    Create {
        id: String,
        name: String,
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
    },
    /// List all portfolios
    List,
    /// Show portfolio details
    Show { id: String },
    /// Dissolve a portfolio (soft delete)
    Dissolve { id: String },
    /// Display cross-project views of tasks
    View {
        /// Group tasks by their story's requirement
        #[arg(long = "by-requirement")]
        by_requirement: bool,
        /// Group tasks by story
        #[arg(long = "by-story")]
        by_story: bool,
        /// Group tasks by tag (optionally filtered to a specific tag value)
        #[arg(long = "by-tag")]
        by_tag: Option<Option<String>>,
        /// Group tasks by project
        #[arg(long = "by-project")]
        by_project: bool,
        /// Output as JSON
        #[arg(long = "json")]
        json: bool,
    },
}

/// A project record — a git repo registered under a portfolio. Projects are
/// the second level of the 7-level hierarchy.
#[derive(Debug, Serialize)]
pub struct Project {
    pub id: i64,
    pub portfolio_id: String,
    pub name: String,
    pub repo_path: String,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub github_account: Option<String>,
    pub tickets_dir: String,
    pub state: String,
    pub registered_at: String,
    pub last_synced_at: Option<String>,
}

/// An app record — a deployable unit within a project. Apps are the third
/// level of the 7-level hierarchy. Single-app projects get an implicit
/// `default` app.
#[derive(Debug, Serialize)]
pub struct App {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub created: String,
}

/// Subcommands for `tkr app`.
#[derive(Subcommand)]
pub enum AppSubcommand {
    /// Create a new app under a project
    Create {
        name: String,
        #[arg(short = 'p', long = "project")]
        project: i64,
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        #[arg(short = 's', long = "state", default_value = "drafted")]
        state: String,
    },
    /// List apps, optionally filtered by project
    List {
        #[arg(short = 'p', long = "project")]
        project: Option<i64>,
    },
    /// Transition an app to the sunset terminal state
    Sunset { id: i64 },
}

/// A requirement record — a durable constraint at the strategic level of
/// the hierarchy. Requirements live under a portfolio and group stories.
#[derive(Debug, Serialize)]
pub struct Requirement {
    pub id: String,
    pub portfolio_id: String,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub created: String,
    pub target_date: Option<String>,
    pub position: i64,
}

/// Subcommands for `tkr requirement`.
#[derive(Subcommand)]
pub enum RequirementSubcommand {
    /// Create a new requirement under a portfolio
    Create {
        title: String,
        #[arg(short = 'p', long = "portfolio")]
        portfolio: String,
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        #[arg(short = 's', long = "state", default_value = "proposed")]
        state: String,
        #[arg(long = "target-date")]
        target_date: Option<String>,
    },
    /// List requirements, optionally filtered by portfolio
    List {
        #[arg(short = 'p', long = "portfolio")]
        portfolio: Option<String>,
    },
    /// Show requirement details
    Show { id: String },
    /// Transition a requirement to the superseded (terminal) state
    Supersede { id: String },
}

/// A story record — a feature slice at the coordination level of the
/// hierarchy. Stories link a requirement (strategic) to an app (deployable
/// unit) and group tasks (operational).
#[derive(Debug, Serialize)]
pub struct Story {
    pub id: String,
    pub requirement_id: Option<String>,
    pub app_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub created: String,
    pub target_date: Option<String>,
    pub position: i64,
}

/// Subcommands for `tkr story`.
#[derive(Subcommand)]
pub enum StorySubcommand {
    /// Create a new story under a requirement and/or app
    Create {
        title: String,
        #[arg(short = 'r', long = "requirement")]
        requirement: Option<String>,
        #[arg(short = 'a', long = "app")]
        app: Option<i64>,
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        #[arg(short = 's', long = "state", default_value = "pitched")]
        state: String,
        #[arg(long = "target-date")]
        target_date: Option<String>,
    },
    /// List stories, optionally filtered by requirement, app, or state
    List {
        #[arg(short = 'r', long = "requirement")]
        requirement: Option<String>,
        #[arg(short = 'a', long = "app")]
        app: Option<i64>,
        #[arg(short = 's', long = "state")]
        state: Option<String>,
    },
    /// Show story details
    Show { id: String },
    /// Transition a story to the shipped (terminal) state
    Ship { id: String },
    /// Link a task to a story (writes `story` into the task's markdown frontmatter)
    Link {
        task_id: String,
        story_id: String,
    },
    /// Unlink a task from its story (removes the `story` field from markdown)
    Unlink { task_id: String },
}

/// Valid requirement state values per the PRD requirement vocabulary.
const VALID_REQUIREMENT_STATES: &[&str] =
    &["surfaced", "proposed", "planned", "current", "superseded"];

/// Validate that `state` is one of the allowed requirement state values.
fn validate_requirement_state(state: &str) -> Result<()> {
    if !VALID_REQUIREMENT_STATES.contains(&state) {
        anyhow::bail!(
            "Invalid requirement state \"{}\". Allowed: surfaced, proposed, planned, current, superseded",
            state
        );
    }
    Ok(())
}

/// Valid app state values per the PRD deployment vocabulary.
const VALID_APP_STATES: &[&str] = &["sketched", "drafted", "deployed", "deprecated", "sunset"];

/// Validate that `state` is one of the allowed app state values.
fn validate_app_state(state: &str) -> Result<()> {
    if !VALID_APP_STATES.contains(&state) {
        anyhow::bail!(
            "Invalid state: {}. Valid states: sketched, drafted, deployed, deprecated, sunset",
            state
        );
    }
    Ok(())
}

/// Valid story state values per the PRD delivery vocabulary.
const VALID_STORY_STATES: &[&str] = &[
    "suggested",
    "pitched",
    "queued",
    "building",
    "stalled",
    "review",
    "shipped",
    "archived",
];

/// Validate that `state` is one of the allowed story state values.
fn validate_story_state(state: &str) -> Result<()> {
    if !VALID_STORY_STATES.contains(&state) {
        anyhow::bail!(
            "Invalid story state \"{}\". Allowed: suggested, pitched, queued, building, stalled, review, shipped, archived",
            state
        );
    }
    Ok(())
}

/// Subcommands for `tkr project`.
#[derive(Subcommand)]
pub enum ProjectSubcommand {
    /// Register a repo under a portfolio
    Register {
        path: String,
        #[arg(short = 'p', long = "portfolio")]
        portfolio: String,
        #[arg(short = 'n', long = "name")]
        name: Option<String>,
    },
    /// List registered projects, optionally filtered by portfolio
    List {
        #[arg(short = 'p', long = "portfolio")]
        portfolio: Option<String>,
    },
    /// Remove a project from the DB (hard delete)
    Unregister { path: String },
}

/// Wrapper around a SQLite connection holding the portfolio database.
pub struct PortfolioDb {
    /// Public so tests and future modules can run ad-hoc queries.
    pub conn: Connection,
}

impl PortfolioDb {
    /// Open (or create) the SQLite database at `path`, enabling WAL journal
    /// mode and foreign-key enforcement on the connection.
    pub fn open<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Ok(Self { conn })
    }

    /// Resolve the portfolio database path. If the `TKR_DB_PATH` environment
    /// variable is set, it is used directly (primarily for testing).
    /// Otherwise the `directories` crate resolves the platform default
    /// (`~/.local/share/tkr/portfolio.db` on Linux,
    /// `~/Library/Application Support/tkr/portfolio.db` on macOS).
    pub fn db_path() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("TKR_DB_PATH") {
            return Ok(PathBuf::from(path));
        }
        use directories::ProjectDirs;
        let proj_dirs = ProjectDirs::from("", "", "tkr")
            .ok_or_else(|| anyhow::anyhow!("could not determine data directory"))?;
        Ok(proj_dirs.data_dir().join("portfolio.db"))
    }

    /// Create all portfolio tables if they do not already exist. Idempotent.
    pub fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA_DDL)?;
        Ok(())
    }

    /// Apply incremental schema migrations, tracking the current version in
    /// the `sync_state` table. Idempotent — calling repeatedly is a no-op
    /// once the database is at [`SCHEMA_VERSION`].
    pub fn migrate(&self) -> Result<()> {
        // Ensure sync_state exists so we can read/write the version.
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT
            );",
        )?;

        let current: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .ok();

        let current_version: u32 = current
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        if current_version < 1 {
            // Migration 1: initial schema.
            self.init_schema()?;
            self.conn.execute(
                "INSERT INTO sync_state (key, value) VALUES ('schema_version', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![SCHEMA_VERSION.to_string()],
            )?;
        }

        Ok(())
    }

    /// Create a new portfolio with the given ID, name, and optional
    /// description. The portfolio is inserted with state `curated` and the
    /// current UTC timestamp. Returns an error if a portfolio with the
    /// same ID already exists.
    pub fn create_portfolio(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM portfolios WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        if exists {
            anyhow::bail!("Portfolio already exists: {}", id);
        }

        let created = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO portfolios (id, name, description, state, created, position)
             VALUES (?1, ?2, ?3, 'curated', ?4, 0)",
            rusqlite::params![id, name, description, created],
        )?;
        Ok(())
    }

    /// List all portfolios, ordered by position then creation time. When
    /// `include_dissolved` is `false`, portfolios with state `dissolved` are
    /// excluded.
    pub fn list_portfolios(&self, include_dissolved: bool) -> Result<Vec<Portfolio>> {
        let query = if include_dissolved {
            "SELECT id, name, description, state, created, position
             FROM portfolios ORDER BY position, created"
        } else {
            "SELECT id, name, description, state, created, position
             FROM portfolios WHERE state != 'dissolved' ORDER BY position, created"
        };
        let mut stmt = self.conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            Ok(Portfolio {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                state: row.get(3)?,
                created: row.get(4)?,
                position: row.get(5)?,
            })
        })?;
        let mut portfolios = Vec::new();
        for row in rows {
            portfolios.push(row?);
        }
        Ok(portfolios)
    }

    /// Retrieve a single portfolio by ID. Returns an error if not found.
    pub fn get_portfolio(&self, id: &str) -> Result<Portfolio> {
        self.conn
            .query_row(
                "SELECT id, name, description, state, created, position
                 FROM portfolios WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(Portfolio {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        state: row.get(3)?,
                        created: row.get(4)?,
                        position: row.get(5)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    anyhow::anyhow!("Portfolio not found: {}", id)
                }
                _ => anyhow::anyhow!(e),
            })
    }

    /// Set a portfolio's state to `dissolved` (soft delete). Idempotent —
    /// dissolving an already-dissolved portfolio is a no-op. Returns an
    /// error if the portfolio does not exist.
    pub fn dissolve_portfolio(&self, id: &str) -> Result<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM portfolios WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        if !exists {
            anyhow::bail!("Portfolio not found: {}", id);
        }

        self.conn.execute(
            "UPDATE portfolios SET state = 'dissolved' WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    /// Register a project under a portfolio. Inserts a row with state `seeded`
    /// and the current UTC timestamp, returning the auto-generated project ID.
    /// Returns an error if a project with the same `repo_path` already exists.
    pub fn register_project(
        &self,
        portfolio_id: &str,
        name: &str,
        repo_path: &str,
        tickets_dir: &str,
        github_owner: Option<&str>,
        github_repo: Option<&str>,
    ) -> Result<i64> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM projects WHERE repo_path = ?1",
            rusqlite::params![repo_path],
            |row| row.get(0),
        )?;
        if exists {
            anyhow::bail!("Project already registered: {}", repo_path);
        }

        let registered_at = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO projects
                (portfolio_id, name, repo_path, github_owner, github_repo, tickets_dir, state, registered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'seeded', ?7)",
            rusqlite::params![
                portfolio_id,
                name,
                repo_path,
                github_owner,
                github_repo,
                tickets_dir,
                registered_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// List registered projects. When `portfolio_id` is `Some`, only projects
    /// in that portfolio are returned. Ordered by registration time.
    pub fn list_projects(&self, portfolio_id: Option<&str>) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, portfolio_id, name, repo_path, github_owner, github_repo,
                    github_account, tickets_dir, state, registered_at, last_synced_at
             FROM projects
             WHERE (?1 IS NULL OR portfolio_id = ?1)
             ORDER BY registered_at",
        )?;
        let rows = stmt.query_map(rusqlite::params![portfolio_id], map_project_row)?;
        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }
        Ok(projects)
    }

    /// Remove a project from the DB by its repo path (hard delete). Also
    /// removes any apps associated with the project to satisfy foreign key
    /// constraints. Returns an error if no project with that repo path exists.
    pub fn unregister_project(&self, repo_path: &str) -> Result<()> {
        // Delete apps for the project first to satisfy FK constraints
        self.conn.execute(
            "DELETE FROM apps WHERE project_id IN (
                SELECT id FROM projects WHERE repo_path = ?1
            )",
            rusqlite::params![repo_path],
        )?;

        let affected = self.conn.execute(
            "DELETE FROM projects WHERE repo_path = ?1",
            rusqlite::params![repo_path],
        )?;
        if affected == 0 {
            anyhow::bail!("Project not found: {}", repo_path);
        }
        Ok(())
    }

    /// Return true if a project with the given repo path is registered.
    #[allow(dead_code)]
    pub fn project_exists(&self, repo_path: &str) -> Result<bool> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM projects WHERE repo_path = ?1",
            rusqlite::params![repo_path],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    /// Create a new app under a project. Validates the state against the
    /// PRD vocabulary, checks that the project exists, and enforces the
    /// `UNIQUE(project_id, name)` constraint at the application layer.
    /// Returns the auto-generated app ID.
    pub fn create_app(
        &self,
        project_id: i64,
        name: &str,
        description: Option<&str>,
        state: &str,
    ) -> Result<i64> {
        validate_app_state(state)?;

        let project_exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |row| row.get(0),
        )?;
        if !project_exists {
            anyhow::bail!("Project not found: {}", project_id);
        }

        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM apps WHERE project_id = ?1 AND name = ?2",
            rusqlite::params![project_id, name],
            |row| row.get(0),
        )?;
        if exists {
            anyhow::bail!("App \"{}\" already exists for project {}", name, project_id);
        }

        let created = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO apps (project_id, name, description, state, created)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![project_id, name, description, state, created],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// List apps, optionally filtered by project. Ordered by app ID.
    pub fn list_apps(&self, project_id: Option<i64>) -> Result<Vec<App>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, name, description, state, created
             FROM apps
             WHERE (?1 IS NULL OR project_id = ?1)
             ORDER BY id",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id], |row| {
            Ok(App {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                state: row.get(4)?,
                created: row.get(5)?,
            })
        })?;
        let mut apps = Vec::new();
        for row in rows {
            apps.push(row?);
        }
        Ok(apps)
    }

    /// Transition an app to the `sunset` terminal state. Returns an error
    /// if the app does not exist.
    pub fn sunset_app(&self, id: i64) -> Result<App> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM apps WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        if !exists {
            anyhow::bail!("App not found: {}", id);
        }

        self.conn.execute(
            "UPDATE apps SET state = 'sunset' WHERE id = ?1",
            rusqlite::params![id],
        )?;

        self.conn.query_row(
            "SELECT id, project_id, name, description, state, created
             FROM apps WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(App {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    state: row.get(4)?,
                    created: row.get(5)?,
                })
            },
        ).map_err(|e| anyhow::anyhow!(e))
    }

    /// Ensure that a `default` app exists for the given project. If no apps
    /// exist for the project, creates one with name `default`, state `drafted`,
    /// and description `Default app`. Returns `Some(app_id)` if a new app was
    /// created, or `None` if apps already exist for the project.
    pub fn ensure_default_app(&self, project_id: i64) -> Result<Option<i64>> {
        let has_app: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM apps WHERE project_id = ?1",
            rusqlite::params![project_id],
            |row| row.get(0),
        )?;
        if has_app {
            return Ok(None);
        }

        let created = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO apps (project_id, name, description, state, created)
             VALUES (?1, 'default', 'Default app', 'drafted', ?2)",
            rusqlite::params![project_id, created],
        )?;
        Ok(Some(self.conn.last_insert_rowid()))
    }

    // -----------------------------------------------------------------
    // Requirement CRUD
    // -----------------------------------------------------------------

    /// Generate a requirement ID with the `req-` prefix using a timestamp
    /// and UUID fragment for entropy, matching the pattern used by
    /// [`TicketManager::generate_id`].
    pub fn generate_requirement_id(&self) -> Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();
        let uuid = uuid::Uuid::new_v4();
        let uuid_str = uuid.as_simple().to_string();
        let hash = format!("{:x}{}", timestamp % 10000, &uuid_str[..4]);
        Ok(format!("req-{}", hash))
    }

    /// Create a new requirement under a portfolio. Validates the state
    /// against the PRD vocabulary, checks that the portfolio exists, and
    /// generates a `req-` prefixed ID. Returns the new requirement ID.
    pub fn create_requirement(
        &self,
        portfolio_id: &str,
        title: &str,
        description: Option<&str>,
        state: &str,
        target_date: Option<&str>,
    ) -> Result<String> {
        validate_requirement_state(state)?;

        // Verify the portfolio exists (FK enforcement)
        let portfolio_exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM portfolios WHERE id = ?1",
            rusqlite::params![portfolio_id],
            |row| row.get(0),
        )?;
        if !portfolio_exists {
            anyhow::bail!("Portfolio not found: {}", portfolio_id);
        }

        let id = self.generate_requirement_id()?;
        let created = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO requirements (id, portfolio_id, title, description, state, created, target_date, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
            rusqlite::params![id, portfolio_id, title, description, state, created, target_date],
        )?;
        Ok(id)
    }

    /// List requirements, optionally filtered by portfolio. Ordered by
    /// position then creation time.
    pub fn list_requirements(&self, portfolio_id: Option<&str>) -> Result<Vec<Requirement>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, portfolio_id, title, description, state, created, target_date, position
             FROM requirements
             WHERE (?1 IS NULL OR portfolio_id = ?1)
             ORDER BY position, created",
        )?;
        let rows = stmt.query_map(rusqlite::params![portfolio_id], |row| {
            Ok(Requirement {
                id: row.get(0)?,
                portfolio_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                state: row.get(4)?,
                created: row.get(5)?,
                target_date: row.get(6)?,
                position: row.get(7)?,
            })
        })?;
        let mut requirements = Vec::new();
        for row in rows {
            requirements.push(row?);
        }
        Ok(requirements)
    }

    /// Retrieve a single requirement by ID. Returns an error if not found.
    pub fn get_requirement(&self, id: &str) -> Result<Requirement> {
        self.conn
            .query_row(
                "SELECT id, portfolio_id, title, description, state, created, target_date, position
                 FROM requirements WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(Requirement {
                        id: row.get(0)?,
                        portfolio_id: row.get(1)?,
                        title: row.get(2)?,
                        description: row.get(3)?,
                        state: row.get(4)?,
                        created: row.get(5)?,
                        target_date: row.get(6)?,
                        position: row.get(7)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    anyhow::anyhow!("Requirement not found: {}", id)
                }
                _ => anyhow::anyhow!(e),
            })
    }

    /// Count the number of stories linked to a requirement. Used by the
    /// `show` command to display a story count and by `supersede` to warn
    /// if stories are still attached.
    pub fn count_requirement_stories(&self, id: &str) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM stories WHERE requirement_id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Transition a requirement to the `superseded` (terminal) state.
    /// Returns an error if the requirement does not exist. Returns the
    /// updated requirement and the count of attached stories (so the
    /// caller can warn the user).
    pub fn supersede_requirement(&self, id: &str) -> Result<(Requirement, i64)> {
        let story_count = self.count_requirement_stories(id).unwrap_or(0);

        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM requirements WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        if !exists {
            anyhow::bail!("Requirement not found: {}", id);
        }

        self.conn.execute(
            "UPDATE requirements SET state = 'superseded' WHERE id = ?1",
            rusqlite::params![id],
        )?;

        let requirement = self.get_requirement(id)?;
        Ok((requirement, story_count))
    }

    // -----------------------------------------------------------------
    // Story CRUD
    // -----------------------------------------------------------------

    /// Generate a story ID with the `story-` prefix using a timestamp
    /// and UUID fragment for entropy, matching the pattern used by
    /// [`PortfolioDb::generate_requirement_id`].
    pub fn generate_story_id(&self) -> Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();
        let uuid = uuid::Uuid::new_v4();
        let uuid_str = uuid.as_simple().to_string();
        let hash = format!("{:x}{}", timestamp % 10000, &uuid_str[..4]);
        Ok(format!("story-{}", hash))
    }

    /// Create a new story. Validates the state against the PRD vocabulary,
    /// checks FK targets if provided, and generates a `story-` prefixed ID.
    /// Both `requirement_id` and `app_id` are optional (a story can be
    /// created without links for triage). Returns the new story ID.
    pub fn create_story(
        &self,
        requirement_id: Option<&str>,
        app_id: Option<i64>,
        title: &str,
        description: Option<&str>,
        state: &str,
        target_date: Option<&str>,
    ) -> Result<String> {
        validate_story_state(state)?;

        // Verify the requirement exists if provided (FK enforcement)
        if let Some(req_id) = requirement_id {
            let req_exists: bool = self.conn.query_row(
                "SELECT COUNT(*) > 0 FROM requirements WHERE id = ?1",
                rusqlite::params![req_id],
                |row| row.get(0),
            )?;
            if !req_exists {
                anyhow::bail!("Requirement not found: {}", req_id);
            }
        }

        // Verify the app exists if provided (FK enforcement)
        if let Some(aid) = app_id {
            let app_exists: bool = self.conn.query_row(
                "SELECT COUNT(*) > 0 FROM apps WHERE id = ?1",
                rusqlite::params![aid],
                |row| row.get(0),
            )?;
            if !app_exists {
                anyhow::bail!("App not found: {}", aid);
            }
        }

        let id = self.generate_story_id()?;
        let created = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO stories (id, requirement_id, app_id, title, description, state, created, target_date, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
            rusqlite::params![id, requirement_id, app_id, title, description, state, created, target_date],
        )?;
        Ok(id)
    }

    /// List stories, optionally filtered by requirement, app, or state.
    /// Ordered by position then creation time.
    pub fn list_stories(
        &self,
        requirement_id: Option<&str>,
        app_id: Option<i64>,
        state: Option<&str>,
    ) -> Result<Vec<Story>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, requirement_id, app_id, title, description, state, created, target_date, position
             FROM stories
             WHERE (?1 IS NULL OR requirement_id = ?1)
               AND (?2 IS NULL OR app_id = ?2)
               AND (?3 IS NULL OR state = ?3)
             ORDER BY position, created",
        )?;
        let rows = stmt.query_map(rusqlite::params![requirement_id, app_id, state], |row| {
            Ok(Story {
                id: row.get(0)?,
                requirement_id: row.get(1)?,
                app_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                state: row.get(5)?,
                created: row.get(6)?,
                target_date: row.get(7)?,
                position: row.get(8)?,
            })
        })?;
        let mut stories = Vec::new();
        for row in rows {
            stories.push(row?);
        }
        Ok(stories)
    }

    /// Retrieve a single story by ID. Returns an error if not found.
    pub fn get_story(&self, id: &str) -> Result<Story> {
        self.conn
            .query_row(
                "SELECT id, requirement_id, app_id, title, description, state, created, target_date, position
                 FROM stories WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(Story {
                        id: row.get(0)?,
                        requirement_id: row.get(1)?,
                        app_id: row.get(2)?,
                        title: row.get(3)?,
                        description: row.get(4)?,
                        state: row.get(5)?,
                        created: row.get(6)?,
                        target_date: row.get(7)?,
                        position: row.get(8)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    anyhow::anyhow!("Story not found: {}", id)
                }
                _ => anyhow::anyhow!(e),
            })
    }

    /// Count the number of tasks linked to a story. Used by the `show`
    /// command to display a task count.
    pub fn count_story_tasks(&self, id: &str) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE story_id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Transition a story to the `shipped` (terminal) state. Returns an
    /// error if the story does not exist. Returns the updated story.
    pub fn ship_story(&self, id: &str) -> Result<Story> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM stories WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        if !exists {
            anyhow::bail!("Story not found: {}", id);
        }

        self.conn.execute(
            "UPDATE stories SET state = 'shipped' WHERE id = ?1",
            rusqlite::params![id],
        )?;

        self.get_story(id)
    }

    /// Return true if a story with the given ID exists in the `stories`
    /// table. Used by the `tkr story link` command to validate the target
    /// story before writing to markdown.
    pub fn story_exists(&self, id: &str) -> Result<bool> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM stories WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    // -----------------------------------------------------------------
    // Tag CRUD
    // -----------------------------------------------------------------

    /// Upsert a tag name into the `tags` table, returning the tag's row id.
    /// If the tag already exists (unique name), returns its existing id.
    pub fn upsert_tag(&self, name: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tags (name) VALUES (?1)
             ON CONFLICT(name) DO NOTHING",
            rusqlite::params![name],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    /// Reconcile the `task_tags` join rows for a single task: delete all
    /// existing rows for `task_id`, then insert a row for each tag in
    /// `tag_names` (upserting the tag name into `tags` first). Returns the
    /// number of join rows inserted.
    pub fn sync_task_tags(&self, task_id: &str, tag_names: &[String]) -> Result<usize> {
        // Delete existing join rows for this task
        self.conn.execute(
            "DELETE FROM task_tags WHERE task_id = ?1",
            rusqlite::params![task_id],
        )?;

        let mut inserted = 0;
        for name in tag_names {
            let tag_id = self.upsert_tag(name)?;
            self.conn.execute(
                "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)
                 ON CONFLICT(task_id, tag_id) DO NOTHING",
                rusqlite::params![task_id, tag_id],
            )?;
            inserted += 1;
        }
        Ok(inserted)
    }

    /// List all distinct tag names in the DB, ordered alphabetically.
    pub fn list_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM tags ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            Ok(name)
        })?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    /// List each tag name with the comma-separated task IDs carrying that
    /// tag. Tags with no tasks are included with an empty task list.
    /// Ordered by tag name.
    pub fn list_tags_with_tasks(&self) -> Result<Vec<(String, Vec<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name, GROUP_CONCAT(tt.task_id, ', ')
             FROM tags t
             LEFT JOIN task_tags tt ON tt.tag_id = t.id
             GROUP BY t.name
             ORDER BY t.name",
        )?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let joined: Option<String> = row.get(1)?;
            let task_ids = joined
                .unwrap_or_default()
                .split(", ")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            Ok((name, task_ids))
        })?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    // -----------------------------------------------------------------
    // AI Task CRUD
    // -----------------------------------------------------------------

    /// Generate an AI Task ID with the `ai-` prefix using a timestamp
    /// and UUID fragment for entropy, matching the pattern used by
    /// [`PortfolioDb::generate_requirement_id`].
    pub fn generate_ai_task_id(&self) -> Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();
        let uuid = uuid::Uuid::new_v4();
        let uuid_str = uuid.as_simple().to_string();
        let hash = format!("{:x}{}", timestamp % 10000, &uuid_str[..4]);
        Ok(format!("ai-{}", hash))
    }

    /// Create a new AI Task linked to a parent task. Validates that the
    /// parent task exists in the `tasks` table, generates an `ai-` prefixed
    /// ID, and sets the initial state to `identified`. Returns the new AI
    /// Task ID.
    pub fn create_ai_task(
        &self,
        task_id: &str,
        title: &str,
        agent_profile: Option<&str>,
    ) -> Result<String> {
        // Verify the parent task exists (FK enforcement)
        let task_exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
            rusqlite::params![task_id],
            |row| row.get(0),
        )?;
        if !task_exists {
            anyhow::bail!("parent task \"{}\" not found", task_id);
        }

        let id = self.generate_ai_task_id()?;
        let created = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO ai_tasks (id, task_id, title, state, agent_profile, created)
             VALUES (?1, ?2, ?3, 'identified', ?4, ?5)",
            rusqlite::params![id, task_id, title, agent_profile, created],
        )?;
        Ok(id)
    }

    /// List AI Tasks, optionally filtered by parent task or state. Ordered
    /// by creation time.
    pub fn list_ai_tasks(
        &self,
        task_id: Option<&str>,
        state: Option<&str>,
    ) -> Result<Vec<AiTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, title, state, agent_profile, created, completed, result_summary
             FROM ai_tasks
             WHERE (?1 IS NULL OR task_id = ?1)
               AND (?2 IS NULL OR state = ?2)
             ORDER BY created",
        )?;
        let rows = stmt.query_map(rusqlite::params![task_id, state], |row| {
            Ok(AiTask {
                id: row.get(0)?,
                task_id: row.get(1)?,
                title: row.get(2)?,
                state: row.get(3)?,
                agent_profile: row.get(4)?,
                created: row.get(5)?,
                completed: row.get(6)?,
                result_summary: row.get(7)?,
            })
        })?;
        let mut ai_tasks = Vec::new();
        for row in rows {
            ai_tasks.push(row?);
        }
        Ok(ai_tasks)
    }

    /// Retrieve a single AI Task by ID. Returns an error if not found.
    pub fn get_ai_task(&self, id: &str) -> Result<AiTask> {
        self.conn
            .query_row(
                "SELECT id, task_id, title, state, agent_profile, created, completed, result_summary
                 FROM ai_tasks WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(AiTask {
                        id: row.get(0)?,
                        task_id: row.get(1)?,
                        title: row.get(2)?,
                        state: row.get(3)?,
                        agent_profile: row.get(4)?,
                        created: row.get(5)?,
                        completed: row.get(6)?,
                        result_summary: row.get(7)?,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    anyhow::anyhow!("AI Task not found: {}", id)
                }
                _ => anyhow::anyhow!(e),
            })
    }

    /// Transition an AI Task to a new state. Validates the state against
    /// the allowed vocabulary. When transitioning to `returned`, stamps
    /// the `completed` column with the current UTC timestamp and optionally
    /// stores `result_summary`. Returns the updated AI Task.
    pub fn update_ai_task_state(
        &self,
        id: &str,
        state: &str,
        result_summary: Option<&str>,
    ) -> Result<AiTask> {
        validate_ai_task_state(state)?;

        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM ai_tasks WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        if !exists {
            anyhow::bail!("AI Task not found: {}", id);
        }

        if state == "returned" {
            let completed = chrono::Utc::now().to_rfc3339();
            self.conn.execute(
                "UPDATE ai_tasks SET state = ?1, completed = ?2, result_summary = ?3
                 WHERE id = ?4",
                rusqlite::params![state, completed, result_summary, id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE ai_tasks SET state = ?1 WHERE id = ?2",
                rusqlite::params![state, id],
            )?;
        }

        self.get_ai_task(id)
    }
}

/// An AI Task record — a delegated subtask linked to a parent task. AI
/// Tasks are the seventh and lowest level of the 7-level hierarchy. They
/// are DB-only (not indexed from markdown) and use their own state
/// vocabulary: `identified`, `dispatched`, `running`, `returned`.
#[derive(Debug, Serialize)]
pub struct AiTask {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub state: String,
    pub agent_profile: Option<String>,
    pub created: String,
    pub completed: Option<String>,
    pub result_summary: Option<String>,
}

/// Subcommands for `tkr ai-task`.
#[derive(Subcommand)]
pub enum AiTaskSubcommand {
    /// Create a new AI Task linked to a parent task
    Create {
        /// The parent task ID (must exist in the `tasks` table)
        task_id: String,
        /// The AI Task title
        title: String,
        /// Optional agent profile name (e.g. `subagent_general`)
        #[arg(long = "agent-profile")]
        agent_profile: Option<String>,
    },
    /// List AI Tasks, optionally filtered by parent task or state
    List {
        /// Filter to AI Tasks under this parent task
        #[arg(long = "task")]
        task: Option<String>,
        /// Filter by state
        #[arg(long = "state")]
        state: Option<String>,
    },
    /// Show AI Task details
    Show {
        /// The AI Task ID
        id: String,
    },
    /// Transition an AI Task to a new state
    UpdateState {
        /// The AI Task ID
        id: String,
        /// The new state (identified, dispatched, running, returned)
        state: String,
        /// Optional result summary (stored when transitioning to `returned`)
        #[arg(long = "summary")]
        summary: Option<String>,
    },
}

/// Valid AI Task state values per the PRD AI Task vocabulary.
const VALID_AI_TASK_STATES: &[&str] = &["identified", "dispatched", "running", "returned"];

/// Validate that `state` is one of the allowed AI Task state values.
fn validate_ai_task_state(state: &str) -> Result<()> {
    if !VALID_AI_TASK_STATES.contains(&state) {
        anyhow::bail!(
            "invalid AI Task state \"{}\". Valid: identified, dispatched, running, returned",
            state
        );
    }
    Ok(())
}

/// Map a rusqlite row into a [`Project`] struct.
fn map_project_row(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        portfolio_id: row.get(1)?,
        name: row.get(2)?,
        repo_path: row.get(3)?,
        github_owner: row.get(4)?,
        github_repo: row.get(5)?,
        github_account: row.get(6)?,
        tickets_dir: row.get(7)?,
        state: row.get(8)?,
        registered_at: row.get(9)?,
        last_synced_at: row.get(10)?,
    })
}

/// Full DDL for the portfolio schema (all 11 tables). Uses `IF NOT EXISTS` so
/// it is safe to run repeatedly.
pub const SCHEMA_DDL: &str = "
-- Portfolios (top level — personal, OSS, business A, client X)
CREATE TABLE IF NOT EXISTS portfolios (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'curated',
    created TEXT NOT NULL,
    position INTEGER DEFAULT 0
);

-- Projects (repos registered in a portfolio)
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY,
    portfolio_id TEXT NOT NULL REFERENCES portfolios(id),
    name TEXT NOT NULL,
    repo_path TEXT NOT NULL UNIQUE,
    github_owner TEXT,
    github_repo TEXT,
    github_account TEXT,
    tickets_dir TEXT NOT NULL,
    state TEXT DEFAULT 'seeded',
    registered_at TEXT NOT NULL,
    last_synced_at TEXT
);

-- Apps (deployable units within a project; 'default' for single-app)
CREATE TABLE IF NOT EXISTS apps (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'drafted',
    created TEXT NOT NULL,
    UNIQUE(project_id, name)
);

-- Requirements (durable constraints — strategic level)
CREATE TABLE IF NOT EXISTS requirements (
    id TEXT PRIMARY KEY,
    portfolio_id TEXT NOT NULL REFERENCES portfolios(id),
    title TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'proposed',
    created TEXT NOT NULL,
    target_date TEXT,
    position INTEGER DEFAULT 0
);

-- Stories (feature slices — coordination level)
CREATE TABLE IF NOT EXISTS stories (
    id TEXT PRIMARY KEY,
    requirement_id TEXT REFERENCES requirements(id),
    app_id INTEGER REFERENCES apps(id),
    title TEXT NOT NULL,
    description TEXT,
    state TEXT DEFAULT 'pitched',
    created TEXT NOT NULL,
    target_date TEXT,
    position INTEGER DEFAULT 0
);

-- Tasks (work items — operational level, indexed from markdown)
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    app_id INTEGER REFERENCES apps(id),
    story_id TEXT REFERENCES stories(id),
    title TEXT NOT NULL,
    state TEXT NOT NULL,
    priority INTEGER DEFAULT 2,
    issue_type TEXT DEFAULT 'task',
    markdown_path TEXT NOT NULL,
    markdown_hash TEXT,
    github_issue_number INTEGER,
    github_issue_url TEXT,
    github_synced_at TEXT,
    github_etag TEXT,
    synced_at TEXT NOT NULL
);

-- AI Tasks (delegated subtasks)
CREATE TABLE IF NOT EXISTS ai_tasks (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    title TEXT NOT NULL,
    state TEXT DEFAULT 'dispatched',
    agent_profile TEXT,
    created TEXT NOT NULL,
    completed TEXT,
    result_summary TEXT
);

-- Tags (cross-project)
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);

CREATE TABLE IF NOT EXISTS task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    PRIMARY KEY (task_id, tag_id)
);

-- Relative priority ordering (drag-and-drop in web UI)
CREATE TABLE IF NOT EXISTS priority_order (
    scope_type TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    position INTEGER NOT NULL,
    PRIMARY KEY (scope_type, scope_id, task_id)
);

-- Sync state
CREATE TABLE IF NOT EXISTS sync_state (
    key TEXT PRIMARY KEY,
    value TEXT
);
";

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_open_creates_database_file() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("portfolio.db");
        assert!(!db_path.exists());
        let _db = PortfolioDb::open(&db_path).unwrap();
        assert!(db_path.exists());
    }

    #[test]
    fn test_init_schema_creates_all_tables() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.init_schema().unwrap();

        let table_names = [
            "portfolios",
            "projects",
            "apps",
            "requirements",
            "stories",
            "tasks",
            "ai_tasks",
            "tags",
            "task_tags",
            "priority_order",
            "sync_state",
        ];
        for name in &table_names {
            let count: i32 = db
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    rusqlite::params![name],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table '{}' was not created", name);
        }
    }

    #[test]
    fn test_init_schema_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.init_schema().unwrap();
        db.init_schema().unwrap(); // should not error

        let count: i32 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 11); // 10 tables + sync_state
    }

    #[test]
    fn test_migrate_sets_schema_version() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();

        let version: String = db
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "1");
    }

    #[test]
    fn test_migrate_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();
        db.migrate().unwrap(); // should not error
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        let fk: i32 = db
            .conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn test_wal_mode_enabled() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        let mode: String = db
            .conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn test_portfolio_default_state() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();

        // Insert a portfolio without specifying state
        db.conn
            .execute(
                "INSERT INTO portfolios (id, name, created) VALUES (?1, ?2, ?3)",
                rusqlite::params!["test-portfolio", "Test", "2026-09-08T00:00:00Z"],
            )
            .unwrap();

        let state: String = db
            .conn
            .query_row(
                "SELECT state FROM portfolios WHERE id = 'test-portfolio'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "curated");
    }

    #[test]
    fn test_project_default_state() {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();

        // Insert parent portfolio first (foreign key constraint is enforced)
        db.conn
            .execute(
                "INSERT INTO portfolios (id, name, created) VALUES (?1, ?2, ?3)",
                rusqlite::params!["test-portfolio", "Test", "2026-09-08T00:00:00Z"],
            )
            .unwrap();

        // Insert a project without specifying state
        db.conn
            .execute(
                "INSERT INTO projects (portfolio_id, name, repo_path, tickets_dir, registered_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    "test-portfolio",
                    "tkr",
                    "/tmp/tkr",
                    "/tmp/tkr/.tickets",
                    "2026-09-08T00:00:00Z"
                ],
            )
            .unwrap();

        let state: String = db
            .conn
            .query_row(
                "SELECT state FROM projects WHERE name = 'tkr'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "seeded");
    }

    #[test]
    fn test_db_path_ends_with_portfolio_db() {
        let path = PortfolioDb::db_path().unwrap();
        assert!(path.to_string_lossy().ends_with("portfolio.db"));
        assert!(path.to_string_lossy().contains("tkr"));
    }
}
