use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply};
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

    // CORS headers
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // API routes
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

    // Serve static files
    let static_files = warp::get()
        .and(warp::fs::dir("web"))
        .or(warp::get().and(warp::path("index.html")).and(warp::fs::file("web/index.html")));

    let routes = static_files
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

    let config_path = PathBuf::from(config_dir).join("ticketr").join("config.yml");

    if config_path.exists() {
        let content = std::fs::read_to_string(config_path).ok()?;
        serde_yaml::from_str(&content).ok()
    } else {
        None
    }
}

fn load_git_root_config() -> Option<WebConfig> {
    // Try to find git root and check for .config/ticketr/config.yml
    let mut current = std::env::current_dir().ok()?;

    loop {
        let config_path = current.join(".config").join("ticketr").join("config.yml");

        if config_path.exists() {
            let content = std::fs::read_to_string(config_path).ok()?;
            if let Some(config) = serde_yaml::from_str(&content).ok() {
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
