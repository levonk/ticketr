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

    let content = fs::read_to_string(ticket_files[0].path()).unwrap();
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
    let content = fs::read_to_string(ticket_path).unwrap();
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
    let content = fs::read_to_string(ticket_path).unwrap();
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
    let content = fs::read_to_string(ticket_path).unwrap();
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
