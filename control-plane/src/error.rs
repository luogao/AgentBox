use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum AppError {
    DockerError(String),
    DatabaseError(String),
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DockerError(e) => write!(f, "Docker error: {}", e),
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::NotFound(e) => write!(f, "Not found: {}", e),
            AppError::BadRequest(e) => write!(f, "Bad request: {}", e),
            AppError::Unauthorized(e) => write!(f, "Unauthorized: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::DockerError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            AppError::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e.clone()),
            AppError::BadRequest(e) => (StatusCode::BAD_REQUEST, e.clone()),
            AppError::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e.clone()),
        };
        tracing::error!("Error: {}", self);
        (status, message).into_response()
    }
}
