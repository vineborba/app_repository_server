use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mongodb::{
    options::{FindOptions, InsertOneOptions},
    Client, Collection,
};

use crate::{
    error::AppError,
    models::user::{CreateUser, User},
};

const DB_NAME: &str = "appdist";
const COLLECTION_NAME: &str = "users";

/// List all users
///
/// List all users in the database.
#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    responses(
        (status = 200, description = "Listed users successfully", body = [User])
    )
)]
pub(crate) async fn get_users(State(client): State<Client>) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let options = FindOptions::default();
    let mut cursor = coll.find(None, options).await?;

    let mut rows: Vec<User> = Vec::new();

    while cursor.advance().await? {
        rows.push(cursor.deserialize_current()?)
    }

    Ok((StatusCode::OK, Json(rows)).into_response())
}

/// Create new user
///
/// Tries to create a new user item database or fails with 400 if it can't be done.
#[utoipa::path(
    post,
    path = "/users",
    tag = "Users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created successfully", body = CreateUser),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn create_user(
    State(client): State<Client>,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let new_user = User::new(payload);

    let options = InsertOneOptions::default();
    coll.insert_one(&new_user, options).await?;

    Ok((StatusCode::CREATED, Json(new_user)).into_response())
}
