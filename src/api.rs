//! Portfolio API endpoints for the warp web server.
//!
//! This module provides a comprehensive REST API for the portfolio layer,
//! exposing CRUD endpoints for all 7 hierarchy levels, a filtered task
//! query endpoint, a priority reordering endpoint, and GitHub sync
//! trigger/status endpoints.

use crate::db::PortfolioDb;
use crate::sync::SyncManager;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use warp::{Filter, Reply};

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

/// Shared database state wrapped in an `Arc<Mutex<>>` for thread-safe
/// access from multiple warp handlers. Uses `std::sync::Mutex` because
/// rusqlite operations are synchronous and the lock is never held across
/// an await point.
pub type DbState = Arc<Mutex<PortfolioDb>>;

/// Warp filter that injects the shared `DbState` into a handler.
pub fn with_db(
    db: DbState,
) -> impl Filter<Extract = (DbState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

// ---------------------------------------------------------------------------
// Helper: JSON error response
// ---------------------------------------------------------------------------

/// Build a JSON error reply with a status code.
fn error_reply(status: warp::http::StatusCode, message: &str) -> warp::reply::Response {
    let body = serde_json::json!({ "error": message });
    warp::reply::with_status(warp::reply::json(&body), status).into_response()
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

/// A task row enriched with project name, app name, story id, and tags.
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub state: String,
    pub project: Option<String>,
    pub app: Option<String>,
    pub story: Option<String>,
    pub priority: i64,
    pub tags: Vec<String>,
    pub markdown_path: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub deps: Vec<String>,
}

/// A tag with its task count.
#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub name: String,
    pub task_count: i64,
}

/// The ordered task list for a priority scope.
#[derive(Debug, Serialize)]
pub struct PriorityResponse {
    pub scope_type: String,
    pub scope_id: String,
    pub ordered_tasks: Vec<String>,
}

/// Sync status returned by `GET /api/sync/status`.
#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub last_sync: Option<String>,
    pub in_progress: bool,
    pub projects_synced: i64,
    pub errors: Vec<String>,
}

/// Sync trigger response returned by `POST /api/sync/github`.
#[derive(Debug, Serialize)]
pub struct SyncTriggerResponse {
    pub status: String,
    pub message: String,
    pub projects: Vec<String>,
}

// ---------------------------------------------------------------------------
// Request structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreatePortfolioRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePortfolioRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub portfolio_id: String,
    pub name: String,
    pub repo_path: String,
    pub tickets_dir: Option<String>,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAppRequest {
    pub project_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAppRequest {
    pub state: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRequirementRequest {
    pub portfolio_id: String,
    pub title: String,
    pub description: Option<String>,
    pub state: Option<String>,
    pub target_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequirementRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub target_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStoryRequest {
    pub requirement_id: Option<String>,
    pub app_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub state: Option<String>,
    pub target_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStoryRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub target_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<i32>,
    pub story: Option<String>,
    pub tags: Option<Vec<String>>,
    pub deps: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAiTaskRequest {
    pub task_id: String,
    pub title: String,
    pub agent_profile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAiTaskRequest {
    pub state: Option<String>,
    pub result_summary: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PriorityReorderRequest {
    pub scope_type: String,
    pub scope_id: String,
    pub task_id: String,
    pub new_position: i64,
}

// ---------------------------------------------------------------------------
// Query parameter structs
// ---------------------------------------------------------------------------

/// Query parameters for `GET /api/tasks`.
#[derive(Debug, Deserialize)]
pub struct TaskQueryParams {
    pub portfolio: Option<String>,
    pub project: Option<String>,
    pub app: Option<String>,
    pub story: Option<String>,
    pub requirement: Option<String>,
    pub tag: Option<String>,
    pub state: Option<String>,
    /// Accepted for API compatibility but not filtered — assignee is stored
    /// in markdown frontmatter, not in the tasks table.
    #[allow(dead_code)]
    pub assignee: Option<String>,
}

/// Query parameters for `GET /api/priority`.
#[derive(Debug, Deserialize)]
pub struct PriorityQueryParams {
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Portfolio endpoints
// ---------------------------------------------------------------------------

pub fn portfolio_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("portfolios"))
        .and(warp::path::end())
        .and(with_db(db.clone()))
        .and_then(get_portfolios);

    let db = db.clone();
    let post = warp::post()
        .and(warp::path("api"))
        .and(warp::path("portfolios"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(create_portfolio);

    let db = db.clone();
    let put = warp::put()
        .and(warp::path("api"))
        .and(warp::path("portfolios"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(update_portfolio);

    let delete = warp::delete()
        .and(warp::path("api"))
        .and(warp::path("portfolios"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(delete_portfolio);

    get.or(post).or(put).or(delete)
}

async fn get_portfolios(db: DbState) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    match db.list_portfolios(true) {
        Ok(portfolios) => Ok(warp::reply::json(&portfolios).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            &e.to_string(),
        )),
    }
}

async fn create_portfolio(
    req: CreatePortfolioRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    match db.create_portfolio(&req.id, &req.name, req.description.as_deref()) {
        Ok(()) => match db.get_portfolio(&req.id) {
            Ok(portfolio) => Ok(warp::reply::with_status(
                warp::reply::json(&portfolio),
                warp::http::StatusCode::CREATED,
            )
            .into_response()),
            Err(e) => Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            )),
        },
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::BAD_REQUEST,
            &e.to_string(),
        )),
    }
}

async fn update_portfolio(
    id: String,
    req: UpdatePortfolioRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    // Check existence first
    if db.get_portfolio(&id).is_err() {
        return Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("Portfolio not found: {}", id),
        ));
    }

    // Build dynamic UPDATE
    let mut sets: Vec<&str> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref name) = req.name {
        sets.push("name = ?");
        params.push(Box::new(name.clone()));
    }
    if let Some(ref desc) = req.description {
        sets.push("description = ?");
        params.push(Box::new(desc.clone()));
    }
    if let Some(ref state) = req.state {
        sets.push("state = ?");
        params.push(Box::new(state.clone()));
    }

    if sets.is_empty() {
        // Nothing to update — return current portfolio
        match db.get_portfolio(&id) {
            Ok(p) => Ok(warp::reply::json(&p).into_response()),
            Err(e) => Ok(error_reply(
                warp::http::StatusCode::NOT_FOUND,
                &e.to_string(),
            )),
        }
    } else {
        let sql = format!("UPDATE portfolios SET {} WHERE id = ?", sets.join(", "));
        params.push(Box::new(id.clone()));
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        match db.conn.execute(&sql, param_refs.as_slice()) {
            Ok(_) => match db.get_portfolio(&id) {
                Ok(p) => Ok(warp::reply::json(&p).into_response()),
                Err(e) => Ok(error_reply(
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    &e.to_string(),
                )),
            },
            Err(e) => Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            )),
        }
    }
}

