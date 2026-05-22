use crate::error::AppError;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, info};
use uuid::Uuid;

const SESSION_DURATION: Duration = Duration::from_secs(3600);

#[derive(Clone)]
pub struct AdminToken {
    tokens: Arc<RwLock<HashMap<[u8; 32], DateTime<Utc>>>>,
}

impl Default for AdminToken {
    fn default() -> Self {
        Self::new()
    }
}

impl AdminToken {
    pub fn new() -> Self {
        let tokens = Arc::new(RwLock::new(HashMap::new()));
        let tokens_weak = Arc::downgrade(&tokens);

        tokio::spawn(async move {
            loop {
                time::sleep(SESSION_DURATION).await;

                let Some(tokens_clone) = tokens_weak.upgrade() else {
                    debug!("AdminToken dropped, stopping session cleanup task");
                    break;
                };

                let now = Utc::now();
                let removed = {
                    let mut guard = tokens_clone.write();
                    let before = guard.len();
                    guard.retain(|_, expires| *expires > now);
                    before - guard.len()
                };

                if removed > 0 {
                    info!("Cleaned up {} expired sessions", removed);
                }
            }
        });

        Self { tokens }
    }

    fn hash_token(token: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hasher.finalize().into()
    }

    pub fn create_session(&self) -> String {
        let token = Uuid::new_v4().to_string();
        let hash = Self::hash_token(&token);

        let chrono_dur = chrono::Duration::from_std(SESSION_DURATION)
            .unwrap_or_else(|_| chrono::Duration::hours(1));
        let expires = Utc::now() + chrono_dur;

        self.tokens.write().insert(hash, expires);

        format!(
            "admin_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/",
            token,
            SESSION_DURATION.as_secs()
        )
    }

    pub fn verify(&self, token: &str) -> bool {
        let hash = Self::hash_token(token);

        self.tokens
            .read()
            .get(&hash)
            .map(|expires| Utc::now() <= *expires)
            .unwrap_or(false)
    }

    pub fn check_auth(&self, cookie_header: Option<&str>) -> Result<(), AppError> {
        let cookie_header = cookie_header.unwrap_or_default();

        let token = cookie_header.split(';').find_map(|cookie| {
            let (key, value) = cookie.trim().split_once('=')?;
            if key == "admin_token" {
                Some(value)
            } else {
                None
            }
        });

        match token {
            Some(token) if self.verify(token) => Ok(()),
            _ => Err(AppError::Unauthorized),
        }
    }

    pub fn logout(&self, token: &str) {
        let hash = Self::hash_token(token);
        if self.tokens.write().remove(&hash).is_some() {
            info!("Session logged out");
        }
    }
}
