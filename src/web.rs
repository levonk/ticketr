use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use warp::{Filter, Reply};
use crate::api;
use crate::db::PortfolioDb;
use crate::ticket::{TicketManager, Ticket};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub default_assignee: Option<String>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            default_assignee: None,
        }
    }
}

pub async fn start_web_server(
    manager: &mut TicketManager,
    cli_host: String,
    cli_port: u16,
) -> Result<()> {
    // Load config from XDG_CONFIG_HOME or HOME
    let _config = load_config().unwrap_or_default();

    // CLI args override config file
    let host = cli_host;
    let port = cli_port;

    println!("Starting web server on http://{}:{}", host, port);

    // Create shared state
    let tickets = Arc::new(RwLock::new(manager.list_tickets()?));
    let manager = Arc::new(RwLock::new(manager.clone()));

    // Open the portfolio SQLite database for the API endpoints
    let db_path = PortfolioDb::db_path()?;
    let portfolio_db = PortfolioDb::open(&db_path)?;
    portfolio_db.migrate()?;
    let db_state: api::DbState = Arc::new(Mutex::new(portfolio_db));

    // CORS headers
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // API routes — existing ticket endpoints (backward compatibility)
    let api_tickets = warp::path("api")
        .and(warp::path("tickets"))
        .and(warp::get())
        .and(with_tickets(tickets.clone()))
        .and_then(get_tickets);

    let api_ticket_update = warp::path("api")
        .and(warp::path("tickets"))
        .and(warp::path::param::<String>())
        .and(warp::put())
        .and(warp::body::json())
        .and(with_manager(manager.clone()))
        .and_then(update_ticket);

    // Portfolio API routes
    let portfolio_api = api::all_api_routes(db_state);

    // Serve static files
    let static_files = warp::get()
        .and(warp::fs::dir("web"))
        .or(warp::get().and(warp::path("index.html")).and(warp::fs::file("web/index.html")));

    let routes = static_files
        .or(portfolio_api)
        .or(api_tickets)
        .or(api_ticket_update)
        .with(cors)
        .with(warp::log("web"));

    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse()?;
    warp::serve(routes).run(addr).await;

    Ok(())
}

fn load_config() -> Option<WebConfig> {
    // First try git root config as override
    if let Some(git_root_config) = load_git_root_config() {
        return Some(git_root_config);
    }

    // Fallback to XDG_CONFIG_HOME or ~/.config
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{}/.config", h)))?;

    let config_path = PathBuf::from(config_dir).join("tkr").join("config.yml");

    if config_path.exists() {
        let content = std::fs::read_to_string(config_path).ok()?;
        serde_yaml::from_str(&content).ok()
    } else {
        None
    }
}

fn load_git_root_config() -> Option<WebConfig> {
    // Try to find git root and check for .config/tkr/config.yml
    let mut current = std::env::current_dir().ok()?;

    loop {
        let config_path = current.join(".config").join("tkr").join("config.yml");

        if config_path.exists() {
            let content = std::fs::read_to_string(config_path).ok()?;
            if let Ok(config) = serde_yaml::from_str(&content) {
                return Some(config);
            }
        }

        if current.join(".git").exists() {
            // Found git root, break the loop
            break;
        }

        if !current.pop() {
            // Reached filesystem root
            break;
        }
    }

    None
}

fn with_tickets(
    tickets: Arc<RwLock<Vec<Ticket>>>,
) -> impl Filter<Extract = (Arc<RwLock<Vec<Ticket>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tickets.clone())
}

fn with_manager(
    manager: Arc<RwLock<TicketManager>>,
) -> impl Filter<Extract = (Arc<RwLock<TicketManager>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || manager.clone())
}

async fn get_tickets(tickets: Arc<RwLock<Vec<Ticket>>>) -> Result<impl Reply, warp::Rejection> {
    let tickets = tickets.read().await;
    let response: Vec<TicketApiResponse> = tickets.iter().map(|t| TicketApiResponse::from(t.clone())).collect();
    Ok(warp::reply::json(&response))
}

