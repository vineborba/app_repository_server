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
