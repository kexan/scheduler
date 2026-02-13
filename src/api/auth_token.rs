use crate::error::AppError;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AdminToken {
    tokens: Arc<RwLock<HashSet<String>>>,
}

impl AdminToken {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn create_session(&self) -> (String, String) {
        let token = format!("admin_token_{}", chrono::Utc::now().timestamp());
        self.tokens.write().await.insert(token.clone());

        let cookie = format!(
            "admin_token={}; HttpOnly; SameSite=Lax; Max-Age=3600; Path=/",
            token
        );

        (token, cookie)
    }

    pub async fn verify(&self, token: &str) -> bool {
        self.tokens.read().await.contains(token)
    }

    pub async fn check_auth(&self, cookie_header: Option<String>) -> Result<(), AppError> {
        let cookie_header = cookie_header.unwrap_or_default();
        if cookie_header.is_empty() {
            return Err(AppError::Unauthorized);
        }

        let token = cookie_header.split(';').find_map(|cookie| {
            let parts = cookie.trim().split_once('=');
            if let Some((key, value)) = parts {
                if key.trim() == "admin_token" {
                    Some(value.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        });

        match token {
            Some(token) => {
                if self.verify(&token).await {
                    Ok(())
                } else {
                    Err(AppError::Unauthorized)
                }
            }
            None => Err(AppError::Unauthorized),
        }
    }
}

impl Default for AdminToken {
    fn default() -> Self {
        Self::new()
    }
}
