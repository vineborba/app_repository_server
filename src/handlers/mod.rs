use axum::{
    async_trait,
    body::{self, BoxBody, Full},
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify,
};

use crate::{error::AppError, models::user::Claims};

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
            AppError::UserAlreadyRegistered => (
                StatusCode::BAD_REQUEST,
                "User already registered".to_string(),
            ),
            AppError::InvalidIosMetadata => {
                (StatusCode::BAD_REQUEST, "Invalid iOS metadata".to_string())
            }
            AppError::FileMissing => (StatusCode::BAD_REQUEST, "File is missing".to_string()),
            AppError::ObjectIdParsingError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::ImageError(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Couldn't parse image".to_string(),
            ),
            AppError::QrCodeError(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Couldn't generate QrCode".to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unkown error".to_string(),
            ),
        };

        Response::builder()
            .status(status)
            .body(body::boxed(Full::from(message)))
            .expect("Couldn't create error response")
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let secret = env::var("JWT_SECRET").expect("JWT_SECRET is not set!");
        let key = DecodingKey::from_secret(secret.as_ref());
        let token_data = decode::<Claims>(bearer.token(), &key, &Validation::default())
            .map_err(|_| AppError::Unauthorized)?;

        Ok(token_data.claims)
    }
}

pub(super) struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
