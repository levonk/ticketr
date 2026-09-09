//! Bidirectional GitHub Issues sync for tkr tasks.
//!
//! This module syncs tkr tasks (stored as markdown files, indexed in SQLite)
//! to/from GitHub Issues across multiple GitHub accounts. Authentication is
//! handled by shelling out to the `gh` CLI (`gh auth token`) — no tokens are
//! stored in the database.
//!
//! ## Conflict resolution
//!
//! When both the local markdown and the GitHub Issue have changed since the
//! last sync, the last-write-wins strategy is applied by comparing timestamps.
//! A warning is logged for every conflict.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use crate::db::{PortfolioDb, Project};
use crate::ticket::TicketManager;

/// A boxed future returned by [`GitHubClient`] trait methods.
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// Maximum length of a GitHub label name.
const MAX_LABEL_LEN: usize = 50;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A GitHub issue as returned by the API (normalised from the raw JSON).
#[derive(Debug, Clone)]
pub struct GithubIssue {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    /// `"open"` or `"closed"`.
    pub state: String,
    /// RFC-3339 timestamp of the last update.
    pub updated_at: String,
    pub html_url: String,
    /// Label names.
    pub labels: Vec<String>,
}

/// Response from listing issues, including the ETag for efficient polling.
#[derive(Debug, Clone)]
pub struct IssueListResponse {
    pub issues: Vec<GithubIssue>,
    pub etag: Option<String>,
    /// `true` when GitHub returned 304 Not Modified.
    pub not_modified: bool,
}

/// Fields for creating a GitHub issue.
#[derive(Debug, Clone)]
pub struct IssueCreate {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
}

/// Fields for updating a GitHub issue.
#[derive(Debug, Clone)]
pub struct IssueUpdate {
    pub title: String,
    pub body: String,
    /// `"open"` or `"closed"`.
    pub state: String,
    pub labels: Vec<String>,
}

// ---------------------------------------------------------------------------
// Sync report types
// ---------------------------------------------------------------------------

/// Per-project GitHub sync result.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GitHubProjectSyncResult {
    pub project_id: i64,
    pub project_name: String,
    pub github_owner: String,
    pub github_repo: String,
    pub github_account: Option<String>,
    pub pushed: usize,
    pub pulled: usize,
    pub conflicts: usize,
    pub errors: usize,
}

/// Aggregate report from a full GitHub sync operation.
#[derive(Debug, Clone, Default)]
pub struct GitHubSyncReport {
    pub projects: Vec<GitHubProjectSyncResult>,
    pub total_pushed: usize,
    pub total_pulled: usize,
    pub total_conflicts: usize,
    pub total_errors: usize,
}

/// GitHub sync state read from the `sync_state` table.
#[derive(Debug, Clone)]
pub struct GitHubSyncState {
    pub last_github_sync: Option<String>,
    pub github_sync_errors: i64,
    pub projects_synced: i64,
    pub total_tasks_linked: i64,
}

// ---------------------------------------------------------------------------
// Conflict resolution
// ---------------------------------------------------------------------------

/// The outcome of comparing local and remote timestamps for a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Neither side changed since the last sync.
    BothUnchanged,
    /// Only the local side changed — push local.
    LocalOnly,
    /// Only the remote side changed — pull remote.
    RemoteOnly,
    /// Both sides changed; local timestamp is later.
    BothChangedLocalNewer,
    /// Both sides changed; remote timestamp is later.
    BothChangedRemoteNewer,
    /// Both sides changed; timestamps are equal (treat as no-op).
    BothChangedEqual,
}

/// Determine the conflict resolution for a task given the local `synced_at`
/// (when the DB was last updated from markdown), the remote `updated_at`
/// (GitHub issue), and the `github_synced_at` (when we last synced with
/// GitHub).
///
/// If both sides changed since `github_synced_at`, last-write-wins by
/// comparing `local_ts` and `remote_ts`.
pub fn resolve_conflict(
    local_ts: &str,
    remote_ts: &str,
    last_sync_ts: Option<&str>,
) -> ConflictResolution {
    let local_changed = match last_sync_ts {
        Some(synced) => local_ts > synced,
        None => true,
    };
    let remote_changed = match last_sync_ts {
        Some(synced) => remote_ts > synced,
        None => true,
    };

    match (local_changed, remote_changed) {
        (false, false) => ConflictResolution::BothUnchanged,
        (true, false) => ConflictResolution::LocalOnly,
        (false, true) => ConflictResolution::RemoteOnly,
        (true, true) => {
            if local_ts > remote_ts {
                ConflictResolution::BothChangedLocalNewer
            } else if remote_ts > local_ts {
                ConflictResolution::BothChangedRemoteNewer
            } else {
                ConflictResolution::BothChangedEqual
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Mapping functions (pure, easily testable)
// ---------------------------------------------------------------------------

/// Map a tkr task state to a GitHub issue state.
///
/// `closed` → GitHub `closed`; all other tkr states → GitHub `open`.
pub fn tkr_state_to_github(state: &str) -> &'static str {
    match state {
        "closed" => "closed",
        _ => "open",
    }
}

/// Map a GitHub issue state to a tkr task state.
///
/// GitHub `closed` → tkr `closed`; GitHub `open` → tkr `open`.
pub fn github_state_to_tkr(state: &str) -> &'static str {
    match state {
        "closed" => "closed",
        _ => "open",
    }
}

/// Sanitise a tag name for use as a GitHub label.
///
/// GitHub label names are lowercased, spaces replaced with hyphens, and
/// truncated to 50 characters.
pub fn sanitize_label_name(tag: &str) -> String {
    let sanitised: String = tag
        .to_lowercase()
        .chars()
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .collect();
    sanitised.chars().take(MAX_LABEL_LEN).collect()
}

/// Map a list of tkr tags to GitHub label names (sanitised).
pub fn tags_to_labels(tags: &[String]) -> Vec<String> {
    tags.iter().map(|t| sanitize_label_name(t)).collect()
}

/// Map a list of GitHub label names back to tkr tags (lowercased).
pub fn labels_to_tags(labels: &[String]) -> Vec<String> {
    labels.iter().map(|l| l.to_lowercase()).collect()
}

// ---------------------------------------------------------------------------
// GitHubClient trait
// ---------------------------------------------------------------------------

/// Abstraction over the GitHub REST API v3, allowing tests to provide a
/// mock implementation that makes no real network calls.
pub trait GitHubClient: Send + Sync {
    /// List issues for a repo. If `etag` is provided, it is sent as
    /// `If-None-Match`; the response may indicate `not_modified`.
    fn list_issues<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        etag: Option<&'a str>,
    ) -> BoxFuture<'a, IssueListResponse>;

    /// Create a new issue in a repo.
    fn create_issue<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        issue: IssueCreate,
    ) -> BoxFuture<'a, GithubIssue>;

    /// Update an existing issue in a repo.
    fn update_issue<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        number: i64,
        issue: IssueUpdate,
    ) -> BoxFuture<'a, GithubIssue>;

    /// Ensure a label exists in a repo, creating it if missing.
    fn ensure_label<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        name: &'a str,
    ) -> BoxFuture<'a, ()>;
}

