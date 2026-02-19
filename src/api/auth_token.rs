use crate::error::AppError;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct AdminToken {
    tokens: Arc<Mutex<HashMap<Vec<u8>, Instant>>>,
}

const SESSION_DURATION: Duration = Duration::from_secs(3600);

impl AdminToken {
    pub fn new() -> Self {
        let tokens = Arc::new(Mutex::new(HashMap::new()));

        let tokens_clone = tokens.clone();
        tokio::spawn(async move {
            loop {
                time::sleep(SESSION_DURATION).await;
                let now = Instant::now();
                let mut tokens_guard = tokens_clone.lock().await;
                let before = tokens_guard.len();
                tokens_guard.retain(|_, expires| *expires > now);
                let removed = before - tokens_guard.len();
                if removed > 0 {
                    info!("Cleaned up {} expired sessions", removed);
                } else {
                    debug!("No expired sessions to clean up");
                }
            }
        });

        Self { tokens }
    }

    fn hash_token(token: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hasher.finalize().to_vec()
    }

    pub async fn create_session(&self) -> String {
        let token = Uuid::new_v4().to_string();
        let hash = Self::hash_token(&token);
        let expires = Instant::now() + SESSION_DURATION;

        self.tokens.lock().await.insert(hash, expires);

        let max_age = SESSION_DURATION.as_secs();

        format!(
            "admin_token={}; HttpOnly; Secure; SameSite=Strict; Max-Age={}; Path=/",
            token, max_age
        )
    }

    pub async fn verify(&self, token: &str) -> bool {
        let hash = Self::hash_token(token);
        let mut tokens = self.tokens.lock().await;

        match tokens.get(&hash) {
            Some(expiration) => {
                if Instant::now() > *expiration {
                    tokens.remove(&hash);
                    false
                } else {
                    true
                }
            }
            None => false,
        }
    }

    pub async fn check_auth(&self, cookie_header: Option<String>) -> Result<(), AppError> {
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
            Some(token) if self.verify(token).await => Ok(()),
            _ => Err(AppError::Unauthorized),
        }
    }
}
