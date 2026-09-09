//! Priority ordering module for the `priority_order` table.
//!
//! Provides CRUD operations for relative task ordering per `(scope_type, scope_id)`
//! pair, enabling drag-and-drop priority reordering. The seven scope types are:
//! `global`, `portfolio`, `project`, `app`, `requirement`, `story`, `tag`.
//!
//! When a task is moved to a new position, other tasks in the same scope are
//! shifted to maintain a gap-free sequence (0, 1, 2, ...). All position updates
//! are atomic (SQLite transaction).

use anyhow::Result;
use rusqlite::Connection;

/// The seven valid scope types for priority ordering.
pub const VALID_SCOPE_TYPES: &[&str] =
    &["global", "portfolio", "project", "app", "requirement", "story", "tag"];

/// Validate that `scope_type` is one of the seven allowed values.
pub fn validate_scope_type(scope_type: &str) -> Result<()> {
    if !VALID_SCOPE_TYPES.contains(&scope_type) {
        anyhow::bail!(
            "Invalid scope type \"{}\". Allowed: global, portfolio, project, app, requirement, story, tag",
            scope_type
        );
    }
    Ok(())
}

/// A manager for relative priority ordering within a `(scope_type, scope_id)`
/// pair. Wraps a SQLite connection and provides atomic, gap-free position
/// management.
pub struct PriorityManager<'a> {
    conn: &'a Connection,
}

