use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};

pub enum WebError {
    /// Something went wrong generating the quiz
    Internal(String),
    /// Session not found or expired
    NoSession,
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        match self {
            WebError::Internal(msg) => {
                tracing::error!("{msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
            WebError::NoSession => Redirect::to("/").into_response(),
        }
    }
}

impl From<quizgen_core::QuizgenError> for WebError {
    fn from(e: quizgen_core::QuizgenError) -> Self {
        WebError::Internal(e.to_string())
    }
}
