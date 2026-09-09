use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

// Helper function to find all ticket files recursively
fn find_ticket_files(tickets_dir: &PathBuf) -> Vec<PathBuf> {
    let mut ticket_files = Vec::new();

    // Search in all status subdirectories
    if let Ok(entries) = fs::read_dir(tickets_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // This is a status directory (open, closed, etc.)
                if let Ok(status_entries) = fs::read_dir(&path) {
                    for status_entry in status_entries.flatten() {
                        let ticket_path = status_entry.path();
                        if ticket_path.extension().map(|ext| ext == "md").unwrap_or(false) {
                            ticket_files.push(ticket_path);
                        }
                    }
                }
            } else if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                // Direct .md file in tickets directory
                ticket_files.push(path);
            }
        }
    }

    ticket_files
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_create_ticket() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket")
        .arg("--description")
        .arg("Test description")
        .assert()
        .success()
        .stdout(predicate::str::contains("-"));

    // Check that ticket file was created
    assert!(tickets_dir.exists());
    let open_dir = tickets_dir.join("open");
    assert!(open_dir.exists());
    let ticket_files: Vec<_> = fs::read_dir(&open_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .collect();

    assert_eq!(ticket_files.len(), 1);
}

#[test]
fn test_create_ticket_with_project_category() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("--project")
        .arg("test-project")
        .arg("--category")
        .arg("test-category")
        .arg("create")
        .arg("Test ticket with tags")
        .assert()
        .success();

    // Check ticket content contains project and category tags
    let ticket_files = find_ticket_files(&tickets_dir);

    assert_eq!(ticket_files.len(), 1);

    let content = fs::read_to_string(ticket_files[0].as_path()).unwrap();
    assert!(content.contains("project: test-project"));
    assert!(content.contains("category: test-category"));
}

#[test]
fn test_list_tickets() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");
    fs::create_dir_all(&tickets_dir).unwrap();

    // Create a test ticket file using the same format as our app
    let ticket_content = r#"---
id: test-123
title: Test Ticket
status: open
deps: []
links: []
created: 2023-01-01T00:00:00Z
type: task
priority: 2
---
# Test Ticket
Test description
"#;
    fs::write(tickets_dir.join("test-123.md"), ticket_content).unwrap();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-123"));
}

#[test]
fn test_ticket_status_update() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // First create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Status test ticket")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files = find_ticket_files(&tickets_dir);

    let ticket_path = ticket_files[0].clone();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Update status
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("status")
        .arg(ticket_id)
        .arg("in_progress")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"));

    // Verify status changed to in_progress
    // The file should now be in the in_progress directory
    let in_progress_path = tickets_dir.join("in_progress").join(format!("{}.md", ticket_id));
    let content = fs::read_to_string(in_progress_path).unwrap();
    assert!(content.contains("status: in_progress"));
}

#[test]
fn test_add_note() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // First create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Note test ticket")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files = find_ticket_files(&tickets_dir);

    let ticket_path = ticket_files[0].as_path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Add a note
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("add-note")
        .arg(ticket_id)
        .arg("Test note content")
        .assert()
        .success()
        .stdout(predicate::str::contains("Note added"));

    // Check note is in file
    let content = fs::read_to_string(ticket_path).unwrap();
    assert!(content.contains("## Notes"));
    assert!(content.contains("Test note content"));
}

