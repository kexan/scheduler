use thiserror::Error;
use warp::{Rejection, Reply, reply::Response};

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

impl warp::reject::Reject for AppError {}

impl AppError {
    pub fn status_code(&self) -> warp::http::StatusCode {
        match self {
            AppError::Unauthorized => warp::http::StatusCode::UNAUTHORIZED,
            AppError::SlotNotFound => warp::http::StatusCode::NOT_FOUND,
            AppError::SlotAlreadyBooked => warp::http::StatusCode::CONFLICT,
            _ => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<Response, std::convert::Infallible> {
    if let Some(app_error) = err.find::<AppError>() {
        let status = app_error.status_code();
        let json = warp::reply::json(&serde_json::json!({
            "error": app_error.to_string()
        }));
        Ok(warp::reply::with_status(json, status).into_response())
    } else {
        let json = warp::reply::json(&serde_json::json!({
            "error": "Internal Server Error"
        }));
        Ok(warp::reply::with_status(
            json,
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response())
    }
}