async fn delete_portfolio(
    id: String,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    match db.dissolve_portfolio(&id) {
        Ok(()) => Ok(warp::reply::json(&serde_json::json!({
            "id": id,
            "state": "dissolved"
        }))
        .into_response()),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                warp::http::StatusCode::NOT_FOUND
            } else {
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            Ok(error_reply(status, &e.to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// Project endpoints
// ---------------------------------------------------------------------------

pub fn project_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("projects"))
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_db(db.clone()))
        .and_then(get_projects);

    let db = db.clone();
    let post = warp::post()
        .and(warp::path("api"))
        .and(warp::path("projects"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(create_project);

    let delete = warp::delete()
        .and(warp::path("api"))
        .and(warp::path("projects"))
        .and(warp::path::param::<i64>())
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(delete_project);

    get.or(post).or(delete)
}

async fn get_projects(
    params: std::collections::HashMap<String, String>,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let portfolio = params.get("portfolio").map(|s| s.as_str());
    match db.list_projects(portfolio) {
        Ok(projects) => Ok(warp::reply::json(&projects).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            &e.to_string(),
        )),
    }
}

async fn create_project(
    req: CreateProjectRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let tickets_dir = req
        .tickets_dir
        .unwrap_or_else(|| format!("{}/.tickets", req.repo_path));
    match db.register_project(
        &req.portfolio_id,
        &req.name,
        &req.repo_path,
        &tickets_dir,
        req.github_owner.as_deref(),
        req.github_repo.as_deref(),
    ) {
        Ok(id) => {
            // Fetch the created project
            let projects = db.list_projects(None).unwrap_or_default();
            let project = projects.into_iter().find(|p| p.id == id);
            match project {
                Some(p) => Ok(warp::reply::with_status(
                    warp::reply::json(&p),
                    warp::http::StatusCode::CREATED,
                )
                .into_response()),
                None => Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({ "id": id })),
                    warp::http::StatusCode::CREATED,
                )
                .into_response()),
            }
        }
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::BAD_REQUEST,
            &e.to_string(),
        )),
    }
}

async fn delete_project(
    id: i64,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    // Check existence first
    let exists: bool = db
        .conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM projects WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| warp::reject::custom(DbLockError(e.to_string())))?;

    if !exists {
        return Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("Project not found: {}", id),
        ));
    }

    // Delete ai_tasks for tasks belonging to this project (FK: ai_tasks.task_id -> tasks.id)
    let _ = db.conn.execute(
        "DELETE FROM ai_tasks WHERE task_id IN (
            SELECT id FROM tasks WHERE project_id = ?1
        )",
        rusqlite::params![id],
    );

    // Delete task_tags for tasks belonging to this project (no ON DELETE CASCADE)
    let _ = db.conn.execute(
        "DELETE FROM task_tags WHERE task_id IN (
            SELECT id FROM tasks WHERE project_id = ?1
        )",
        rusqlite::params![id],
    );

    // Delete priority_order entries for tasks belonging to this project
    let _ = db.conn.execute(
        "DELETE FROM priority_order WHERE task_id IN (
            SELECT id FROM tasks WHERE project_id = ?1
        )",
        rusqlite::params![id],
    );

    // Delete tasks for the project (FK constraint)
    let _ = db.conn.execute(
        "DELETE FROM tasks WHERE project_id = ?1",
        rusqlite::params![id],
    );

    // Null out story app_id references to this project's apps (FK: stories.app_id -> apps.id)
    let _ = db.conn.execute(
        "UPDATE stories SET app_id = NULL WHERE app_id IN (
            SELECT id FROM apps WHERE project_id = ?1
        )",
        rusqlite::params![id],
    );

    // Delete apps for the project (FK constraint)
    let _ = db.conn.execute(
        "DELETE FROM apps WHERE project_id = ?1",
        rusqlite::params![id],
    );

    // Delete the project
    let _ = db
        .conn
        .execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| warp::reject::custom(DbLockError(e.to_string())))?;

    Ok(warp::reply::json(&serde_json::json!({ "id": id, "deleted": true }))
        .into_response())
}

// ---------------------------------------------------------------------------
// App endpoints
// ---------------------------------------------------------------------------

pub fn app_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("apps"))
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_db(db.clone()))
        .and_then(get_apps);

    let db = db.clone();
    let post = warp::post()
        .and(warp::path("api"))
        .and(warp::path("apps"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(create_app);

    let put = warp::put()
        .and(warp::path("api"))
        .and(warp::path("apps"))
        .and(warp::path::param::<i64>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(update_app);

    get.or(post).or(put)
}

async fn get_apps(
    params: std::collections::HashMap<String, String>,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let project_id = params.get("project").and_then(|s| s.parse::<i64>().ok());
    match db.list_apps(project_id) {
        Ok(apps) => Ok(warp::reply::json(&apps).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            &e.to_string(),
        )),
    }
}

async fn create_app(
    req: CreateAppRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let state = req.state.as_deref().unwrap_or("drafted");
    match db.create_app(req.project_id, &req.name, req.description.as_deref(), state) {
        Ok(id) => {
            let apps = db.list_apps(Some(req.project_id)).unwrap_or_default();
            let app = apps.into_iter().find(|a| a.id == id);
            match app {
                Some(a) => Ok(warp::reply::with_status(
                    warp::reply::json(&a),
                    warp::http::StatusCode::CREATED,
                )
                .into_response()),
                None => Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({ "id": id })),
                    warp::http::StatusCode::CREATED,
                )
                .into_response()),
            }
        }
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::BAD_REQUEST,
            &e.to_string(),
        )),
    }
}

async fn update_app(
    id: i64,
    req: UpdateAppRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    // Check existence
    let exists: bool = db
        .conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM apps WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| warp::reject::custom(DbLockError(e.to_string())))?;

    if !exists {
        return Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("App not found: {}", id),
        ));
    }

    let mut sets: Vec<&str> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref state) = req.state {
        sets.push("state = ?");
        params.push(Box::new(state.clone()));
    }
    if let Some(ref desc) = req.description {
        sets.push("description = ?");
        params.push(Box::new(desc.clone()));
    }

    if !sets.is_empty() {
        let sql = format!("UPDATE apps SET {} WHERE id = ?", sets.join(", "));
        params.push(Box::new(id));
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        if let Err(e) = db.conn.execute(&sql, param_refs.as_slice()) {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ));
        }
    }

    // Return updated app
    let apps = db.list_apps(None).unwrap_or_default();
    let app = apps.into_iter().find(|a| a.id == id);
    match app {
        Some(a) => Ok(warp::reply::json(&a).into_response()),
        None => Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("App not found: {}", id),
        )),
    }
}

// ---------------------------------------------------------------------------
// Requirement endpoints
// ---------------------------------------------------------------------------

