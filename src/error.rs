use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use yougile_api_client::YougileError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Invalid time format: {0}")]
    InvalidTimeFormat(String),

    #[error("Invalid time range: start must be before end")]
    InvalidTimeRange,

    #[error("Slot not found")]
    SlotNotFound,

    #[error("Slot already booked")]
    SlotAlreadyBooked,

    #[error("Yougile error: {0}")]
    Yougile(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::SlotNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::SlotAlreadyBooked => (StatusCode::CONFLICT, self.to_string()),
            AppError::InvalidTimeRange => (StatusCode::BAD_REQUEST, self.to_string()),
            other => {
                tracing::error!("Internal server error: {:?}", other);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Внутренняя ошибка сервера".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

impl From<YougileError> for AppError {
    fn from(e: YougileError) -> Self {
        AppError::Yougile(e.to_string())
    }
}
pub trait IsFatalError {
    fn is_fatal(&self) -> bool;
}

impl IsFatalError for AppError {
    fn is_fatal(&self) -> bool {
        match self {
            AppError::Io(_) | AppError::Json(_) => true,
            AppError::InvalidUuid(_)
            | AppError::InvalidTimeFormat(_)
            | AppError::InvalidTimeRange => true,
            AppError::SlotNotFound | AppError::SlotAlreadyBooked => true,
            AppError::Unauthorized => true,
            AppError::Yougile(msg) => {
                let msg_lower = msg.to_lowercase();
                if msg_lower.contains("401") || msg_lower.contains("unauthorized") {
                    return true;
                }
                if msg_lower.contains("403") || msg_lower.contains("forbidden") {
                    return true;
                }
                if msg_lower.contains("400") || msg_lower.contains("bad request") {
                    return true;
                }
                if msg_lower.contains("404") || msg_lower.contains("not found") {
                    return true;
                }
                false
            }
            AppError::Other(_) => false,
        }
    }
}