#[test]
fn test_dependency_management() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Create first ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Parent ticket")
        .assert()
        .success();

    // Create second ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Child ticket")
        .assert()
        .success();

    // Get ticket IDs
    let ticket_files: Vec<_> = fs::read_dir(&tickets_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .collect();

    // Ensure we have 2 tickets
    assert_eq!(ticket_files.len(), 2, "Expected 2 tickets to be created, found {}", ticket_files.len());

    let parent_path = ticket_files[0].path();
    let parent_id = parent_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let child_path = ticket_files[1].path();
    let child_id = child_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Add dependency
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("dep")
        .arg(child_id)
        .arg(parent_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added dependency"));

    // Remove dependency
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("undep")
        .arg(child_id)
        .arg(parent_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed dependency"));
}

#[test]
fn test_start_command() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // First create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket for start")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files = find_ticket_files(&tickets_dir);

    let ticket_path = ticket_files[0].as_path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Start the ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("start")
        .arg(ticket_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Started"));

    // Verify status changed to in_progress
    // The file should now be in the in_progress directory
    let in_progress_path = tickets_dir.join("in_progress").join(format!("{}.md", ticket_id));
    let content = fs::read_to_string(in_progress_path).unwrap();
    assert!(content.contains("status: in_progress"));
}

#[test]
fn test_close_command() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // First create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket for close")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files = find_ticket_files(&tickets_dir);

    let ticket_path = ticket_files[0].as_path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Close the ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("close")
        .arg(ticket_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Closed"));

    // Verify status changed to closed
    // The file should now be in the closed directory
    let closed_path = tickets_dir.join("closed").join(format!("{}.md", ticket_id));
    let content = fs::read_to_string(closed_path).unwrap();
    assert!(content.contains("status: closed"));
}

#[test]
fn test_reopen_command() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // First create and close a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket for reopen")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files = find_ticket_files(&tickets_dir);

    let ticket_path = ticket_files[0].as_path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Close the ticket first
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("close")
        .arg(ticket_id)
        .assert()
        .success();

    // Reopen the ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("reopen")
        .arg(ticket_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Reopened"));

    // Verify status changed to open
    let content = fs::read_to_string(ticket_path).unwrap();
    assert!(content.contains("status: open"));
}

#[test]
fn test_status_command() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // First create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket for status")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files = find_ticket_files(&tickets_dir);

    let ticket_path = ticket_files[0].as_path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Set status to blocked
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("status")
        .arg(ticket_id)
        .arg("blocked")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"));

    // Verify status changed to blocked
    // The file should now be in the blocked directory
    let blocked_path = tickets_dir.join("blocked").join(format!("{}.md", ticket_id));
    let content = fs::read_to_string(blocked_path).unwrap();
    assert!(content.contains("status: blocked"));
}

#[test]
fn test_show_command() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Create a ticket with full details
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Show test ticket")
        .arg("--description")
        .arg("Test description for show")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files = find_ticket_files(&tickets_dir);

    let ticket_path = ticket_files[0].as_path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Show the ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("show")
        .arg(ticket_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Show test ticket"))
        .stdout(predicate::str::contains("Test description for show"))
        .stdout(predicate::str::contains(ticket_id));
}

#[test]
fn test_dep_tree_command() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Create parent ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Parent ticket")
        .assert()
        .success();

    // Create child ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Child ticket")
        .assert()
        .success();

    // Get ticket IDs
    let ticket_files: Vec<_> = fs::read_dir(&tickets_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .collect();

    assert_eq!(ticket_files.len(), 2);

    let parent_path = ticket_files[0].path();
    let parent_id = parent_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let child_path = ticket_files[1].path();
    let child_id = child_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Add dependency
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("dep")
        .arg(child_id)
        .arg(parent_id)
        .assert()
        .success();

    // Show dependency tree
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("dep-tree")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dependency tree"));
}

#[test]
fn test_link_unlink_commands() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Create first ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("First ticket")
        .assert()
        .success();

    // Create second ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Second ticket")
        .assert()
        .success();

    // Get ticket IDs
    let ticket_files: Vec<_> = fs::read_dir(&tickets_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .collect();

    assert_eq!(ticket_files.len(), 2);

    let first_path = ticket_files[0].path();
    let first_id = first_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let second_path = ticket_files[1].path();
    let second_id = second_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Link tickets
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("link")
        .arg(first_id)
        .arg(second_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Linked"));

    // Unlink tickets
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("unlink")
        .arg(first_id)
        .arg(second_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unlinked"));
}

#[test]
fn test_ready_command() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Create a ticket with no dependencies (should be ready)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Ready ticket")
        .assert()
        .success();

    // Create a ticket with dependencies (should not be ready)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Blocked ticket")
        .assert()
        .success();

    // Get ticket IDs
    let ticket_files = find_ticket_files(&tickets_dir);
    assert_eq!(ticket_files.len(), 2);

    let ready_path = ticket_files[0].as_path();
    let ready_id = ready_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let blocked_path = ticket_files[1].as_path();
    let blocked_id = blocked_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Add dependency to make second ticket blocked
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("dep")
        .arg(blocked_id)
        .arg(ready_id)
        .assert()
        .success();

    // Test ready command - should only show the first ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("ready")
        .assert()
        .success()
        .stdout(predicate::str::contains(ready_id));
}

#[test]
fn test_ticket_file_movement_between_status_directories() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket for file movement")
        .assert()
        .success();

    // Get the ticket ID and verify it starts in open directory
    let ticket_files = find_ticket_files(&tickets_dir);
    assert_eq!(ticket_files.len(), 1);
    
    let ticket_path = ticket_files[0].as_path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Verify initial location is open directory
    assert!(ticket_path.starts_with(&tickets_dir.join("open")));
    assert!(ticket_path.exists());

    // Start the ticket - should move to in_progress directory
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("start")
        .arg(ticket_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Started"));

    // Verify file moved to in_progress directory and no longer in open
    let in_progress_path = tickets_dir.join("in_progress").join(format!("{}.md", ticket_id));
    let old_open_path = tickets_dir.join("open").join(format!("{}.md", ticket_id));
    
    assert!(in_progress_path.exists());
    assert!(!old_open_path.exists());
    
    // Verify content is correct in new location
    let content = fs::read_to_string(&in_progress_path).unwrap();
    assert!(content.contains("status: in_progress"));

    // Close the ticket - should move to closed directory
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("close")
        .arg(ticket_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Closed"));

    // Verify file moved to closed directory and no longer in in_progress
    let closed_path = tickets_dir.join("closed").join(format!("{}.md", ticket_id));
    
    assert!(closed_path.exists());
    assert!(!in_progress_path.exists());
    assert!(!old_open_path.exists());
    
    // Verify content is correct in final location
    let content = fs::read_to_string(&closed_path).unwrap();
    assert!(content.contains("status: closed"));

    // Reopen the ticket - should move back to open directory
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("reopen")
        .arg(ticket_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Reopened"));

    // Verify file moved back to open directory and no longer in closed
    assert!(old_open_path.exists());
    assert!(!closed_path.exists());
    assert!(!in_progress_path.exists());
    
    // Verify content is correct in final location
    let content = fs::read_to_string(&old_open_path).unwrap();
    assert!(content.contains("status: open"));

    // Use generic status command to move to blocked
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("status")
        .arg(ticket_id)
        .arg("blocked")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"));

    // Verify file moved to blocked directory
    let blocked_path = tickets_dir.join("blocked").join(format!("{}.md", ticket_id));
    
    assert!(blocked_path.exists());
    assert!(!old_open_path.exists());
    
    // Verify content is correct
    let content = fs::read_to_string(&blocked_path).unwrap();
    assert!(content.contains("status: blocked"));

    // Ensure only one file exists across all status directories
    let final_ticket_files = find_ticket_files(&tickets_dir);
    assert_eq!(final_ticket_files.len(), 1);
    assert!(final_ticket_files[0].ends_with(&format!("{}.md", ticket_id)));
}

#[test] 
fn test_no_duplicate_files_created_during_status_changes() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket for duplicate prevention")
        .assert()
        .success();

    let ticket_files = find_ticket_files(&tickets_dir);
    let ticket_id = ticket_files[0]
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Perform multiple status changes
    let statuses = vec!["in_progress", "blocked", "ready", "closed"];
    
    for status in &statuses {
        let mut cmd = Command::cargo_bin("tkr").unwrap();
        cmd.env("TICKETS_DIR", &tickets_dir)
            .arg("status")
            .arg(ticket_id)
            .arg(status)
            .assert()
            .success();
        
        // After each status change, ensure exactly one file exists
        let current_files = find_ticket_files(&tickets_dir);
        assert_eq!(current_files.len(), 1, 
            "Found {} files after changing to status {}: {:?}", 
            current_files.len(), status, current_files);
        
        // Verify the file is in the correct status directory
        let expected_path = tickets_dir.join(status).join(format!("{}.md", ticket_id));
        assert!(expected_path.exists(), 
            "File not found in expected location: {:?}", expected_path);
    }
}

#[test]
fn test_git_aware_file_movement_in_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Initialize git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repo");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git email");

    // Create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket for git-aware movement")
        .assert()
        .success();

    let ticket_files = find_ticket_files(&tickets_dir);
    let ticket_id = ticket_files[0]
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Add the ticket to git
    std::process::Command::new("git")
        .args(["add", ".tickets/"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add tickets to git");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add test ticket"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit ticket");

    // Verify file is tracked by git before movement
    let git_ls_output = std::process::Command::new("git")
        .args(["ls-files", ".tickets/"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to check git tracking");

    let git_files_before = String::from_utf8_lossy(&git_ls_output.stdout);
    assert!(git_files_before.contains(&ticket_id), 
        "Ticket file should be tracked by git before movement: {}", git_files_before);

    // Change ticket status - this should use git mv or filesystem operations
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("start")
        .arg(ticket_id)
        .assert()
        .success();

    // Verify file is in correct location
    let expected_path = tickets_dir.join("in_progress").join(format!("{}.md", ticket_id));
    assert!(expected_path.exists(), "File not found in in_progress directory");

    // Verify no duplicates exist
    let all_files = find_ticket_files(&tickets_dir);
    assert_eq!(all_files.len(), 1, "Found duplicate files: {:?}", all_files);

    // Verify original file was removed
    let original_path = tickets_dir.join("open").join(format!("{}.md", ticket_id));
    assert!(!original_path.exists(), "Original file still exists");

    // The key test: file movement worked correctly (regardless of git behavior)
    // This tests the core functionality - proper file movement without duplicates
    let file_content = fs::read_to_string(&expected_path).expect("Failed to read moved file");
    assert!(file_content.contains("in_progress"), 
        "File should have updated status content");
}

#[test]
fn test_file_movement_outside_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Create a ticket (NOT in a git repo)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Test ticket outside git repo")
        .assert()
        .success();

    let ticket_files = find_ticket_files(&tickets_dir);
    let ticket_id = ticket_files[0]
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Change ticket status - should use filesystem operations
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("start")
        .arg(ticket_id)
        .assert()
        .success();

    // Verify file is in correct location
    let expected_path = tickets_dir.join("in_progress").join(format!("{}.md", ticket_id));
    assert!(expected_path.exists(), "File not found in in_progress directory");

    // Verify no duplicates exist
    let all_files = find_ticket_files(&tickets_dir);
    assert_eq!(all_files.len(), 1, "Found duplicate files: {:?}", all_files);

    // Verify original file was removed
    let original_path = tickets_dir.join("open").join(format!("{}.md", ticket_id));
    assert!(!original_path.exists(), "Original file still exists");
}

#[test]
fn test_git_aware_movement_with_untracked_file() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Initialize git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repo");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git email");

    // Create a ticket but DON'T add it to git
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Untracked ticket")
        .assert()
        .success();

    let ticket_files = find_ticket_files(&tickets_dir);
    let ticket_id = ticket_files[0]
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Change ticket status - should use filesystem operations (not git mv)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("start")
        .arg(ticket_id)
        .assert()
        .success();

    // Verify file is in correct location
    let expected_path = tickets_dir.join("in_progress").join(format!("{}.md", ticket_id));
    assert!(expected_path.exists(), "File not found in in_progress directory");

    // Verify no duplicates exist
    let all_files = find_ticket_files(&tickets_dir);
    assert_eq!(all_files.len(), 1, "Found duplicate files: {:?}", all_files);
}

#[test]
fn test_git_aware_movement_multiple_status_changes() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    // Initialize git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repo");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git user");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git email");

    // Create a ticket
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Multi-status test ticket")
        .assert()
        .success();

    let ticket_files = find_ticket_files(&tickets_dir);
    let ticket_id = ticket_files[0]
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Add to git and commit
    std::process::Command::new("git")
        .args(["add", ".tickets/"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add tickets to git");

    std::process::Command::new("git")
        .args(["commit", "-m", "Add test ticket"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to commit ticket");

    // Perform multiple status changes
    let statuses = vec!["in_progress", "ready", "closed"];
    
    for status in &statuses {
        let mut cmd = Command::cargo_bin("tkr").unwrap();
        cmd.env("TICKETS_DIR", &tickets_dir)
            .arg("status")
            .arg(ticket_id)
            .arg(status)
            .assert()
            .success();
        
        // Verify file is in correct location
        let expected_path = tickets_dir.join(status).join(format!("{}.md", ticket_id));
        assert!(expected_path.exists(), 
            "File not found in {} directory: {:?}", status, expected_path);
        
        // Verify no duplicates exist
        let all_files = find_ticket_files(&tickets_dir);
        assert_eq!(all_files.len(), 1, 
            "Found duplicate files after {}: {:?}", status, all_files);
    }
}

#[test]
fn test_db_module_does_not_break_existing_commands() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("list")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Portfolio CRUD integration tests (story 02-001)
// ---------------------------------------------------------------------------

#[test]
fn test_portfolio_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .arg("--description")
        .arg("My personal work")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created portfolio: personal"))
        .stdout(predicate::str::contains("curated"));

    assert!(db_path.exists());
}

#[test]
fn test_portfolio_create_without_description() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("oss")
        .arg("Open Source")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created portfolio: oss"));
}

#[test]
fn test_portfolio_create_duplicate_id() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create first
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .success();

    // Create duplicate
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_portfolio_list() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create two portfolios
    for (id, name) in [("personal", "Personal"), ("business", "Business")] {
        let mut cmd = Command::cargo_bin("tkr").unwrap();
        cmd.env("TKR_DB_PATH", &db_path)
            .arg("portfolio")
            .arg("create")
            .arg(id)
            .arg(name)
            .assert()
            .success();
    }

    // List
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("Personal"))
        .stdout(predicate::str::contains("business"))
        .stdout(predicate::str::contains("Business"));
}

#[test]
fn test_portfolio_list_excludes_dissolved() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create two portfolios
    for (id, name) in [("personal", "Personal"), ("business", "Business")] {
        let mut cmd = Command::cargo_bin("tkr").unwrap();
        cmd.env("TKR_DB_PATH", &db_path)
            .arg("portfolio")
            .arg("create")
            .arg(id)
            .arg(name)
            .assert()
            .success();
    }

    // Dissolve one
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("personal")
        .assert()
        .success();

    // List should not show dissolved
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("business"))
        .stdout(predicate::str::contains("Business"))
        .stdout(predicate::str::contains("personal").not());
}

