//! Portfolio views — read-only cross-project task groupings.
//!
//! Each view function runs a SQL query against the portfolio SQLite DB and
//! returns a [`ViewResult`] of grouped tasks. The CLI renders the result as
//! text (default) or JSON (`--json`).

use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;

/// A single task row in a view group.
#[derive(Debug, Serialize)]
pub struct ViewTask {
    pub id: String,
    pub title: String,
    pub state: String,
}

/// A named group of tasks (e.g. a requirement, story, tag, or project).
#[derive(Debug, Serialize)]
pub struct ViewGroup {
    pub group: String,
    pub count: usize,
    pub tasks: Vec<ViewTask>,
}

/// The full result of a portfolio view query, ready for rendering.
#[derive(Debug, Serialize)]
pub struct ViewResult {
    /// JSON shape version — bump on breaking changes.
    pub version: u32,
    pub groups: Vec<ViewGroup>,
    pub total_tasks: usize,
    pub group_count: usize,
}

impl ViewResult {
    /// Build a [`ViewResult`] from a list of groups, computing the totals.
    fn from_groups(groups: Vec<ViewGroup>) -> Self {
        let total_tasks: usize = groups.iter().map(|g| g.count).sum();
        let group_count = groups.len();
        Self {
            version: 1,
            groups,
            total_tasks,
            group_count,
        }
    }
}

/// Count the total number of tasks in the DB.
fn count_tasks(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;
    Ok(count)
}

// -----------------------------------------------------------------
// Grouping queries
// -----------------------------------------------------------------

/// Group tasks by their story's requirement (via `stories.requirement_id`).
/// Requirements with zero tasks are included as empty groups.
pub fn view_by_requirement(conn: &Connection) -> Result<ViewResult> {
    if count_tasks(conn)? == 0 {
        return Ok(ViewResult::from_groups(Vec::new()));
    }

    let mut stmt = conn.prepare(
        "SELECT r.id, r.title, t.id, t.title, t.state
         FROM requirements r
         LEFT JOIN stories s ON s.requirement_id = r.id
         LEFT JOIN tasks t ON t.story_id = s.id
         ORDER BY r.position, r.created, t.id",
    )?;

    let rows = stmt.query_map([], |row| {
        let req_id: String = row.get(0)?;
        let req_title: String = row.get(1)?;
        let task_id: Option<String> = row.get(2)?;
        let task_title: Option<String> = row.get(3)?;
        let task_state: Option<String> = row.get(4)?;
        Ok((req_id, req_title, task_id, task_title, task_state))
    })?;

    let mut groups: Vec<ViewGroup> = Vec::new();
    for row in rows {
        let (_req_id, req_title, task_id, task_title, task_state) = row?;
        // Find or create the group for this requirement.
        let group = groups
            .iter_mut()
            .find(|g| g.group == req_title)
            .map(|g| g as *mut ViewGroup);
        let group = if let Some(g) = group {
            // SAFETY: the pointer is valid for the duration of this iteration.
            unsafe { &mut *g }
        } else {
            groups.push(ViewGroup {
                group: req_title,
                count: 0,
                tasks: Vec::new(),
            });
            groups.last_mut().unwrap()
        };

        if let (Some(id), Some(title), Some(state)) = (task_id, task_title, task_state) {
            group.tasks.push(ViewTask { id, title, state });
            group.count += 1;
        }
    }

    Ok(ViewResult::from_groups(groups))
}

