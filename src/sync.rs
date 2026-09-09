use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::db::{PortfolioDb, Project};
use crate::ticket::Ticket;

/// Status subdirectories to scan for markdown ticket files.
const STATUS_DIRS: &[&str] = &[
    "open",
    "in_progress",
    "closed",
    "blocked",
    "ready",
    "icebox",
    "archive",
];

/// Per-project sync result.
pub struct ProjectSyncResult {
    pub project_id: i64,
    pub project_name: String,
    pub tasks_indexed: usize,
    pub tasks_updated: usize,
    pub tasks_removed: usize,
    pub files_skipped: usize,
}

/// Aggregate report from a full sync operation.
pub struct SyncReport {
    pub projects: Vec<ProjectSyncResult>,
    pub tasks_indexed: usize,
    pub tasks_updated: usize,
    pub tasks_removed: usize,
    pub files_skipped: usize,
}

/// Sync state read from the `sync_state` table.
pub struct SyncState {
    pub last_sync_at: Option<String>,
    pub project_count: i64,
    pub task_count: i64,
}

/// Result of upserting a single task.
pub enum UpsertResult {
    Inserted,
    Updated,
    Skipped,
}

/// Manages markdown-to-SQLite sync operations.
pub struct SyncManager<'a> {
    db: &'a PortfolioDb,
}

impl<'a> SyncManager<'a> {
    pub fn new(db: &'a PortfolioDb) -> Self {
        Self { db }
    }

