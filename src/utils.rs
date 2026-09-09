use std::path::{Path, PathBuf};
use anyhow::Result;
use std::process::Command as StdCommand;

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

/// Detect the GitHub owner and repo name from `git remote get-url origin` run
/// in the given repo path. Returns `Ok(None)` if the repo has no `origin`
/// remote or the URL cannot be parsed as a GitHub URL. Supports HTTPS, SSH,
/// and `ssh://` URL formats.
pub fn detect_github_info(repo_path: &Path) -> Result<Option<(String, String)>> {
    let output = StdCommand::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(repo_path)
        .output();

    let url = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Ok(None),
    };

    Ok(parse_github_url(&url))
}

/// Parse a GitHub remote URL into `(owner, repo)`. Returns `None` if the URL
/// is not a recognised GitHub format. Trailing `.git` suffix is stripped.
fn parse_github_url(url: &str) -> Option<(String, String)> {
    // Strip trailing .git
    let url = url.trim();
    let url = url.strip_suffix(".git").unwrap_or(url);

    // SSH format: git@github.com:owner/repo
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        return split_owner_repo(rest);
    }
    // HTTPS or SSH-with-scheme: https://github.com/owner/repo
    //                          ssh://git@github.com/owner/repo
    if let Some(rest) = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("ssh://git@github.com/"))
        .or_else(|| url.strip_prefix("ssh://github.com/"))
    {
        return split_owner_repo(rest);
    }
    None
}

/// Split `owner/repo` (ignoring any extra path after the repo segment) into
/// `(owner, repo)`.
fn split_owner_repo(s: &str) -> Option<(String, String)> {
    let mut parts = s.split('/');
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_https() {
        let (owner, repo) = parse_github_url("https://github.com/levonk/tkr.git").unwrap();
        assert_eq!(owner, "levonk");
        assert_eq!(repo, "tkr");
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let (owner, repo) = parse_github_url("git@github.com:levonk/tkr.git").unwrap();
        assert_eq!(owner, "levonk");
        assert_eq!(repo, "tkr");
    }

    #[test]
    fn test_parse_github_url_ssh_scheme() {
        let (owner, repo) =
            parse_github_url("ssh://git@github.com/levonk/tkr.git").unwrap();
        assert_eq!(owner, "levonk");
        assert_eq!(repo, "tkr");
    }

    #[test]
    fn test_parse_github_url_no_git_suffix() {
        let (owner, repo) = parse_github_url("https://github.com/levonk/tkr").unwrap();
        assert_eq!(owner, "levonk");
        assert_eq!(repo, "tkr");
    }

    #[test]
    fn test_parse_github_url_non_github() {
        assert!(parse_github_url("https://gitlab.com/levonk/tkr.git").is_none());
    }
}
