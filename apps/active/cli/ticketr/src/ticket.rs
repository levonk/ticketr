use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: String,
    pub deps: Vec<String>,
    pub links: Vec<String>,
    pub created: DateTime<Utc>,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acceptance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<Note>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct TicketManager {
    pub tickets_dir: PathBuf,
    pub project: Option<String>,
    pub category: Option<String>,
}

impl TicketManager {
    pub fn new(tickets_dir: PathBuf, project: Option<String>, category: Option<String>) -> Self {
        Self {
            tickets_dir,
            project,
            category,
        }
    }

    pub fn ensure_status_directories(&self) -> Result<()> {
        let statuses = ["open", "in_progress", "closed", "blocked", "ready", "icebox", "archive"];
        for status in &statuses {
            let status_dir = self.tickets_dir.join(status);
            if !status_dir.exists() {
                fs::create_dir_all(&status_dir)?;
            }
        }
        Ok(())
    }

    fn get_status_dir(&self, status: &str) -> PathBuf {
        self.tickets_dir.join(status)
    }

    fn ticket_path_by_status(&self, id: &str, status: &str) -> Result<PathBuf> {
        let status_dir = self.get_status_dir(status);
        let exact_path = status_dir.join(format!("{}.md", id));

        if exact_path.exists() {
            return Ok(exact_path);
        }

        // Try partial ID matching in the specific status directory
        if let Ok(entries) = fs::read_dir(&status_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(id) && filename.ends_with(".md") {
                        return Ok(path);
                    }
                }
            }
        }

        // Fallback: search all status directories
        self.ticket_path(id)
    }

    pub fn ensure_tickets_dir(&self) -> Result<()> {
        if !self.tickets_dir.exists() {
            fs::create_dir_all(&self.tickets_dir)?;
        }
        Ok(())
    }

    pub fn generate_id(&self) -> Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Get directory name for prefix
        let dir_name = self.tickets_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unk");

        // Extract first letter of each segment
        let re = regex::Regex::new(r"[-_]").unwrap();
        let segments: Vec<&str> = re.split(dir_name).collect();
        let prefix: String = segments.iter()
            .filter_map(|s| s.chars().next())
            .collect();

        let prefix = if prefix.is_empty() {
            dir_name.chars().take(3).collect()
        } else {
            prefix
        };

        // Generate unique ID using timestamp and random component
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis();
        let uuid = uuid::Uuid::new_v4();
        let uuid_str = uuid.as_simple().to_string();
        let hash = format!("{:x}{}", timestamp % 10000, &uuid_str[..4]);

        Ok(format!("{}-{}", prefix, hash))
    }

    pub fn ticket_path(&self, id: &str) -> Result<PathBuf> {
        let exact_path = self.tickets_dir.join(format!("{}.md", id));

        if exact_path.exists() {
            return Ok(exact_path);
        }

        // Try partial ID matching
        if let Ok(entries) = fs::read_dir(&self.tickets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(id) && filename.ends_with(".md") {
                        return Ok(path);
                    }
                }
            }
        }

        Ok(exact_path)
    }

    pub fn load_ticket(&self, id: &str) -> Result<Ticket> {
        let path = self.ticket_path(id)?;
        let content = fs::read_to_string(&path)?;

        // Split YAML frontmatter and content
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            anyhow::bail!("Invalid ticket format");
        }

        let yaml_content = parts[1].trim();
        let ticket: Ticket = serde_yaml::from_str(yaml_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML for ticket {}: {}", id, e))?;

        Ok(ticket)
    }

    pub fn save_ticket(&self, ticket: &Ticket) -> Result<()> {
        self.ensure_status_directories()?;

        let path = self.get_status_dir(&ticket.status).join(format!("{}.md", ticket.id));

        // Serialize YAML frontmatter
        let yaml_content = serde_yaml::to_string(ticket)?;

        // Format as markdown with frontmatter
        let mut content = format!("---\n{}---\n\n# {}\n",
            yaml_content.trim(),
            ticket.title
        );

        // Add description if present
        if let Some(desc) = &ticket.description {
            content.push_str(&format!("\n\n{}", desc));
        }

        // Add notes if present
        if let Some(notes) = &ticket.notes {
            content.push_str("\n\n## Notes\n");
            for note in notes {
                content.push_str(&format!("\n**{}**: {}",
                    note.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    note.content
                ));
            }
        }

        fs::write(&path, content)?;

        Ok(())
    }

    pub fn move_ticket_to_status(&self, ticket_id: &str, new_status: &str) -> Result<()> {
        let mut ticket = self.load_ticket(ticket_id)?;
        let old_status = ticket.status.clone();

        if old_status == new_status {
            return Ok(()); // Already in the right status
        }

        // Handle special closing logic
        if new_status == "closed" {
            self.handle_ticket_closure(&ticket)?;
        }

        // Update status and save to new location
        ticket.status = new_status.to_string();
        self.save_ticket(&ticket)?;

        // Remove from old location if it exists
        let old_path = self.ticket_path_by_status(ticket_id, &old_status)?;
        if old_path.exists() && old_path != self.get_status_dir(new_status).join(format!("{}.md", ticket_id)) {
            fs::remove_file(old_path)?;
        }

        Ok(())
    }

    fn handle_ticket_closure(&self, closing_ticket: &Ticket) -> Result<()> {
        // Find tickets that depend on this closing ticket
        let all_tickets = self.list_tickets()?;

        for ticket in &all_tickets {
            if ticket.deps.contains(&closing_ticket.id) && ticket.status == "blocked" {
                // Unblock this ticket by removing the dependency and setting to ready
                let mut updated_ticket = ticket.clone();
                updated_ticket.deps.retain(|dep| dep != &closing_ticket.id);
                updated_ticket.status = "ready".to_string();

                println!("Unblocking ticket {} (was blocked by {})",
                    updated_ticket.id, closing_ticket.id);

                self.save_ticket(&updated_ticket)?;

                // Remove from blocked directory
                let blocked_path = self.ticket_path_by_status(&updated_ticket.id, "blocked")?;
                if blocked_path.exists() {
                    fs::remove_file(blocked_path)?;
                }
            }
        }

        Ok(())
    }

    pub fn list_tickets(&self) -> Result<Vec<Ticket>> {
        self.ensure_status_directories()?;

        let mut tickets = Vec::new();

        if !self.tickets_dir.exists() {
            return Ok(tickets);
        }

        // Use traditional directory scan with status directories
        let statuses = ["open", "in_progress", "closed", "blocked", "ready", "icebox", "archive"];
        for status in &statuses {
            let status_dir = self.get_status_dir(status);
            if status_dir.exists() {
                if let Ok(entries) = fs::read_dir(&status_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                                let ticket_id = filename.trim_end_matches(".md");
                                if let Ok(ticket) = self.load_ticket(ticket_id) {
                                    tickets.push(ticket);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort by creation date (newest first)
        tickets.sort_by(|a, b| b.created.cmp(&a.created));

        Ok(tickets)
    }

    pub fn search_tickets(&self, query: &str) -> Result<Vec<Ticket>> {
        let all_tickets = self.list_tickets()?;
        let query_lower = query.to_lowercase();

        let filtered_tickets: Vec<Ticket> = all_tickets.into_iter()
            .filter(|ticket| {
                ticket.title.to_lowercase().contains(&query_lower) ||
                ticket.description.as_ref().map(|d| d.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                ticket.id.to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(filtered_tickets)
    }

    pub fn migrate_tickets(&self, source: &str) -> Result<()> {
        self.ensure_status_directories()?;

        let migration_type = if source == "auto" {
            self.detect_migration_source()?
        } else {
            source.to_string()
        };

        match migration_type.as_str() {
            "bash-tk" => self.migrate_from_bash_tk()?,
            "beads" => self.migrate_from_beads()?,
            _ => anyhow::bail!("Unsupported migration source: {}. Use 'auto', 'bash-tk', or 'beads'", migration_type),
        }

        println!("Migration completed successfully from {}", migration_type);
        Ok(())
    }

    fn detect_migration_source(&self) -> Result<String> {
        // Check for bash tk format (flat .tickets directory with .md files)
        if self.tickets_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.tickets_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map(|ext| ext == "md").unwrap_or(false) {
                        // Check if it's bash tk format (no YAML frontmatter)
                        if let Ok(content) = fs::read_to_string(&path) {
                            if !content.starts_with("---") {
                                return Ok("bash-tk".to_string());
                            }
                        }
                    }
                }
            }
        }

        // Check for beads format (look for beads-specific patterns)
        // This would need to be implemented based on beads format specifics
        // For now, default to bash-tk if no YAML frontmatter found

        Ok("bash-tk".to_string())
    }

    fn migrate_from_bash_tk(&self) -> Result<()> {
        println!("Migrating from bash tk format...");

        if !self.tickets_dir.exists() {
            anyhow::bail!("No .tickets directory found");
        }

        let mut migrated_count = 0;

        for entry in fs::read_dir(&self.tickets_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map(|ext| ext == "md").unwrap_or(false) {
                // Read the bash tk format file
                let content = fs::read_to_string(&path)?;

                // Skip if already in new format
                if content.starts_with("---") {
                    continue;
                }

                // Parse bash tk format
                let ticket = self.parse_bash_tk_ticket(&content, &path)?;

                // Save in new format with status directory
                self.save_ticket(&ticket)?;

                // Remove old file
                fs::remove_file(&path)?;

                migrated_count += 1;
            }
        }

        println!("Migrated {} tickets from bash tk format", migrated_count);
        Ok(())
    }

    fn parse_bash_tk_ticket(&self, content: &str, path: &std::path::Path) -> Result<Ticket> {
        let lines: Vec<&str> = content.lines().collect();

        // Extract title (first non-empty line)
        let title = lines.iter()
            .find(|line| !line.trim().is_empty())
            .map_or("Untitled", |line| *line)
            .trim_start_matches('#')
            .trim();

        // Extract ID from filename
        let filename = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Parse metadata from bash tk format
        let mut status = "open".to_string();
        let mut description = None;
        let mut notes = Vec::new();
        let mut current_section = "description";

        for line in lines.iter().skip(1) {
            let line = line.trim();

            if line.starts_with("Status:") {
                status = line.replace("Status:", "").trim().to_lowercase();
                current_section = "description";
            } else if line.starts_with("Description:") {
                description = Some(line.replace("Description:", "").trim().to_string());
                current_section = "description";
            } else if line.starts_with("Notes:") {
                current_section = "notes";
            } else if !line.is_empty() {
                match current_section {
                    "notes" => {
                        // Parse note format: **YYYY-MM-DD HH:MM:SS**: content
                        if let Some(timestamp_start) = line.find("**") {
                            if let Some(timestamp_end) = line[timestamp_start + 2..].find("**") {
                                let timestamp_str = &line[timestamp_start + 2..timestamp_start + 2 + timestamp_end];
                                let note_content = line[timestamp_start + 2 + timestamp_end + 2..].trim_start_matches(':').trim();

                                if let Ok(timestamp) = DateTime::parse_from_str(&format!("{} +00:00", timestamp_str), "%Y-%m-%d %H:%M:%S %z") {
                                    notes.push(Note {
                                        timestamp: timestamp.with_timezone(&Utc),
                                        content: note_content.to_string(),
                                    });
                                }
                            }
                        }
                    }
                    "description" => {
                        if description.is_none() {
                            description = Some(line.to_string());
                        } else if let Some(ref mut desc) = description {
                            desc.push_str("\n");
                            desc.push_str(line);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(Ticket {
            id: filename.to_string(),
            title: title.to_string(),
            status,
            deps: Vec::new(),
            links: Vec::new(),
            created: Utc::now(), // Would need to extract from file metadata if available
            issue_type: "task".to_string(),
            priority: 2,
            description,
            design: None,
            acceptance: None,
            assignee: None,
            external_ref: None,
            parent: None,
            project: self.project.clone(),
            category: self.category.clone(),
            notes: if notes.is_empty() { None } else { Some(notes) },
        })
    }

    fn migrate_from_beads(&self) -> Result<()> {
        println!("Migrating from beads format...");
        // TODO: Implement beads format migration
        // This would need to be implemented based on the specific beads format
        anyhow::bail!("Beads format migration not yet implemented");
    }

    pub fn validate_status(&self, status: &str) -> Result<()> {
        let valid_statuses = ["open", "in_progress", "closed", "blocked", "ready", "icebox", "archive"];
        if !valid_statuses.contains(&status) {
            anyhow::bail!("Invalid status: {}. Valid statuses: {}",
                status, valid_statuses.join(", "));
        }
        Ok(())
    }

    pub fn create_ticket(&mut self, title: String, options: CreateOptions) -> Result<String> {
        let id = self.generate_id()?;
        let now = Utc::now();

        let ticket = Ticket {
            id: id.clone(),
            title: title.to_string(),
            status: "open".to_string(),
            deps: Vec::new(),
            links: Vec::new(),
            created: now,
            issue_type: options.issue_type.to_string(),
            priority: options.priority,
            description: options.description,
            design: options.design,
            acceptance: options.acceptance,
            assignee: options.assignee,
            external_ref: options.external_ref,
            parent: options.parent,
            project: self.project.clone(),
            category: self.category.clone(),
            notes: None,
        };

        self.save_ticket(&ticket)?;
        println!("{}", id);
        Ok(id)
    }

    #[allow(dead_code)]
    pub fn get_git_user(&self) -> Option<String> {
        std::process::Command::new("git")
            .args(["config", "user.name"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
    }

    pub fn update_status(&self, id: &str, status: &str) -> Result<()> {
        let mut ticket = self.load_ticket(id)?;
        self.validate_status(status)?;
        ticket.status = status.to_string();
        self.save_ticket(&ticket)?;
        println!("Updated {} -> {}", id, status);
        Ok(())
    }

    pub fn add_dependency(&self, id: &str, dep_id: &str) -> Result<()> {
        let mut ticket = self.load_ticket(id)?;
        if !ticket.deps.contains(&dep_id.to_string()) {
            ticket.deps.push(dep_id.to_string());
            self.save_ticket(&ticket)?;
            println!("Added dependency: {} -> {}", id, dep_id);
        } else {
            println!("Dependency already exists: {} -> {}", id, dep_id);
        }
        Ok(())
    }

    pub fn remove_dependency(&self, id: &str, dep_id: &str) -> Result<()> {
        let mut ticket = self.load_ticket(id)?;
        if let Some(pos) = ticket.deps.iter().position(|d| d == dep_id) {
            ticket.deps.remove(pos);
            self.save_ticket(&ticket)?;
            println!("Removed dependency: {} -> {}", id, dep_id);
        } else {
            println!("Dependency not found: {} -> {}", id, dep_id);
        }
        Ok(())
    }

    pub fn add_note(&self, id: &str, note_content: &str) -> Result<()> {
        let mut ticket = self.load_ticket(id)?;

        let note = Note {
            timestamp: Utc::now(),
            content: note_content.to_string(),
        };

        match &mut ticket.notes {
            Some(notes) => notes.push(note),
            None => ticket.notes = Some(vec![note]),
        }

        self.save_ticket(&ticket)?;
        println!("Note added to {}", id);
        Ok(())
    }

    pub fn show_ticket(&self, id: &str) -> Result<()> {
        let _ticket = self.load_ticket(id)?;
        let path = self.ticket_path(id)?;
        let content = fs::read_to_string(&path)?;

        println!("{}", content);
        Ok(())
    }
}

#[derive(Debug)]
pub struct CreateOptions {
    pub issue_type: String,
    pub priority: i32,
    pub description: Option<String>,
    pub design: Option<String>,
    pub acceptance: Option<String>,
    pub assignee: Option<String>,
    pub external_ref: Option<String>,
    pub parent: Option<String>,
}
