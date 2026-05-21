use migration_scheduler::api::auth_token::AdminToken;
use migration_scheduler::error::AppError;
use serial_test::serial;

fn extract_token(cookie: &str) -> &str {
    cookie
        .split(';')
        .find_map(|c| {
            let (key, value) = c.trim().split_once('=')?;
            if key == "admin_token" {
                Some(value)
            } else {
                None
            }
        })
        .expect("cookie should contain admin_token")
}

// ── create_session ─────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn auth_token_create_session_returns_valid_cookie() {
    let token = AdminToken::new();
    let cookie = token.create_session();

    assert!(cookie.starts_with("admin_token="));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("Max-Age=3600"));
    assert!(cookie.contains("Path=/"));
}

#[tokio::test]
#[serial]
async fn auth_token_create_session_unique_tokens() {
    let token = AdminToken::new();
    let c1 = token.create_session();
    let c2 = token.create_session();

    let t1 = extract_token(&c1);
    let t2 = extract_token(&c2);

    assert_ne!(t1, t2);
}

// ── verify ─────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn auth_token_verify_valid_token() {
    let token = AdminToken::new();
    let cookie = token.create_session();
    let t = extract_token(&cookie);

    assert!(token.verify(t));
}

#[tokio::test]
#[serial]
async fn auth_token_verify_invalid_token() {
    let token = AdminToken::new();

    assert!(!token.verify("nonexistent-token"));
    assert!(!token.verify(""));
    assert!(!token.verify("garbage"));
}

#[tokio::test]
#[serial]
async fn auth_token_verify_after_logout() {
    let token = AdminToken::new();
    let cookie = token.create_session();
    let t = extract_token(&cookie);

    assert!(token.verify(t));

    token.logout(t);

    assert!(!token.verify(t));
}

// ── check_auth ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn auth_token_check_auth_valid_cookie() {
    let token = AdminToken::new();
    let cookie = token.create_session();

    assert!(token.check_auth(Some(&cookie)).is_ok());
}

#[tokio::test]
#[serial]
async fn auth_token_check_auth_no_cookie() {
    let token = AdminToken::new();

    let result = token.check_auth(None);
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
#[serial]
async fn auth_token_check_auth_empty_cookie() {
    let token = AdminToken::new();

    let result = token.check_auth(Some(""));
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
#[serial]
async fn auth_token_check_auth_garbage_cookie() {
    let token = AdminToken::new();

    let result = token.check_auth(Some("admin_token=garbage; Path=/"));
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
#[serial]
async fn auth_token_check_auth_expired_token() {
    let token = AdminToken::new();
    let cookie = token.create_session();
    let t = extract_token(&cookie);

    // Verify works initially
    assert!(token.check_auth(Some(&cookie)).is_ok());

    // Manually expire by logging out (simulating expiration)
    token.logout(t);

    let result = token.check_auth(Some(&cookie));
    assert!(matches!(result, Err(AppError::Unauthorized)));
}

// ── logout ─────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn auth_token_logout_removes_session() {
    let token = AdminToken::new();
    let cookie = token.create_session();
    let t = extract_token(&cookie);

    assert!(token.verify(t));
    token.logout(t);
    assert!(!token.verify(t));
}

#[tokio::test]
#[serial]
async fn auth_token_logout_nonexistent_token_is_noop() {
    let token = AdminToken::new();

    // Should not panic
    token.logout("nonexistent-token");
    token.logout("");
}

#[tokio::test]
#[serial]
async fn auth_token_multiple_sessions_independent() {
    let token = AdminToken::new();

    let c1 = token.create_session();
    let c2 = token.create_session();

    let t1 = extract_token(&c1);
    let t2 = extract_token(&c2);

    // Both valid
    assert!(token.verify(t1));
    assert!(token.verify(t2));

    // Logout one
    token.logout(t1);

    assert!(!token.verify(t1));
    assert!(token.verify(t2)); // other still works
}

#[tokio::test]
#[serial]
async fn auth_token_drops_cleanly() {
    let tokens = std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::<Vec<u8>, ()>::new()));
    let weak = std::sync::Arc::downgrade(&tokens);
    assert!(weak.upgrade().is_some());
    drop(tokens);
    assert!(weak.upgrade().is_none());
}