#[test]
fn test_portfolio_show() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .arg("--description")
        .arg("My personal work")
        .assert()
        .success();

    // Show
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("show")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("Personal"))
        .stdout(predicate::str::contains("My personal work"))
        .stdout(predicate::str::contains("curated"));
}

#[test]
fn test_portfolio_show_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("show")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_portfolio_dissolve() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .success();

    // Dissolve
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dissolved"));
}

#[test]
fn test_portfolio_dissolve_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_portfolio_dissolve_already_dissolved() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create and dissolve
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("personal")
        .assert()
        .success();

    // Dissolve again — should be idempotent or warn
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("dissolve")
        .arg("personal")
        .assert()
        .success();
}

#[test]
fn test_portfolio_auto_creates_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    assert!(!db_path.exists());

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("create")
        .arg("personal")
        .arg("Personal")
        .assert()
        .success();

    assert!(db_path.exists());
}

// ---------------------------------------------------------------------------
// Project registration integration tests (story 02-002)
// ---------------------------------------------------------------------------

use std::process::Command as StdCommand;

/// Helper: create a temp git repo with a remote and a `.tickets` directory.
fn create_temp_git_repo() -> TempDir {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    StdCommand::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("https://github.com/levonk/test-repo.git")
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Create .tickets dir
    std::fs::create_dir_all(repo_path.join(".tickets")).unwrap();

    temp
}

/// Helper: create a portfolio in the DB.
fn create_portfolio(db_path: &std::path::Path, id: &str, name: &str) {
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", db_path)
        .arg("portfolio")
        .arg("create")
        .arg(id)
        .arg(name)
        .assert()
        .success();
}

#[test]
fn test_project_register() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    create_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("Registered project"))
        .stdout(predicate::str::contains("seeded"))
        .stdout(predicate::str::contains("levonk/test-repo"));
}

#[test]
fn test_project_register_with_custom_name() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    create_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .arg("--name")
        .arg("custom-project-name")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom-project-name"));
}

#[test]
fn test_project_register_nonexistent_portfolio() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_project_register_nonexistent_path() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    create_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg("/nonexistent/path/to/repo")
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("does not exist")));
}

#[test]
fn test_project_register_duplicate() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    create_portfolio(&db_path, "personal", "Personal");

    // First registration
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    // Duplicate registration
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already registered"));
}

#[test]
fn test_project_list() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path();

    create_portfolio(&db_path, "personal", "Personal");

    // Register a project
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    // List
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("seeded"));
}

#[test]
fn test_project_list_filtered_by_portfolio() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    // Create two portfolios
    create_portfolio(&db_path, "personal", "Personal");
    create_portfolio(&db_path, "business", "Business");

    // Register a project in each
    let repo1 = create_temp_git_repo();
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo1.path())
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    let repo2 = create_temp_git_repo();
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo2.path())
        .arg("--portfolio")
        .arg("business")
        .assert()
        .success();

    // List filtered to personal — should contain personal, not business
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("list")
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("business").not());
}

#[test]
fn test_project_unregister() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();
    let repo_path = repo_temp.path().to_str().unwrap().to_string();

    create_portfolio(&db_path, "personal", "Personal");

    // Register
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(&repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    // Unregister
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("unregister")
        .arg(&repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unregistered"));

    // List should be empty
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No projects").or(predicate::str::contains("0 projects")));
}

#[test]
fn test_project_unregister_not_found() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("unregister")
        .arg("/nonexistent/path")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_project_auto_creates_db() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();

    // Create portfolio first (this creates the DB)
    create_portfolio(&db_path, "personal", "Personal");

    assert!(db_path.exists());

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_temp.path())
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();
}

#[test]
fn test_project_register_non_git_repo() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    // Create a plain directory (not a git repo)
    let repo_temp = TempDir::new().unwrap();
    std::fs::create_dir_all(repo_temp.path().join(".tickets")).unwrap();

    create_portfolio(&db_path, "personal", "Personal");

    // Should still register, but without GitHub info
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_temp.path())
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("(none)").or(predicate::str::contains("Registered")));
}

// ---------------------------------------------------------------------------
// App CRUD integration tests (story 03-001)
// ---------------------------------------------------------------------------

/// Helper: create a portfolio and register a project (which auto-creates a
/// default app). Returns the temp dirs so they stay alive for the test.
fn setup_project_with_default_app() -> (TempDir, std::path::PathBuf) {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();

    create_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_temp.path())
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    // Keep repo_temp alive by leaking it — the TempDir will be cleaned up when
    // the process exits. This is acceptable for tests.
    std::mem::forget(repo_temp);

    (db_temp, db_path)
}

