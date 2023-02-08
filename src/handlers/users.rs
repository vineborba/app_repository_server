use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use mongodb::{
    bson::oid::ObjectId,
    error::{ErrorKind, WriteError, WriteFailure},
    options::{FindOneOptions, FindOptions, InsertOneOptions, UpdateOptions},
    Client, Collection,
};

use crate::{
    error::AppError,
    models::user::{
        AuthOutput, Claims, CreateUserInput, LoginInput, UpdateFavoriteProjectsInput, User,
        UserOutput,
    },
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
    ),
    security(
        ("jwt_auth" = [])
    ),
)]
pub(crate) async fn get_user_data(
    State(client): State<Client>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let options = FindOneOptions::default();
    let oid = ObjectId::parse_str(claims.user_id)?;
    let filter = doc! { "_id": oid };
    match coll.find_one(filter, options).await? {
        Some(user) => Ok((StatusCode::OK, Json(UserOutput::new(user))).into_response()),
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
    request_body = CreateUserInput,
    responses(
        (status = 201, description = "User created successfully", body = AuthOutput),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn create_user(
    State(client): State<Client>,
    Json(payload): Json<CreateUserInput>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let new_user = User::new(payload)?;

    let options = InsertOneOptions::default();
    match coll.insert_one(&new_user, options).await {
        Ok(_) => {
            let response = AuthOutput::new(new_user.email, new_user.id)?;
            Ok((StatusCode::CREATED, Json(response)).into_response())
        }
        Err(e) => match *e.kind.to_owned() {
            ErrorKind::Write(WriteFailure::WriteError(WriteError { code: 11000, .. })) => {
                Err(AppError::UserAlreadyRegistered)
            }
            _ => Err(AppError::MongoError(e)),
        },
    }
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

/// Edit favorite projects
///
/// Adds or removes a project from user's favorite projects list
#[utoipa::path(
    patch,
    path = "/users/favorite-projects",
    tag = "Users",
    request_body = LoginInput,
    responses(
        (status = 204, description = "User logged in successfully"),
        (status = 400, description = "Bad Request"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("jwt_auth" = [])
    ),
)]
pub(crate) async fn edit_favorite_projects(
    State(client): State<Client>,
    claims: Claims,
    Json(payload): Json<UpdateFavoriteProjectsInput>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<User> = client.database(DB_NAME).collection::<User>(COLLECTION_NAME);

    let options = FindOneOptions::default();

    let oid = ObjectId::parse_str(claims.user_id)?;
    let filter = doc! { "_id": oid };

    let user = match coll.find_one(filter.clone(), options).await? {
        Some(u) => u,
        None => return Err(AppError::Forbidden),
    };

    let options = UpdateOptions::default();
    let update;
    if user
        .favorite_projects
        .iter()
        .any(|id| id.eq(&payload.project_id))
    {
        update = doc! {
            "$pull": doc! {
                "favoriteProjects": payload.project_id
            }
        };
    } else {
        update = doc! {
            "$addToSet": doc! {
                "favoriteProjects": payload.project_id
            }
        };
    }
    coll.update_one(filter, update, options).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
