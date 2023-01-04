use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mongodb::{
    options::{FindOptions, InsertOneOptions},
    Client, Collection,
};

use crate::models::users::{CreateUser, User};

const DB_NAME: &str = "appdist";
const COLLECTION_NAME: &str = "users";

pub async fn get_users(State(client): State<Client>) -> impl IntoResponse {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let options = FindOptions::default();
    let mut cursor = coll
        .find(None, options)
        .await
        .expect("failed to load users data");

    let mut rows: Vec<User> = Vec::new();

    while cursor.advance().await.expect("can't advance cursor") {
        rows.push(
            cursor
                .deserialize_current()
                .expect("can't deserialize user"),
        )
    }

    (StatusCode::OK, Json(rows)).into_response()
}

pub async fn create_user(State(client): State<Client>, Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let new_user = User::new(payload);

    let options = InsertOneOptions::default();
    coll.insert_one(&new_user, options)
        .await
        .expect("failed to insert user");

    (StatusCode::CREATED, Json(new_user)).into_response()
}
