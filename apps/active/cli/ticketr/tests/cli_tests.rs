use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("tk").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_create_ticket() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tk").unwrap();
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

    assert_eq!(ticket_files.len(), 1);
}

#[test]
fn test_create_ticket_with_project_category() {
    let temp_dir = TempDir::new().unwrap();
    let tickets_dir = temp_dir.path().join(".tickets");

    let mut cmd = Command::cargo_bin("tk").unwrap();
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
    let ticket_files: Vec<_> = fs::read_dir(&tickets_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();

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

    let mut cmd = Command::cargo_bin("tk").unwrap();
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
    let mut cmd = Command::cargo_bin("tk").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Status test ticket")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files: Vec<_> = fs::read_dir(&tickets_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();

    let ticket_path = ticket_files[0].path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Update status
    let mut cmd = Command::cargo_bin("tk").unwrap();
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
    let mut cmd = Command::cargo_bin("tk").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Note test ticket")
        .assert()
        .success();

    // Get the ticket ID
    let ticket_files: Vec<_> = fs::read_dir(&tickets_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();

    let ticket_path = ticket_files[0].path();
    let ticket_id = ticket_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    // Add a note
    let mut cmd = Command::cargo_bin("tk").unwrap();
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
    let mut cmd = Command::cargo_bin("tk").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("create")
        .arg("Parent ticket")
        .assert()
        .success();

    // Create second ticket
    let mut cmd = Command::cargo_bin("tk").unwrap();
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
    let mut cmd = Command::cargo_bin("tk").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("dep")
        .arg(child_id)
        .arg(parent_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added dependency"));

    // Remove dependency
    let mut cmd = Command::cargo_bin("tk").unwrap();
    cmd.env("TICKETS_DIR", &tickets_dir)
        .arg("undep")
        .arg(child_id)
        .arg(parent_id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed dependency"));
}