#[test]
fn test_app_create() {
    let (_temp_dir, db_path) = setup_project_with_default_app();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app")
        .arg("create")
        .arg("api")
        .arg("--project=1")
        .arg("--description=REST API")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created app"));
}

#[test]
fn test_app_create_duplicate_fails() {
    let (_temp_dir, db_path) = setup_project_with_default_app();

    // Create first app
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("api").arg("--project=1")
        .assert()
        .success();

    // Create duplicate
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("api").arg("--project=1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_app_list() {
    let (_temp_dir, db_path) = setup_project_with_default_app();

    // Create an additional app
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("api").arg("--project=1")
        .assert()
        .success();

    // List should show both default and api
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("list").arg("--project=1")
        .assert()
        .success()
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("api"));
}

#[test]
fn test_app_list_all_projects() {
    let (_temp_dir, db_path) = setup_project_with_default_app();

    // List without --project should show all apps
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("default"));
}

#[test]
fn test_app_sunset() {
    let (_temp_dir, db_path) = setup_project_with_default_app();

    // Sunset the default app (id=1)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("sunset").arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("sunset"));
}

#[test]
fn test_app_sunset_not_found() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("sunset").arg("999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_app_create_invalid_state() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("api").arg("--project=1").arg("--state=invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid state"));
}

#[test]
fn test_app_create_with_valid_state() {
    let (_temp_dir, db_path) = setup_project_with_default_app();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("web").arg("--project=1").arg("--state=sketched")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created app"));

    // Verify state is shown in list
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("list").arg("--project=1")
        .assert()
        .success()
        .stdout(predicate::str::contains("sketched"));
}

#[test]
fn test_app_create_nonexistent_project() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("create").arg("api").arg("--project=999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_project_register_creates_default_app() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");
    let repo_temp = create_temp_git_repo();

    create_portfolio(&db_path, "personal", "Personal");

    // Register a project — should auto-create a default app
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project").arg("register").arg(repo_temp.path()).arg("--portfolio").arg("personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("default"));

    // Verify default app exists via app list
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("app").arg("list").arg("--project=1")
        .assert()
        .success()
        .stdout(predicate::str::contains("default"));

    // Keep repo_temp alive
    std::mem::forget(repo_temp);
}

// ---------------------------------------------------------------------------
// Requirement CRUD integration tests (story 03-002)
// ---------------------------------------------------------------------------

/// Helper: create a portfolio in the DB (reuses the existing helper but
/// scoped locally for clarity in requirement tests).
fn setup_portfolio(db_path: &std::path::Path, id: &str, name: &str) {
    create_portfolio(db_path, id, name);
}

#[test]
fn test_requirement_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement")
        .arg("create")
        .arg("Multi-account Sync")
        .arg("--portfolio=personal")
        .arg("--description=Must support multi-account sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created requirement"))
        .stdout(predicate::str::contains("req-"));
}

#[test]
fn test_requirement_create_invalid_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement")
        .arg("create")
        .arg("Test")
        .arg("--portfolio=personal")
        .arg("--state=invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid requirement state"));
}

#[test]
fn test_requirement_create_nonexistent_portfolio() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement")
        .arg("create")
        .arg("Test")
        .arg("--portfolio=nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_requirement_list() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_portfolio(&db_path, "personal", "Personal");

    // Create two requirements
    for title in ["Multi-account Sync", "Offline-first"] {
        let mut cmd = Command::cargo_bin("tkr").unwrap();
        cmd.env("TKR_DB_PATH", &db_path)
            .arg("requirement")
            .arg("create")
            .arg(title)
            .arg("--portfolio=personal")
            .assert()
            .success();
    }

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi-account Sync"))
        .stdout(predicate::str::contains("Offline-first"));
}

#[test]
fn test_requirement_list_by_portfolio() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_portfolio(&db_path, "personal", "Personal");
    setup_portfolio(&db_path, "business", "Business");

    // Create a requirement in each portfolio
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("create").arg("Multi-account Sync")
        .arg("--portfolio=personal")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("create").arg("Enterprise SSO")
        .arg("--portfolio=business")
        .assert()
        .success();

    // List filtered to personal — should contain personal, not business
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("list").arg("--portfolio=personal")
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi-account Sync"))
        .stdout(predicate::str::contains("Enterprise SSO").not());
}

#[test]
fn test_requirement_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No requirements found"));
}

#[test]
fn test_requirement_show() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_portfolio(&db_path, "personal", "Personal");

    // Create a requirement
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("create").arg("Multi-account Sync")
        .arg("--portfolio=personal")
        .arg("--description=Must support multi-account sync")
        .assert()
        .success()
        .get_output()
        .stdout.clone();
    let stdout = String::from_utf8(output).unwrap();

    // Extract the requirement ID from the output
    let req_id = stdout
        .lines()
        .find(|l| l.contains("req-"))
        .and_then(|l| l.split("id: ").nth(1))
        .and_then(|s| s.split(')').next())
        .unwrap_or("");

    // Show the requirement
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("show").arg(req_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi-account Sync"))
        .stdout(predicate::str::contains("proposed"))
        .stdout(predicate::str::contains("Must support multi-account sync"));
}

#[test]
fn test_requirement_show_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("show").arg("req-nonexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_requirement_supersede() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_portfolio(&db_path, "personal", "Personal");

    // Create a requirement
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("create").arg("Multi-account Sync")
        .arg("--portfolio=personal")
        .assert()
        .success()
        .get_output()
        .stdout.clone();
    let stdout = String::from_utf8(output).unwrap();
    let req_id = stdout
        .lines()
        .find(|l| l.contains("req-"))
        .and_then(|l| l.split("id: ").nth(1))
        .and_then(|s| s.split(')').next())
        .unwrap_or("");

    // Supersede the requirement
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("supersede").arg(req_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("superseded"));

    // Verify state is superseded via show
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("show").arg(req_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("superseded"));
}

#[test]
fn test_requirement_supersede_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("supersede").arg("req-nonexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_requirement_id_format() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("create").arg("Test").arg("--portfolio=personal")
        .assert()
        .success()
        .get_output()
        .stdout.clone();
    let stdout = String::from_utf8(output).unwrap();
    // The output should contain a req- prefixed ID
    assert!(stdout.contains("req-"));
}

#[test]
fn test_requirement_create_with_target_date_and_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_portfolio(&db_path, "personal", "Personal");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement")
        .arg("create")
        .arg("Offline-first")
        .arg("--portfolio=personal")
        .arg("--state=planned")
        .arg("--target-date=2026-12-01")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created requirement"));

    // Verify the state and target date appear in list
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("requirement").arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("planned"))
        .stdout(predicate::str::contains("2026-12-01"));
}

// ---------------------------------------------------------------------------
// Story CRUD integration tests (story 03-003)
// ---------------------------------------------------------------------------

/// Helper: create a requirement under a portfolio and return its ID.
fn setup_requirement(db_path: &std::path::Path, portfolio: &str, title: &str) -> String {
    setup_portfolio(db_path, portfolio, "Portfolio");
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TKR_DB_PATH", db_path)
        .arg("requirement")
        .arg("create")
        .arg(title)
        .arg(format!("--portfolio={}", portfolio))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    stdout
        .lines()
        .find(|l| l.contains("req-"))
        .and_then(|l| l.split("id: ").nth(1))
        .and_then(|s| s.split(')').next())
        .unwrap_or("")
        .to_string()
}