impl<'a> PriorityManager<'a> {
    /// Create a new `PriorityManager` wrapping the given connection.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Move a task to `new_position` within `(scope_type, scope_id)`. Shifts
    /// other tasks to maintain gap-free ordering. Uses a SQLite transaction
    /// for atomicity. Returns the updated ordered task ID list for the scope.
    pub fn set_position(
        &self,
        scope_type: &str,
        scope_id: &str,
        task_id: &str,
        new_position: i64,
    ) -> Result<Vec<String>> {
        validate_scope_type(scope_type)?;
        if new_position < 0 {
            anyhow::bail!("Position must be non-negative, got {}", new_position);
        }

        self.conn.execute_batch("BEGIN IMMEDIATE")?;

        let result = (|| -> Result<()> {
            // Get the task's current position in this scope (if any)
            let old_position: Option<i64> = self
                .conn
                .query_row(
                    "SELECT position FROM priority_order
                     WHERE scope_type = ?1 AND scope_id = ?2 AND task_id = ?3",
                    rusqlite::params![scope_type, scope_id, task_id],
                    |row| row.get(0),
                )
                .ok();

            // If the task already has a position, close the gap at old position
            if let Some(old_pos) = old_position {
                self.conn.execute(
                    "DELETE FROM priority_order
                     WHERE scope_type = ?1 AND scope_id = ?2 AND task_id = ?3",
                    rusqlite::params![scope_type, scope_id, task_id],
                )?;
                self.conn.execute(
                    "UPDATE priority_order SET position = position - 1
                     WHERE scope_type = ?1 AND scope_id = ?2 AND position > ?3",
                    rusqlite::params![scope_type, scope_id, old_pos],
                )?;
            }

            // Shift tasks at or after new_position up by 1 to make room
            self.conn.execute(
                "UPDATE priority_order SET position = position + 1
                 WHERE scope_type = ?1 AND scope_id = ?2 AND position >= ?3",
                rusqlite::params![scope_type, scope_id, new_position],
            )?;

            // Insert the task at the new position
            self.conn.execute(
                "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![scope_type, scope_id, task_id, new_position],
            )?;

            // Compact: renumber to 0, 1, 2, ... to maintain gap-free sequence
            self.compact(scope_type, scope_id)?;

            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT")?;
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }

        self.get_ordering(scope_type, scope_id)
    }

    /// Return the ordered list of task IDs for a scope, ordered by position.
    /// Auto-assigns tasks that belong to the scope but have no
    /// `priority_order` entry to the end of the ordering.
    pub fn get_ordering(&self, scope_type: &str, scope_id: &str) -> Result<Vec<String>> {
        validate_scope_type(scope_type)?;

        // Auto-assign tasks in the scope that have no priority_order entry
        self.auto_assign(scope_type, scope_id)?;

        let mut stmt = self.conn.prepare(
            "SELECT task_id FROM priority_order
             WHERE scope_type = ?1 AND scope_id = ?2
             ORDER BY position",
        )?;
        let rows = stmt.query_map(rusqlite::params![scope_type, scope_id], |row| {
            let id: String = row.get(0)?;
            Ok(id)
        })?;
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row?);
        }
        Ok(tasks)
    }

    /// Return the position of a single task in a scope, or `None` if no
    /// entry exists.
    #[allow(dead_code)]
    pub fn get_position(
        &self,
        scope_type: &str,
        scope_id: &str,
        task_id: &str,
    ) -> Result<Option<i64>> {
        validate_scope_type(scope_type)?;
        let position = self
            .conn
            .query_row(
                "SELECT position FROM priority_order
                 WHERE scope_type = ?1 AND scope_id = ?2 AND task_id = ?3",
                rusqlite::params![scope_type, scope_id, task_id],
                |row| row.get(0),
            )
            .ok();
        Ok(position)
    }

    /// Remove all `priority_order` entries for a task across all scopes.
    /// Shifts positions in each affected scope to close gaps.
    #[allow(dead_code)]
    pub fn remove_task(&self, task_id: &str) -> Result<()> {
        // Find all scopes the task has entries in
        let mut stmt = self
            .conn
            .prepare("SELECT scope_type, scope_id FROM priority_order WHERE task_id = ?1")?;
        let scopes: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![task_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        if scopes.is_empty() {
            return Ok(());
        }

        self.conn.execute_batch("BEGIN IMMEDIATE")?;

        let result = (|| -> Result<()> {
            for (scope_type, scope_id) in &scopes {
                self.conn.execute(
                    "DELETE FROM priority_order
                     WHERE scope_type = ?1 AND scope_id = ?2 AND task_id = ?3",
                    rusqlite::params![scope_type, scope_id, task_id],
                )?;
                self.compact(scope_type, scope_id)?;
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT")?;
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }

        Ok(())
    }

    /// Add a new task at the end of a scope's ordering (position = max + 1,
    /// or 0 if the scope is empty). Returns the assigned position.
    pub fn init_task(&self, scope_type: &str, scope_id: &str, task_id: &str) -> Result<i64> {
        validate_scope_type(scope_type)?;

        let max_pos: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), -1) FROM priority_order
                 WHERE scope_type = ?1 AND scope_id = ?2",
                rusqlite::params![scope_type, scope_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        let position = max_pos + 1;
        self.conn.execute(
            "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(scope_type, scope_id, task_id) DO NOTHING",
            rusqlite::params![scope_type, scope_id, task_id, position],
        )?;

        Ok(position)
    }

    /// Renumber all entries in a scope to 0, 1, 2, ... ordered by current
    /// position, maintaining a gap-free sequence.
    fn compact(&self, scope_type: &str, scope_id: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT task_id FROM priority_order
             WHERE scope_type = ?1 AND scope_id = ?2
             ORDER BY position",
        )?;
        let task_ids: Vec<String> = stmt
            .query_map(rusqlite::params![scope_type, scope_id], |row| {
                let id: String = row.get(0)?;
                Ok(id)
            })?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        for (i, task_id) in task_ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE priority_order SET position = ?1
                 WHERE scope_type = ?2 AND scope_id = ?3 AND task_id = ?4",
                rusqlite::params![i as i64, scope_type, scope_id, task_id],
            )?;
        }

        Ok(())
    }

    /// Auto-assign tasks that belong to the scope but have no
    /// `priority_order` entry to the end of the ordering.
    fn auto_assign(&self, scope_type: &str, scope_id: &str) -> Result<()> {
        let scope_task_ids = self.get_scope_task_ids(scope_type, scope_id)?;

        for task_id in &scope_task_ids {
            let has_entry: bool = self.conn.query_row(
                "SELECT COUNT(*) > 0 FROM priority_order
                 WHERE scope_type = ?1 AND scope_id = ?2 AND task_id = ?3",
                rusqlite::params![scope_type, scope_id, task_id],
                |row| row.get(0),
            )?;
            if !has_entry {
                self.init_task(scope_type, scope_id, task_id)?;
            }
        }

        Ok(())
    }

    /// Return the task IDs that belong to a given scope, based on the
    /// scope type and ID.
    fn get_scope_task_ids(&self, scope_type: &str, scope_id: &str) -> Result<Vec<String>> {
        let sql = match scope_type {
            "global" => "SELECT id FROM tasks",
            "portfolio" => {
                "SELECT t.id FROM tasks t
                 JOIN projects p ON t.project_id = p.id
                 WHERE p.portfolio_id = ?1"
            }
            "project" => {
                "SELECT t.id FROM tasks t
                 JOIN projects p ON t.project_id = p.id
                 WHERE p.name = ?1"
            }
            "app" => {
                "SELECT t.id FROM tasks t
                 JOIN apps a ON t.app_id = a.id
                 WHERE a.name = ?1"
            }
            "requirement" => {
                "SELECT t.id FROM tasks t
                 WHERE t.story_id IN (
                     SELECT id FROM stories WHERE requirement_id = ?1
                 )"
            }
            "story" => "SELECT id FROM tasks WHERE story_id = ?1",
            "tag" => {
                "SELECT t.id FROM tasks t
                 JOIN task_tags tt ON t.id = tt.task_id
                 JOIN tags tag ON tt.tag_id = tag.id
                 WHERE tag.name = ?1"
            }
            _ => anyhow::bail!("Invalid scope type: {}", scope_type),
        };

        let mut stmt = self.conn.prepare(sql)?;
        let mut ids = Vec::new();
        if scope_type == "global" {
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            for row in rows {
                ids.push(row?);
            }
        } else {
            let rows = stmt.query_map(rusqlite::params![scope_id], |row| {
                row.get::<_, String>(0)
            })?;
            for row in rows {
                ids.push(row?);
            }
        }
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::PortfolioDb;
    use tempfile::TempDir;

    /// Set up a test database with the schema migrated.
    fn setup_db() -> (TempDir, PortfolioDb) {
        let temp = TempDir::new().unwrap();
        let db = PortfolioDb::open(temp.path().join("portfolio.db")).unwrap();
        db.migrate().unwrap();
        (temp, db)
    }

    /// Seed a full hierarchy: portfolio -> project -> app -> requirement ->
    /// story -> tasks, plus a tag.
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
            "INSERT INTO tasks (id, project_id, app_id, story_id, title, state, priority, markdown_path, synced_at)
             VALUES ('ja-2', 1, 1, 'story-1', 'Task Two', 'in_progress', 3, '/m2', '2026-01-02')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, project_id, title, state, priority, markdown_path, synced_at)
             VALUES ('ja-3', 1, 'Task Three', 'open', 1, '/m3', '2026-01-03')",
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

    #[test]
    fn test_set_position_basic() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
        }
        let pm = PriorityManager::new(&db.conn);
        let ordered = pm.set_position("story", "story-1", "ja-1", 0).unwrap();
        assert_eq!(ordered, vec!["ja-1".to_string(), "ja-2".to_string()]);
    }

    #[test]
    fn test_set_position_shifts_others() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
            // Initialize ordering: ja-1 at 0, ja-2 at 1
            db.conn
                .execute(
                    "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                     VALUES ('story', 'story-1', 'ja-1', 0)",
                    [],
                )
                .unwrap();
            db.conn
                .execute(
                    "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                     VALUES ('story', 'story-1', 'ja-2', 1)",
                    [],
                )
                .unwrap();
        }
        let pm = PriorityManager::new(&db.conn);
        // Move ja-2 from position 1 to position 0
        let ordered = pm.set_position("story", "story-1", "ja-2", 0).unwrap();
        assert_eq!(ordered, vec!["ja-2".to_string(), "ja-1".to_string()]);
    }

    #[test]
    fn test_set_position_gap_free() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
        }
        let pm = PriorityManager::new(&db.conn);
        // Multiple set_position calls
        pm.set_position("story", "story-1", "ja-1", 0).unwrap();
        pm.set_position("story", "story-1", "ja-2", 0).unwrap();
        pm.set_position("story", "story-1", "ja-1", 1).unwrap();

        let ordered = pm.get_ordering("story", "story-1").unwrap();
        assert_eq!(ordered.len(), 2);

        // Verify positions are gap-free
        let positions: Vec<i64> = db
            .conn
            .prepare("SELECT position FROM priority_order WHERE scope_type='story' AND scope_id='story-1' ORDER BY position")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(positions, vec![0, 1]);
    }

    #[test]
    fn test_get_ordering_empty_scope() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
        }
        let pm = PriorityManager::new(&db.conn);
        // Scope with no tasks at all
        let ordered = pm.get_ordering("story", "nonexistent-story").unwrap();
        assert!(ordered.is_empty());
    }

    #[test]
    fn test_get_ordering_auto_assign() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
            // Only ja-1 has a priority_order entry
            db.conn
                .execute(
                    "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                     VALUES ('story', 'story-1', 'ja-1', 0)",
                    [],
                )
                .unwrap();
        }
        let pm = PriorityManager::new(&db.conn);
        // ja-2 has no entry — should be auto-assigned to the end
        let ordered = pm.get_ordering("story", "story-1").unwrap();
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0], "ja-1");
        assert_eq!(ordered[1], "ja-2");
    }

    #[test]
    fn test_remove_task_closes_gap() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
            db.conn
                .execute(
                    "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                     VALUES ('story', 'story-1', 'ja-1', 0)",
                    [],
                )
                .unwrap();
            db.conn
                .execute(
                    "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                     VALUES ('story', 'story-1', 'ja-2', 1)",
                    [],
                )
                .unwrap();
        }
        let pm = PriorityManager::new(&db.conn);
        pm.remove_task("ja-1").unwrap();

        // ja-2 should now be at position 0 (gap closed)
        let pos = pm.get_position("story", "story-1", "ja-2").unwrap();
        assert_eq!(pos, Some(0));

        // ja-1 should have no entry
        let pos = pm.get_position("story", "story-1", "ja-1").unwrap();
        assert_eq!(pos, None);
    }

    #[test]
    fn test_init_task_appends() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
            db.conn
                .execute(
                    "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                     VALUES ('story', 'story-1', 'ja-1', 0)",
                    [],
                )
                .unwrap();
            db.conn
                .execute(
                    "INSERT INTO priority_order (scope_type, scope_id, task_id, position)
                     VALUES ('story', 'story-1', 'ja-2', 1)",
                    [],
                )
                .unwrap();
        }
        let pm = PriorityManager::new(&db.conn);
        // ja-3 is not in story-1's scope (no story_id), but we can still init it
        // in a different scope
        let pos = pm.init_task("global", "global", "ja-3").unwrap();
        assert_eq!(pos, 0);

        let pos2 = pm.init_task("global", "global", "ja-1").unwrap();
        assert_eq!(pos2, 1);
    }

    #[test]
    fn test_all_scope_types() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
        }
        let pm = PriorityManager::new(&db.conn);

        for scope_type in VALID_SCOPE_TYPES {
            let scope_id = match *scope_type {
                "global" => "global",
                "portfolio" => "p",
                "project" => "tkr",
                "app" => "default",
                "requirement" => "req-1",
                "story" => "story-1",
                "tag" => "security",
                _ => unreachable!(),
            };
            // set_position should work for all scope types
            let ordered = pm.set_position(scope_type, scope_id, "ja-1", 0).unwrap();
            assert!(!ordered.is_empty(), "scope_type={} should have tasks", scope_type);

            // get_ordering should return the same list
            let ordered2 = pm.get_ordering(scope_type, scope_id).unwrap();
            assert_eq!(ordered, ordered2);
        }
    }

    #[test]
    fn test_invalid_scope_type_rejected() {
        let (_temp, db) = setup_db();
        let pm = PriorityManager::new(&db.conn);
        let result = pm.set_position("invalid_scope", "scope1", "ja-1", 0);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid scope type"));
    }

    #[test]
    fn test_negative_position_rejected() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
        }
        let pm = PriorityManager::new(&db.conn);
        let result = pm.set_position("story", "story-1", "ja-1", -1);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_position_no_entry() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
        }
        let pm = PriorityManager::new(&db.conn);
        let pos = pm.get_position("story", "story-1", "ja-1").unwrap();
        assert_eq!(pos, None);
    }

    #[test]
    fn test_remove_task_no_entries() {
        let (_temp, db) = setup_db();
        {
            seed_hierarchy(&db.conn);
        }
        let pm = PriorityManager::new(&db.conn);
        // Should not error even if the task has no entries
        pm.remove_task("ja-1").unwrap();
    }
}
