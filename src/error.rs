use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Mongo failed to complete operation {}", .0)]
    MongoError(#[from] mongodb::error::Error),
    #[error("Failed to decode form-data field")]
    MultipartError(#[from] axum::extract::multipart::MultipartError),
    #[error("Not found")]
    NotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
    #[error("Invalid ObjectId")]
    ObjectIdParsingError(#[from] mongodb::bson::oid::Error),
    #[error("Unknown error")]
    Unspecified(#[from] ring::error::Unspecified),
    #[error("Unknown error")]
    Decode(#[from] data_encoding::DecodeError),
    #[error("Unknown error")]
    Encode(#[from] jsonwebtoken::errors::Error),
    #[error("Unknown error")]
    Never, // kinda like Typescript never type
}
