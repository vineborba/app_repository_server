use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Mongo failed to complete operation {}", .0)]
    MongoError(#[from] mongodb::error::Error),
    #[error("Failed to decode form-data field")]
    MultipartError(#[from] axum::extract::multipart::MultipartError),
    #[error("Image error")]
    ImageError(#[from] image::ImageError),
    #[error("QrCode error")]
    QrCodeError(#[from] qrcode_generator::QRCodeError),
    #[error("Invalid iOS metadata")]
    InvalidIosMetadata,
    #[error("Failed to insert data")]
    FailedInsertion,
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
    #[error("Invalid ObjectId")]
    ObjectIdParsingError(#[from] mongodb::bson::oid::Error),
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