pub fn requirement_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("requirements"))
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_db(db.clone()))
        .and_then(get_requirements);

    let db = db.clone();
    let post = warp::post()
        .and(warp::path("api"))
        .and(warp::path("requirements"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(create_requirement);

    let db = db.clone();
    let put = warp::put()
        .and(warp::path("api"))
        .and(warp::path("requirements"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(update_requirement);

    let delete = warp::delete()
        .and(warp::path("api"))
        .and(warp::path("requirements"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(delete_requirement);

    get.or(post).or(put).or(delete)
}

async fn get_requirements(
    params: std::collections::HashMap<String, String>,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let portfolio = params.get("portfolio").map(|s| s.as_str());
    match db.list_requirements(portfolio) {
        Ok(requirements) => Ok(warp::reply::json(&requirements).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            &e.to_string(),
        )),
    }
}

async fn create_requirement(
    req: CreateRequirementRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let state = req.state.as_deref().unwrap_or("proposed");
    match db.create_requirement(
        &req.portfolio_id,
        &req.title,
        req.description.as_deref(),
        state,
        req.target_date.as_deref(),
    ) {
        Ok(id) => match db.get_requirement(&id) {
            Ok(r) => Ok(warp::reply::with_status(
                warp::reply::json(&r),
                warp::http::StatusCode::CREATED,
            )
            .into_response()),
            Err(e) => Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            )),
        },
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::BAD_REQUEST,
            &e.to_string(),
        )),
    }
}

async fn update_requirement(
    id: String,
    req: UpdateRequirementRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    if db.get_requirement(&id).is_err() {
        return Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("Requirement not found: {}", id),
        ));
    }

    let mut sets: Vec<&str> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref title) = req.title {
        sets.push("title = ?");
        params.push(Box::new(title.clone()));
    }
    if let Some(ref desc) = req.description {
        sets.push("description = ?");
        params.push(Box::new(desc.clone()));
    }
    if let Some(ref state) = req.state {
        sets.push("state = ?");
        params.push(Box::new(state.clone()));
    }
    if let Some(ref td) = req.target_date {
        sets.push("target_date = ?");
        params.push(Box::new(td.clone()));
    }

    if !sets.is_empty() {
        let sql = format!("UPDATE requirements SET {} WHERE id = ?", sets.join(", "));
        params.push(Box::new(id.clone()));
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        if let Err(e) = db.conn.execute(&sql, param_refs.as_slice()) {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ));
        }
    }

    match db.get_requirement(&id) {
        Ok(r) => Ok(warp::reply::json(&r).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &e.to_string(),
        )),
    }
}

async fn delete_requirement(
    id: String,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    match db.supersede_requirement(&id) {
        Ok((req, _count)) => Ok(warp::reply::json(&req).into_response()),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                warp::http::StatusCode::NOT_FOUND
            } else {
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            Ok(error_reply(status, &e.to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// Story endpoints
// ---------------------------------------------------------------------------

pub fn story_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("stories"))
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_db(db.clone()))
        .and_then(get_stories);

    let db = db.clone();
    let post = warp::post()
        .and(warp::path("api"))
        .and(warp::path("stories"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(create_story);

    let db = db.clone();
    let put = warp::put()
        .and(warp::path("api"))
        .and(warp::path("stories"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(update_story);

    let delete = warp::delete()
        .and(warp::path("api"))
        .and(warp::path("stories"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(delete_story);

    get.or(post).or(put).or(delete)
}

async fn get_stories(
    params: std::collections::HashMap<String, String>,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let requirement_id = params.get("requirement").map(|s| s.as_str());
    let app_id = params.get("app").and_then(|s| s.parse::<i64>().ok());
    let state = params.get("state").map(|s| s.as_str());
    match db.list_stories(requirement_id, app_id, state) {
        Ok(stories) => Ok(warp::reply::json(&stories).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            &e.to_string(),
        )),
    }
}

async fn create_story(
    req: CreateStoryRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let state = req.state.as_deref().unwrap_or("pitched");
    match db.create_story(
        req.requirement_id.as_deref(),
        req.app_id,
        &req.title,
        req.description.as_deref(),
        state,
        req.target_date.as_deref(),
    ) {
        Ok(id) => match db.get_story(&id) {
            Ok(s) => Ok(warp::reply::with_status(
                warp::reply::json(&s),
                warp::http::StatusCode::CREATED,
            )
            .into_response()),
            Err(e) => Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            )),
        },
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::BAD_REQUEST,
            &e.to_string(),
        )),
    }
}

async fn update_story(
    id: String,
    req: UpdateStoryRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    if db.get_story(&id).is_err() {
        return Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("Story not found: {}", id),
        ));
    }

    let mut sets: Vec<&str> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref title) = req.title {
        sets.push("title = ?");
        params.push(Box::new(title.clone()));
    }
    if let Some(ref desc) = req.description {
        sets.push("description = ?");
        params.push(Box::new(desc.clone()));
    }
    if let Some(ref state) = req.state {
        sets.push("state = ?");
        params.push(Box::new(state.clone()));
    }
    if let Some(ref td) = req.target_date {
        sets.push("target_date = ?");
        params.push(Box::new(td.clone()));
    }

    if !sets.is_empty() {
        let sql = format!("UPDATE stories SET {} WHERE id = ?", sets.join(", "));
        params.push(Box::new(id.clone()));
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        if let Err(e) = db.conn.execute(&sql, param_refs.as_slice()) {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ));
        }
    }

    match db.get_story(&id) {
        Ok(s) => Ok(warp::reply::json(&s).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &e.to_string(),
        )),
    }
}

