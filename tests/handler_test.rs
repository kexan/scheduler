use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use migration_scheduler::api::routes;
use migration_scheduler::scheduler::Scheduler;
use migration_scheduler::yougile::YougileClient;
use serial_test::serial;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tower::ServiceExt;

const TEST_SLOTS_PATH: &str = "data/test_handler_slots.json";
const TEST_YOUGILE_PATH: &str = "data/test_handler_yougile.json";

fn clean_test_files() {
    for path in &[TEST_SLOTS_PATH, TEST_YOUGILE_PATH] {
        let p = Path::new(path);
        if p.exists() {
            fs::remove_file(p).expect("failed to remove test file");
        }
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).expect("failed to create data directory");
        }
    }
}

async fn create_test_app() -> axum::Router {
    clean_test_files();

    let scheduler = Scheduler::with_path(TEST_SLOTS_PATH).await.unwrap();
    let yougile = YougileClient::with_path(TEST_YOUGILE_PATH).await.unwrap();
    routes(Arc::new(scheduler), Arc::new(yougile))
}

async fn admin_login(app: &axum::Router) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/admin")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"password":"admin123"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .to_string()
}

fn make_slot_body(date: &str, start: &str, end: &str) -> String {
    format!(
        r#"{{"date":"{}","start_time":"{}","end_time":"{}"}}"#,
        date, start, end
    )
}

fn make_booking_body() -> String {
    r#"{"company_name":"TestCorp","admin_email":"admin@test.com","company_id":"c1","download_email":"dl@test.com"}"#.to_string()
}

// ── GET /api/slots ──────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_get_slots_empty() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let slots: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(slots.is_empty());
}

#[tokio::test]
#[serial]
async fn handler_get_slots_with_data() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    // Create a slot first
    app.clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2026-07-01", "10:00", "11:00")))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let slots: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0]["date"], "2026-07-01");
}

#[tokio::test]
#[serial]
async fn handler_get_slots_with_date_filter() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    for date in &["2026-06-01", "2026-07-01", "2026-08-01"] {
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/api/slots")
                    .method("POST")
                    .header("content-type", "application/json")
                    .header("cookie", &cookie)
                    .body(Body::from(make_slot_body(date, "10:00", "11:00")))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots?from=2026-07-01&to=2026-07-31")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let slots: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0]["date"], "2026-07-01");
}

// ── POST /api/slots ─────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_create_slot_success() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2026-09-15", "14:00", "15:30")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let slot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(slot["date"], "2026-09-15");
    assert_eq!(slot["start_time"], "14:00:00");
    assert_eq!(slot["end_time"], "15:30:00");
    assert_eq!(slot["is_available"], true);
    assert!(slot["id"].is_string());
}

#[tokio::test]
#[serial]
async fn handler_create_slot_unauthorized() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(make_slot_body("2026-09-15", "14:00", "15:30")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn handler_create_slot_invalid_body() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"date":"not-a-date"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[serial]
async fn handler_create_slot_invalid_time_range() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2026-09-15", "15:30", "14:00")))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(err["error"]
        .as_str()
        .unwrap()
        .contains("Invalid time range"));
}

// ── POST /api/slots/{id}/book ───────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_book_slot_success() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    // Create a slot
    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2026-10-01", "09:00", "10:00")))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let slot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let slot_id = slot["id"].as_str().unwrap();

    // Book it
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}/book", slot_id))
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(make_booking_body()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let booked: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(booked["is_available"], false);
    assert_eq!(booked["booking"]["company_name"], "TestCorp");
}

#[tokio::test]
#[serial]
async fn handler_book_slot_past_date_rejected() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    // Create a slot in the past
    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2020-01-01", "09:00", "10:00")))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let slot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let slot_id = slot["id"].as_str().unwrap();

    // Try to book it
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}/book", slot_id))
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(make_booking_body()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(err["error"]
        .as_str()
        .unwrap()
        .contains("Внутренняя ошибка сервера"));
}

