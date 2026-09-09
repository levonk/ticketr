use anyhow::{Context, Result};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::db::PortfolioDb;
use crate::sync::SyncManager;

/// Debounce window for coalescing rapid file events (milliseconds).
const DEBOUNCE_MS: u64 = 500;

/// Maximum time to wait for the daemon process to exit after SIGTERM (seconds).
const STOP_TIMEOUT_SECS: u64 = 5;

// ---------------------------------------------------------------------------
// Path resolution (XDG compliant, overridable for tests)
// ---------------------------------------------------------------------------

/// Resolve the daemon data directory.
///
/// If `TKR_DAEMON_DIR` is set, it is used directly (primarily for testing).
/// Otherwise the `directories` crate resolves the platform default
/// (`~/.local/share/tkr` on Linux, `~/Library/Application Support/tkr` on macOS).
fn daemon_data_dir() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("TKR_DAEMON_DIR") {
        return Ok(PathBuf::from(path));
    }
    use directories::ProjectDirs;
    let proj_dirs = ProjectDirs::from("", "", "tkr")
        .ok_or_else(|| anyhow::anyhow!("could not determine data directory"))?;
    Ok(proj_dirs.data_dir().to_path_buf())
}

/// Path to the PID file.
fn pid_file_path() -> Result<PathBuf> {
    Ok(daemon_data_dir()?.join("daemon.pid"))
}

/// Path to the daemon log file.
fn log_file_path() -> Result<PathBuf> {
    Ok(daemon_data_dir()?.join("daemon.log"))
}

// ---------------------------------------------------------------------------
// PID file management
// ---------------------------------------------------------------------------

/// Write the current process PID to the PID file.
fn write_pid_file() -> Result<()> {
    let path = pid_file_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pid = std::process::id();
    std::fs::write(&path, pid.to_string())?;
    Ok(())
}

/// Read the PID from the PID file. Returns `None` if the file does not exist.
fn read_pid_file() -> Result<Option<u32>> {
    let path = pid_file_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let pid: u32 = content
        .trim()
        .parse()
        .with_context(|| format!("Invalid PID in file: {}", content.trim()))?;
    Ok(Some(pid))
}

