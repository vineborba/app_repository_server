use axum::{
    body::{self, BoxBody, Full},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::error::AppError;

pub(super) mod projects;
pub(super) mod users;

impl IntoResponse for AppError {
    fn into_response(self) -> Response<BoxBody> {
        let (status, message) = match self {
            AppError::MongoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.kind),
        };

        Response::builder()
            .status(status)
            .body(body::boxed(Full::from(format!("{message}"))))
            .expect("couldn't create error response")
    }
}
