use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use mongodb::{
    options::{FindOneOptions, FindOptions, InsertOneOptions},
    Client, Collection,
};

use crate::{
    error::AppError,
    models::{
        project::{CreateProject, Project},
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
    let project = coll.find_one(filter, options).await?;

    match project {
        Some(p) => Ok((StatusCode::OK, Json(p)).into_response()),
        None => Err(AppError::NotFound),
    }
}

/// Create new project
///
/// Tries to create a new project database or fails with 400 if it can't be done.
#[utoipa::path(
    post,
    path = "/projects",
    request_body = CreateProject,
    tag = "Projects",
    responses(
        (status = 201, description = "Project created successfully", body = Project),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn create_project(
    State(client): State<Client>,
    claims: Claims,
    Json(payload): Json<CreateProject>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Project> = client
        .database(DB_NAME)
        .collection::<Project>(COLLECTION_NAME);
    let new_project = Project::new(payload, claims.user_id);
    let options = InsertOneOptions::default();
    coll.insert_one(&new_project, options).await?;

    Ok((StatusCode::CREATED, Json(new_project)).into_response())
}
