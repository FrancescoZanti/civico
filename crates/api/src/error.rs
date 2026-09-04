use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

/// A structured API error returned to clients in the form:
///
/// ```json
/// { "error": { "code": "...", "message": "...", "details": {} } }
/// ```
///
/// Database/`sqlx` internals are never leaked: they are mapped to a generic
/// internal message with the cause logged server-side only.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub details: Value,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details: Value::Null,
        }
    }

    pub fn unauthenticated() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "authentication is required",
        )
    }

    pub fn not_found(entity: &str, id: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("{entity} `{id}` not found"),
        )
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "conflict", message)
    }

    pub fn validation(message: impl Into<String>, details: Value) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "validation_error",
            message: message.into(),
            details,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = details;
        self
    }

    pub fn internal(context: &str) -> Self {
        tracing::error!("internal error while {context}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "an internal error occurred",
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({
            "error": {
                "code": self.code,
                "message": self.message,
                "details": self.details,
            }
        });
        (self.status, Json(body)).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}", self.status, self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Convenience result alias.
pub type ApiResult<T> = Result<T, ApiError>;
