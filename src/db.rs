use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

/// Current schema version. Increment every time a new migration step is added
/// to [`PortfolioDb::migrate`].
pub const SCHEMA_VERSION: u32 = 1;

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

    /// Resolve the default portfolio database path using the `directories`
    /// crate (`~/.local/share/tkr/portfolio.db` on Linux,
    /// `~/Library/Application Support/tkr/portfolio.db` on macOS).
    pub fn db_path() -> Result<PathBuf> {
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