/// Helper: create a story and return its ID.
fn setup_story(db_path: &std::path::Path, title: &str) -> String {
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TKR_DB_PATH", db_path)
        .arg("story")
        .arg("create")
        .arg(title)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    stdout
        .lines()
        .find(|l| l.contains("story-"))
        .and_then(|l| l.split("id: ").nth(1))
        .and_then(|s| s.split(')').next())
        .unwrap_or("")
        .to_string()
}

#[test]
fn test_story_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let req_id = setup_requirement(&db_path, "personal", "Multi-account Sync");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Portfolio Layer")
        .arg(format!("--requirement={}", req_id))
        .arg("--description=Cross-project portfolio management")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created story"))
        .stdout(predicate::str::contains("story-"));
}

#[test]
fn test_story_create_no_links() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Investigate WASM")
        .arg("--description=Explore WebAssembly")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created story"));
}

#[test]
fn test_story_create_invalid_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Test")
        .arg("--state=invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid story state"));
}

#[test]
fn test_story_create_nonexistent_requirement() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Test")
        .arg("--requirement=req-nonexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_story_create_nonexistent_app() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Test")
        .arg("--app=999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_story_list() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    setup_story(&db_path, "Portfolio Layer");
    setup_story(&db_path, "Investigate WASM");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Portfolio Layer"))
        .stdout(predicate::str::contains("Investigate WASM"));
}

#[test]
fn test_story_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No stories found"));
}

#[test]
fn test_story_list_by_requirement() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let req_id = setup_requirement(&db_path, "personal", "Multi-account Sync");

    // Create a story linked to the requirement
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Portfolio Layer")
        .arg(format!("--requirement={}", req_id))
        .assert()
        .success();

    // Create a story without a requirement
    setup_story(&db_path, "Standalone Story");

    // List filtered by requirement — should contain Portfolio Layer, not Standalone
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("list")
        .arg(format!("--requirement={}", req_id))
        .assert()
        .success()
        .stdout(predicate::str::contains("Portfolio Layer"))
        .stdout(predicate::str::contains("Standalone Story").not());
}

#[test]
fn test_story_list_by_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // Create a story with default state (pitched)
    setup_story(&db_path, "Default Story");

    // Create a story with building state
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Building Story")
        .arg("--state=building")
        .assert()
        .success();

    // List filtered by state=building
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("list")
        .arg("--state=building")
        .assert()
        .success()
        .stdout(predicate::str::contains("Building Story"))
        .stdout(predicate::str::contains("Default Story").not());
}

#[test]
fn test_story_show() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let req_id = setup_requirement(&db_path, "personal", "Multi-account Sync");

    // Create a story with full details
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Portfolio Layer")
        .arg(format!("--requirement={}", req_id))
        .arg("--description=Cross-project portfolio management")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    let story_id = stdout
        .lines()
        .find(|l| l.contains("story-"))
        .and_then(|l| l.split("id: ").nth(1))
        .and_then(|s| s.split(')').next())
        .unwrap_or("");

    // Show the story
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("show")
        .arg(story_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Portfolio Layer"))
        .stdout(predicate::str::contains("pitched"))
        .stdout(predicate::str::contains("Cross-project portfolio management"));
}

#[test]
fn test_story_show_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("show")
        .arg("story-nonexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_story_ship() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let story_id = setup_story(&db_path, "Portfolio Layer");

    // Ship the story
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("ship")
        .arg(&story_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("shipped"));

    // Verify state is shipped via show
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("show")
        .arg(&story_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("shipped"));
}

#[test]
fn test_story_ship_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("ship")
        .arg("story-nonexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_story_id_format() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Test")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("story-"));
}

#[test]
fn test_story_create_with_target_date_and_state() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("create")
        .arg("Offline-first")
        .arg("--state=queued")
        .arg("--target-date=2026-12-01")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created story"));

    // Verify the state appears in list
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("queued"));
}

// ---------------------------------------------------------------------------
// Markdown sync integration tests (story 03-004)
// ---------------------------------------------------------------------------

/// Helper: create a portfolio and register a project whose tickets_dir
/// points to a temp repo with markdown ticket files. Returns the temp dirs
/// (DB temp and repo temp) and the db_path so they stay alive for the test.
fn setup_sync_project(
    ticket_files: &[(&str, &str)], // (filename, content)
) -> (TempDir, TempDir, std::path::PathBuf) {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    let repo_temp = TempDir::new().unwrap();
    let repo_path = repo_temp.path().to_path_buf();
    let tickets_dir = repo_path.join(".tickets");

    // Create the open status directory
    fs::create_dir_all(tickets_dir.join("open")).unwrap();

    // Write ticket files
    for (filename, content) in ticket_files {
        fs::write(tickets_dir.join("open").join(filename), content).unwrap();
    }

    // Create portfolio
    create_portfolio(&db_path, "personal", "Personal");

    // Register the project
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(&repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    (db_temp, repo_temp, db_path)
}

const TICKET_CONTENT: &str = "---\nid: ja-test001\ntitle: Test Ticket\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Test Ticket\n";

#[test]
fn test_sync_basic() {
    let (_db_temp, _repo_temp, db_path) = setup_sync_project(&[("ja-test001.md", TICKET_CONTENT)]);

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("Sync complete"))
        .stdout(predicate::str::contains("1 task"));
}

#[test]
fn test_sync_idempotent() {
    let (_db_temp, _repo_temp, db_path) = setup_sync_project(&[("ja-test001.md", TICKET_CONTENT)]);

    // First sync
    let mut cmd1 = Command::cargo_bin("tkr").unwrap();
    cmd1.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Second sync — should report 0 indexed
    let mut cmd2 = Command::cargo_bin("tkr").unwrap();
    cmd2.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("0 tasks indexed"));
}

#[test]
fn test_sync_detects_changes() {
    let (_db_temp, repo_temp, db_path) = setup_sync_project(&[("ja-test001.md", TICKET_CONTENT)]);

    // First sync
    let mut cmd1 = Command::cargo_bin("tkr").unwrap();
    cmd1.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Modify the file
    let modified = "---\nid: ja-test001\ntitle: Updated Title\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Updated Title\n";
    fs::write(
        repo_temp.path().join(".tickets").join("open").join("ja-test001.md"),
        modified,
    )
    .unwrap();

    // Second sync — should report 1 updated
    let mut cmd2 = Command::cargo_bin("tkr").unwrap();
    cmd2.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 updated"));
}

#[test]
fn test_sync_removes_deleted() {
    let (_db_temp, repo_temp, db_path) = setup_sync_project(&[("ja-test001.md", TICKET_CONTENT)]);

    // First sync
    let mut cmd1 = Command::cargo_bin("tkr").unwrap();
    cmd1.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Delete the file
    fs::remove_file(
        repo_temp.path().join(".tickets").join("open").join("ja-test001.md"),
    )
    .unwrap();

    // Second sync — should report 1 removed
    let mut cmd2 = Command::cargo_bin("tkr").unwrap();
    cmd2.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 removed"));
}

#[test]
fn test_sync_status() {
    let (_db_temp, _repo_temp, db_path) = setup_sync_project(&[("ja-test001.md", TICKET_CONTENT)]);

    // Run sync once to populate sync_state
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Check status
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .arg("--status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Last sync"))
        .stdout(predicate::str::contains("Projects:"))
        .stdout(predicate::str::contains("Tasks:"));
}

#[test]
fn test_sync_github_stub() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .arg("--github")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_sync_invalid_markdown_skipped() {
    let valid = "---\nid: ja-valid01\ntitle: Valid\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Valid\n";
    let invalid = "This is not valid YAML\n";

    let (_db_temp, _repo_temp, db_path) =
        setup_sync_project(&[("ja-valid01.md", valid), ("broken.md", invalid)]);

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 task"))
        .stdout(predicate::str::contains("skipped"));
}

