use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Surreal failed to complete operation {}", .0)]
    SurrealError(#[from] surrealdb::Error),
    #[error("Surreal failed to complete operation {}", .0)]
    SurrealInternalError(#[from] surrealdb::err::Error),
    #[error("Failed to decode form-data field")]
    MultipartError(#[from] axum::extract::multipart::MultipartError),
    #[error("Image error")]
    ImageError(#[from] image::ImageError),
    #[error("QrCode error")]
    QrCodeError(#[from] qrcode_generator::QRCodeError),
    #[error("Invalid iOS metadata")]
    InvalidIosMetadata,
    #[error("File missing")]
    FileMissing,
    #[error("Not found")]
    NotFound,
    #[error("User already registered")]
    UserAlreadyRegistered,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("System Time")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
    #[error("Unknown error")]
    AxumError(#[from] axum::http::Error),
    #[error("Unknown error")]
    Unspecified(#[from] ring::error::Unspecified),
    #[error("Unknown error")]
    Decode(#[from] data_encoding::DecodeError),
    #[error("Unknown error")]
    Encode(#[from] jsonwebtoken::errors::Error),
    #[error("Unknown error")]
    Never, // kinda like Typescript never type
}