async fn update_ticket(
    id: String,
    update: TicketUpdate,
    manager: Arc<RwLock<TicketManager>>,
) -> Result<impl Reply, warp::Rejection> {
    let manager = manager.write().await;

    // Load the ticket
    let mut ticket = match manager.load_ticket(&id) {
        Ok(t) => t,
        Err(_) => return Ok(warp::reply::with_status("", warp::http::StatusCode::NOT_FOUND)),
    };

    // Apply updates
    if let Some(status) = update.status {
        ticket.status = status;
    }
    if let Some(title) = update.title {
        ticket.title = title;
    }
    if let Some(description) = update.description {
        ticket.description = Some(description);
    }
    if let Some(assignee) = update.assignee {
        ticket.assignee = Some(assignee);
    }
    if let Some(priority) = update.priority {
        ticket.priority = priority;
    }

    // Save the ticket
    if let Err(e) = manager.save_ticket(&ticket) {
        eprintln!("Failed to save ticket: {}", e);
        return Ok(warp::reply::with_status("", warp::http::StatusCode::INTERNAL_SERVER_ERROR));
    }

    Ok(warp::reply::with_status("", warp::http::StatusCode::OK))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketApiResponse {
    pub id: String,
    pub title: String,
    pub status: String,
    pub project: Option<String>,
    pub category: Option<String>,
    pub assignee: Option<String>,
    pub priority: i32,
    pub issue_type: String,
    pub description: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub deps: Vec<String>,
    pub links: Vec<String>,
}

impl From<Ticket> for TicketApiResponse {
    fn from(ticket: Ticket) -> Self {
        Self {
            id: ticket.id,
            title: ticket.title,
            status: ticket.status,
            project: ticket.project,
            category: ticket.category,
            assignee: ticket.assignee,
            priority: ticket.priority,
            issue_type: ticket.issue_type,
            description: ticket.description,
            created: ticket.created,
            deps: ticket.deps,
            links: ticket.links,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TicketUpdate {
    pub status: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<i32>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    /// Resolve the `web/` directory relative to the crate root
    /// (CARGO_MANIFEST_DIR), independent of the test working directory.
    fn web_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web")
    }

    /// Read a file from the `web/` directory, panicking with a clear message
    /// if it does not exist.
    fn read_web_file(rel: &str) -> String {
        let path = web_dir().join(rel);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Expected web file not found: {}", path.display()))
    }

    // -----------------------------------------------------------------------
    // File existence tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_index_html_exists() {
        let content = read_web_file("index.html");
        assert!(!content.is_empty(), "index.html should not be empty");
    }

    #[test]
    fn test_css_file_exists() {
        let content = read_web_file("css/kanban.css");
        assert!(!content.is_empty(), "kanban.css should not be empty");
    }

    #[test]
    fn test_kanban_js_exists() {
        let content = read_web_file("js/kanban.js");
        assert!(!content.is_empty(), "kanban.js should not be empty");
    }

    #[test]
    fn test_filters_js_exists() {
        let content = read_web_file("js/filters.js");
        assert!(!content.is_empty(), "filters.js should not be empty");
    }

    #[test]
    fn test_drag_drop_js_exists() {
        let content = read_web_file("js/drag-drop.js");
        assert!(!content.is_empty(), "drag-drop.js should not be empty");
    }

    #[test]
    fn test_detail_panel_js_exists() {
        let content = read_web_file("js/detail-panel.js");
        assert!(!content.is_empty(), "detail-panel.js should not be empty");
    }

    // -----------------------------------------------------------------------
    // HTML structure tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_html_has_six_kanban_columns() {
        let html = read_web_file("index.html");
        // All 6 state columns must be present
        for state in &["logged", "open", "in_progress", "blocked", "ready", "closed"] {
            let id = format!("col-{}", state);
            assert!(
                html.contains(&id),
                "index.html should contain column id '{}' for state '{}'",
                id,
                state
            );
        }
    }

    #[test]
    fn test_html_has_column_count_badges() {
        let html = read_web_file("index.html");
        for state in &["logged", "open", "in_progress", "blocked", "ready", "closed"] {
            let id = format!("count-{}", state);
            assert!(
                html.contains(&id),
                "index.html should contain count badge id '{}' for state '{}'",
                id,
                state
            );
        }
    }

    #[test]
    fn test_html_has_filter_sections() {
        let html = read_web_file("index.html");
        // All 8 filter types must be present
        for filter in &[
            "portfolio", "project", "app", "story", "requirement", "tag", "state", "assignee",
        ] {
            let id = format!("filter-{}", filter);
            assert!(
                html.contains(&id),
                "index.html should contain filter section id '{}'",
                id
            );
        }
    }

    #[test]
    fn test_html_has_total_count_display() {
        let html = read_web_file("index.html");
        assert!(
            html.contains("id=\"totalCount\""),
            "index.html should contain total count display element"
        );
    }

    #[test]
    fn test_html_has_active_filters_container() {
        let html = read_web_file("index.html");
        assert!(
            html.contains("id=\"activeFilters\""),
            "index.html should contain active filters container for removable chips"
        );
    }

    #[test]
    fn test_html_has_detail_panel() {
        let html = read_web_file("index.html");
        assert!(
            html.contains("id=\"detailPanel\""),
            "index.html should contain detail panel for card editing"
        );
        // Detail panel should have fields for title, description, tags, story, deps
        assert!(html.contains("detailTitle"), "detail panel should have title field");
        assert!(
            html.contains("detailDescription"),
            "detail panel should have description field"
        );
        assert!(html.contains("detailTags"), "detail panel should have tags field");
        assert!(html.contains("detailStory"), "detail panel should have story field");
        assert!(
            html.contains("detailDeps"),
            "detail panel should have dependencies field"
        );
    }

    #[test]
    fn test_html_has_cross_altitude_tooltip() {
        let html = read_web_file("index.html");
        assert!(
            html.contains("crossAltitudeTooltip"),
            "index.html should contain cross-altitude tooltip element"
        );
        assert!(
            html.contains("cross-altitude prioritization is not meaningful"),
            "index.html should contain the exact cross-altitude rule tooltip text"
        );
    }

    #[test]
    fn test_html_links_css_and_js() {
        let html = read_web_file("index.html");
        assert!(
            html.contains("/css/kanban.css"),
            "index.html should link to css/kanban.css"
        );
        assert!(
            html.contains("/js/kanban.js"),
            "index.html should load js/kanban.js"
        );
        assert!(
            html.contains("/js/filters.js"),
            "index.html should load js/filters.js"
        );
        assert!(
            html.contains("/js/drag-drop.js"),
            "index.html should load js/drag-drop.js"
        );
        assert!(
            html.contains("/js/detail-panel.js"),
            "index.html should load js/detail-panel.js"
        );
    }

    #[test]
    fn test_html_no_external_dependencies() {
        let html = read_web_file("index.html");
        // Must not reference any CDN or external JS/CSS
        assert!(
            !html.contains("cdn.tailwindcss.com"),
            "index.html must not use external CDN (tailwind)"
        );
        assert!(
            !html.contains("unpkg.com"),
            "index.html must not use external CDN (unpkg)"
        );
        assert!(
            !html.contains("cdn.jsdelivr.net"),
            "index.html must not use external CDN (jsdelivr)"
        );
        assert!(
            !html.contains("https://"),
            "index.html must not load any external resources via https"
        );
    }

    // -----------------------------------------------------------------------
    // JS structure tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_kanban_js_has_six_columns() {
        let js = read_web_file("js/kanban.js");
        assert!(
            js.contains("logged") && js.contains("open") && js.contains("in_progress")
                && js.contains("blocked") && js.contains("ready") && js.contains("closed"),
            "kanban.js should define all 6 column states"
        );
    }

    #[test]
    fn test_kanban_js_has_auto_refresh() {
        let js = read_web_file("js/kanban.js");
        assert!(
            js.contains("REFRESH_INTERVAL"),
            "kanban.js should define a refresh interval constant"
        );
        assert!(
            js.contains("setInterval"),
            "kanban.js should use setInterval for auto-refresh"
        );
    }

    #[test]
    fn test_kanban_js_has_fetch_tasks() {
        let js = read_web_file("js/kanban.js");
        assert!(
            js.contains("fetchTasks"),
            "kanban.js should have a fetchTasks function"
        );
        assert!(
            js.contains("/api/tasks"),
            "kanban.js should call the /api/tasks endpoint"
        );
    }

    #[test]
    fn test_filters_js_has_count_annotations() {
        let js = read_web_file("js/filters.js");
        assert!(
            js.contains("toSuperscript"),
            "filters.js should have a toSuperscript function for count badges"
        );
        assert!(
            js.contains("computeScopedCount"),
            "filters.js should have a computeScopedCount function for scoped counts"
        );
    }

    #[test]
    fn test_filters_js_has_all_filter_types() {
        let js = read_web_file("js/filters.js");
        for filter in &[
            "portfolio", "project", "app", "story", "requirement", "tag", "state", "assignee",
        ] {
            assert!(
                js.contains(filter),
                "filters.js should reference filter type '{}'",
                filter
            );
        }
    }

    #[test]
    fn test_drag_drop_js_has_priority_reorder() {
        let js = read_web_file("js/drag-drop.js");
        assert!(
            js.contains("/api/priority"),
            "drag-drop.js should call PUT /api/priority for within-column reorder"
        );
        assert!(
            js.contains("reorderPriority"),
            "drag-drop.js should have a reorderPriority function"
        );
    }

    #[test]
    fn test_drag_drop_js_has_state_change() {
        let js = read_web_file("js/drag-drop.js");
        assert!(
            js.contains("/api/tasks/"),
            "drag-drop.js should call PUT /api/tasks/:id for between-column state change"
        );
        assert!(
            js.contains("updateTaskState"),
            "drag-drop.js should have an updateTaskState function"
        );
    }

    #[test]
    fn test_drag_drop_js_has_cross_altitude_rule() {
        let js = read_web_file("js/drag-drop.js");
        assert!(
            js.contains("isStoryFiltered"),
            "drag-drop.js should check isStoryFiltered for cross-altitude rule"
        );
        assert!(
            js.contains("cross-altitude"),
            "drag-drop.js should reference the cross-altitude rule"
        );
    }

    #[test]
    fn test_detail_panel_js_has_save() {
        let js = read_web_file("js/detail-panel.js");
        assert!(
            js.contains("saveTaskDetail"),
            "detail-panel.js should have a saveTaskDetail function"
        );
        assert!(
            js.contains("/api/tasks/"),
            "detail-panel.js should call PUT /api/tasks/:id to save"
        );
    }

    #[test]
    fn test_detail_panel_js_has_open() {
        let js = read_web_file("js/detail-panel.js");
        assert!(
            js.contains("openDetailPanel"),
            "detail-panel.js should have an openDetailPanel function"
        );
        assert!(
            js.contains("closeDetailPanel"),
            "detail-panel.js should have a closeDetailPanel function"
        );
    }

    // -----------------------------------------------------------------------
    // CSS structure tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_css_has_column_styles() {
        let css = read_web_file("css/kanban.css");
        assert!(css.contains(".column"), "kanban.css should style .column");
        assert!(
            css.contains(".column-header"),
            "kanban.css should style .column-header"
        );
        assert!(
            css.contains(".column-count"),
            "kanban.css should style .column-count badge"
        );
    }

    #[test]
    fn test_css_has_card_styles() {
        let css = read_web_file("css/kanban.css");
        assert!(css.contains(".card"), "kanban.css should style .card");
        assert!(
            css.contains(".card-title"),
            "kanban.css should style .card-title"
        );
        assert!(
            css.contains(".card-badge"),
            "kanban.css should style .card-badge"
        );
    }

    #[test]
    fn test_css_has_drag_disabled_style() {
        let css = read_web_file("css/kanban.css");
        assert!(
            css.contains("drag-disabled"),
            "kanban.css should style the drag-disabled state for cross-altitude rule"
        );
    }

    #[test]
    fn test_css_has_detail_panel_styles() {
        let css = read_web_file("css/kanban.css");
        assert!(
            css.contains("#detailPanel"),
            "kanban.css should style the detail panel"
        );
    }

    #[test]
    fn test_css_has_filter_sidebar_styles() {
        let css = read_web_file("css/kanban.css");
        assert!(
            css.contains("#filterSidebar"),
            "kanban.css should style the filter sidebar"
        );
        assert!(
            css.contains(".filter-count"),
            "kanban.css should style the filter count badges"
        );
        assert!(
            css.contains(".zero-count"),
            "kanban.css should style the zero-count (greyed out) state"
        );
    }
}