// ---------------------------------------------------------------------------
// Task-story linking integration tests (story 04-001)
// ---------------------------------------------------------------------------

/// Helper: create a ticket in a temp tickets dir and return its ID.
fn setup_ticket(tickets_dir: &std::path::Path, title: &str) -> String {
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TICKETS_DIR", tickets_dir)
        .arg("create")
        .arg(title)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap().trim().to_string()
}

#[test]
fn test_story_link_writes_story_to_frontmatter() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");
    let db_path = temp_dir.path().join("portfolio.db");

    // Create a story in the DB
    let story_id = setup_story(&db_path, "Portfolio Layer");

    // Create a task
    let task_id = setup_ticket(&tickets_dir, "Link test task");

    // Link the task to the story
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("link")
        .arg(&task_id)
        .arg(&story_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated"))
        .stdout(predicate::str::contains(&story_id));

    // Assert the markdown file contains the story field
    let ticket_files = find_ticket_files(&tickets_dir);
    let content = fs::read_to_string(&ticket_files[0]).unwrap();
    assert!(content.contains(&format!("story: {}", story_id)));
}

#[test]
fn test_story_unlink_removes_story_from_frontmatter() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");
    let db_path = temp_dir.path().join("portfolio.db");

    // Create a story and a task, then link them
    let story_id = setup_story(&db_path, "Portfolio Layer");
    let task_id = setup_ticket(&tickets_dir, "Unlink test task");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("link")
        .arg(&task_id)
        .arg(&story_id)
        .assert()
        .success();

    // Verify the story field is present
    let ticket_files = find_ticket_files(&tickets_dir);
    let content = fs::read_to_string(&ticket_files[0]).unwrap();
    assert!(content.contains("story:"));

    // Unlink
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("unlink")
        .arg(&task_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared story link"));

    // Assert the story field is gone from the markdown
    let ticket_files = find_ticket_files(&tickets_dir);
    let content = fs::read_to_string(&ticket_files[0]).unwrap();
    assert!(!content.contains("story:"));
}

#[test]
fn test_story_link_nonexistent_story_fails() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");
    let db_path = temp_dir.path().join("portfolio.db");

    // Create a task (no story created in DB)
    let task_id = setup_ticket(&tickets_dir, "Missing story test task");

    // Capture the markdown content before the failed link
    let ticket_files = find_ticket_files(&tickets_dir);
    let content_before = fs::read_to_string(&ticket_files[0]).unwrap();

    // Attempt to link to a non-existent story
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("link")
        .arg(&task_id)
        .arg("story-doesnotexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));

    // Assert the markdown is unchanged
    let ticket_files = find_ticket_files(&tickets_dir);
    let content_after = fs::read_to_string(&ticket_files[0]).unwrap();
    assert_eq!(content_before, content_after);
}

#[test]
fn test_sync_populates_story_id_from_markdown() {
    // Set up a sync project with a ticket (story field added after)
    let ticket_no_story = "---\nid: ja-story01\ntitle: Story Task\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Story Task\n";
    let (_db_temp, repo_temp, db_path) =
        setup_sync_project(&[("ja-story01.md", ticket_no_story)]);

    // Create a story in the same DB that setup_sync_project uses
    let story_id = setup_story(&db_path, "Sync Story");

    // Rewrite the ticket with the story field
    let ticket_with_story = format!(
        "---\nid: ja-story01\ntitle: Story Task\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\nstory: {}\n---\n\n# Story Task\n",
        story_id
    );
    fs::write(
        repo_temp.path().join(".tickets").join("open").join("ja-story01.md"),
        &ticket_with_story,
    )
    .unwrap();

    // Run sync
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success();

    // Query the DB to verify story_id was populated
    let db = tkr_test_db::open_db(&db_path);
    let stored_story_id: Option<String> = db
        .query_row(
            "SELECT story_id FROM tasks WHERE id = 'ja-story01'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored_story_id, Some(story_id));
}

#[test]
fn test_sync_unlink_clears_story_id() {
    // Set up a sync project with a ticket (no story yet)
    let ticket_no_story = "---\nid: ja-unlink01\ntitle: Unlink Task\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Unlink Task\n";
    let (_db_temp, repo_temp, db_path) =
        setup_sync_project(&[("ja-unlink01.md", ticket_no_story)]);

    // Create a story in the same DB
    let story_id = setup_story(&db_path, "Unlink Sync Story");

    // Write the ticket with the story field
    let ticket_with_story = format!(
        "---\nid: ja-unlink01\ntitle: Unlink Task\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\nstory: {}\n---\n\n# Unlink Task\n",
        story_id
    );
    fs::write(
        repo_temp.path().join(".tickets").join("open").join("ja-unlink01.md"),
        &ticket_with_story,
    )
    .unwrap();

    // First sync — populates story_id
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Verify story_id is set
    let db = tkr_test_db::open_db(&db_path);
    let stored: Option<String> = db
        .query_row(
            "SELECT story_id FROM tasks WHERE id = 'ja-unlink01'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored, Some(story_id.clone()));

    // Remove the story field from the markdown
    let ticket_without_story = "---\nid: ja-unlink01\ntitle: Unlink Task\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Unlink Task\n";
    fs::write(
        repo_temp.path().join(".tickets").join("open").join("ja-unlink01.md"),
        ticket_without_story,
    )
    .unwrap();

    // Second sync — should clear story_id (set to NULL)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    let stored: Option<String> = db
        .query_row(
            "SELECT story_id FROM tasks WHERE id = 'ja-unlink01'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored, None);
}

#[test]
fn test_story_field_backward_compat() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");
    fs::create_dir_all(tickets_dir.join("open")).unwrap();

    // Write a ticket with no story field (old format)
    let content = "---\nid: ja-old01\ntitle: Old Ticket\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Old Ticket\n";
    fs::write(tickets_dir.join("open").join("ja-old01.md"), content).unwrap();

    // List should still work (no parse error)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("ja-old01"));

    // Show should still work
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("show")
        .arg("ja-old01")
        .assert()
        .success()
        .stdout(predicate::str::contains("Old Ticket"));
}

#[test]
fn test_story_field_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");
    let db_path = temp_dir.path().join("portfolio.db");

    // Create a story
    let story_id = setup_story(&db_path, "Round Trip Story");

    // Create a task
    let task_id = setup_ticket(&tickets_dir, "Round trip task");

    // Link it
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("link")
        .arg(&task_id)
        .arg(&story_id)
        .assert()
        .success();

    // Show the ticket — should contain the story field
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TICKETS_DIR", &tickets_dir)
        .arg("show")
        .arg(&task_id)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains(&format!("story: {}", story_id)));

    // Unlink it
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .env("TKR_DB_PATH", &db_path)
        .arg("story")
        .arg("unlink")
        .arg(&task_id)
        .assert()
        .success();

    // Show again — story field should be gone
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TICKETS_DIR", &tickets_dir)
        .arg("show")
        .arg(&task_id)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(!stdout.contains("story:"));
}

/// Helper module to open a read-only DB connection for test assertions.
mod tkr_test_db {
    use rusqlite::Connection;
    use std::path::Path;

    pub fn open_db(path: &Path) -> Connection {
        Connection::open(path).unwrap()
    }
}

// ---------------------------------------------------------------------------
// Tag integration tests (story 04-002)
// ---------------------------------------------------------------------------

#[test]
fn test_tag_add_writes_tags_to_frontmatter() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let task_id = setup_ticket(&tickets_dir, "Tag add test task");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("tag")
        .arg("add")
        .arg(&task_id)
        .arg("security")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added tag"))
        .stdout(predicate::str::contains("security"));

    // Assert the markdown file contains the tags field
    let ticket_files = find_ticket_files(&tickets_dir);
    let content = fs::read_to_string(&ticket_files[0]).unwrap();
    assert!(content.contains("tags:"));
    assert!(content.contains("security"));
}

