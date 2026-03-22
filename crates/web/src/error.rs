use quizgen_core::QuizgenError;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};

pub enum WebError {
    /// Invalid user input (bad quiz type, etc.)
    BadRequest(String),
    /// Session not found or expired
    NoSession,
    /// Invalid question index
    NotFound,
    /// Upstream API failure
    ServiceUnavailable,
    /// Unexpected server error
    Internal(String),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        match self {
            WebError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            WebError::NoSession => Redirect::to("/").into_response(),
            WebError::NotFound => StatusCode::NOT_FOUND.into_response(),
            WebError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE.into_response(),
            WebError::Internal(msg) => {
                tracing::error!("{msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}

impl From<QuizgenError> for WebError {
    fn from(e: QuizgenError) -> Self {
        match e {
            QuizgenError::ApiError(_) => WebError::ServiceUnavailable,
            QuizgenError::DataError => WebError::Internal("Data error".into()),
            QuizgenError::FileError(e) => WebError::Internal(e.to_string()),
        }
    }
}
