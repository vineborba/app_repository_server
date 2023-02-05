use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use mongodb::{
    options::{FindOneAndUpdateOptions, FindOneOptions, FindOptions, InsertOneOptions},
    Client, Collection,
};

use crate::{
    error::AppError,
    models::{
        project::{BaseProjectInput, Project},
        user::Claims,
    },
};

const DB_NAME: &str = "appdist";
const COLLECTION_NAME: &str = "projects";

/// List all projects
///
/// List all projects in the database.
#[utoipa::path(
    get,
    path = "/projects",
    tag = "Projects",
    responses(
        (status = 200, description = "Listed projects successfully", body = [Project])
    )
)]
pub(crate) async fn get_projects(
    State(client): State<Client>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Project> = client
        .database(DB_NAME)
        .collection::<Project>(COLLECTION_NAME);

    let options = FindOptions::default();
    let mut cursor = coll.find(None, options).await?;

    let mut rows: Vec<Project> = Vec::new();

    while cursor.advance().await? {
        rows.push(cursor.deserialize_current()?);
    }

    Ok((StatusCode::OK, Json(rows)).into_response())
}

/// Get project data
///
/// Get a single project data.
#[utoipa::path(
    get,
    path = "/projects/:project_id",
    tag = "Projects",
    params(
        ("project_id" = String, Path, description = "id of the requested project data")
    ),
    responses(
        (status = 200, description = "Found project successfully", body = Project),
        (status = 404, description = "Project not found")
    )
)]
pub(crate) async fn get_project(
    State(client): State<Client>,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Project> = client
        .database(DB_NAME)
        .collection::<Project>(COLLECTION_NAME);

    let options = FindOneOptions::default();
    let oid = ObjectId::parse_str(project_id)?;
    let filter = doc! { "_id": oid };
    match coll.find_one(filter, options).await? {
        Some(project) => Ok((StatusCode::OK, Json(project)).into_response()),
        None => Err(AppError::NotFound),
    }
}

/// Create new project
///
/// Tries to create a new project database or fails with 400 if it can't be done.
#[utoipa::path(
    post,
    path = "/projects",
    request_body = BaseProjectInput,
    tag = "Projects",
    responses(
        (status = 201, description = "Project created successfully", body = Project),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn create_project(
    State(client): State<Client>,
    claims: Claims,
    Json(payload): Json<BaseProjectInput>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Project> = client
        .database(DB_NAME)
        .collection::<Project>(COLLECTION_NAME);
    let new_project = Project::new(payload, claims.user_id);
    let options = InsertOneOptions::default();
    coll.insert_one(&new_project, options).await?;

    Ok((StatusCode::CREATED, Json(new_project)).into_response())
}

/// Update project
///
/// Tries to update a project in database. Fails with 400 if it can't be done or with 404 if not found.
#[utoipa::path(
    patch,
    path = "/projects/:project_id",
    request_body = BaseProjectInput,
    tag = "Projects",
    params(
        ("project_id" = String, Path, description = "id of the requested project to be update")
    ),
    responses(
        (status = 204, description = "Project updated successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Not Found")
    )
)]
pub(crate) async fn update_project(
    State(client): State<Client>,
    Path(project_id): Path<String>,
    Json(payload): Json<BaseProjectInput>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Project> = client
        .database(DB_NAME)
        .collection::<Project>(COLLECTION_NAME);

    let oid = ObjectId::parse_str(project_id)?;
    let filter = doc! { "_id": oid };

    let updated_platforms = bson::to_bson(&payload.platforms).unwrap();
    let update = doc! {
        "$set": doc! {
            "name": payload.name,
            "description": payload.description,
            "platforms": updated_platforms
        }
    };

    let options = FindOneAndUpdateOptions::default();

    match coll.find_one_and_update(filter, update, options).await? {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(AppError::NotFound),
    }
}