async fn delete_story(
    id: String,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    if db.get_story(&id).is_err() {
        return Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("Story not found: {}", id),
        ));
    }

    // Archive = set state to 'archived'
    if let Err(e) = db.conn.execute(
        "UPDATE stories SET state = 'archived' WHERE id = ?1",
        rusqlite::params![id],
    ) {
        return Ok(error_reply(
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            &e.to_string(),
        ));
    }

    match db.get_story(&id) {
        Ok(s) => Ok(warp::reply::json(&s).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            &e.to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// Task endpoints
// ---------------------------------------------------------------------------

pub fn task_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let get_list = warp::get()
        .and(warp::path("api"))
        .and(warp::path("tasks"))
        .and(warp::path::end())
        .and(warp::query::<TaskQueryParams>())
        .and(with_db(db.clone()))
        .and_then(get_tasks);

    let db = db.clone();
    let get_one = warp::get()
        .and(warp::path("api"))
        .and(warp::path("tasks"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(with_db(db.clone()))
        .and_then(get_task_by_id);

    let put = warp::put()
        .and(warp::path("api"))
        .and(warp::path("tasks"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(update_task);

    get_list.or(get_one).or(put)
}

/// Query a single task's tags from the DB.
fn get_task_tags(conn: &Connection, task_id: &str) -> Vec<String> {
    let mut stmt = match conn.prepare(
        "SELECT t.name FROM tags t
         JOIN task_tags tt ON tt.tag_id = t.id
         WHERE tt.task_id = ?1
         ORDER BY t.name",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map(rusqlite::params![task_id], |row| row.get::<_, String>(0)) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut tags = Vec::new();
    for tag in rows.flatten() {
        tags.push(tag);
    }
    tags
}

/// Read description, assignee, and deps from a task's markdown frontmatter.
/// Returns `(None, None, Vec::new())` if the file cannot be read or parsed.
fn read_task_frontmatter(markdown_path: &str) -> (Option<String>, Option<String>, Vec<String>) {
    use crate::ticket::Ticket;

    let content = match std::fs::read_to_string(markdown_path) {
        Ok(c) => c,
        Err(_) => return (None, None, Vec::new()),
    };

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return (None, None, Vec::new());
    }

    let yaml_content = parts[1].trim();
    let ticket: Ticket = match serde_yaml::from_str(yaml_content) {
        Ok(t) => t,
        Err(_) => return (None, None, Vec::new()),
    };

    (ticket.description, ticket.assignee, ticket.deps)
}

/// Map a rusqlite row into a `TaskResponse`, including project name, app
/// name, story id, and tags.
fn map_task_row(conn: &Connection, row: &rusqlite::Row) -> rusqlite::Result<TaskResponse> {
    let id: String = row.get(0)?;
    let title: String = row.get(1)?;
    let state: String = row.get(2)?;
    let project_id: i64 = row.get(3)?;
    let app_id: Option<i64> = row.get(4)?;
    let story_id: Option<String> = row.get(5)?;
    let priority: i64 = row.get(6)?;
    let markdown_path: String = row.get(7)?;

    // Look up project name
    let project: Option<String> = conn
        .query_row(
            "SELECT name FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |r| r.get(0),
        )
        .ok();

    // Look up app name
    let app: Option<String> = match app_id {
        Some(aid) => conn
            .query_row(
                "SELECT name FROM apps WHERE id = ?1",
                rusqlite::params![aid],
                |r| r.get(0),
            )
            .ok(),
        None => None,
    };

    let tags = get_task_tags(conn, &id);

    // Read description, assignee, and deps from the markdown source-of-truth.
    let (description, assignee, deps) = read_task_frontmatter(&markdown_path);

    Ok(TaskResponse {
        id,
        title,
        state,
        project,
        app,
        story: story_id,
        priority,
        tags,
        markdown_path,
        description,
        assignee,
        deps,
    })
}

async fn get_tasks(
    params: TaskQueryParams,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let conn = &db.conn;

    // Build dynamic WHERE clause with parameterized queries
    let mut conditions: Vec<String> = Vec::new();
    let mut sql_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref portfolio) = params.portfolio {
        conditions.push(
            "t.project_id IN (SELECT id FROM projects WHERE portfolio_id = ?".to_string(),
        );
        sql_params.push(Box::new(portfolio.clone()));
        conditions.last_mut().unwrap().push(')');
    }
    if let Some(ref project) = params.project {
        conditions.push(
            "t.project_id IN (SELECT id FROM projects WHERE name = ?".to_string(),
        );
        sql_params.push(Box::new(project.clone()));
        conditions.last_mut().unwrap().push(')');
    }
    if let Some(ref app) = params.app {
        conditions.push("t.app_id IN (SELECT id FROM apps WHERE name = ?".to_string());
        sql_params.push(Box::new(app.clone()));
        conditions.last_mut().unwrap().push(')');
    }
    if let Some(ref story) = params.story {
        conditions.push("t.story_id = ?".to_string());
        sql_params.push(Box::new(story.clone()));
    }
    if let Some(ref requirement) = params.requirement {
        conditions.push(
            "t.story_id IN (SELECT id FROM stories WHERE requirement_id = ?".to_string(),
        );
        sql_params.push(Box::new(requirement.clone()));
        conditions.last_mut().unwrap().push(')');
    }
    if let Some(ref tag) = params.tag {
        conditions.push(
            "t.id IN (SELECT task_id FROM task_tags WHERE tag_id IN (SELECT id FROM tags WHERE name = ?))"
                .to_string(),
        );
        sql_params.push(Box::new(tag.clone()));
    }
    if let Some(ref state) = params.state {
        conditions.push("t.state = ?".to_string());
        sql_params.push(Box::new(state.clone()));
    }
    // assignee: not stored in tasks table — accepted but not filtered

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT t.id, t.title, t.state, t.project_id, t.app_id, t.story_id, t.priority, t.markdown_path
         FROM tasks t{}",
        where_clause
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };

    let param_refs: Vec<&dyn rusqlite::ToSql> =
        sql_params.iter().map(|p| p.as_ref()).collect();

    let rows = match stmt.query_map(param_refs.as_slice(), |row| map_task_row(conn, row)) {
        Ok(r) => r,
        Err(e) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };

    let mut tasks = Vec::new();
    for row in rows {
        match row {
            Ok(t) => tasks.push(t),
            Err(e) => {
                return Ok(error_reply(
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    &e.to_string(),
                ))
            }
        }
    }

    // Sort tasks by priority_order position for the relevant scope.
    // Determine the scope from the query params, falling back to global.
    let (scope_type, scope_id) = if let Some(ref story) = params.story {
        ("story", story.clone())
    } else if let Some(ref project) = params.project {
        ("project", project.clone())
    } else if let Some(ref portfolio) = params.portfolio {
        ("portfolio", portfolio.clone())
    } else if let Some(ref app) = params.app {
        ("app", app.clone())
    } else if let Some(ref requirement) = params.requirement {
        ("requirement", requirement.clone())
    } else if let Some(ref tag) = params.tag {
        ("tag", tag.clone())
    } else {
        ("global", "global".to_string())
    };

    let pm = crate::priority::PriorityManager::new(conn);
    let ordering = pm.get_ordering(scope_type, &scope_id).unwrap_or_default();

    // Build a position map: task_id -> position index
    let pos_map: std::collections::HashMap<String, usize> = ordering
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), i))
        .collect();

    // Sort: tasks with priority_order entries first (by position), then
    // fall back to priority field (lower number = higher priority), then
    // by id (alphabetical) as a stable tiebreaker.
    tasks.sort_by(|a, b| {
        let pa = pos_map.get(&a.id);
        let pb = pos_map.get(&b.id);
        match (pa, pb) {
            (Some(i), Some(j)) => i.cmp(j),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                // Fall back to priority field, then id
                a.priority
                    .cmp(&b.priority)
                    .then_with(|| a.id.cmp(&b.id))
            }
        }
    });

    Ok(warp::reply::json(&tasks).into_response())
}