    /// Compute SHA-256 hash of content as a lowercase hex string.
    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }

    /// List all registered projects from the DB.
    pub fn list_registered_projects(&self) -> Result<Vec<Project>> {
        self.db.list_projects(None)
    }

    /// Scan a tickets directory for all markdown files in status subdirectories.
    pub fn scan_markdown_files(tickets_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for status in STATUS_DIRS {
            let status_dir = tickets_dir.join(status);
            if status_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&status_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                            files.push(path);
                        }
                    }
                }
            }
        }
        Ok(files)
    }

    /// Parse a markdown ticket file, returning the parsed Ticket and content hash.
    pub fn parse_ticket_file(path: &Path) -> Result<(Ticket, String)> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))?;

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            anyhow::bail!("Invalid ticket format - expected frontmatter with --- separators");
        }

        let yaml_content = parts[1].trim();
        let ticket: Ticket = serde_yaml::from_str(yaml_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML: {}", e))?;

        let hash = Self::compute_hash(&content);
        Ok((ticket, hash))
    }

    /// Upsert a task into the tasks table. Returns whether the task was
    /// inserted, updated, or skipped (hash unchanged).
    pub fn upsert_task(
        &self,
        project_id: i64,
        ticket: &Ticket,
        markdown_path: &str,
        hash: &str,
    ) -> Result<UpsertResult> {
        // Check if task exists and get current hash
        let existing_hash: Option<String> = self
            .db
            .conn
            .query_row(
                "SELECT markdown_hash FROM tasks WHERE id = ?1",
                rusqlite::params![ticket.id],
                |row| row.get(0),
            )
            .ok();

        let synced_at = chrono::Utc::now().to_rfc3339();

        match existing_hash {
            Some(current_hash) if current_hash == hash => Ok(UpsertResult::Skipped),
            Some(_) => {
                self.db.conn.execute(
                    "UPDATE tasks SET
                        project_id = ?1, title = ?2, state = ?3, priority = ?4,
                        issue_type = ?5, markdown_path = ?6, markdown_hash = ?7,
                        synced_at = ?8
                     WHERE id = ?9",
                    rusqlite::params![
                        project_id,
                        ticket.title,
                        ticket.status,
                        ticket.priority,
                        ticket.issue_type,
                        markdown_path,
                        hash,
                        synced_at,
                        ticket.id,
                    ],
                )?;
                Ok(UpsertResult::Updated)
            }
            None => {
                self.db.conn.execute(
                    "INSERT INTO tasks
                        (id, project_id, title, state, priority, issue_type,
                         markdown_path, markdown_hash, synced_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    rusqlite::params![
                        ticket.id,
                        project_id,
                        ticket.title,
                        ticket.status,
                        ticket.priority,
                        ticket.issue_type,
                        markdown_path,
                        hash,
                        synced_at,
                    ],
                )?;
                Ok(UpsertResult::Inserted)
            }
        }
    }

    /// Remove tasks for a project whose IDs are not in the current set.
    /// Returns the number of removed tasks.
    pub fn remove_stale_tasks(&self, project_id: i64, current_ids: &[String]) -> Result<usize> {
        let mut stmt = self.db.conn.prepare("SELECT id FROM tasks WHERE project_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![project_id], |row| {
            let id: String = row.get(0)?;
            Ok(id)
        })?;

        let mut stale_ids = Vec::new();
        for row in rows {
            let id = row?;
            if !current_ids.contains(&id) {
                stale_ids.push(id);
            }
        }

        for id in &stale_ids {
            self.db
                .conn
                .execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![id])?;
        }

        Ok(stale_ids.len())
    }

    /// Orchestrate the full sync: iterate over all registered projects,
    /// scan their tickets directories, parse and upsert tasks, remove stale ones.
    pub fn sync_all(&self) -> Result<SyncReport> {
        let mut report = SyncReport {
            projects: Vec::new(),
            tasks_indexed: 0,
            tasks_updated: 0,
            tasks_removed: 0,
            files_skipped: 0,
        };

        let projects = self.list_registered_projects()?;

        for project in &projects {
            let mut project_result = ProjectSyncResult {
                project_id: project.id,
                project_name: project.name.clone(),
                tasks_indexed: 0,
                tasks_updated: 0,
                tasks_removed: 0,
                files_skipped: 0,
            };

            let tickets_dir = PathBuf::from(&project.tickets_dir);
            let files = Self::scan_markdown_files(&tickets_dir)?;

            let mut current_ids = Vec::new();

            for file_path in &files {
                match Self::parse_ticket_file(file_path) {
                    Ok((ticket, hash)) => {
                        let markdown_path = file_path
                            .canonicalize()
                            .unwrap_or_else(|_| file_path.clone())
                            .to_string_lossy()
                            .to_string();

                        match self.upsert_task(project.id, &ticket, &markdown_path, &hash)? {
                            UpsertResult::Inserted => {
                                project_result.tasks_indexed += 1;
                            }
                            UpsertResult::Updated => {
                                project_result.tasks_updated += 1;
                            }
                            UpsertResult::Skipped => {}
                        }
                        current_ids.push(ticket.id);
                    }
                    Err(e) => {
                        eprintln!(
                            "  Warning: Failed to parse {}: {}",
                            file_path.display(),
                            e
                        );
                        project_result.files_skipped += 1;
                    }
                }
            }

            let removed = self.remove_stale_tasks(project.id, &current_ids)?;
            project_result.tasks_removed = removed;

            report.tasks_indexed += project_result.tasks_indexed;
            report.tasks_updated += project_result.tasks_updated;
            report.tasks_removed += project_result.tasks_removed;
            report.files_skipped += project_result.files_skipped;
            report.projects.push(project_result);
        }

        // Update sync_state
        let now = chrono::Utc::now().to_rfc3339();
        let total_tasks: i64 = self
            .db
            .conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;

        self.db.conn.execute(
            "INSERT INTO sync_state (key, value) VALUES ('last_sync_at', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![now],
        )?;
        self.db.conn.execute(
            "INSERT INTO sync_state (key, value) VALUES ('total_tasks', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![total_tasks.to_string()],
        )?;

        Ok(report)
    }

    /// Read sync state from the sync_state table.
    pub fn sync_status(&self) -> Result<SyncState> {
        let last_sync_at: Option<String> = self
            .db
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'last_sync_at'",
                [],
                |row| row.get(0),
            )
            .ok();

        let project_count: i64 = self
            .db
            .conn
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))?;

        let task_count: i64 = self
            .db
            .conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;

        Ok(SyncState {
            last_sync_at,
            project_count,
            task_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::PortfolioDb;
    use std::fs;
    use tempfile::TempDir;

    fn setup_db() -> (TempDir, PortfolioDb) {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();
        (temp, db)
    }

    #[test]
    fn test_sync_hash() {
        let content = "---\nid: ja-test\ntitle: Test\n---\n";
        let hash = SyncManager::compute_hash(content);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_sync_hash_deterministic() {
        let content = "hello world";
        let h1 = SyncManager::compute_hash(content);
        let h2 = SyncManager::compute_hash(content);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sync_list_projects() {
        let (_temp, db) = setup_db();
        let sync = SyncManager::new(&db);
        let projects = sync.list_registered_projects().unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn test_sync_scan_files() {
        let temp = TempDir::new().unwrap();
        let tickets_dir = temp.path().join(".tickets");
        let open_dir = tickets_dir.join("open");
        fs::create_dir_all(&open_dir).unwrap();
        fs::write(open_dir.join("ja-test001.md"), "---\n---\n").unwrap();

        let files = SyncManager::scan_markdown_files(&tickets_dir).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("ja-test001.md"));
    }

    #[test]
    fn test_sync_scan_files_empty() {
        let temp = TempDir::new().unwrap();
        let tickets_dir = temp.path().join(".tickets");
        let files = SyncManager::scan_markdown_files(&tickets_dir).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_sync_parse_ticket() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.md");
        let content = "---\nid: ja-test001\ntitle: Test Ticket\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Test Ticket\n";
        fs::write(&path, content).unwrap();

        let (ticket, hash) = SyncManager::parse_ticket_file(&path).unwrap();
        assert_eq!(ticket.id, "ja-test001");
        assert_eq!(ticket.title, "Test Ticket");
        assert_eq!(ticket.status, "open");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sync_parse_ticket_invalid() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("broken.md");
        fs::write(&path, "This is not valid YAML\n").unwrap();

        let result = SyncManager::parse_ticket_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_upsert() {
        let (_temp, db) = setup_db();

        // Create portfolio and project
        db.create_portfolio("personal", "Personal", None).unwrap();
        let project_id = db
            .register_project(
                "personal",
                "test-project",
                "/tmp/test-repo",
                "/tmp/test-repo/.tickets",
                None,
                None,
            )
            .unwrap();

        let sync = SyncManager::new(&db);

        let ticket = Ticket {
            id: "ja-test001".to_string(),
            title: "Test Ticket".to_string(),
            status: "open".to_string(),
            deps: Vec::new(),
            links: Vec::new(),
            created: chrono::Utc::now(),
            issue_type: "task".to_string(),
            priority: 2,
            description: None,
            design: None,
            acceptance: None,
            assignee: None,
            external_ref: None,
            parent: None,
            project: None,
            category: None,
            notes: None,
        };

        // Insert
        let result = sync
            .upsert_task(project_id, &ticket, "/tmp/test.md", "abc123", )
            .unwrap();
        assert!(matches!(result, UpsertResult::Inserted));

        // Skip (same hash)
        let result = sync
            .upsert_task(project_id, &ticket, "/tmp/test.md", "abc123")
            .unwrap();
        assert!(matches!(result, UpsertResult::Skipped));

        // Update (different hash)
        let result = sync
            .upsert_task(project_id, &ticket, "/tmp/test.md", "def456")
            .unwrap();
        assert!(matches!(result, UpsertResult::Updated));
    }

    #[test]
    fn test_sync_remove_stale() {
        let (_temp, db) = setup_db();

        db.create_portfolio("personal", "Personal", None).unwrap();
        let project_id = db
            .register_project(
                "personal",
                "test-project",
                "/tmp/test-repo",
                "/tmp/test-repo/.tickets",
                None,
                None,
            )
            .unwrap();

        let sync = SyncManager::new(&db);

        // Insert two tasks
        for id in &["ja-001", "ja-002"] {
            let ticket = Ticket {
                id: id.to_string(),
                title: "Test".to_string(),
                status: "open".to_string(),
                deps: Vec::new(),
                links: Vec::new(),
                created: chrono::Utc::now(),
                issue_type: "task".to_string(),
                priority: 2,
                description: None,
                design: None,
                acceptance: None,
                assignee: None,
                external_ref: None,
                parent: None,
                project: None,
                category: None,
                notes: None,
            };
            sync.upsert_task(project_id, &ticket, "/tmp/test.md", "hash").unwrap();
        }

        // Only ja-001 is current — ja-002 should be removed
        let removed = sync.remove_stale_tasks(project_id, &["ja-001".to_string()]).unwrap();
        assert_eq!(removed, 1);

        // Verify only one task remains
        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sync_all() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("portfolio.db");
        let db = PortfolioDb::open(&db_path).unwrap();
        db.migrate().unwrap();

        // Create portfolio and project
        db.create_portfolio("personal", "Personal", None).unwrap();

        // Create tickets directory with a markdown file
        let repo_dir = temp.path().join("myrepo");
        let tickets_dir = repo_dir.join(".tickets");
        let open_dir = tickets_dir.join("open");
        fs::create_dir_all(&open_dir).unwrap();

        let ticket_content = "---\nid: ja-test001\ntitle: Test Ticket\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Test Ticket\n";
        fs::write(open_dir.join("ja-test001.md"), ticket_content).unwrap();

        let _project_id = db
            .register_project(
                "personal",
                "myrepo",
                repo_dir.to_str().unwrap(),
                tickets_dir.to_str().unwrap(),
                None,
                None,
            )
            .unwrap();

        let sync = SyncManager::new(&db);
        let report = sync.sync_all().unwrap();

        assert_eq!(report.projects.len(), 1);
        assert_eq!(report.tasks_indexed, 1);
        assert_eq!(report.tasks_updated, 0);
        assert_eq!(report.tasks_removed, 0);

        // Second sync — should be idempotent
        let report2 = sync.sync_all().unwrap();
        assert_eq!(report2.tasks_indexed, 0);
        assert_eq!(report2.tasks_updated, 0);
    }

    #[test]
    fn test_sync_status() {
        let (_temp, db) = setup_db();
        let sync = SyncManager::new(&db);

        // Before any sync
        let state = sync.sync_status().unwrap();
        assert!(state.last_sync_at.is_none());
        assert_eq!(state.project_count, 0);
        assert_eq!(state.task_count, 0);
    }
}
