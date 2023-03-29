use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use image::{imageops::FilterType, io::Reader as ImageReader, ImageOutputFormat};
use std::io::Cursor;

use crate::{
    error::AppError,
    helpers::base64::encode_base64,
    schemas::{project::{BaseProjectInput, Project},user::Claims},
    repositories::project::ProjectRepository,
};

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
    State(repository): State<ProjectRepository>,
) -> Result<impl IntoResponse, AppError> {
    let projects = repository.get_all().await?;
    Ok((StatusCode::OK, Json(projects)))
}

/// Get project data
///
/// Get a single project data.
#[utoipa::path(
    get,
    path = "/projects/{project_id}",
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
    State(repository): State<ProjectRepository>,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(p) = repository.get(project_id).await? {
        Ok((StatusCode::OK, Json(p)))
    } else {
        Err(AppError::NotFound)
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
    ),
    security(
        ("jwt_auth" = [])
    ), 
)]
pub(crate) async fn create_project(
    State(repository): State<ProjectRepository>,
    claims: Claims,
    Json(payload): Json<BaseProjectInput>,
) -> Result<impl IntoResponse, AppError> {
    let new_project = Project::new(payload, claims.user_id);
    let new_project = repository.create(new_project).await?;
    Ok((StatusCode::CREATED, Json(new_project)))
}

/// Update project
///
/// Tries to update a project in database. Fails with 400 if it can't be done or with 404 if not found.
#[utoipa::path(
    patch,
    path = "/projects/{project_id}",
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
    State(repository): State<ProjectRepository>,
    Path(project_id): Path<String>,
    Json(payload): Json<BaseProjectInput>,
) -> Result<impl IntoResponse, AppError> {
    repository.update(project_id, payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Update project image
///
/// Updates a project image, resizeing it to 160x160 and then saving it as a base64 encoded string.
#[utoipa::path(
    patch,
    tag = "Projects",
    path = "/projects/{project_id}/image",
    request_body(content = EditImageInput, description = "Image", content_type = "multipart/form-data"),
    params(
        ("project_id" = String, Path, description = "id of the project that the image belongs to")
    ),
    responses(
        (status = 204, description = "Image resized and stored successfully"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Not Found")
    )
)]
pub(crate) async fn update_project_image(
    State(repository): State<ProjectRepository>,
    Path(project_id): Path<String>,
    mut payload: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut encoded_image = String::from("data:image/png;base64,");
    while let Some(field) = payload.next_field().await? {
        match field.name() {
            Some("file") => match field.bytes().await {
                Ok(file) => {
                    let img = ImageReader::new(Cursor::new(file))
                        .with_guessed_format()?
                        .decode()?
                        .resize_to_fill(160, 160, FilterType::Nearest);
                    let mut img_buffer = vec![];
                    img.write_to(&mut Cursor::new(&mut img_buffer), ImageOutputFormat::Png)?;
                    encoded_image.push_str(encode_base64(img_buffer.as_slice())?.as_str());
                }
                Err(e) => return Err(AppError::MultipartError(e)),
            },
            _ => (),
        };
    }

    repository.update_image(project_id, Some(encoded_image)).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Remove project image
///
/// Removes a project image.
#[utoipa::path(
    delete,
    tag = "Projects",
    path = "/projects/{project_id}/image",
    params(
        ("project_id" = String, Path, description = "id of the project that the image belongs to")
    ),
    responses(
        (status = 204, description = "Image removed successfully"),
        (status = 404, description = "Not Found")
    )
)]
pub(crate) async fn remove_project_image(
    State(repository): State<ProjectRepository>,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    repository.update_image(project_id, None).await?;
    Ok(StatusCode::NO_CONTENT)
}