async fn get_task_by_id(
    id: String,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let conn = &db.conn;

    let sql =
        "SELECT t.id, t.title, t.state, t.project_id, t.app_id, t.story_id, t.priority, t.markdown_path
         FROM tasks t WHERE t.id = ?1";

    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };

    let result = stmt
        .query_map(rusqlite::params![id], |row| map_task_row(conn, row))
        .map_err(|e| warp::reject::custom(DbLockError(e.to_string())))?;

    let mut tasks = Vec::new();
    for row in result {
        match row {
            Ok(t) => tasks.push(t),
            Err(e) => {
                return Ok(error_reply(
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    &e.to_string(),
                ))
            }
        }
    }

    match tasks.into_iter().next() {
        Some(t) => Ok(warp::reply::json(&t).into_response()),
        None => Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("Task not found: {}", id),
        )),
    }
}

async fn update_task(
    id: String,
    req: UpdateTaskRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let conn = &db.conn;

    // Check task exists and get markdown_path
    let markdown_path: Option<String> = conn
        .query_row(
            "SELECT markdown_path FROM tasks WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok();

    let markdown_path = match markdown_path {
        Some(p) => p,
        None => {
            return Ok(error_reply(
                warp::http::StatusCode::NOT_FOUND,
                &format!("Task not found: {}", id),
            ))
        }
    };

    // Validate the status field against the valid task states.
    const VALID_TASK_STATES: &[&str] =
        &["logged", "open", "in_progress", "blocked", "ready", "closed"];
    if let Some(ref status) = req.status {
        if !VALID_TASK_STATES.contains(&status.as_str()) {
            return Ok(error_reply(
                warp::http::StatusCode::BAD_REQUEST,
                &format!(
                    "Invalid task state: '{}'. Valid states: {}",
                    status,
                    VALID_TASK_STATES.join(", "),
                ),
            ));
        }
    }

    // Update DB columns for the fields we can directly set
    let mut sets: Vec<&str> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref title) = req.title {
        sets.push("title = ?");
        params.push(Box::new(title.clone()));
    }
    if let Some(ref status) = req.status {
        sets.push("state = ?");
        params.push(Box::new(status.clone()));
    }
    if let Some(ref priority) = req.priority {
        sets.push("priority = ?");
        params.push(Box::new(*priority));
    }
    if let Some(ref story) = req.story {
        sets.push("story_id = ?");
        params.push(Box::new(story.clone()));
    }

    if !sets.is_empty() {
        let sql = format!("UPDATE tasks SET {} WHERE id = ?", sets.join(", "));
        params.push(Box::new(id.clone()));
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        if let Err(e) = conn.execute(&sql, param_refs.as_slice()) {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ));
        }
    }

    // Sync tags if provided
    if let Some(ref tags) = req.tags {
        if let Err(e) = db.sync_task_tags(&id, tags) {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ));
        }
    }

    // Write to markdown file (source of truth) if title, description,
    // status, assignee, priority, story, tags, or deps changed.
    let should_write_markdown = req.title.is_some()
        || req.description.is_some()
        || req.status.is_some()
        || req.assignee.is_some()
        || req.priority.is_some()
        || req.story.is_some()
        || req.tags.is_some()
        || req.deps.is_some();

    if should_write_markdown {
        if let Err(e) = write_task_markdown(
            &markdown_path,
            &id,
            &req,
        ) {
            // Markdown is the source of truth — a write failure means the
            // DB and markdown are now inconsistent.  Return a 500 so the
            // caller knows the update did not fully succeed.
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to write markdown for task {}: {}", id, e),
            ));
        }
    }

    // Return updated task
    let sql = "SELECT t.id, t.title, t.state, t.project_id, t.app_id, t.story_id, t.priority, t.markdown_path
         FROM tasks t WHERE t.id = ?1";
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };
    let result = stmt
        .query_map(rusqlite::params![id], |row| map_task_row(conn, row))
        .map_err(|e| warp::reject::custom(DbLockError(e.to_string())))?;

    let mut tasks = Vec::new();
    for t in result.flatten() {
        tasks.push(t);
    }

    match tasks.into_iter().next() {
        Some(t) => Ok(warp::reply::json(&t).into_response()),
        None => Ok(error_reply(
            warp::http::StatusCode::NOT_FOUND,
            &format!("Task not found: {}", id),
        )),
    }
}

/// Write updated fields to the task's markdown file (source of truth).
/// Reads the existing file, parses the YAML frontmatter, applies updates,
/// and writes back.
fn write_task_markdown(
    path: &str,
    task_id: &str,
    req: &UpdateTaskRequest,
) -> anyhow::Result<()> {
    use crate::ticket::Ticket;

    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read markdown: {}", e))?;

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid ticket format for task {}", task_id);
    }

    let yaml_content = parts[1].trim();
    let mut ticket: Ticket = serde_yaml::from_str(yaml_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse YAML: {}", e))?;

    // Apply updates
    if let Some(ref title) = req.title {
        ticket.title = title.clone();
    }
    if let Some(ref desc) = req.description {
        ticket.description = Some(desc.clone());
    }
    if let Some(ref status) = req.status {
        ticket.status = status.clone();
    }
    if let Some(ref assignee) = req.assignee {
        ticket.assignee = Some(assignee.clone());
    }
    if let Some(ref priority) = req.priority {
        ticket.priority = *priority;
    }
    if let Some(ref story) = req.story {
        ticket.story = Some(story.clone());
    }
    if let Some(ref tags) = req.tags {
        ticket.tags = tags.clone();
    }
    if let Some(ref deps) = req.deps {
        ticket.deps = deps.clone();
    }

    // Serialize back to markdown
    let yaml_content = serde_yaml::to_string(&ticket)?;
    let mut new_content = format!("---\n{}\n---\n\n# {}\n", yaml_content.trim(), ticket.title);

    if let Some(desc) = &ticket.description {
        new_content.push_str(&format!("\n\n{}", desc));
    }

    std::fs::write(path, new_content)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// AI Task endpoints
// ---------------------------------------------------------------------------

pub fn ai_task_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("ai-tasks"))
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_db(db.clone()))
        .and_then(get_ai_tasks);

    let db = db.clone();
    let post = warp::post()
        .and(warp::path("api"))
        .and(warp::path("ai-tasks"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(create_ai_task);

    let put = warp::put()
        .and(warp::path("api"))
        .and(warp::path("ai-tasks"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(update_ai_task);

    get.or(post).or(put)
}

async fn get_ai_tasks(
    params: std::collections::HashMap<String, String>,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let task_id = params.get("task").map(|s| s.as_str());
    match db.list_ai_tasks(task_id, None) {
        Ok(ai_tasks) => Ok(warp::reply::json(&ai_tasks).into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            &e.to_string(),
        )),
    }
}

async fn create_ai_task(
    req: CreateAiTaskRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    match db.create_ai_task(&req.task_id, &req.title, req.agent_profile.as_deref()) {
        Ok(id) => match db.get_ai_task(&id) {
            Ok(t) => Ok(warp::reply::with_status(
                warp::reply::json(&t),
                warp::http::StatusCode::CREATED,
            )
            .into_response()),
            Err(e) => Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            )),
        },
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::BAD_REQUEST,
            &e.to_string(),
        )),
    }
}