#[test]
fn test_tag_add_normalizes_to_lowercase() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let task_id = setup_ticket(&tickets_dir, "Tag normalize test task");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("tag")
        .arg("add")
        .arg(&task_id)
        .arg("Backend")
        .assert()
        .success()
        .stdout(predicate::str::contains("backend"));

    let ticket_files = find_ticket_files(&tickets_dir);
    let content = fs::read_to_string(&ticket_files[0]).unwrap();
    assert!(content.contains("backend"));
    assert!(!content.contains("Backend"));
}

#[test]
fn test_tag_add_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let task_id = setup_ticket(&tickets_dir, "Tag idempotent test task");

    // First add
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("tag")
        .arg("add")
        .arg(&task_id)
        .arg("security")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added tag"));

    // Second add — should say "already on"
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("tag")
        .arg("add")
        .arg(&task_id)
        .arg("security")
        .assert()
        .success()
        .stdout(predicate::str::contains("already on"));

    // Assert only one tag in the markdown
    let ticket_files = find_ticket_files(&tickets_dir);
    let content = fs::read_to_string(&ticket_files[0]).unwrap();
    let count = content.matches("security").count();
    assert_eq!(count, 1);
}

#[test]
fn test_tag_remove_removes_from_frontmatter() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let task_id = setup_ticket(&tickets_dir, "Tag remove test task");

    // Add a tag first
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("tag")
        .arg("add")
        .arg(&task_id)
        .arg("security")
        .assert()
        .success();

    // Remove it
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("tag")
        .arg("remove")
        .arg(&task_id)
        .arg("security")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed tag"));

    // Assert the tags field is gone (skipped when empty)
    let ticket_files = find_ticket_files(&tickets_dir);
    let content = fs::read_to_string(&ticket_files[0]).unwrap();
    assert!(!content.contains("tags:"));
}

#[test]
fn test_tag_remove_missing_is_noop() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let task_id = setup_ticket(&tickets_dir, "Tag remove missing test task");

    // Capture content before
    let ticket_files = find_ticket_files(&tickets_dir);
    let content_before = fs::read_to_string(&ticket_files[0]).unwrap();

    // Remove a tag that doesn't exist
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("tag")
        .arg("remove")
        .arg(&task_id)
        .arg("nope")
        .assert()
        .success()
        .stdout(predicate::str::contains("not found"));

    // Assert markdown is unchanged
    let ticket_files = find_ticket_files(&tickets_dir);
    let content_after = fs::read_to_string(&ticket_files[0]).unwrap();
    assert_eq!(content_before, content_after);
}

#[test]
fn test_tag_add_empty_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let task_id = setup_ticket(&tickets_dir, "Tag empty reject test task");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("tag")
        .arg("add")
        .arg(&task_id)
        .arg("   ")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn test_tag_list_shows_all_tags() {
    // Set up a sync project with two tagged tickets
    let ticket_a = "---\nid: ja-taga01\ntitle: Tag A\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\ntags: [security]\n---\n\n# Tag A\n";
    let ticket_b = "---\nid: ja-tagb01\ntitle: Tag B\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\ntags: [backend]\n---\n\n# Tag B\n";
    let (_db_temp, _repo_temp, db_path) =
        setup_sync_project(&[("ja-taga01.md", ticket_a), ("ja-tagb01.md", ticket_b)]);

    // Sync to populate tags + task_tags
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // List tags
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("tag")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("backend"))
        .stdout(predicate::str::contains("security"));
}

#[test]
fn test_tag_list_tasks_shows_task_ids() {
    let ticket_a = "---\nid: ja-taga01\ntitle: Tag A\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\ntags: [security]\n---\n\n# Tag A\n";
    let ticket_b = "---\nid: ja-tagb01\ntitle: Tag B\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\ntags: [backend]\n---\n\n# Tag B\n";
    let (_db_temp, _repo_temp, db_path) =
        setup_sync_project(&[("ja-taga01.md", ticket_a), ("ja-tagb01.md", ticket_b)]);

    // Sync to populate tags + task_tags
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // List tags with tasks
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("tag")
        .arg("list")
        .arg("--tasks")
        .assert()
        .success()
        .stdout(predicate::str::contains("backend: ja-tagb01"))
        .stdout(predicate::str::contains("security: ja-taga01"));
}

#[test]
fn test_tag_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("tag")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No tags found"));
}

#[test]
fn test_sync_reconciles_tags_add() {
    let ticket_no_tags = "---\nid: ja-tagc01\ntitle: Tag Sync\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Tag Sync\n";
    let (_db_temp, repo_temp, db_path) =
        setup_sync_project(&[("ja-tagc01.md", ticket_no_tags)]);

    // First sync
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Add a tag to the markdown
    let ticket_with_tags = "---\nid: ja-tagc01\ntitle: Tag Sync\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\ntags: [security]\n---\n\n# Tag Sync\n";
    fs::write(
        repo_temp.path().join(".tickets").join("open").join("ja-tagc01.md"),
        ticket_with_tags,
    )
    .unwrap();

    // Second sync
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Verify task_tags row exists
    let db = tkr_test_db::open_db(&db_path);
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM task_tags tt
             JOIN tags t ON t.id = tt.tag_id
             WHERE tt.task_id = 'ja-tagc01' AND t.name = 'security'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_sync_reconciles_tags_remove() {
    let ticket_with_tags = "---\nid: ja-tagd01\ntitle: Tag Sync Remove\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\ntags: [security]\n---\n\n# Tag Sync Remove\n";
    let (_db_temp, repo_temp, db_path) =
        setup_sync_project(&[("ja-tagd01.md", ticket_with_tags)]);

    // First sync — populates the tag
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Verify the tag row exists
    let db = tkr_test_db::open_db(&db_path);
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM task_tags tt
             JOIN tags t ON t.id = tt.tag_id
             WHERE tt.task_id = 'ja-tagd01' AND t.name = 'security'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    // Remove the tag from the markdown
    let ticket_no_tags = "---\nid: ja-tagd01\ntitle: Tag Sync Remove\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Tag Sync Remove\n";
    fs::write(
        repo_temp.path().join(".tickets").join("open").join("ja-tagd01.md"),
        ticket_no_tags,
    )
    .unwrap();

    // Second sync
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path).arg("sync").assert().success();

    // Verify the task_tags row is gone
    let db = tkr_test_db::open_db(&db_path);
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM task_tags tt
             JOIN tags t ON t.id = tt.tag_id
             WHERE tt.task_id = 'ja-tagd01' AND t.name = 'security'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_tag_field_backward_compat() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");
    fs::create_dir_all(tickets_dir.join("open")).unwrap();

    // Write a ticket with no tags field (old format)
    let content = "---\nid: ja-oldtag01\ntitle: Old Tag Ticket\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Old Tag Ticket\n";
    fs::write(tickets_dir.join("open").join("ja-oldtag01.md"), content).unwrap();

    // List should still work (no parse error)
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("ja-oldtag01"));
}

// ---------------------------------------------------------------------------
// Portfolio views integration tests (story 04-003)
// ---------------------------------------------------------------------------

