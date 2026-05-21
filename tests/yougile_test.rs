use chrono::{NaiveDate, NaiveTime};
use migration_scheduler::scheduler::{Booking, TimeSlot};
use migration_scheduler::yougile::{YougileClient, YougileConfig};
use serial_test::serial;
use std::fs;
use std::path::Path;

const TEST_YOUGILE_PATH: &str = "data/test_yougile_config.json";

fn clean_test_file() {
    let path = Path::new(TEST_YOUGILE_PATH);
    if path.exists() {
        fs::remove_file(path).expect("failed to remove test yougile config");
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create data directory");
    }
}

fn get_test_url() -> String {
    std::env::var("YOUGILE_TEST_API_URL").unwrap_or_default()
}

fn get_test_token() -> String {
    std::env::var("YOUGILE_TEST_API_TOKEN").unwrap_or_default()
}

fn should_skip_test() -> bool {
    get_test_url().is_empty() || get_test_token().is_empty()
}

fn test_config() -> YougileConfig {
    YougileConfig {
        enabled: true,
        api_url: get_test_url(),
        api_token: Some(get_test_token()),
        project_id: None,
        board_id: None,
        column_id: None,
        assignee_id: None,
    }
}

async fn create_yougile_client() -> YougileClient {
    clean_test_file();
    YougileClient::with_config(test_config(), TEST_YOUGILE_PATH)
        .await
        .unwrap()
}

fn make_test_slot() -> TimeSlot {
    let booking = Booking {
        company_name: "TestCorp Integration".to_string(),
        admin_email: "test@test.com".to_string(),
        company_id: "test-comp-001".to_string(),
        download_email: "dl@test.com".to_string(),
        created_at: chrono::Utc::now(),
    };

    TimeSlot {
        id: uuid::Uuid::new_v4(),
        date: NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        start_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        end_time: NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
        is_available: false,
        booking: Some(booking),
        yougile_task_id: None,
        completed: false,
    }
}

// ── load_projects ───────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_load_projects() {
    if should_skip_test() { return; }
    let client = create_yougile_client().await;

    let projects = client.load_projects().await.unwrap();
    assert!(!projects.is_empty(), "Expected at least one project");

    let project = &projects[0];
    assert!(!project.id.is_empty());
    assert!(!project.title.is_empty());

    println!("Loaded {} projects", projects.len());
    for p in &projects {
        println!("  - {}: {}", p.title, p.id);
    }
}

// ── load_boards ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_load_boards() {
    if should_skip_test() { return; }
    let client = create_yougile_client().await;

    let projects = client.load_projects().await.unwrap();
    assert!(!projects.is_empty());

    let project_id = &projects[0].id;
    let boards = client.load_boards(project_id).await.unwrap();

    println!("Loaded {} boards for project {}", boards.len(), project_id);
    for b in &boards {
        println!("  - {}: {}", b.title, b.id);
    }
}

// ── load_columns ────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_load_columns() {
    if should_skip_test() { return; }
    let client = create_yougile_client().await;

    let projects = client.load_projects().await.unwrap();
    let boards = client.load_boards(&projects[0].id).await.unwrap();

    if boards.is_empty() {
        println!("No boards found, skipping columns test");
        return;
    }

    let board_id = &boards[0].id;
    let columns = client.load_columns(board_id).await.unwrap();

    assert!(!columns.is_empty(), "Expected at least one column");
    println!("Loaded {} columns for board {}", columns.len(), board_id);
    for c in &columns {
        println!("  - {}: {}", c.title, c.id);
    }
}

// ── load_users ──────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_load_users() {
    if should_skip_test() { return; }
    let client = create_yougile_client().await;

    let projects = client.load_projects().await.unwrap();
    assert!(!projects.is_empty());

    let project_id = &projects[0].id;
    let users = client.load_users(project_id).await.unwrap();

    assert!(!users.is_empty(), "Expected at least one user");
    println!("Loaded {} users for project {}", users.len(), project_id);
    for u in &users {
        println!("  - {} <{}>: {}", u.name, u.email, u.id);
    }
}

// ── create_task ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_create_task() {
    if should_skip_test() { return; }
    let client = create_yougile_client().await;

    let slot = make_test_slot();
    let task_id = client.create_task(&slot).await.unwrap();

    assert!(!task_id.is_empty(), "Expected non-empty task ID");
    println!("Created task: {}", task_id);

    // Clean up
    client.delete_task(&task_id).await.unwrap();
}

// ── update_task ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_create_and_update_task() {
    if should_skip_test() { return; }
    let client = create_yougile_client().await;

    let mut slot = make_test_slot();
    let task_id = client.create_task(&slot).await.unwrap();
    assert!(!task_id.is_empty());

    slot.yougile_task_id = Some(task_id.clone());
    slot.completed = true;

    client.update_task(&slot).await.unwrap();
    println!("Updated task: {}", task_id);

    // Clean up
    client.delete_task(&task_id).await.unwrap();
}

// ── delete_task ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_delete_task() {
    if should_skip_test() { return; }
    let client = create_yougile_client().await;

    let slot = make_test_slot();
    let task_id = client.create_task(&slot).await.unwrap();
    assert!(!task_id.is_empty());

    client.delete_task(&task_id).await.unwrap();
    println!("Deleted task: {}", task_id);
}

// ── disabled integration ────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_disabled_returns_empty() {
    if should_skip_test() { return; }
    clean_test_file();

    let disabled_config = YougileConfig {
        enabled: false,
        api_url: "http://invalid".to_string(),
        api_token: Some("invalid".to_string()),
        project_id: None,
        board_id: None,
        column_id: None,
        assignee_id: None,
    };

    let client = YougileClient::with_config(disabled_config, TEST_YOUGILE_PATH)
        .await
        .unwrap();

    let projects = client.load_projects().await.unwrap();
    assert!(projects.is_empty());

    let slot = make_test_slot();
    let task_id = client.create_task(&slot).await.unwrap();
    assert!(task_id.is_empty());
}

// ── config roundtrip ────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_config_update_persists() {
    if should_skip_test() { return; }
    clean_test_file();

    let client = create_yougile_client().await;

    let mut updated = test_config();
    updated.enabled = false;
    updated.project_id = Some("test-project-123".to_string());

    client.update_config(updated).await.unwrap();

    let fetched = client.config();
    assert!(!fetched.enabled);
    assert_eq!(fetched.project_id, Some("test-project-123".to_string()));
    assert!(fetched.api_token.is_some(), "api_token should be preserved");
}

// ── full integration flow ───────────────────────────────────────────

#[tokio::test]
#[serial]
async fn yougile_full_task_lifecycle() {
    if should_skip_test() { return; }
    let client = create_yougile_client().await;

    let projects = client.load_projects().await.unwrap();
    assert!(!projects.is_empty());

    let boards = client.load_boards(&projects[0].id).await.unwrap();
    assert!(!boards.is_empty());

    let columns = client.load_columns(&boards[0].id).await.unwrap();
    assert!(!columns.is_empty());

    let users = client.load_users(&projects[0].id).await.unwrap();
    assert!(!users.is_empty());

    let mut slot = make_test_slot();
    let task_id = client.create_task(&slot).await.unwrap();
    assert!(!task_id.is_empty());
    println!("Created task: {}", task_id);

    slot.yougile_task_id = Some(task_id.clone());
    slot.completed = true;
    client.update_task(&slot).await.unwrap();
    println!("Updated task to completed");

    client.delete_task(&task_id).await.unwrap();
    println!("Deleted task");
}
