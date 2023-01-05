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
