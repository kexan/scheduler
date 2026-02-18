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
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
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