/// Group tasks by story. Tasks with no story link fall into a `(none)` group.
pub fn view_by_story(conn: &Connection) -> Result<ViewResult> {
    if count_tasks(conn)? == 0 {
        return Ok(ViewResult::from_groups(Vec::new()));
    }

    // Query tasks joined to their story (story may be NULL).
    let mut stmt = conn.prepare(
        "SELECT s.id, s.title, t.id, t.title, t.state
         FROM tasks t
         LEFT JOIN stories s ON s.id = t.story_id
         ORDER BY s.position, s.created, t.id",
    )?;

    let rows = stmt.query_map([], |row| {
        let story_id: Option<String> = row.get(0)?;
        let story_title: Option<String> = row.get(1)?;
        let task_id: String = row.get(2)?;
        let task_title: String = row.get(3)?;
        let task_state: String = row.get(4)?;
        Ok((story_id, story_title, task_id, task_title, task_state))
    })?;

    let mut groups: Vec<ViewGroup> = Vec::new();
    for row in rows {
        let (_story_id, story_title, task_id, task_title, task_state) = row?;
        let group_name = story_title.unwrap_or_else(|| "(none)".to_string());

        let group = groups
            .iter_mut()
            .find(|g| g.group == group_name)
            .map(|g| g as *mut ViewGroup);
        let group = if let Some(g) = group {
            unsafe { &mut *g }
        } else {
            groups.push(ViewGroup {
                group: group_name,
                count: 0,
                tasks: Vec::new(),
            });
            groups.last_mut().unwrap()
        };

        group.tasks.push(ViewTask {
            id: task_id,
            title: task_title,
            state: task_state,
        });
        group.count += 1;
    }

    Ok(ViewResult::from_groups(groups))
}

/// Group tasks by project.
pub fn view_by_project(conn: &Connection) -> Result<ViewResult> {
    if count_tasks(conn)? == 0 {
        return Ok(ViewResult::from_groups(Vec::new()));
    }

    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, t.id, t.title, t.state
         FROM projects p
         LEFT JOIN tasks t ON t.project_id = p.id
         ORDER BY p.id, t.id",
    )?;

    let rows = stmt.query_map([], |row| {
        let _project_id: i64 = row.get(0)?;
        let project_name: String = row.get(1)?;
        let task_id: Option<String> = row.get(2)?;
        let task_title: Option<String> = row.get(3)?;
        let task_state: Option<String> = row.get(4)?;
        Ok((project_name, task_id, task_title, task_state))
    })?;

    let mut groups: Vec<ViewGroup> = Vec::new();
    for row in rows {
        let (project_name, task_id, task_title, task_state) = row?;
        let group = groups
            .iter_mut()
            .find(|g| g.group == project_name)
            .map(|g| g as *mut ViewGroup);
        let group = if let Some(g) = group {
            unsafe { &mut *g }
        } else {
            groups.push(ViewGroup {
                group: project_name,
                count: 0,
                tasks: Vec::new(),
            });
            groups.last_mut().unwrap()
        };

        if let (Some(id), Some(title), Some(state)) = (task_id, task_title, task_state) {
            group.tasks.push(ViewTask { id, title, state });
            group.count += 1;
        }
    }

    Ok(ViewResult::from_groups(groups))
}

/// Group tasks by tag. When `tag` is `Some(name)`, only tasks carrying that
/// tag are shown, grouped by project. When `tag` is `None`, all tags are
/// listed with their task counts.
pub fn view_by_tag(conn: &Connection, tag: Option<&str>) -> Result<ViewResult> {
    match tag {
        Some(tag_name) => view_by_tag_filtered(conn, tag_name),
        None => view_by_tag_summary(conn),
    }
}

/// Show tasks carrying `tag_name`, grouped by project.
fn view_by_tag_filtered(conn: &Connection, tag_name: &str) -> Result<ViewResult> {
    let mut stmt = conn.prepare(
        "SELECT p.name, t.id, t.title, t.state
         FROM tasks t
         JOIN task_tags tt ON tt.task_id = t.id
         JOIN tags tag ON tag.id = tt.tag_id AND tag.name = ?1
         JOIN projects p ON p.id = t.project_id
         ORDER BY p.name, t.id",
    )?;

    let rows = stmt.query_map(rusqlite::params![tag_name], |row| {
        let project_name: String = row.get(0)?;
        let task_id: String = row.get(1)?;
        let task_title: String = row.get(2)?;
        let task_state: String = row.get(3)?;
        Ok((project_name, task_id, task_title, task_state))
    })?;

    let mut groups: Vec<ViewGroup> = Vec::new();
    for row in rows {
        let (project_name, task_id, task_title, task_state) = row?;
        let group = groups
            .iter_mut()
            .find(|g| g.group == project_name)
            .map(|g| g as *mut ViewGroup);
        let group = if let Some(g) = group {
            unsafe { &mut *g }
        } else {
            groups.push(ViewGroup {
                group: project_name,
                count: 0,
                tasks: Vec::new(),
            });
            groups.last_mut().unwrap()
        };

        group.tasks.push(ViewTask {
            id: task_id,
            title: task_title,
            state: task_state,
        });
        group.count += 1;
    }

    Ok(ViewResult::from_groups(groups))
}