#[tokio::test]
#[serial]
async fn handler_book_slot_not_found() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots/00000000-0000-0000-0000-000000000000/book")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(make_booking_body()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn handler_book_slot_already_booked() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2026-10-01", "09:00", "10:00")))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let slot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let slot_id = slot["id"].as_str().unwrap();

    // First booking
    app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}/book", slot_id))
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(make_booking_body()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Second booking should fail
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}/book", slot_id))
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(make_booking_body()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ── DELETE /api/slots/{id} ──────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_delete_slot_success() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2026-11-01", "10:00", "11:00")))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let slot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let slot_id = slot["id"].as_str().unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}", slot_id))
                .method("DELETE")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result, true);
}

#[tokio::test]
#[serial]
async fn handler_delete_slot_not_found() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots/00000000-0000-0000-0000-000000000000")
                .method("DELETE")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn handler_delete_slot_unauthorized() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots/00000000-0000-0000-0000-000000000000")
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── PUT /api/slots/{id} ─────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_update_slot_time() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2026-12-01", "10:00", "11:00")))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let slot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let slot_id = slot["id"].as_str().unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}", slot_id))
                .method("PUT")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"start_time":"14:00","end_time":"16:00"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["start_time"], "14:00:00");
    assert_eq!(updated["end_time"], "16:00:00");
}

#[tokio::test]
#[serial]
async fn handler_update_slot_not_found() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots/00000000-0000-0000-0000-000000000000")
                .method("PUT")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"start_time":"14:00"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn handler_update_slot_unauthorized() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slots/00000000-0000-0000-0000-000000000000")
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"start_time":"14:00"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── POST /api/auth/admin ────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_auth_admin_valid_password() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/admin")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"password":"admin123"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("set-cookie"));
}

#[tokio::test]
#[serial]
async fn handler_auth_admin_invalid_password() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/admin")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"password":"wrong"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── GET /api/auth/check ─────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_auth_check_not_authenticated() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["authenticated"], false);
}

#[tokio::test]
#[serial]
async fn handler_auth_check_authenticated() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/check")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["authenticated"], true);
}

// ── POST /api/auth/logout ───────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_logout_success() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    // Verify authenticated before logout
    let check_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/check")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(check_resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["authenticated"], true);

    // Logout
    let logout_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/logout")
                .method("POST")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout_resp.status(), StatusCode::OK);
    assert!(logout_resp.headers().contains_key("set-cookie"));

    let body = axum::body::to_bytes(logout_resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["message"], "Выход выполнен");

    // Verify not authenticated after logout
    let check_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/check")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(check_resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["authenticated"], false);
}

#[tokio::test]
#[serial]
async fn handler_logout_without_token() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/logout")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ── GET /api/yougile/settings ───────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_get_yougile_settings() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/yougile/settings")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let settings: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(settings.is_object());
    assert!(
        settings.get("api_token").is_none(),
        "api_token must not be exposed"
    );
}

#[tokio::test]
#[serial]
async fn handler_get_yougile_settings_unauthorized() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/yougile/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── PUT /api/yougile/settings ───────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_update_yougile_settings() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/yougile/settings")
                .method("PUT")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(
                    r#"{"enabled":true,"api_url":"https://test.yougile.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the update persisted
    let get_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/yougile/settings")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(get_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let settings: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(settings["enabled"], true);
    assert_eq!(settings["api_url"], "https://test.yougile.com");
}

// ── Full CRUD flow ──────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_full_crud_flow() {
    let app = create_test_app().await;
    let cookie = admin_login(&app).await;

    // Create
    let create_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(make_slot_body("2027-01-15", "10:00", "12:00")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let slot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let slot_id = slot["id"].as_str().unwrap();

    // Read
    let slots_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(slots_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let slots: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(slots.len(), 1);

    // Update
    let update_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}", slot_id))
                .method("PUT")
                .header("content-type", "application/json")
                .header("cookie", &cookie)
                .body(Body::from(r#"{"end_time":"13:00"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_resp.status(), StatusCode::OK);

    // Book
    let book_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}/book", slot_id))
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(make_booking_body()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(book_resp.status(), StatusCode::OK);

    // Verify booked
    let slots_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(slots_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let slots: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(slots[0]["is_available"], false);

    // Delete
    let delete_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/slots/{}", slot_id))
                .method("DELETE")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_resp.status(), StatusCode::OK);

    // Verify deleted
    let slots_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(slots_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let slots: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(slots.is_empty());
}

// ── Static files ────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn handler_static_index() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/index.html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn handler_static_js() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/js/main.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
