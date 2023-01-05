use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AppError {
    #[error("Mongo failed to complete operation {}", .0)]
    MongoError(#[from] mongodb::error::Error),
}