// ---------------------------------------------------------------------------
// ReqwestClient — real implementation using reqwest + gh auth token
// ---------------------------------------------------------------------------

/// Real GitHub API client using `reqwest` for HTTP and `gh auth token` for
/// authentication. No tokens are stored — they are fetched on demand.
pub struct ReqwestClient {
    http: reqwest::Client,
    /// Optional GitHub account name (used for logging; the token is always
    /// fetched from the active `gh auth` context).
    #[allow(dead_code)]
    account: Option<String>,
}

impl ReqwestClient {
    /// Create a new client. If `account` is `Some`, it is used for
    /// informational logging; the token is still fetched from `gh auth token`.
    pub fn new(account: Option<String>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent("tkr-github-sync")
            .build()?;
        Ok(Self { http, account })
    }

    /// Fetch a GitHub auth token by shelling out to `gh auth token`.
    fn get_token(&self) -> Result<String> {
        let output = std::process::Command::new("gh")
            .arg("auth")
            .arg("token")
            .output()
            .context("Failed to run `gh auth token` — is the gh CLI installed?")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("`gh auth token` failed: {}", stderr.trim());
        }
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if token.is_empty() {
            anyhow::bail!("`gh auth token` returned an empty token");
        }
        Ok(token)
    }

    /// Build an authenticated request builder.
    fn request(&self, method: reqwest::Method, url: &str) -> Result<reqwest::RequestBuilder> {
        let token = self.get_token()?;
        Ok(self
            .http
            .request(method, url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28"))
    }

    const API_BASE: &'static str = "https://api.github.com";
}

impl GitHubClient for ReqwestClient {
    fn list_issues<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        etag: Option<&'a str>,
    ) -> BoxFuture<'a, IssueListResponse> {
        Box::pin(async move {
            let url = format!("{}/repos/{}/{}/issues?state=all&per_page=100", Self::API_BASE, owner, repo);
            let mut req = self.request(reqwest::Method::GET, &url)?;
            if let Some(etag) = etag {
                req = req.header("If-None-Match", etag);
            }
            let resp = req.send().await?;
            let status = resp.status();
            let response_etag = resp
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            if status == reqwest::StatusCode::NOT_MODIFIED {
                return Ok(IssueListResponse {
                    issues: Vec::new(),
                    etag: response_etag,
                    not_modified: true,
                });
            }

            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("GitHub API error {}: {}", status, body);
            }

            let raw_issues: Vec<GithubIssueResponse> = resp.json().await?;
            let issues = raw_issues.into_iter().map(|r| r.into()).collect();
            Ok(IssueListResponse {
                issues,
                etag: response_etag,
                not_modified: false,
            })
        })
    }

    fn create_issue<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        issue: IssueCreate,
    ) -> BoxFuture<'a, GithubIssue> {
        Box::pin(async move {
            let url = format!("{}/repos/{}/{}/issues", Self::API_BASE, owner, repo);
            let body = IssueCreateRequest {
                title: issue.title,
                body: issue.body,
                labels: issue.labels,
            };
            let resp = self
                .request(reqwest::Method::POST, &url)?
                .json(&body)
                .send()
                .await?;
            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                anyhow::bail!("GitHub API error creating issue: {} - {}", status, text);
            }
            let raw: GithubIssueResponse = resp.json().await?;
            Ok(raw.into())
        })
    }

    fn update_issue<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        number: i64,
        issue: IssueUpdate,
    ) -> BoxFuture<'a, GithubIssue> {
        Box::pin(async move {
            let url = format!("{}/repos/{}/{}/issues/{}", Self::API_BASE, owner, repo, number);
            let body = IssueUpdateRequest {
                title: issue.title,
                body: issue.body,
                state: issue.state,
                labels: issue.labels,
            };
            let resp = self
                .request(reqwest::Method::PATCH, &url)?
                .json(&body)
                .send()
                .await?;
            let status = resp.status();
            if !status.is_success() {
                let text = resp.text().await.unwrap_or_default();
                anyhow::bail!("GitHub API error updating issue: {} - {}", status, text);
            }
            let raw: GithubIssueResponse = resp.json().await?;
            Ok(raw.into())
        })
    }

    fn ensure_label<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        name: &'a str,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            // Check if the label exists.
            let check_url = format!("{}/repos/{}/{}/labels/{}", Self::API_BASE, owner, repo, name);
            let resp = self.request(reqwest::Method::GET, &check_url)?.send().await?;
            if resp.status().is_success() {
                return Ok(());
            }
            // Label doesn't exist — create it.
            let create_url = format!("{}/repos/{}/{}/labels", Self::API_BASE, owner, repo);
            let body = LabelCreateRequest {
                name: name.to_string(),
            };
            let resp = self
                .request(reqwest::Method::POST, &create_url)?
                .json(&body)
                .send()
                .await?;
            if !resp.status().is_success() {
                // Another process may have created it concurrently — check again.
                let text = resp.text().await.unwrap_or_default();
                if text.contains("already_exists") {
                    return Ok(());
                }
                anyhow::bail!("GitHub API error creating label: {}", text);
            }
            Ok(())
        })
    }
}

// ---------------------------------------------------------------------------
// Serde structs for GitHub API JSON
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GithubIssueResponse {
    number: i64,
    title: String,
    body: Option<String>,
    state: String,
    updated_at: String,
    html_url: String,
    labels: Vec<GithubLabelResponse>,
}

