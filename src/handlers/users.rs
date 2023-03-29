use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{
    error::AppError,
    repositories::user::UserRepository,
    schemas::user::{
        AuthOutput, Claims, CreateUserInput, LoginInput, UpdateFavoriteProjectsInput, User,
        UserOutput,
    },
};

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
pub(crate) async fn get_users(
    State(repository): State<UserRepository>,
) -> Result<impl IntoResponse, AppError> {
    let users = repository.get_all().await?;

    Ok((StatusCode::OK, Json(users)))
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
    State(repository): State<UserRepository>,
    claims: Claims,
) -> Result<impl IntoResponse, AppError> {
    if let Some(user) = repository.get(claims.user_id.as_str()).await? {
        Ok((StatusCode::OK, Json(UserOutput::new(user))))
    } else {
        Err(AppError::Forbidden)
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
    State(repository): State<UserRepository>,
    Json(payload): Json<CreateUserInput>,
) -> Result<impl IntoResponse, AppError> {
    let new_user = User::new(payload)?;
    let new_user = repository.create(new_user).await?;
    let response = AuthOutput::new(new_user.email, new_user.id)?;
    Ok((StatusCode::CREATED, Json(response)))
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
    State(repository): State<UserRepository>,
    Json(payload): Json<LoginInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = match repository.get_by_email(payload.email).await? {
        Some(u) => u,
        None => return Err(AppError::InvalidCredentials),
    };

    user.validate_password(payload.password)?;
    let response = AuthOutput::new(user.email, user.id)?;

    Ok((StatusCode::OK, Json(response)))
}

/// Edit favorite projects
///
/// Adds or removes a project from user's favorite projects list
#[utoipa::path(
    patch,
    path = "/users/favorite-projects",
    tag = "Users",
    request_body = UpdateFavoriteProjectsInput,
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
    State(repository): State<UserRepository>,
    claims: Claims,
    Json(payload): Json<UpdateFavoriteProjectsInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = match repository.get(&claims.user_id).await? {
        Some(u) => u,
        None => return Err(AppError::Forbidden),
    };

    if user
        .favorite_projects
        .iter()
        .any(|id| id.eq(payload.project_id.as_str()))
    {
        repository
            .remove_favorite_project(claims.user_id, payload.project_id)
            .await?;
    } else {
        repository
            .insert_favorite_project(claims.user_id, payload.project_id)
            .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}