/// List all tags with their task counts (no grouping by project).
fn view_by_tag_summary(conn: &Connection) -> Result<ViewResult> {
    let mut stmt = conn.prepare(
        "SELECT t.name, COUNT(tt.task_id)
         FROM tags t
         LEFT JOIN task_tags tt ON tt.tag_id = t.id
         GROUP BY t.name
         ORDER BY t.name",
    )?;

    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        Ok((name, count))
    })?;

    let mut groups: Vec<ViewGroup> = Vec::new();
    for row in rows {
        let (name, count) = row?;
        groups.push(ViewGroup {
            group: name,
            count: count as usize,
            tasks: Vec::new(),
        });
    }

    Ok(ViewResult::from_groups(groups))
}

// -----------------------------------------------------------------
// Renderers
// -----------------------------------------------------------------

/// Render a [`ViewResult`] as human-readable text.
pub fn render_text(result: &ViewResult) -> String {
    if result.groups.is_empty() {
        return "No tasks found. Run 'tkr sync' first.".to_string();
    }

    let mut out = String::new();
    for group in &result.groups {
        out.push_str(&format!("{} ({})\n", group.group, group.count));
        if group.tasks.is_empty() {
            out.push_str("  (empty)\n");
        } else {
            for task in &group.tasks {
                out.push_str(&format!("  {} - {} ({})\n", task.id, task.title, task.state));
            }
        }
    }
    out.push_str(&format!(
        "Showing {} tasks across {} groups\n",
        result.total_tasks, result.group_count
    ));
    out
}

