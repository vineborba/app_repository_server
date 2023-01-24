use axum::{
    body::{self, BoxBody, Full},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::error::AppError;

pub(super) mod artifacts;
pub(super) mod projects;
pub(super) mod users;

impl IntoResponse for AppError {
    fn into_response(self) -> Response<BoxBody> {
        let (status, message) = match self {
            AppError::MongoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::MultipartError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::InvalidCredentials => {
                (StatusCode::BAD_REQUEST, "Invalid credentials".to_string())
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unkown error".to_string(),
            ),
        };

        Response::builder()
            .status(status)
            .body(body::boxed(Full::from(format!("{message}"))))
            .expect("Couldn't create error response")
    }
}