impl From<GithubIssueResponse> for GithubIssue {
    fn from(r: GithubIssueResponse) -> Self {
        Self {
            number: r.number,
            title: r.title,
            body: r.body,
            state: r.state,
            updated_at: r.updated_at,
            html_url: r.html_url,
            labels: r.labels.into_iter().map(|l| l.name).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GithubLabelResponse {
    name: String,
}

#[derive(Debug, Serialize)]
struct IssueCreateRequest {
    title: String,
    body: String,
    labels: Vec<String>,
}

#[derive(Debug, Serialize)]
struct IssueUpdateRequest {
    title: String,
    body: String,
    state: String,
    labels: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LabelCreateRequest {
    name: String,
}

// ---------------------------------------------------------------------------
// GitHubSync — orchestrator
// ---------------------------------------------------------------------------

/// Manages bidirectional GitHub Issues sync for all registered projects.
pub struct GitHubSync<'a> {
    db: &'a PortfolioDb,
}

/// Internal representation of a task's sync state, read from the DB.
#[allow(dead_code)]
struct TaskSyncState {
    id: String,
    title: String,
    state: String,
    markdown_path: String,
    synced_at: String,
    github_issue_number: Option<i64>,
    github_issue_url: Option<String>,
    github_synced_at: Option<String>,
    tags: Vec<String>,
}

impl<'a> GitHubSync<'a> {
    pub fn new(db: &'a PortfolioDb) -> Self {
        Self { db }
    }

    /// List all registered projects that have GitHub owner and repo set.
    fn list_github_projects(&self) -> Result<Vec<Project>> {
        let projects = self.db.list_projects(None)?;
        Ok(projects
            .into_iter()
            .filter(|p| p.github_owner.is_some() && p.github_repo.is_some())
            .collect())
    }

    /// Read all tasks for a project from the DB, including their tags.
    fn list_tasks_for_project(&self, project_id: i64) -> Result<Vec<TaskSyncState>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, title, state, markdown_path, synced_at,
                    github_issue_number, github_issue_url, github_synced_at
             FROM tasks WHERE project_id = ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id], |row| {
            Ok(TaskSyncState {
                id: row.get(0)?,
                title: row.get(1)?,
                state: row.get(2)?,
                markdown_path: row.get(3)?,
                synced_at: row.get(4)?,
                github_issue_number: row.get(5)?,
                github_issue_url: row.get(6)?,
                github_synced_at: row.get(7)?,
                tags: Vec::new(),
            })
        })?;

        let mut tasks: Vec<TaskSyncState> = rows.filter_map(|r| r.ok()).collect();
        for task in &mut tasks {
            task.tags = self.get_task_tags(&task.id)?;
        }
        Ok(tasks)
    }

    /// Get the tag names for a single task.
    fn get_task_tags(&self, task_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT t.name FROM tags t
             JOIN task_tags tt ON tt.tag_id = t.id
             WHERE tt.task_id = ?1
             ORDER BY t.name",
        )?;
        let rows = stmt.query_map(rusqlite::params![task_id], |row| row.get::<_, String>(0))?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    /// Store GitHub issue info for a task in the DB.
    fn update_task_github_info(
        &self,
        task_id: &str,
        issue_number: Option<i64>,
        issue_url: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.conn.execute(
            "UPDATE tasks SET github_issue_number = ?1, github_issue_url = ?2,
                              github_synced_at = ?3 WHERE id = ?4",
            rusqlite::params![issue_number, issue_url, now, task_id],
        )?;
        Ok(())
    }

    /// Update only the `github_synced_at` timestamp for a task.
    fn touch_task_github_synced_at(&self, task_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.conn.execute(
            "UPDATE tasks SET github_synced_at = ?1 WHERE id = ?2",
            rusqlite::params![now, task_id],
        )?;
        Ok(())
    }