/// Remove the PID file if it exists.
fn remove_pid_file() -> Result<()> {
    let path = pid_file_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Process management helpers (Unix)
// ---------------------------------------------------------------------------

/// Check whether a process with the given PID is alive.
fn is_process_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Send SIGTERM to a process.
fn send_sigterm(pid: u32) -> Result<()> {
    std::process::Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status()?;
    Ok(())
}

/// Send SIGKILL to a process.
fn send_sigkill(pid: u32) -> Result<()> {
    std::process::Command::new("kill")
        .arg("-KILL")
        .arg(pid.to_string())
        .status()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

/// Append a structured log line to the daemon log file.
fn log_message(level: &str, message: &str) -> Result<()> {
    let path = log_file_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let timestamp = chrono::Utc::now().to_rfc3339();
    let line = format!("[{}] {}  {}\n", timestamp, level, message);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Daemon run loop
// ---------------------------------------------------------------------------

/// Run the daemon loop: watch registered projects' `.tickets/` directories and
/// sync changes to SQLite. Blocks until the watcher is disconnected or the
/// process is killed.
pub fn run_daemon() -> Result<()> {
    write_pid_file()?;
    log_message("INFO", "Daemon started")?;

    // Install a ctrl-c handler so the PID file is cleaned up on SIGINT.
    let pid_path = pid_file_path()?;
    let _ = ctrlc::set_handler(move || {
        let _ = std::fs::remove_file(&pid_path);
        std::process::exit(0);
    });

    // Open the portfolio DB.
    let db_path = PortfolioDb::db_path()?;
    let db = PortfolioDb::open(&db_path)?;
    db.migrate()?;

    let sync = SyncManager::new(&db);

    // List registered projects and watch their `.tickets/` directories.
    let projects = sync.list_registered_projects()?;
    let (tx, rx) = channel::<notify::Result<notify::Event>>();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;

    let mut watched_count = 0usize;
    for project in &projects {
        let tickets_dir = PathBuf::from(&project.tickets_dir);
        if tickets_dir.exists() {
            match watcher.watch(&tickets_dir, RecursiveMode::Recursive) {
                Ok(()) => {
                    watched_count += 1;
                    log_message(
                        "INFO",
                        &format!("Watching: {} ({})", project.name, tickets_dir.display()),
                    )?;
                }
                Err(e) => {
                    log_message(
                        "WARN",
                        &format!("Failed to watch {}: {}", tickets_dir.display(), e),
                    )?;
                }
            }
        } else {
            log_message(
                "WARN",
                &format!("Tickets dir does not exist: {}", tickets_dir.display()),
            )?;
        }
    }

    log_message("INFO", &format!("Watching {} projects", watched_count))?;

    // Run an initial full sync so the DB is up to date on startup.
    match sync.sync_all() {
        Ok(_) => log_message("INFO", "Initial sync complete")?,
        Err(e) => log_message("ERROR", &format!("Initial sync failed: {}", e))?,
    }

    // Event loop with debouncing.
    let debounce = Duration::from_millis(DEBOUNCE_MS);
    let mut pending_sync = false;
    let mut last_event_time: Option<Instant> = None;

    loop {
        match rx.recv_timeout(debounce) {
            Ok(event) => match event {
                Ok(ev) => {
                    if matches!(
                        ev.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        log_message("INFO", &format!("File event: {:?}", ev.paths))?;
                        last_event_time = Some(Instant::now());
                        pending_sync = true;
                    }
                }
                Err(e) => {
                    log_message("ERROR", &format!("Watch error: {}", e))?;
                }
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if pending_sync && last_event_time.is_some_and(|t| t.elapsed() >= debounce) {
                    log_message("INFO", "Running debounced sync")?;
                    match sync.sync_all() {
                        Ok(report) => {
                            if report.tasks_indexed > 0
                                || report.tasks_updated > 0
                                || report.tasks_removed > 0
                            {
                                log_message(
                                    "INFO",
                                    &format!(
                                        "Sync: {} indexed, {} updated, {} removed",
                                        report.tasks_indexed,
                                        report.tasks_updated,
                                        report.tasks_removed
                                    ),
                                )?;
                            }
                        }
                        Err(e) => {
                            log_message("ERROR", &format!("Sync failed: {}", e))?;
                        }
                    }
                    pending_sync = false;
                    last_event_time = None;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                log_message("ERROR", "Watcher disconnected, shutting down")?;
                break;
            }
        }
    }

    remove_pid_file()?;
    log_message("INFO", "Daemon stopped")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Daemon CLI actions
// ---------------------------------------------------------------------------

/// Start the daemon as a detached background process.
pub fn start_daemon() -> Result<()> {
    // Check if already running.
    if let Some(pid) = read_pid_file()? {
        if is_process_alive(pid) {
            anyhow::bail!("Daemon already running (PID {})", pid);
        }
        // Stale PID file — remove it.
        remove_pid_file()?;
    }

    // Spawn the daemon as a detached child process running `tkr daemon run`.
    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("daemon")
        .arg("run")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    let child = cmd.spawn()?;
    let pid = child.id();

    // Give the child a moment to write its PID file.
    std::thread::sleep(Duration::from_millis(500));

    // Verify the daemon started by checking the PID file.
    match read_pid_file()? {
        Some(file_pid) if file_pid == pid => {
            // Open the DB to list projects for display.
            let db_path = PortfolioDb::db_path()?;
            let db = PortfolioDb::open(&db_path)?;
            db.migrate()?;
            let sync = SyncManager::new(&db);
            let projects = sync.list_registered_projects()?;

            println!("Daemon started (PID {})", pid);
            println!("Watching {} projects:", projects.len());
            for p in &projects {
                println!("  - {} ({})", p.name, p.tickets_dir);
            }
            let log_path = log_file_path()?;
            println!("Logs: {}", log_path.display());
        }
        Some(other) => {
            anyhow::bail!("Daemon PID mismatch: expected {}, got {}", pid, other);
        }
        None => {
            anyhow::bail!("Daemon failed to start (PID file not created)");
        }
    }

    Ok(())
}

/// Stop the daemon by reading the PID file and sending SIGTERM.
pub fn stop_daemon() -> Result<()> {
    let pid = read_pid_file()?
        .ok_or_else(|| anyhow::anyhow!("Daemon not running (no PID file)"))?;

    if !is_process_alive(pid) {
        remove_pid_file()?;
        println!("Daemon was not running (stale PID file removed)");
        return Ok(());
    }

    send_sigterm(pid)?;

    // Wait up to STOP_TIMEOUT_SECS for the process to exit.
    let poll_interval = Duration::from_millis(100);
    let deadline = Duration::from_secs(STOP_TIMEOUT_SECS);
    let waited = Duration::ZERO;
    let mut waited = waited;
    while waited < deadline {
        if !is_process_alive(pid) {
            break;
        }
        std::thread::sleep(poll_interval);
        waited += poll_interval;
    }

    if is_process_alive(pid) {
        send_sigkill(pid)?;
        std::thread::sleep(poll_interval);
    }

    remove_pid_file()?;
    println!("Daemon stopped (PID {})", pid);
    println!("PID file removed.");

    Ok(())
}

/// Report daemon status: running/stale/not-running, plus project and sync info.
pub fn status_daemon() -> Result<()> {
    match read_pid_file()? {
        Some(pid) if is_process_alive(pid) => {
            let db_path = PortfolioDb::db_path()?;
            let db = PortfolioDb::open(&db_path)?;
            db.migrate()?;
            let sync = SyncManager::new(&db);
            let state = sync.sync_status()?;

            println!("Daemon: running (PID {})", pid);
            println!("Watching {} projects", state.project_count);
            match state.last_sync_at {
                Some(ts) => println!("Last sync: {}", ts),
                None => println!("Last sync: (never)"),
            }
            println!("Tasks: {}", state.task_count);
        }
        Some(_) => {
            // Stale PID — clean up.
            remove_pid_file()?;
            println!("Daemon: not running (stale PID file removed)");
        }
        None => {
            println!("Daemon: not running");
        }
    }

    Ok(())
}

/// Restart the daemon (stop then start).
pub fn restart_daemon() -> Result<()> {
    if read_pid_file()?.is_some() {
        stop_daemon()?;
    }
    start_daemon()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    /// Serialise tests that touch the `TKR_DAEMON_DIR` env var so they don't
    /// interfere with each other when run in parallel.
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    /// Override `TKR_DAEMON_DIR` for the duration of the test.
    fn with_temp_daemon_dir() -> (TempDir, std::sync::MutexGuard<'static, ()>) {
        let guard = lock().lock().unwrap();
        let temp = TempDir::new().unwrap();
        std::env::set_var("TKR_DAEMON_DIR", temp.path());
        (temp, guard)
    }

    #[test]
    fn test_pid_file_write_read() {
        let (_dir, _guard) = with_temp_daemon_dir();

        write_pid_file().unwrap();
        let pid = read_pid_file().unwrap();
        assert_eq!(pid, Some(std::process::id()));
    }

    #[test]
    fn test_pid_file_stale_detection() {
        let (_dir, _guard) = with_temp_daemon_dir();

        // Write a PID for a process that almost certainly does not exist.
        let path = pid_file_path().unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "999999").unwrap();

        let pid = read_pid_file().unwrap();
        assert_eq!(pid, Some(999999));
        assert!(!is_process_alive(999999));
    }

    #[test]
    fn test_pid_file_missing() {
        let (_dir, _guard) = with_temp_daemon_dir();
        let pid = read_pid_file().unwrap();
        assert_eq!(pid, None);
    }

    #[test]
    fn test_remove_pid_file() {
        let (_dir, _guard) = with_temp_daemon_dir();

        write_pid_file().unwrap();
        assert!(pid_file_path().unwrap().exists());

        remove_pid_file().unwrap();
        assert!(!pid_file_path().unwrap().exists());

        // Removing a non-existent file is a no-op.
        remove_pid_file().unwrap();
    }

    #[test]
    fn test_log_message() {
        let (_dir, _guard) = with_temp_daemon_dir();

        log_message("INFO", "test message").unwrap();
        let content = std::fs::read_to_string(log_file_path().unwrap()).unwrap();
        assert!(content.contains("INFO"));
        assert!(content.contains("test message"));
    }
}