async fn update_ai_task(
    id: String,
    req: UpdateAiTaskRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    let state = match req.state {
        Some(s) => s,
        None => {
            return Ok(error_reply(
                warp::http::StatusCode::BAD_REQUEST,
                "state is required",
            ))
        }
    };

    match db.update_ai_task_state(&id, &state, req.result_summary.as_deref()) {
        Ok(t) => Ok(warp::reply::json(&t).into_response()),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                warp::http::StatusCode::NOT_FOUND
            } else {
                warp::http::StatusCode::BAD_REQUEST
            };
            Ok(error_reply(status, &e.to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// Tag endpoints
// ---------------------------------------------------------------------------

pub fn tag_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("tags"))
        .and(warp::path::end())
        .and(with_db(db.clone()))
        .and_then(get_tags);

    let post = warp::post()
        .and(warp::path("api"))
        .and(warp::path("tags"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db))
        .and_then(create_tag);

    get.or(post)
}

async fn get_tags(db: DbState) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    let conn = &db.conn;

    let mut stmt = match conn.prepare(
        "SELECT t.name, COUNT(tt.task_id)
         FROM tags t
         LEFT JOIN task_tags tt ON tt.tag_id = t.id
         GROUP BY t.name
         ORDER BY t.name",
    ) {
        Ok(s) => s,
        Err(e) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };

    let rows = match stmt.query_map([], |row| {
        Ok(TagResponse {
            name: row.get(0)?,
            task_count: row.get(1)?,
        })
    }) {
        Ok(r) => r,
        Err(e) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };

    let mut tags = Vec::new();
    for t in rows.flatten() {
        tags.push(t);
    }

    Ok(warp::reply::json(&tags).into_response())
}

