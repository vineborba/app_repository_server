use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use mongodb::{
    bson::oid::ObjectId,
    options::{FindOneOptions, FindOptions, InsertOneOptions},
    Client, Collection,
};

use crate::{
    error::AppError,
    models::user::{AuthOutput, Claims, CreateUser, LoginInput, User, UserOutput},
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

/// Get user information
///
/// Get user information using the token in request header.
#[utoipa::path(
    get,
    path = "/users/me",
    tag = "Users",
    responses(
        (status = 200, description = "Returned user successfully", body = UserOutput),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn get_user_data(
    State(client): State<Client>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let options = FindOneOptions::default();
    let oid = ObjectId::parse_str(claims.user_id)?;
    let filter = doc! { "_id": oid };
    let user = coll.find_one(filter, options).await?;

    match user {
        Some(u) => Ok((StatusCode::OK, Json(UserOutput::new(u))).into_response()),
        None => Err(AppError::Unauthorized),
    }
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
        (status = 201, description = "User created successfully", body = AuthOutput),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn create_user(
    State(client): State<Client>,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let new_user = User::new(payload)?;

    let options = InsertOneOptions::default();
    coll.insert_one(&new_user, options).await?;
    let response = AuthOutput::new(new_user.email, new_user.id)?;
    Ok((StatusCode::CREATED, Json(response)).into_response())
}

/// Log in
///
/// Log in user into the platform
#[utoipa::path(
    post,
    path = "/users/login",
    tag = "Users",
    request_body = LoginInput,
    responses(
        (status = 201, description = "User logged in successfully", body = AuthOutput),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn login_user(
    State(client): State<Client>,
    Json(payload): Json<LoginInput>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let filter = doc! { "email": payload.email };
    let options = FindOneOptions::default();
    let user = coll.find_one(filter, options).await?;

    let user = match user {
        Some(u) => u,
        None => return Err(AppError::InvalidCredentials),
    };

    user.validate_password(payload.password)?;
    let response = AuthOutput::new(user.email, user.id)?;

    Ok((StatusCode::OK, Json(response)).into_response())
}