    /// Get the stored ETag for a project from the `sync_state` table.
    fn get_project_etag(&self, project_id: i64) -> Result<Option<String>> {
        let key = format!("github_etag:{}", project_id);
        let etag: Option<String> = self
            .db
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            )
            .ok();
        Ok(etag)
    }

    /// Store the ETag for a project in the `sync_state` table.
    fn set_project_etag(&self, project_id: i64, etag: Option<&str>) -> Result<()> {
        let key = format!("github_etag:{}", project_id);
        match etag {
            Some(value) => {
                self.db.conn.execute(
                    "INSERT INTO sync_state (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    rusqlite::params![key, value],
                )?;
            }
            None => {
                self.db.conn.execute(
                    "DELETE FROM sync_state WHERE key = ?1",
                    rusqlite::params![key],
                )?;
            }
        }
        Ok(())
    }

    /// Update the `sync_state` table after a GitHub sync.
    fn update_sync_state(&self, report: &GitHubSyncReport) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.db.conn.execute(
            "INSERT INTO sync_state (key, value) VALUES ('last_github_sync', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![now],
        )?;
        self.db.conn.execute(
            "INSERT INTO sync_state (key, value) VALUES ('github_sync_errors', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![report.total_errors.to_string()],
        )?;
        self.db.conn.execute(
            "INSERT INTO sync_state (key, value) VALUES ('github_projects_synced', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![report.projects.len().to_string()],
        )?;
        Ok(())
    }

    /// Run a full bidirectional GitHub sync for all registered projects.
    ///
    /// If `dry_run` is `true`, no API calls are made — the method reports what
    /// *would* be synced based on the DB state.
    pub async fn sync(
        &self,
        client: Option<&dyn GitHubClient>,
        dry_run: bool,
    ) -> Result<GitHubSyncReport> {
        let projects = self.list_github_projects()?;
        let mut report = GitHubSyncReport::default();

        for project in &projects {
            let result = self
                .sync_project(project, client, dry_run)
                .await
                .unwrap_or_else(|e| {
                    eprintln!(
                        "  Error syncing project {}: {}",
                        project.name, e
                    );
                    GitHubProjectSyncResult {
                        project_id: project.id,
                        project_name: project.name.clone(),
                        github_owner: project.github_owner.clone().unwrap_or_default(),
                        github_repo: project.github_repo.clone().unwrap_or_default(),
                        github_account: project.github_account.clone(),
                        pushed: 0,
                        pulled: 0,
                        conflicts: 0,
                        errors: 1,
                    }
                });

            report.total_pushed += result.pushed;
            report.total_pulled += result.pulled;
            report.total_conflicts += result.conflicts;
            report.total_errors += result.errors;
            report.projects.push(result);
        }

        if !dry_run {
            self.update_sync_state(&report)?;
        }

        Ok(report)
    }

    /// Sync a single project (pull first, then push).
    ///
    /// Pulling first ensures that conflict resolution sees the remote state
    /// before the push overwrites it. Tasks that were pulled (remote was newer)
    /// are skipped during the push phase.
    async fn sync_project(
        &self,
        project: &Project,
        client: Option<&dyn GitHubClient>,
        dry_run: bool,
    ) -> Result<GitHubProjectSyncResult> {
        let owner = project.github_owner.as_ref().unwrap();
        let repo = project.github_repo.as_ref().unwrap();
        let tasks = self.list_tasks_for_project(project.id)?;

        // Pull: GitHub → tkr (first, so conflict resolution sees remote state)
        let (pulled, conflicts, pulled_ids) = self
            .pull_project(project, owner, repo, &tasks, client, dry_run)
            .await?;

        // Push: tkr → GitHub (skip tasks that were just pulled)
        let pushed = self
            .push_project(owner, repo, &tasks, &pulled_ids, client, dry_run)
            .await?;

        Ok(GitHubProjectSyncResult {
            project_id: project.id,
            project_name: project.name.clone(),
            github_owner: owner.clone(),
            github_repo: repo.clone(),
            github_account: project.github_account.clone(),
            pushed,
            pulled,
            conflicts,
            errors: 0,
        })
    }

    /// Push changed tasks to GitHub Issues. Tasks whose IDs are in
    /// `skip_ids` are skipped (they were just pulled from GitHub).
    async fn push_project(
        &self,
        owner: &str,
        repo: &str,
        tasks: &[TaskSyncState],
        skip_ids: &std::collections::HashSet<String>,
        client: Option<&dyn GitHubClient>,
        dry_run: bool,
    ) -> Result<usize> {
        let mut pushed = 0;

        for task in tasks {
            if skip_ids.contains(&task.id) {
                continue;
            }

            // Determine if the local content changed since the last GitHub sync.
            let local_changed = task.github_synced_at.is_none()
                || task.synced_at.as_str()
                    > task.github_synced_at.as_deref().unwrap_or("");

            if !local_changed {
                continue;
            }

            let labels = tags_to_labels(&task.tags);
            let body = self
                .read_task_description(&task.markdown_path)
                .unwrap_or_default();

            if dry_run {
                if task.github_issue_number.is_some() {
                    eprintln!(
                        "  [dry-run] Would update issue #{} for task {}",
                        task.github_issue_number.unwrap(),
                        task.id
                    );
                } else {
                    eprintln!(
                        "  [dry-run] Would create issue for task {} ({})",
                        task.id, task.title
                    );
                }
                pushed += 1;
                continue;
            }

            let client = match client {
                Some(c) => c,
                None => anyhow::bail!("GitHub client required for non-dry-run sync"),
            };

            // Ensure labels exist on the repo.
            for label in &labels {
                if let Err(e) = client.ensure_label(owner, repo, label).await {
                    eprintln!("  Warning: could not ensure label '{}': {}", label, e);
                }
            }

            if let Some(number) = task.github_issue_number {
                // Update existing issue.
                let update = IssueUpdate {
                    title: task.title.clone(),
                    body: body.clone(),
                    state: tkr_state_to_github(&task.state).to_string(),
                    labels: labels.clone(),
                };
                if let Err(e) = client.update_issue(owner, repo, number, update).await {
                    eprintln!("  Error updating issue #{} for task {}: {}", number, task.id, e);
                    continue;
                }
                self.touch_task_github_synced_at(&task.id)?;
            } else {
                // Create new issue (GitHub always creates issues as "open").
                let create = IssueCreate {
                    title: task.title.clone(),
                    body: body.clone(),
                    labels: labels.clone(),
                };
                match client.create_issue(owner, repo, create).await {
                    Ok(issue) => {
                        self.update_task_github_info(
                            &task.id,
                            Some(issue.number),
                            Some(&issue.html_url),
                        )?;
                        // If the task is closed, close the newly created issue.
                        if tkr_state_to_github(&task.state) == "closed" {
                            let close_update = IssueUpdate {
                                title: task.title.clone(),
                                body: body.clone(),
                                state: "closed".to_string(),
                                labels: labels.clone(),
                            };
                            if let Err(e) =
                                client.update_issue(owner, repo, issue.number, close_update).await
                            {
                                eprintln!(
                                    "  Warning: could not close issue #{} for task {}: {}",
                                    issue.number, task.id, e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("  Error creating issue for task {}: {}", task.id, e);
                        continue;
                    }
                }
            }
            pushed += 1;
        }

        Ok(pushed)
    }

    /// Pull updated issues from GitHub and update tkr markdown files.
    ///
    /// Returns `(pulled_count, conflict_count, set_of_pulled_task_ids)`.
    async fn pull_project(
        &self,
        project: &Project,
        owner: &str,
        repo: &str,
        tasks: &[TaskSyncState],
        client: Option<&dyn GitHubClient>,
        dry_run: bool,
    ) -> Result<(usize, usize, std::collections::HashSet<String>)> {
        let mut pulled_ids = std::collections::HashSet::new();

        if dry_run {
            return Ok((0, 0, pulled_ids));
        }

        let client = match client {
            Some(c) => c,
            None => return Ok((0, 0, pulled_ids)),
        };

        let etag = self.get_project_etag(project.id)?;
        let response = client.list_issues(owner, repo, etag.as_deref()).await?;

        if response.not_modified {
            return Ok((0, 0, pulled_ids));
        }

        // Store the new ETag.
        self.set_project_etag(project.id, response.etag.as_deref())?;

        let mut pulled = 0;
        let mut conflicts = 0;

        for issue in &response.issues {
            // Find matching task by github_issue_number.
            let matching_task = tasks
                .iter()
                .find(|t| t.github_issue_number == Some(issue.number));

            let task = match matching_task {
                Some(t) => t,
                None => continue, // No matching task — skip (could create in future)
            };

            let resolution = resolve_conflict(
                &task.synced_at,
                &issue.updated_at,
                task.github_synced_at.as_deref(),
            );

            match resolution {
                ConflictResolution::BothUnchanged | ConflictResolution::LocalOnly => continue,
                ConflictResolution::RemoteOnly => {}
                ConflictResolution::BothChangedLocalNewer => {
                    eprintln!(
                        "  WARN  Conflict: task {}\n    Local changed at: {}\n    Remote changed at: {}\n    Resolution: local is newer, keeping local",
                        task.id, task.synced_at, issue.updated_at
                    );
                    conflicts += 1;
                    continue;
                }
                ConflictResolution::BothChangedRemoteNewer => {
                    eprintln!(
                        "  WARN  Conflict: task {}\n    Local changed at: {}\n    Remote changed at: {}\n    Resolution: remote is newer, pulling remote",
                        task.id, task.synced_at, issue.updated_at
                    );
                    conflicts += 1;
                }
                ConflictResolution::BothChangedEqual => {
                    conflicts += 1;
                    continue;
                }
            }

            // Update the markdown file from the issue.
            if let Err(e) = self.update_markdown_from_issue(project, task, issue) {
                eprintln!("  Error updating markdown for task {}: {}", task.id, e);
                continue;
            }

            self.touch_task_github_synced_at(&task.id)?;
            pulled_ids.insert(task.id.clone());
            pulled += 1;
        }

        Ok((pulled, conflicts, pulled_ids))
    }

    /// Read a task's description from its markdown file.
    fn read_task_description(&self, markdown_path: &str) -> Result<String> {
        let content = std::fs::read_to_string(markdown_path)
            .with_context(|| format!("Failed to read markdown: {}", markdown_path))?;
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Ok(String::new());
        }
        // The body is everything after the second `---`, minus the heading.
        let body = parts[2].trim();
        // Strip the leading `# Title` line if present.
        let body = body
            .lines()
            .skip_while(|l| l.starts_with('#') || l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(body.trim().to_string())
    }

    /// Update a task's markdown file from a GitHub issue.
    ///
    /// If the issue state differs from the current ticket status, the file
    /// is moved to the new status directory and the old file is removed.
    fn update_markdown_from_issue(
        &self,
        project: &Project,
        task: &TaskSyncState,
        issue: &GithubIssue,
    ) -> Result<()> {
        let tickets_dir = PathBuf::from(&project.tickets_dir);
        let manager = TicketManager::new(tickets_dir.clone(), None, None);

        let mut ticket = manager.load_ticket(&task.id)?;
        let old_status = ticket.status.clone();

        // Update title.
        ticket.title = issue.title.clone();
        // Update description from issue body.
        ticket.description = issue.body.clone();
        // Update state.
        let new_state = github_state_to_tkr(&issue.state);
        ticket.status = new_state.to_string();
        // Update tags from labels.
        ticket.tags = labels_to_tags(&issue.labels);

        manager.save_ticket(&ticket)?;

        // If the status changed, remove the old file from the old status dir.
        if old_status != new_state {
            let old_file = tickets_dir.join(&old_status).join(format!("{}.md", task.id));
            let new_file = tickets_dir.join(new_state).join(format!("{}.md", task.id));
            if old_file.exists() && old_file != new_file {
                std::fs::remove_file(&old_file)?;
            }
        }

        Ok(())
    }

    /// Read GitHub sync state from the `sync_state` table.
    pub fn status(&self) -> Result<GitHubSyncState> {
        let last_github_sync: Option<String> = self
            .db
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'last_github_sync'",
                [],
                |row| row.get(0),
            )
            .ok();

        let github_sync_errors: i64 = self
            .db
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'github_sync_errors'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let projects_synced: i64 = self
            .db
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'github_projects_synced'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let total_tasks_linked: i64 = self
            .db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE github_issue_number IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(GitHubSyncState {
            last_github_sync,
            github_sync_errors,
            projects_synced,
            total_tasks_linked,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::PortfolioDb;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // Pure function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_tkr_state_to_github_closed() {
        assert_eq!(tkr_state_to_github("closed"), "closed");
    }

    #[test]
    fn test_tkr_state_to_github_open_states() {
        assert_eq!(tkr_state_to_github("open"), "open");
        assert_eq!(tkr_state_to_github("in_progress"), "open");
        assert_eq!(tkr_state_to_github("blocked"), "open");
        assert_eq!(tkr_state_to_github("ready"), "open");
        assert_eq!(tkr_state_to_github("logged"), "open");
    }

    #[test]
    fn test_github_state_to_tkr_closed() {
        assert_eq!(github_state_to_tkr("closed"), "closed");
    }

    #[test]
    fn test_github_state_to_tkr_open() {
        assert_eq!(github_state_to_tkr("open"), "open");
    }

    #[test]
    fn test_sanitize_label_name_lowercase() {
        assert_eq!(sanitize_label_name("Backend"), "backend");
    }

    #[test]
    fn test_sanitize_label_name_spaces_to_hyphens() {
        assert_eq!(sanitize_label_name("high priority"), "high-priority");
    }

    #[test]
    fn test_sanitize_label_name_truncation() {
        let long = "a".repeat(100);
        let label = sanitize_label_name(&long);
        assert_eq!(label.len(), MAX_LABEL_LEN);
    }

    #[test]
    fn test_tags_to_labels() {
        let tags = vec!["Security".to_string(), "Back End".to_string()];
        let labels = tags_to_labels(&tags);
        assert_eq!(labels, vec!["security", "back-end"]);
    }

    #[test]
    fn test_labels_to_tags() {
        let labels = vec!["bug".to_string(), "Feature".to_string()];
        let tags = labels_to_tags(&labels);
        assert_eq!(tags, vec!["bug", "feature"]);
    }

    // -----------------------------------------------------------------------
    // Conflict resolution tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_resolve_conflict_both_unchanged() {
        let res = resolve_conflict("2026-01-01T10:00:00Z", "2026-01-01T10:00:00Z", Some("2026-01-01T10:00:00Z"));
        assert_eq!(res, ConflictResolution::BothUnchanged);
    }

    #[test]
    fn test_resolve_conflict_local_only() {
        let res = resolve_conflict("2026-01-01T11:00:00Z", "2026-01-01T10:00:00Z", Some("2026-01-01T10:00:00Z"));
        assert_eq!(res, ConflictResolution::LocalOnly);
    }

    #[test]
    fn test_resolve_conflict_remote_only() {
        let res = resolve_conflict("2026-01-01T10:00:00Z", "2026-01-01T11:00:00Z", Some("2026-01-01T10:00:00Z"));
        assert_eq!(res, ConflictResolution::RemoteOnly);
    }

    #[test]
    fn test_resolve_conflict_both_changed_local_newer() {
        let res = resolve_conflict("2026-01-01T12:00:00Z", "2026-01-01T11:00:00Z", Some("2026-01-01T10:00:00Z"));
        assert_eq!(res, ConflictResolution::BothChangedLocalNewer);
    }

    #[test]
    fn test_resolve_conflict_both_changed_remote_newer() {
        let res = resolve_conflict("2026-01-01T11:00:00Z", "2026-01-01T12:00:00Z", Some("2026-01-01T10:00:00Z"));
        assert_eq!(res, ConflictResolution::BothChangedRemoteNewer);
    }

    #[test]
    fn test_resolve_conflict_both_changed_equal() {
        let res = resolve_conflict("2026-01-01T11:00:00Z", "2026-01-01T11:00:00Z", Some("2026-01-01T10:00:00Z"));
        assert_eq!(res, ConflictResolution::BothChangedEqual);
    }

    #[test]
    fn test_resolve_conflict_no_last_sync() {
        // No last sync — both sides are "changed".
        let res = resolve_conflict("2026-01-01T11:00:00Z", "2026-01-01T12:00:00Z", None);
        assert_eq!(res, ConflictResolution::BothChangedRemoteNewer);
    }

    // -----------------------------------------------------------------------
    // Mock GitHub client
    // -----------------------------------------------------------------------

    /// A mock GitHub client for testing. Stores issues in memory and supports
    /// ETag-based 304 responses.
    #[derive(Default)]
    struct MockClient {
        issues: Arc<Mutex<HashMap<i64, GithubIssue>>>,
        next_number: Arc<Mutex<i64>>,
        labels: Arc<Mutex<Vec<String>>>,
        etag: Arc<Mutex<Option<String>>>,
        call_count: Arc<Mutex<MockCallCount>>,
    }

    #[derive(Default, Debug)]
    struct MockCallCount {
        list_issues: usize,
        create_issue: usize,
        update_issue: usize,
        ensure_label: usize,
    }

    impl MockClient {
        fn new() -> Self {
            Self::default()
        }

        fn seed_issue(&self, issue: GithubIssue) {
            self.issues.lock().unwrap().insert(issue.number, issue);
        }

        fn call_count(&self) -> std::sync::MutexGuard<'_, MockCallCount> {
            self.call_count.lock().unwrap()
        }
    }

    impl GitHubClient for MockClient {
        fn list_issues<'a>(
            &'a self,
            _owner: &'a str,
            _repo: &'a str,
            etag: Option<&'a str>,
        ) -> BoxFuture<'a, IssueListResponse> {
            Box::pin(async move {
                self.call_count.lock().unwrap().list_issues += 1;
                let stored_etag = self.etag.lock().unwrap().clone();
                if let (Some(req_etag), Some(stored)) = (etag, &stored_etag) {
                    if req_etag == stored {
                        return Ok(IssueListResponse {
                            issues: Vec::new(),
                            etag: stored_etag,
                            not_modified: true,
                        });
                    }
                }
                let issues: Vec<GithubIssue> =
                    self.issues.lock().unwrap().values().cloned().collect();
                let new_etag = Some(format!("etag-{}", chrono::Utc::now().timestamp()));
                *self.etag.lock().unwrap() = new_etag.clone();
                Ok(IssueListResponse {
                    issues,
                    etag: new_etag,
                    not_modified: false,
                })
            })
        }

        fn create_issue<'a>(
            &'a self,
            _owner: &'a str,
            _repo: &'a str,
            issue: IssueCreate,
        ) -> BoxFuture<'a, GithubIssue> {
            Box::pin(async move {
                self.call_count.lock().unwrap().create_issue += 1;
                let mut next = self.next_number.lock().unwrap();
                *next += 1;
                let number = *next;
                let gh_issue = GithubIssue {
                    number,
                    title: issue.title,
                    body: Some(issue.body),
                    state: "open".to_string(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    html_url: format!("https://github.com/test/repo/issues/{}", number),
                    labels: issue.labels,
                };
                self.issues.lock().unwrap().insert(number, gh_issue.clone());
                Ok(gh_issue)
            })
        }

        fn update_issue<'a>(
            &'a self,
            _owner: &'a str,
            _repo: &'a str,
            number: i64,
            issue: IssueUpdate,
        ) -> BoxFuture<'a, GithubIssue> {
            Box::pin(async move {
                self.call_count.lock().unwrap().update_issue += 1;
                let mut issues = self.issues.lock().unwrap();
                let existing = issues
                    .get(&number)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Issue #{} not found", number))?;
                let updated = GithubIssue {
                    number,
                    title: issue.title,
                    body: Some(issue.body),
                    state: issue.state,
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    html_url: existing.html_url,
                    labels: issue.labels,
                };
                issues.insert(number, updated.clone());
                Ok(updated)
            })
        }

        fn ensure_label<'a>(
            &'a self,
            _owner: &'a str,
            _repo: &'a str,
            name: &'a str,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.call_count.lock().unwrap().ensure_label += 1;
                let mut labels = self.labels.lock().unwrap();
                if !labels.contains(&name.to_string()) {
                    labels.push(name.to_string());
                }
                Ok(())
            })
        }
    }

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    fn setup_db() -> (TempDir, PortfolioDb) {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();
        (temp, db)
    }

    fn setup_github_project(db: &PortfolioDb, name: &str, owner: &str, repo: &str, account: Option<&str>) -> i64 {
        // Create portfolio only if it doesn't already exist.
        let portfolio_exists: bool = db
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM portfolios WHERE id = 'personal'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);
        if !portfolio_exists {
            db.create_portfolio("personal", "Personal", None).unwrap();
        }
        let project_id = db
            .register_project(
                "personal",
                name,
                &format!("/tmp/{}", name),
                &format!("/tmp/{}/.tickets", name),
                Some(owner),
                Some(repo),
            )
            .unwrap();
        if let Some(acct) = account {
            db.conn
                .execute(
                    "UPDATE projects SET github_account = ?1 WHERE id = ?2",
                    rusqlite::params![acct, project_id],
                )
                .unwrap();
        }
        project_id
    }

    fn insert_task(
        db: &PortfolioDb,
        project_id: i64,
        id: &str,
        title: &str,
        state: &str,
        markdown_path: &str,
        synced_at: &str,
        tags: &[&str],
    ) {
        db.conn
            .execute(
                "INSERT INTO tasks (id, project_id, title, state, priority, issue_type,
                                    markdown_path, markdown_hash, synced_at)
                 VALUES (?1, ?2, ?3, ?4, 2, 'task', ?5, 'hash', ?6)",
                rusqlite::params![id, project_id, title, state, markdown_path, synced_at],
            )
            .unwrap();
        for tag in tags {
            let tag_id = db.upsert_tag(tag).unwrap();
            db.conn
                .execute(
                    "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)
                     ON CONFLICT DO NOTHING",
                    rusqlite::params![id, tag_id],
                )
                .unwrap();
        }
    }

    // -----------------------------------------------------------------------
    // Sync integration tests (using mock client)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_sync_push_creates_issue() {
        let (_temp, db) = setup_db();
        let project_id = setup_github_project(&db, "test-repo", "owner", "repo", None);

        insert_task(
            &db,
            project_id,
            "ja-001",
            "Test Task",
            "open",
            "/tmp/test.md",
            "2026-01-01T12:00:00Z",
            &["security", "backend"],
        );

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();

        let report = sync.sync(Some(&client), false).await.unwrap();

        assert_eq!(report.total_pushed, 1);
        assert_eq!(report.total_pulled, 0);
        assert_eq!(report.projects.len(), 1);

        // Verify the issue was created in the mock.
        let calls = client.call_count();
        assert_eq!(calls.create_issue, 1);
        assert!(calls.ensure_label >= 2); // security + backend

        // Verify the DB was updated with the issue number.
        let issue_number: Option<i64> = db
            .conn
            .query_row(
                "SELECT github_issue_number FROM tasks WHERE id = 'ja-001'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(issue_number.is_some());
    }

    #[tokio::test]
    async fn test_sync_push_updates_existing_issue() {
        let (_temp, db) = setup_db();
        let project_id = setup_github_project(&db, "test-repo", "owner", "repo", None);

        // Insert a task that already has a GitHub issue linked.
        db.conn
            .execute(
                "INSERT INTO tasks (id, project_id, title, state, priority, issue_type,
                                    markdown_path, markdown_hash, synced_at,
                                    github_issue_number, github_issue_url, github_synced_at)
                 VALUES (?1, ?2, ?3, ?4, 2, 'task', ?5, 'hash', ?6,
                         42, 'https://github.com/owner/repo/issues/42', '2026-01-01T10:00:00Z')",
                rusqlite::params!["ja-001", project_id, "Old Title", "open", "/tmp/test.md", "2026-01-01T12:00:00Z"],
            )
            .unwrap();

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();
        client.seed_issue(GithubIssue {
            number: 42,
            title: "Old Title".to_string(),
            body: None,
            state: "open".to_string(),
            updated_at: "2026-01-01T10:00:00Z".to_string(),
            html_url: "https://github.com/owner/repo/issues/42".to_string(),
            labels: vec![],
        });

        let report = sync.sync(Some(&client), false).await.unwrap();

        assert_eq!(report.total_pushed, 1);
        let calls = client.call_count();
        assert_eq!(calls.update_issue, 1);
        assert_eq!(calls.create_issue, 0);
    }

    #[tokio::test]
    async fn test_sync_push_skips_unchanged() {
        let (_temp, db) = setup_db();
        let project_id = setup_github_project(&db, "test-repo", "owner", "repo", None);

        // Task synced_at equals github_synced_at — no change.
        db.conn
            .execute(
                "INSERT INTO tasks (id, project_id, title, state, priority, issue_type,
                                    markdown_path, markdown_hash, synced_at,
                                    github_issue_number, github_synced_at)
                 VALUES (?1, ?2, ?3, ?4, 2, 'task', ?5, 'hash', ?6,
                         42, ?6)",
                rusqlite::params!["ja-001", project_id, "Title", "open", "/tmp/test.md", "2026-01-01T12:00:00Z"],
            )
            .unwrap();

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();

        let report = sync.sync(Some(&client), false).await.unwrap();

        assert_eq!(report.total_pushed, 0);
        let calls = client.call_count();
        assert_eq!(calls.create_issue, 0);
        assert_eq!(calls.update_issue, 0);
    }

    #[tokio::test]
    async fn test_sync_pull_updates_markdown() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("portfolio.db");
        let db = PortfolioDb::open(&db_path).unwrap();
        db.migrate().unwrap();

        db.create_portfolio("personal", "Personal", None).unwrap();

        // Create a real tickets directory with a markdown file.
        let tickets_dir = temp.path().join(".tickets").join("open");
        std::fs::create_dir_all(&tickets_dir).unwrap();
        let ticket_content = "---\nid: ja-001\ntitle: Old Title\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Old Title\n";
        std::fs::write(tickets_dir.join("ja-001.md"), ticket_content).unwrap();

        let project_id = db
            .register_project(
                "personal",
                "test-repo",
                temp.path().to_str().unwrap(),
                temp.path().join(".tickets").to_str().unwrap(),
                Some("owner"),
                Some("repo"),
            )
            .unwrap();

        // Insert task with existing GitHub link.
        let markdown_path = tickets_dir.join("ja-001.md").to_string_lossy().to_string();
        db.conn
            .execute(
                "INSERT INTO tasks (id, project_id, title, state, priority, issue_type,
                                    markdown_path, markdown_hash, synced_at,
                                    github_issue_number, github_synced_at)
                 VALUES (?1, ?2, ?3, ?4, 2, 'task', ?5, 'hash', '2026-01-01T10:00:00Z',
                         42, '2026-01-01T10:00:00Z')",
                rusqlite::params!["ja-001", project_id, "Old Title", "open", markdown_path],
            )
            .unwrap();

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();
        client.seed_issue(GithubIssue {
            number: 42,
            title: "New Title from GitHub".to_string(),
            body: Some("Updated description".to_string()),
            state: "closed".to_string(),
            updated_at: "2026-01-01T11:00:00Z".to_string(),
            html_url: "https://github.com/owner/repo/issues/42".to_string(),
            labels: vec!["bug".to_string()],
        });

        let report = sync.sync(Some(&client), false).await.unwrap();

        assert_eq!(report.total_pulled, 1);

        // Verify the markdown was updated (file moved to closed/ dir since state is "closed").
        let updated_path = temp.path().join(".tickets").join("closed").join("ja-001.md");
        let content = std::fs::read_to_string(&updated_path).unwrap();
        assert!(content.contains("New Title from GitHub"));
        assert!(content.contains("closed"));
        assert!(content.contains("bug"));
    }

    #[tokio::test]
    async fn test_sync_conflict_remote_newer() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("portfolio.db");
        let db = PortfolioDb::open(&db_path).unwrap();
        db.migrate().unwrap();

        db.create_portfolio("personal", "Personal", None).unwrap();

        let tickets_dir = temp.path().join(".tickets").join("open");
        std::fs::create_dir_all(&tickets_dir).unwrap();
        let ticket_content = "---\nid: ja-001\ntitle: Local Title\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Local Title\n";
        std::fs::write(tickets_dir.join("ja-001.md"), ticket_content).unwrap();

        let project_id = db
            .register_project(
                "personal",
                "test-repo",
                temp.path().to_str().unwrap(),
                temp.path().join(".tickets").to_str().unwrap(),
                Some("owner"),
                Some("repo"),
            )
            .unwrap();

        let markdown_path = tickets_dir.join("ja-001.md").to_string_lossy().to_string();
        // synced_at (local) is 11:00, github_synced_at is 10:00, remote updated_at is 12:00.
        // Both changed, remote is newer.
        db.conn
            .execute(
                "INSERT INTO tasks (id, project_id, title, state, priority, issue_type,
                                    markdown_path, markdown_hash, synced_at,
                                    github_issue_number, github_synced_at)
                 VALUES (?1, ?2, ?3, ?4, 2, 'task', ?5, 'hash', '2026-01-01T11:00:00Z',
                         42, '2026-01-01T10:00:00Z')",
                rusqlite::params!["ja-001", project_id, "Local Title", "open", markdown_path],
            )
            .unwrap();

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();
        client.seed_issue(GithubIssue {
            number: 42,
            title: "Remote Title".to_string(),
            body: Some("Remote body".to_string()),
            state: "closed".to_string(),
            updated_at: "2026-01-01T12:00:00Z".to_string(),
            html_url: "https://github.com/owner/repo/issues/42".to_string(),
            labels: vec![],
        });

        let report = sync.sync(Some(&client), false).await.unwrap();

        assert_eq!(report.total_conflicts, 1);
        assert_eq!(report.total_pulled, 1); // remote is newer, pulled

        // Verify markdown was updated to remote (file moved to closed/ dir).
        let updated_path = temp.path().join(".tickets").join("closed").join("ja-001.md");
        let content = std::fs::read_to_string(&updated_path).unwrap();
        assert!(content.contains("Remote Title"));
    }

    #[tokio::test]
    async fn test_sync_conflict_local_newer() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("portfolio.db");
        let db = PortfolioDb::open(&db_path).unwrap();
        db.migrate().unwrap();

        db.create_portfolio("personal", "Personal", None).unwrap();

        let tickets_dir = temp.path().join(".tickets").join("open");
        std::fs::create_dir_all(&tickets_dir).unwrap();
        let ticket_content = "---\nid: ja-001\ntitle: Local Title\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Local Title\n";
        std::fs::write(tickets_dir.join("ja-001.md"), ticket_content).unwrap();

        let project_id = db
            .register_project(
                "personal",
                "test-repo",
                temp.path().to_str().unwrap(),
                temp.path().join(".tickets").to_str().unwrap(),
                Some("owner"),
                Some("repo"),
            )
            .unwrap();

        let markdown_path = tickets_dir.join("ja-001.md").to_string_lossy().to_string();
        // synced_at (local) is 12:00, github_synced_at is 10:00, remote updated_at is 11:00.
        // Both changed, local is newer.
        db.conn
            .execute(
                "INSERT INTO tasks (id, project_id, title, state, priority, issue_type,
                                    markdown_path, markdown_hash, synced_at,
                                    github_issue_number, github_synced_at)
                 VALUES (?1, ?2, ?3, ?4, 2, 'task', ?5, 'hash', '2026-01-01T12:00:00Z',
                         42, '2026-01-01T10:00:00Z')",
                rusqlite::params!["ja-001", project_id, "Local Title", "open", markdown_path],
            )
            .unwrap();

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();
        client.seed_issue(GithubIssue {
            number: 42,
            title: "Remote Title".to_string(),
            body: Some("Remote body".to_string()),
            state: "closed".to_string(),
            updated_at: "2026-01-01T11:00:00Z".to_string(),
            html_url: "https://github.com/owner/repo/issues/42".to_string(),
            labels: vec![],
        });

        let report = sync.sync(Some(&client), false).await.unwrap();

        assert_eq!(report.total_conflicts, 1);
        assert_eq!(report.total_pulled, 0); // local is newer, not pulled

        // Verify markdown was NOT changed.
        let content = std::fs::read_to_string(&markdown_path).unwrap();
        assert!(content.contains("Local Title"));
        assert!(!content.contains("Remote Title"));
    }

    #[tokio::test]
    async fn test_sync_etag_not_modified() {
        let (_temp, db) = setup_db();
        let project_id = setup_github_project(&db, "test-repo", "owner", "repo", None);

        db.conn
            .execute(
                "INSERT INTO tasks (id, project_id, title, state, priority, issue_type,
                                    markdown_path, markdown_hash, synced_at,
                                    github_issue_number, github_synced_at)
                 VALUES (?1, ?2, ?3, ?4, 2, 'task', ?5, 'hash', '2026-01-01T10:00:00Z',
                         42, '2026-01-01T10:00:00Z')",
                rusqlite::params!["ja-001", project_id, "Title", "open", "/tmp/test.md"],
            )
            .unwrap();

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();

        // First sync — stores an ETag.
        let _report = sync.sync(Some(&client), false).await.unwrap();
        let etag = sync.get_project_etag(project_id).unwrap();
        assert!(etag.is_some());

        // Second sync — should get 304 Not Modified.
        let report2 = sync.sync(Some(&client), false).await.unwrap();
        assert_eq!(report2.total_pulled, 0);
    }

    #[tokio::test]
    async fn test_sync_dry_run_no_api_calls() {
        let (_temp, db) = setup_db();
        let project_id = setup_github_project(&db, "test-repo", "owner", "repo", None);

        insert_task(
            &db,
            project_id,
            "ja-001",
            "Test Task",
            "open",
            "/tmp/test.md",
            "2026-01-01T12:00:00Z",
            &["security"],
        );

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();

        let report = sync.sync(Some(&client), true).await.unwrap();

        assert_eq!(report.total_pushed, 1);
        // No API calls should have been made.
        let calls = client.call_count();
        assert_eq!(calls.create_issue, 0);
        assert_eq!(calls.update_issue, 0);
        assert_eq!(calls.ensure_label, 0);
        assert_eq!(calls.list_issues, 0);
    }

    #[tokio::test]
    async fn test_sync_no_github_projects() {
        let (_temp, db) = setup_db();
        // Register a project without GitHub info.
        db.create_portfolio("personal", "Personal", None).unwrap();
        db.register_project(
            "personal",
            "no-github",
            "/tmp/no-github",
            "/tmp/no-github/.tickets",
            None,
            None,
        )
        .unwrap();

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();

        let report = sync.sync(Some(&client), false).await.unwrap();
        assert!(report.projects.is_empty());
        assert_eq!(report.total_pushed, 0);
    }

    #[tokio::test]
    async fn test_sync_multi_account() {
        let (_temp, db) = setup_db();
        let _id1 = setup_github_project(&db, "repo-a", "levonk", "repo-a", Some("levonk"));
        let _id2 = setup_github_project(&db, "repo-b", "lrepo52", "repo-b", Some("lrepo52"));

        insert_task(&db, _id1, "ja-a01", "Task A", "open", "/tmp/a.md", "2026-01-01T12:00:00Z", &[]);
        insert_task(&db, _id2, "ja-b01", "Task B", "open", "/tmp/b.md", "2026-01-01T12:00:00Z", &[]);

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();

        let report = sync.sync(Some(&client), false).await.unwrap();

        assert_eq!(report.projects.len(), 2);
        assert_eq!(report.total_pushed, 2);

        // Verify both accounts are recorded.
        assert_eq!(report.projects[0].github_account, Some("levonk".to_string()));
        assert_eq!(report.projects[1].github_account, Some("lrepo52".to_string()));
    }

    #[tokio::test]
    async fn test_sync_state_mapping_closed() {
        let (_temp, db) = setup_db();
        let project_id = setup_github_project(&db, "test-repo", "owner", "repo", None);

        insert_task(
            &db,
            project_id,
            "ja-001",
            "Closed Task",
            "closed",
            "/tmp/test.md",
            "2026-01-01T12:00:00Z",
            &[],
        );

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();

        let report = sync.sync(Some(&client), false).await.unwrap();
        assert_eq!(report.total_pushed, 1);

        // The mock should have received state "closed" for the created issue.
        let issues = client.issues.lock().unwrap();
        let issue = issues.values().next().unwrap();
        assert_eq!(issue.state, "closed");
    }

    #[tokio::test]
    async fn test_sync_tag_label_mapping() {
        let (_temp, db) = setup_db();
        let project_id = setup_github_project(&db, "test-repo", "owner", "repo", None);

        insert_task(
            &db,
            project_id,
            "ja-001",
            "Tagged Task",
            "open",
            "/tmp/test.md",
            "2026-01-01T12:00:00Z",
            &["security", "Back End"],
        );

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();

        let _report = sync.sync(Some(&client), false).await.unwrap();

        // Verify labels were created (sanitised).
        let labels = client.labels.lock().unwrap();
        assert!(labels.contains(&"security".to_string()));
        assert!(labels.contains(&"back-end".to_string()));

        // Verify the created issue has the right labels.
        let issues = client.issues.lock().unwrap();
        let issue = issues.values().next().unwrap();
        assert!(issue.labels.contains(&"security".to_string()));
        assert!(issue.labels.contains(&"back-end".to_string()));
    }

    #[tokio::test]
    async fn test_sync_status_empty() {
        let (_temp, db) = setup_db();
        let sync = GitHubSync::new(&db);
        let state = sync.status().unwrap();
        assert!(state.last_github_sync.is_none());
        assert_eq!(state.github_sync_errors, 0);
        assert_eq!(state.total_tasks_linked, 0);
    }

    #[tokio::test]
    async fn test_sync_status_after_sync() {
        let (_temp, db) = setup_db();
        let project_id = setup_github_project(&db, "test-repo", "owner", "repo", None);

        insert_task(
            &db,
            project_id,
            "ja-001",
            "Test Task",
            "open",
            "/tmp/test.md",
            "2026-01-01T12:00:00Z",
            &[],
        );

        let sync = GitHubSync::new(&db);
        let client = MockClient::new();
        let _report = sync.sync(Some(&client), false).await.unwrap();

        let state = sync.status().unwrap();
        assert!(state.last_github_sync.is_some());
        assert_eq!(state.total_tasks_linked, 1);
        assert_eq!(state.github_sync_errors, 0);
    }
}