async fn create_tag(
    req: CreateTagRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;
    match db.upsert_tag(&req.name) {
        Ok(_id) => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "name": req.name })),
            warp::http::StatusCode::CREATED,
        )
        .into_response()),
        Err(e) => Ok(error_reply(
            warp::http::StatusCode::BAD_REQUEST,
            &e.to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// Priority endpoints
// ---------------------------------------------------------------------------

pub fn priority_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let put = warp::put()
        .and(warp::path("api"))
        .and(warp::path("priority"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(reorder_priority);

    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("priority"))
        .and(warp::path::end())
        .and(warp::query::<PriorityQueryParams>())
        .and(with_db(db))
        .and_then(get_priority);

    put.or(get)
}

async fn reorder_priority(
    req: PriorityReorderRequest,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    let pm = crate::priority::PriorityManager::new(&db.conn);
    match pm.set_position(&req.scope_type, &req.scope_id, &req.task_id, req.new_position) {
        Ok(ordered) => Ok(warp::reply::json(&PriorityResponse {
            scope_type: req.scope_type,
            scope_id: req.scope_id,
            ordered_tasks: ordered,
        })
        .into_response()),
        Err(e) => {
            let status = if e.to_string().contains("Invalid scope type") {
                warp::http::StatusCode::BAD_REQUEST
            } else {
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            Ok(error_reply(status, &e.to_string()))
        }
    }
}

async fn get_priority(
    params: PriorityQueryParams,
    db: DbState,
) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    let scope_type = match params.scope_type {
        Some(s) => s,
        None => {
            return Ok(error_reply(
                warp::http::StatusCode::BAD_REQUEST,
                "scope_type is required",
            ))
        }
    };
    let scope_id = match params.scope_id {
        Some(s) => s,
        None => {
            return Ok(error_reply(
                warp::http::StatusCode::BAD_REQUEST,
                "scope_id is required",
            ))
        }
    };

    let pm = crate::priority::PriorityManager::new(&db.conn);
    match pm.get_ordering(&scope_type, &scope_id) {
        Ok(ordered) => Ok(warp::reply::json(&PriorityResponse {
            scope_type,
            scope_id,
            ordered_tasks: ordered,
        })
        .into_response()),
        Err(e) => {
            let status = if e.to_string().contains("Invalid scope type") {
                warp::http::StatusCode::BAD_REQUEST
            } else {
                warp::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            Ok(error_reply(status, &e.to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// Sync endpoints
// ---------------------------------------------------------------------------

pub fn sync_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let db = db.clone();
    let post = warp::post()
        .and(warp::path("api"))
        .and(warp::path("sync"))
        .and(warp::path("github"))
        .and(warp::path::end())
        .and(with_db(db.clone()))
        .and_then(trigger_sync);

    let get = warp::get()
        .and(warp::path("api"))
        .and(warp::path("sync"))
        .and(warp::path("status"))
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(get_sync_status);

    post.or(get)
}

async fn trigger_sync(db: DbState) -> Result<warp::reply::Response, warp::Rejection> {
    // First, check if there are any projects configured for GitHub sync.
    // This is a synchronous DB read — we lock, read, and drop the guard
    // before any async work.
    let (project_names, has_projects) = {
        let db = db.lock().map_err(|e| {
            warp::reject::custom(DbLockError(e.to_string()))
        })?;
        let gh_sync = crate::github_sync::GitHubSync::new(&db);
        let projects = match gh_sync.list_github_projects() {
            Ok(p) => p,
            Err(e) => {
                return Ok(error_reply(
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    &e.to_string(),
                ))
            }
        };
        let names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();
        (names, !projects.is_empty())
    }; // MutexGuard dropped here

    if !has_projects {
        return Ok(error_reply(
            warp::http::StatusCode::BAD_REQUEST,
            "No projects configured for GitHub sync (missing github_owner/github_repo)",
        ));
    }

    // Create a real GitHub API client. If the gh CLI is not available,
    // fall back to a dry run so the endpoint still reports something
    // useful instead of silently succeeding.
    let client = crate::github_sync::ReqwestClient::new(None);
    let dry_run = client.is_err();

    // Run the GitHub sync inside `spawn_blocking` because `PortfolioDb`
    // (and thus `MutexGuard`) is not `Send` and cannot be held across an
    // `.await` point.  We re-acquire the lock inside the blocking task and
    // drive the async sync with `Handle::block_on`.
    let db_for_blocking = db.clone();
    let sync_result = tokio::task::spawn_blocking(move || -> anyhow::Result<crate::github_sync::GitHubSyncReport> {
        let db = db_for_blocking
            .lock()
            .map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?;
        let gh_sync = crate::github_sync::GitHubSync::new(&db);
        let handle = tokio::runtime::Handle::current();
        let client_ref: Option<&dyn crate::github_sync::GitHubClient> = match &client {
            Ok(c) => Some(c),
            Err(_) => None,
        };
        handle.block_on(gh_sync.sync(client_ref, dry_run))
    })
    .await;

    let report = match sync_result {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
        Err(e) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Sync task failed: {}", e),
            ))
        }
    };

    let status = if dry_run { "dry_run" } else { "completed" };
    let message = if dry_run {
        format!(
            "gh CLI not available — dry run only. \
             {} project(s), {} error(s)",
            report.projects.len(),
            report.total_errors,
        )
    } else {
        format!(
            "GitHub sync complete: {} pushed, {} pulled, {} conflicts, {} errors",
            report.total_pushed,
            report.total_pulled,
            report.total_conflicts,
            report.total_errors,
        )
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&SyncTriggerResponse {
            status: status.to_string(),
            message,
            projects: project_names,
        }),
        warp::http::StatusCode::OK,
    )
    .into_response())
}

async fn get_sync_status(db: DbState) -> Result<warp::reply::Response, warp::Rejection> {
    let db = db.lock().map_err(|e| {
        warp::reject::custom(DbLockError(e.to_string()))
    })?;

    let sync = SyncManager::new(&db);
    let state = match sync.sync_status() {
        Ok(s) => s,
        Err(e) => {
            return Ok(error_reply(
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            ))
        }
    };

    Ok(warp::reply::json(&SyncStatusResponse {
        last_sync: state.last_sync_at,
        in_progress: false,
        projects_synced: state.project_count,
        errors: Vec::new(),
    })
    .into_response())
}

// ---------------------------------------------------------------------------
// Combined API routes
// ---------------------------------------------------------------------------

/// Build all portfolio API routes combined into a single filter.
pub fn all_api_routes(
    db: DbState,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    portfolio_routes(db.clone())
        .or(project_routes(db.clone()))
        .or(app_routes(db.clone()))
        .or(requirement_routes(db.clone()))
        .or(story_routes(db.clone()))
        .or(task_routes(db.clone()))
        .or(ai_task_routes(db.clone()))
        .or(tag_routes(db.clone()))
        .or(priority_routes(db.clone()))
        .or(sync_routes(db))
}

// ---------------------------------------------------------------------------
// Rejection handler for DB lock errors
// ---------------------------------------------------------------------------

/// Custom rejection for database lock errors.
#[derive(Debug)]
#[allow(dead_code)]
pub struct DbLockError(pub String);

impl warp::reject::Reject for DbLockError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::PortfolioDb;
    use tempfile::TempDir;

    /// Set up a test database with the schema migrated.
    fn setup_db() -> (TempDir, DbState) {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();
        let state = Arc::new(Mutex::new(db));
        (temp, state)
    }

    /// Seed a full hierarchy: portfolio → project → app → requirement →
    /// story → task, plus a tag.
    fn seed_hierarchy(conn: &Connection) {
        conn.execute(
            "INSERT INTO portfolios (id, name, created) VALUES ('p', 'Personal', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO projects (id, portfolio_id, name, repo_path, tickets_dir, registered_at)
             VALUES (1, 'p', 'tkr', '/r', '/r/.tickets', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO apps (id, project_id, name, state, created)
             VALUES (1, 1, 'default', 'drafted', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO requirements (id, portfolio_id, title, created)
             VALUES ('req-1', 'p', 'Sync', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO stories (id, requirement_id, app_id, title, created)
             VALUES ('story-1', 'req-1', 1, 'Portfolio', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, project_id, app_id, story_id, title, state, priority, markdown_path, synced_at)
             VALUES ('ja-1', 1, 1, 'story-1', 'Task One', 'open', 2, '/m1', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, project_id, title, state, priority, markdown_path, synced_at)
             VALUES ('ja-2', 1, 'Task Two', 'in_progress', 3, '/m2', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO tags (id, name) VALUES (1, 'security')", []).unwrap();
        conn.execute(
            "INSERT INTO task_tags (task_id, tag_id) VALUES ('ja-1', 1)",
            [],
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_portfolios() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/portfolios")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.as_array().unwrap().len() >= 1);
    }

    #[tokio::test]
    async fn test_create_portfolio() {
        let (_temp, db) = setup_db();
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("POST")
            .path("/api/portfolios")
            .json(&serde_json::json!({
                "id": "work",
                "name": "Work",
                "description": "Work projects"
            }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 201);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["id"], "work");
        assert_eq!(body["name"], "Work");
        assert_eq!(body["state"], "curated");
    }

    #[tokio::test]
    async fn test_update_portfolio() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("PUT")
            .path("/api/portfolios/p")
            .json(&serde_json::json!({ "name": "Personal Updated" }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["name"], "Personal Updated");
    }

    #[tokio::test]
    async fn test_delete_portfolio() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("DELETE")
            .path("/api/portfolios/p")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["state"], "dissolved");
    }

    #[tokio::test]
    async fn test_get_tasks_filtered() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/tasks?project=tkr&tag=security")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let tasks = body.as_array().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0]["id"], "ja-1");
    }

    #[tokio::test]
    async fn test_get_tasks_compose_filters() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/tasks?portfolio=p&project=tkr&state=open")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let tasks = body.as_array().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0]["id"], "ja-1");
    }

    #[tokio::test]
    async fn test_get_task_by_id() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/tasks/ja-1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["id"], "ja-1");
        assert_eq!(body["title"], "Task One");
        assert_eq!(body["project"], "tkr");
        assert_eq!(body["app"], "default");
        assert_eq!(body["story"], "story-1");
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let (_temp, db) = setup_db();
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/tasks/nonexistent")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 404);
    }

    #[tokio::test]
    async fn test_get_tags() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/tags")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let tags = body.as_array().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0]["name"], "security");
        assert_eq!(tags[0]["task_count"], 1);
    }

    #[tokio::test]
    async fn test_put_priority() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
            // Insert initial priority entries
            d.conn.execute(
                "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                 VALUES ('story', 'story-1', 'ja-1', 0)",
                [],
            )
            .unwrap();
            d.conn.execute(
                "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                 VALUES ('story', 'story-1', 'ja-2', 1)",
                [],
            )
            .unwrap();
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("PUT")
            .path("/api/priority")
            .json(&serde_json::json!({
                "scope_type": "story",
                "scope_id": "story-1",
                "task_id": "ja-2",
                "new_position": 0
            }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let ordered = body["ordered_tasks"].as_array().unwrap();
        assert_eq!(ordered[0], "ja-2");
        assert_eq!(ordered[1], "ja-1");
    }

    #[tokio::test]
    async fn test_get_priority() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
            d.conn.execute(
                "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                 VALUES ('story', 'story-1', 'ja-1', 0)",
                [],
            )
            .unwrap();
            d.conn.execute(
                "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                 VALUES ('story', 'story-1', 'ja-2', 1)",
                [],
            )
            .unwrap();
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/priority?scope_type=story&scope_id=story-1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let ordered = body["ordered_tasks"].as_array().unwrap();
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0], "ja-1");
        assert_eq!(ordered[1], "ja-2");
    }

    #[tokio::test]
    async fn test_api_tasks_sorted_by_priority() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
            // Set story_id on ja-2 so both tasks are in story-1 scope
            d.conn.execute(
                "UPDATE tasks SET story_id = 'story-1' WHERE id = 'ja-2'",
                [],
            )
            .unwrap();
            // Set explicit ordering: ja-2 at position 0, ja-1 at position 1
            d.conn.execute(
                "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                 VALUES ('story', 'story-1', 'ja-2', 0)",
                [],
            )
            .unwrap();
            d.conn.execute(
                "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                 VALUES ('story', 'story-1', 'ja-1', 1)",
                [],
            )
            .unwrap();
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/tasks?story=story-1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let tasks = body.as_array().unwrap();
        // Both ja-1 and ja-2 have story_id = story-1
        assert_eq!(tasks.len(), 2);
        // ja-2 should be first (position 0), ja-1 second (position 1)
        assert_eq!(tasks[0]["id"], "ja-2");
        assert_eq!(tasks[1]["id"], "ja-1");
    }

    #[tokio::test]
    async fn test_api_tasks_fallback_sort() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
            // Set story_id on ja-2 so both tasks are in story-1 scope
            d.conn.execute(
                "UPDATE tasks SET story_id = 'story-1' WHERE id = 'ja-2'",
                [],
            )
            .unwrap();
            // No priority_order entries — should auto-assign and fall back
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/tasks?story=story-1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let tasks = body.as_array().unwrap();
        assert_eq!(tasks.len(), 2);
        // After auto-assignment, tasks are ordered by insertion (ja-1 first, ja-2 second)
        // since auto-assign appends in scope task order
        assert_eq!(tasks[0]["id"], "ja-1");
        assert_eq!(tasks[1]["id"], "ja-2");
    }

    #[tokio::test]
    async fn test_sync_github() {
        let (_temp, db) = setup_db();
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("POST")
            .path("/api/sync/github")
            .reply(&routes)
            .await;
        // No projects are configured for GitHub sync, so the endpoint
        // returns 400 Bad Request.
        assert_eq!(resp.status(), 400);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert!(body["error"].as_str().unwrap().contains("No projects configured"));
    }

    #[tokio::test]
    async fn test_sync_status() {
        let (_temp, db) = setup_db();
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/sync/status")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert!(body["in_progress"].is_boolean());
        assert!(body["errors"].is_array());
    }

    #[tokio::test]
    async fn test_create_project() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            d.conn.execute(
                "INSERT INTO portfolios (id, name, created) VALUES ('p', 'P', '2026-01-01')",
                [],
            )
            .unwrap();
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("POST")
            .path("/api/projects")
            .json(&serde_json::json!({
                "portfolio_id": "p",
                "name": "test-repo",
                "repo_path": "/tmp/test-repo",
                "tickets_dir": "/tmp/test-repo/.tickets"
            }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 201);
    }

    #[tokio::test]
    async fn test_delete_project() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("DELETE")
            .path("/api/projects/1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_create_app() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("POST")
            .path("/api/apps")
            .json(&serde_json::json!({
                "project_id": 1,
                "name": "api",
                "state": "drafted"
            }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 201);
    }

    #[tokio::test]
    async fn test_update_app() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("PUT")
            .path("/api/apps/1")
            .json(&serde_json::json!({ "state": "deployed" }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["state"], "deployed");
    }

    #[tokio::test]
    async fn test_create_requirement() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("POST")
            .path("/api/requirements")
            .json(&serde_json::json!({
                "portfolio_id": "p",
                "title": "New Requirement"
            }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 201);
    }

    #[tokio::test]
    async fn test_delete_requirement() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("DELETE")
            .path("/api/requirements/req-1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["state"], "superseded");
    }

    #[tokio::test]
    async fn test_create_story() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("POST")
            .path("/api/stories")
            .json(&serde_json::json!({
                "requirement_id": "req-1",
                "title": "New Story"
            }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 201);
    }

    #[tokio::test]
    async fn test_delete_story() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("DELETE")
            .path("/api/stories/story-1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["state"], "archived");
    }

    #[tokio::test]
    async fn test_create_ai_task() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("POST")
            .path("/api/ai-tasks")
            .json(&serde_json::json!({
                "task_id": "ja-1",
                "title": "AI Subtask"
            }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 201);
    }

    #[tokio::test]
    async fn test_create_tag() {
        let (_temp, db) = setup_db();
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("POST")
            .path("/api/tags")
            .json(&serde_json::json!({ "name": "backend" }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 201);
    }

    #[tokio::test]
    async fn test_get_projects_filtered() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/projects?portfolio=p")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let projects = body.as_array().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0]["name"], "tkr");
    }

    #[tokio::test]
    async fn test_get_stories_filtered() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/stories?requirement=req-1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let stories = body.as_array().unwrap();
        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0]["id"], "story-1");
    }

    #[tokio::test]
    async fn test_get_ai_tasks_filtered() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
            d.conn.execute(
                "INSERT INTO ai_tasks (id, task_id, title, state, created)
                 VALUES ('ai-1', 'ja-1', 'AI Task', 'identified', '2026-01-01')",
                [],
            )
            .unwrap();
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("GET")
            .path("/api/ai-tasks?task=ja-1")
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let tasks = body.as_array().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0]["id"], "ai-1");
    }

    #[tokio::test]
    async fn test_update_ai_task() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
            d.conn.execute(
                "INSERT INTO ai_tasks (id, task_id, title, state, created)
                 VALUES ('ai-1', 'ja-1', 'AI Task', 'identified', '2026-01-01')",
                [],
            )
            .unwrap();
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("PUT")
            .path("/api/ai-tasks/ai-1")
            .json(&serde_json::json!({ "state": "dispatched" }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["state"], "dispatched");
    }

    #[tokio::test]
    async fn test_update_requirement() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("PUT")
            .path("/api/requirements/req-1")
            .json(&serde_json::json!({ "title": "Updated Req" }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["title"], "Updated Req");
    }

    #[tokio::test]
    async fn test_update_story() {
        let (_temp, db) = setup_db();
        {
            let d = db.lock().unwrap();
            seed_hierarchy(&d.conn);
        }
        let routes = all_api_routes(db);
        let resp = warp::test::request()
            .method("PUT")
            .path("/api/stories/story-1")
            .json(&serde_json::json!({ "title": "Updated Story" }))
            .reply(&routes)
            .await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["title"], "Updated Story");
    }
}