/// Render a [`ViewResult`] as a JSON string.
pub fn render_json(result: &ViewResult) -> Result<String> {
    Ok(serde_json::to_string_pretty(result)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_db() -> (TempDir, Connection) {
        let temp = TempDir::new().unwrap();
        let db = crate::db::PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();
        let conn = db.conn;
        (temp, conn)
    }

    fn seed_tasks(conn: &Connection) {
        // portfolio + project + app
        conn.execute(
            "INSERT INTO portfolios (id, name, created) VALUES ('p', 'P', '2026-01-01')",
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
            "INSERT INTO projects (id, portfolio_id, name, repo_path, tickets_dir, registered_at)
             VALUES (2, 'p', 'dotfiles', '/d', '/d/.tickets', '2026-01-01')",
            [],
        )
        .unwrap();
        // requirement + story
        conn.execute(
            "INSERT INTO requirements (id, portfolio_id, title, created) VALUES ('req-1', 'p', 'Multi-account Sync', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO stories (id, requirement_id, title, created) VALUES ('story-1', 'req-1', 'Portfolio Layer', '2026-01-01')",
            [],
        )
        .unwrap();
        // tasks: two linked to story, one unlinked, across two projects
        conn.execute(
            "INSERT INTO tasks (id, project_id, title, state, markdown_path, synced_at, story_id)
             VALUES ('ja-1', 1, 'Task One', 'open', '/m1', '2026-01-01', 'story-1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, project_id, title, state, markdown_path, synced_at, story_id)
             VALUES ('ja-2', 1, 'Task Two', 'in_progress', '/m2', '2026-01-01', 'story-1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, project_id, title, state, markdown_path, synced_at, story_id)
             VALUES ('ja-3', 2, 'Task Three', 'open', '/m3', '2026-01-01', NULL)",
            [],
        )
        .unwrap();
        // tags
        conn.execute("INSERT INTO tags (id, name) VALUES (1, 'security')", []).unwrap();
        conn.execute(
            "INSERT INTO task_tags (task_id, tag_id) VALUES ('ja-1', 1)",
            [],
        )
        .unwrap();
    }

    #[test]
    fn test_view_by_requirement() {
        let (_temp, conn) = setup_db();
        seed_tasks(&conn);
        let result = view_by_requirement(&conn).unwrap();
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].group, "Multi-account Sync");
        assert_eq!(result.groups[0].count, 2);
        assert_eq!(result.total_tasks, 2);
        assert_eq!(result.group_count, 1);
    }

    #[test]
    fn test_view_by_requirement_empty_group() {
        let (_temp, conn) = setup_db();
        // requirement with no stories/tasks
        conn.execute(
            "INSERT INTO portfolios (id, name, created) VALUES ('p', 'P', '2026-01-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO requirements (id, portfolio_id, title, created) VALUES ('req-e', 'p', 'Empty Req', '2026-01-01')",
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
            "INSERT INTO tasks (id, project_id, title, state, markdown_path, synced_at)
             VALUES ('ja-x', 1, 'X', 'open', '/x', '2026-01-01')",
            [],
        )
        .unwrap();
        let result = view_by_requirement(&conn).unwrap();
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].group, "Empty Req");
        assert_eq!(result.groups[0].count, 0);
    }

    #[test]
    fn test_view_by_story_includes_none() {
        let (_temp, conn) = setup_db();
        seed_tasks(&conn);
        let result = view_by_story(&conn).unwrap();
        // Two groups: "Portfolio Layer" (2 tasks) and "(none)" (1 task)
        assert_eq!(result.groups.len(), 2);
        let none_group = result.groups.iter().find(|g| g.group == "(none)").unwrap();
        assert_eq!(none_group.count, 1);
        let story_group = result
            .groups
            .iter()
            .find(|g| g.group == "Portfolio Layer")
            .unwrap();
        assert_eq!(story_group.count, 2);
        assert_eq!(result.total_tasks, 3);
    }

    #[test]
    fn test_view_by_project() {
        let (_temp, conn) = setup_db();
        seed_tasks(&conn);
        let result = view_by_project(&conn).unwrap();
        assert_eq!(result.groups.len(), 2);
        assert_eq!(result.total_tasks, 3);
    }

    #[test]
    fn test_view_by_tag_filtered() {
        let (_temp, conn) = setup_db();
        seed_tasks(&conn);
        let result = view_by_tag(&conn, Some("security")).unwrap();
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].group, "tkr");
        assert_eq!(result.groups[0].count, 1);
    }

    #[test]
    fn test_view_by_tag_summary() {
        let (_temp, conn) = setup_db();
        seed_tasks(&conn);
        let result = view_by_tag(&conn, None).unwrap();
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].group, "security");
        assert_eq!(result.groups[0].count, 1);
    }

    #[test]
    fn test_view_empty_db() {
        let (_temp, conn) = setup_db();
        let result = view_by_project(&conn).unwrap();
        assert!(result.groups.is_empty());
        let text = render_text(&result);
        assert!(text.contains("No tasks found"));
    }

    #[test]
    fn test_render_text() {
        let (_temp, conn) = setup_db();
        seed_tasks(&conn);
        let result = view_by_project(&conn).unwrap();
        let text = render_text(&result);
        assert!(text.contains("tkr (2)"));
        assert!(text.contains("dotfiles (1)"));
        assert!(text.contains("Showing 3 tasks across 2 groups"));
    }

    #[test]
    fn test_render_json() {
        let (_temp, conn) = setup_db();
        seed_tasks(&conn);
        let result = view_by_project(&conn).unwrap();
        let json = render_json(&result).unwrap();
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"group\""));
        assert!(json.contains("\"total_tasks\": 3"));
    }
}
