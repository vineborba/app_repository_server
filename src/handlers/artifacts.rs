use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use mongodb::{
    bson::doc,
    options::{FindOptions, InsertOneOptions},
    Client, Collection,
};

use crate::{
    error::AppError,
    models::artifact::{Artifact, ArtifactExtensions, ArtifactToCreate, CreateArtifact},
};

const DB_NAME: &str = "appdist";
const COLLECTION_NAME: &str = "artifacts";

/// List all artifacts
///
/// List all artifacts in the database.
#[utoipa::path(
    get,
    path = "/artifacts",
    tag = "Artifacts",
    responses(
        (status = 200, description = "Listed artifacts successfully", body = [Artifact])
    )
)]
pub(crate) async fn get_artifacts(
    State(client): State<Client>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Artifact> = client
        .database(DB_NAME)
        .collection::<Artifact>(COLLECTION_NAME);

    let options = FindOptions::default();
    let mut cursor = coll.find(None, options).await?;

    let mut rows: Vec<Artifact> = Vec::new();

    while cursor.advance().await? {
        rows.push(cursor.deserialize_current()?);
    }

    Ok((StatusCode::OK, Json(rows)).into_response())
}

/// Create new artifact
///
/// Tries to store a new artifact in disk and save relevant data in database or fails with 400 if it can't be done.
#[utoipa::path(
    post,
    tag = "Projects",
    path = "/projects/{project_id}/artifacts",
    request_body(content = CreateArtifactInput, description = "Artifact data", content_type = "multipart/form-data"),
    params(
        ("project_id" = String, Path, description = "id of the project that the artifact belongs to")
    ),
    responses(
        (status = 201, description = "Artifact created and stored successfully", body = Artifact),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn create_artifact(
    State(client): State<Client>,
    Path(project_id): Path<String>,
    mut payload: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut artifact_to_create = CreateArtifact::default();
    while let Some(field) = payload.next_field().await? {
        match field.name() {
            Some("branch") => artifact_to_create.branch = Some(field.text().await?),
            Some("identifier") => artifact_to_create.identifier = Some(field.text().await?),
            Some("file") => {
                let file_name = match field.file_name() {
                    Some(v) => v.to_string(),
                    None => return Err(AppError::Never),
                };
                artifact_to_create.original_filename = Some(file_name.clone());

                match file_name.get(file_name.len() - 3..) {
                    Some("apk") => artifact_to_create.extension = Some(ArtifactExtensions::Apk),
                    Some("ipa") => artifact_to_create.extension = Some(ArtifactExtensions::Ipa),
                    _ => artifact_to_create.extension = Some(ArtifactExtensions::Aab),
                }

                match field.content_type() {
                    Some(mime_type) => artifact_to_create.mime_type = Some(mime_type.to_string()),
                    None => return Err(AppError::Never),
                }

                artifact_to_create.size = Some(field.bytes().await?.len());
            }
            _ => (),
        }
    }

    let artifact_to_create = ArtifactToCreate::new(artifact_to_create, project_id)?;
    let coll: Collection<Artifact> = client
        .database(DB_NAME)
        .collection::<Artifact>(COLLECTION_NAME);
    let new_artifact = Artifact::new(artifact_to_create);

    let options = InsertOneOptions::default();
    coll.insert_one(&new_artifact, options).await?;
    Ok((StatusCode::CREATED, Json(new_artifact)).into_response())
}

/// List artifacts by project
///
/// List all artifacts that belongs to a project.
#[utoipa::path(
    get,
    tag = "Projects",
    path = "/projects/{project_id}/artifacts",
    request_body(content = CreateArtifactInput, description = "Artifact data", content_type = "multipart/form-data"),
    params(
        ("project_id" = String, Path, description = "id of the project that the artifact belongs to")
    ),
    responses(
        (status = 201, description = "Artifact created and stored successfully", body = Artifact),
        (status = 400, description = "Bad Request")
    )
)]
pub(crate) async fn list_project_artifacts(
    State(client): State<Client>,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Artifact> = client
        .database(DB_NAME)
        .collection::<Artifact>(COLLECTION_NAME);

    let filter = doc! { "projectId": project_id };
    let options = FindOptions::builder().sort(doc! { "createdAt": 1 }).build();
    let mut cursor = coll.find(filter, options).await?;

    let mut rows: Vec<Artifact> = Vec::new();

    while cursor.advance().await? {
        rows.push(cursor.deserialize_current()?);
    }

    Ok((StatusCode::OK, Json(rows)).into_response())
}
