use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mongodb::{
    options::{FindOptions, InsertOneOptions},
    Client, Collection,
};

use crate::{
    error::AppError,
    models::project::{CreateProject, Project},
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
        (status = 200, description = "List projects successfully", body = [Project])
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

/// Create new project
///
/// Tries to create a new project item database or fails with 400 if it can't be done.
#[utoipa::path(
    post,
    path = "/projects",
    request_body = CreateProject,
    tag = "Projects",
    responses(
        (status = 201, description = "Project item created successfully", body = Project),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn create_project(
    State(client): State<Client>,
    Json(payload): Json<CreateProject>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Project> = client
        .database(DB_NAME)
        .collection::<Project>(COLLECTION_NAME);

    let new_project = Project::new(payload, String::from("placeholder"));

    let options = InsertOneOptions::default();
    coll.insert_one(&new_project, options).await?;

    Ok((StatusCode::CREATED, Json(new_project)).into_response())
}
