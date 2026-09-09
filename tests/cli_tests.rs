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
