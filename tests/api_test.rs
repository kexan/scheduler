use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use migration_scheduler::api::routes;
use migration_scheduler::scheduler::Scheduler;
use migration_scheduler::yougile::YougileClient;
use std::sync::Arc;
use tower::ServiceExt;

async fn create_test_app() -> axum::Router {
    let scheduler = Scheduler::new().await.unwrap();
    let yougile = YougileClient::new().await.unwrap();
    routes(Arc::new(scheduler), Arc::new(yougile))
}

#[tokio::test]
async fn test_get_slots() {
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
    let slots: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(slots.is_array());
}

#[tokio::test]
async fn test_create_slot_with_auth() {
    let app = create_test_app().await;

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

    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .to_string();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from(
                    r#"{"date":"2026-02-20","start_time":"10:00","end_time":"11:00"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_admin_invalid_password() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/admin")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"password":"wrongpassword"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    eprintln!("Status: {:?}", response.status());

    assert!(
        response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn test_auth_admin_valid_password() {
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
async fn test_check_auth_not_authenticated() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/check")
                .method("GET")
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
async fn test_protected_route_without_auth_returns_401() {
    let app = create_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/slots")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"date":"2026-02-20","start_time":"10:00","end_time":"11:00"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_yougile_settings() {
    let app = create_test_app().await;

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

    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .to_string();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/yougile/settings")
                .method("GET")
                .header("cookie", cookie)
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
        "api_token should not be exposed in response"
    );
}

#[tokio::test]
async fn test_static_index() {
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
async fn test_static_js() {
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
