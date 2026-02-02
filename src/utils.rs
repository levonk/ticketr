use std::path::{Path, PathBuf};
use anyhow::Result;

pub fn find_tickets_dir(repo_root: Option<String>) -> Result<PathBuf> {
    if let Some(root) = repo_root {
        let root_path = PathBuf::from(root);
        check_tickets_locations(&root_path)
            .ok_or_else(|| anyhow::anyhow!("No tickets directory found in {}", root_path.display()))
    } else {
        // Start from current directory and go up
        let mut current = std::env::current_dir()?;
        loop {
            if let Some(tickets_dir) = check_tickets_locations(&current) {
                return Ok(tickets_dir);
            }

            if !current.pop() {
                break; // Reached filesystem root
            }
        }

        // Fallback to current directory
        Ok(std::env::current_dir()?.join(".tickets"))
    }
}

fn check_tickets_locations(base: &Path) -> Option<PathBuf> {
    let locations = [
        base.join(".tickets"),
        base.join("tickets"),
        base.join(".ticket"),
        base.join("ticket"),
    ];

    for location in &locations {
        if location.exists() || location.parent().is_some_and(|p| p.exists()) {
            return Some(location.clone());
        }
    }

    None
}

#[allow(dead_code)]
pub fn get_repo_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }

        if !current.pop() {
            anyhow::bail!("Not in a git repository");
        }
    }
}