/// Helper: set up a synced portfolio DB with two projects, a requirement, a
/// story, tasks (some linked to the story, some not), and tags. Returns the
/// temp dirs (kept alive) and the db_path.
fn setup_view_data() -> (TempDir, TempDir, std::path::PathBuf) {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    let repo_temp = TempDir::new().unwrap();
    let repo_path = repo_temp.path().to_path_buf();
    let tickets_dir = repo_path.join(".tickets");
    fs::create_dir_all(tickets_dir.join("open")).unwrap();

    // Create a story in the DB first (so we can reference it in markdown)
    let req_id = setup_requirement(&db_path, "personal", "Multi-account Sync");
    let story_id = setup_story_with_requirement(&db_path, "Portfolio Layer", &req_id);

    // Ticket 1: linked to story, tagged security
    let t1 = format!(
        "---\nid: ja-v001\ntitle: Add rusqlite dependency\nstatus: in_progress\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\nstory: {}\ntags: [security]\n---\n\n# Add rusqlite\n",
        story_id
    );
    // Ticket 2: linked to story, tagged backend
    let t2 = format!(
        "---\nid: ja-v002\ntitle: Add notify watcher\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\nstory: {}\ntags: [backend]\n---\n\n# Add notify\n",
        story_id
    );
    // Ticket 3: unlinked, tagged security
    let t3 = "---\nid: ja-v003\ntitle: Tidy docs\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\ntags: [security]\n---\n\n# Tidy docs\n";

    fs::write(tickets_dir.join("open").join("ja-v001.md"), t1).unwrap();
    fs::write(tickets_dir.join("open").join("ja-v002.md"), t2).unwrap();
    fs::write(tickets_dir.join("open").join("ja-v003.md"), t3).unwrap();

    // Register the project and sync
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(&repo_path)
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success();

    (db_temp, repo_temp, db_path)
}

/// Helper: create a story linked to a requirement and return its ID.
fn setup_story_with_requirement(db_path: &std::path::Path, title: &str, req_id: &str) -> String {
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TKR_DB_PATH", db_path)
        .arg("story")
        .arg("create")
        .arg(title)
        .arg(format!("--requirement={}", req_id))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    stdout
        .lines()
        .find(|l| l.contains("story-"))
        .and_then(|l| l.split("id: ").nth(1))
        .and_then(|s| s.split(')').next())
        .unwrap_or("")
        .to_string()
}

#[test]
fn test_portfolio_view_by_requirement() {
    let (_db_temp, _repo_temp, db_path) = setup_view_data();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-requirement")
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi-account Sync (2)"))
        .stdout(predicate::str::contains("ja-v001 - Add rusqlite dependency (in_progress)"))
        .stdout(predicate::str::contains("ja-v002 - Add notify watcher (open)"))
        .stdout(predicate::str::contains("Showing 2 tasks across 1 groups"));
}

#[test]
fn test_portfolio_view_by_story() {
    let (_db_temp, _repo_temp, db_path) = setup_view_data();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-story")
        .assert()
        .success()
        .stdout(predicate::str::contains("Portfolio Layer (2)"))
        .stdout(predicate::str::contains("(none) (1)"))
        .stdout(predicate::str::contains("ja-v003 - Tidy docs (open)"))
        .stdout(predicate::str::contains("Showing 3 tasks across 2 groups"));
}

#[test]
fn test_portfolio_view_by_tag_filtered() {
    let (_db_temp, _repo_temp, db_path) = setup_view_data();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-tag=security")
        .assert()
        .success()
        .stdout(predicate::str::contains("ja-v001 - Add rusqlite dependency (in_progress)"))
        .stdout(predicate::str::contains("ja-v003 - Tidy docs (open)"))
        .stdout(predicate::str::contains("Showing 2 tasks across 1 groups"));
}

#[test]
fn test_portfolio_view_by_tag_summary() {
    let (_db_temp, _repo_temp, db_path) = setup_view_data();

    // --by-tag with no value lists all tags with counts
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-tag")
        .assert()
        .success()
        .stdout(predicate::str::contains("security"))
        .stdout(predicate::str::contains("backend"));
}

#[test]
fn test_portfolio_view_by_project() {
    let (_db_temp, _repo_temp, db_path) = setup_view_data();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-project")
        .assert()
        .success()
        .stdout(predicate::str::contains("Showing 3 tasks across 1 groups"))
        .stdout(predicate::str::contains("ja-v001"))
        .stdout(predicate::str::contains("ja-v002"))
        .stdout(predicate::str::contains("ja-v003"));
}

#[test]
fn test_portfolio_view_empty_group() {
    let db_temp = TempDir::new().unwrap();
    let db_path = db_temp.path().join("portfolio.db");

    // Create a requirement with no stories/tasks
    setup_requirement(&db_path, "personal", "Empty Requirement");

    // Also create a project with a task (so the DB isn't empty)
    let repo_temp = TempDir::new().unwrap();
    let tickets_dir = repo_temp.path().join(".tickets");
    fs::create_dir_all(tickets_dir.join("open")).unwrap();
    let ticket = "---\nid: ja-eg01\ntitle: Lone task\nstatus: open\ndeps: []\nlinks: []\ncreated: 2026-09-08T11:00:00Z\ntype: task\npriority: 2\n---\n\n# Lone\n";
    fs::write(tickets_dir.join("open").join("ja-eg01.md"), ticket).unwrap();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("project")
        .arg("register")
        .arg(repo_temp.path())
        .arg("--portfolio")
        .arg("personal")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("sync")
        .assert()
        .success();

    // The requirement should appear with count 0 and an empty marker
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-requirement")
        .assert()
        .success()
        .stdout(predicate::str::contains("Empty Requirement (0)"))
        .stdout(predicate::str::contains("(empty)"));
}

#[test]
fn test_portfolio_view_no_flag_errors() {
    let (_db_temp, _repo_temp, db_path) = setup_view_data();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .assert()
        .failure()
        .stderr(predicate::str::contains("specify one of"));
}

#[test]
fn test_portfolio_view_multiple_flags_errors() {
    let (_db_temp, _repo_temp, db_path) = setup_view_data();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-story")
        .arg("--by-project")
        .assert()
        .failure()
        .stderr(predicate::str::contains("exactly one grouping flag"));
}

#[test]
fn test_portfolio_view_json_output() {
    let (_db_temp, _repo_temp, db_path) = setup_view_data();

    let mut cmd = Command::cargo_bin("tkr").unwrap();
    let output = cmd
        .env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-story")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    // Should be valid JSON with expected keys
    assert!(stdout.contains("\"version\""));
    assert!(stdout.contains("\"groups\""));
    assert!(stdout.contains("\"group\""));
    assert!(stdout.contains("\"count\""));
    assert!(stdout.contains("\"total_tasks\""));
    assert!(stdout.contains("\"Portfolio Layer\""));
    // Verify it parses as JSON
    serde_json::from_str::<serde_json::Value>(&stdout).unwrap();
}

#[test]
fn test_portfolio_view_empty_db_message() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("portfolio.db");

    // No projects, no tasks
    let mut cmd = Command::cargo_bin("tkr").unwrap();
    cmd.env("TKR_DB_PATH", &db_path)
        .arg("portfolio")
        .arg("view")
        .arg("--by-project")
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks found"));
}

#[test]
fn test_portfolio_view_readonly() {
    let (_db_temp, repo_temp, db_path) = setup_view_data();

    // Capture all markdown file contents before running the view
    let tickets_dir = repo_temp.path().join(".tickets");
    let open_dir = tickets_dir.join("open");
    let mut before: Vec<(std::path::PathBuf, String)> = Vec::new();
    for entry in fs::read_dir(&open_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = fs::read_to_string(&path).unwrap();
            before.push((path.clone(), content));
        }
    }

    // Run all four views
    for flag in &["--by-requirement", "--by-story", "--by-project", "--by-tag"] {
        let mut cmd = Command::cargo_bin("tkr").unwrap();
        cmd.env("TKR_DB_PATH", &db_path)
            .arg("portfolio")
            .arg("view")
            .arg(flag)
            .assert()
            .success();
    }

    // Verify no markdown files changed
    for (path, content) in &before {
        let after = fs::read_to_string(path).unwrap();
        assert_eq!(&after, content, "markdown file {} was modified by view", path.display());
    }
}
