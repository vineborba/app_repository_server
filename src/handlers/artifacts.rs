use axum::{
    body::{self, boxed, Bytes, StreamBody},
    extract::{Multipart, Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use bson::oid::ObjectId;
use mongodb::{
    bson::doc,
    options::{AggregateOptions, FindOneOptions, FindOptions, InsertOneOptions, UpdateOptions},
    Client, Collection,
};
use qrcode_generator::QrCodeEcc;
use tokio_util::io::ReaderStream;

use crate::{
    error::AppError,
    helpers::{
        artifact::{
            create_file_url, create_itms_service_url, parse_plist_template, write_file_to_disk,
        },
        base64::encode_base64,
    },
    models::artifact::{
        Artifact, ArtifactExtensions, ArtifactToCreate, CreateArtifact, IosMetadata,
    },
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
    let mut ios_metadata = IosMetadata::default();
    let mut file_content: Option<Bytes> = None;
    while let Some(field) = payload.next_field().await? {
        match field.name() {
            Some("branch") => artifact_to_create.branch = Some(field.text().await?),
            Some("identifier") => artifact_to_create.identifier = Some(field.text().await?),
            Some("bundle_identifier") => ios_metadata.bundle_identifier = field.text().await?,
            Some("bundle_version") => ios_metadata.bundle_version = field.text().await?,
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

                let bytes = field.bytes().await?;
                artifact_to_create.size = Some(bytes.len());
                file_content = Some(bytes);
            }
            _ => (),
        }
    }

    if let Some(e) = &artifact_to_create.extension {
        match e {
            ArtifactExtensions::Ipa
                if ios_metadata.bundle_identifier.is_empty()
                    || ios_metadata.bundle_version.is_empty() =>
            {
                return Err(AppError::InvalidIosMetadata);
            }
            ArtifactExtensions::Ipa => {
                artifact_to_create.metadata = Some(ios_metadata);
            }
            _ => (),
        }
    }

    if let None = file_content {
        return Err(AppError::FileMissing);
    }

    let artifact_to_create = ArtifactToCreate::new(artifact_to_create, project_id)?;
    let coll: Collection<Artifact> = client
        .database(DB_NAME)
        .collection::<Artifact>(COLLECTION_NAME);
    let new_artifact = Artifact::new(artifact_to_create)?;

    let options = InsertOneOptions::default();
    let insert_result = coll.insert_one(&new_artifact, options).await?;

    if let Some(oid) = insert_result.inserted_id.as_object_id() {
        let inserted_id = oid.to_string();
        let url = match new_artifact.get_extension() {
            ArtifactExtensions::Ipa => create_itms_service_url(inserted_id),
            _ => create_file_url(inserted_id),
        };
        let qrcode = qrcode_generator::to_svg_to_string(url, QrCodeEcc::Low, 240, None::<&str>)?;
        let mut encoded_code = String::from("data:image/svg+xml;base64,");
        encoded_code.push_str(encode_base64(qrcode.as_bytes())?.as_str());
        let filter = doc! { "_id": oid };
        let update = doc! { "$set": doc! { "qrcode": encoded_code } };
        let options = UpdateOptions::default();
        coll.update_one(filter, update, options).await?;
        if let Some(file_content) = file_content {
            let content_vec = file_content.to_vec();
            write_file_to_disk(new_artifact.get_path(), content_vec.as_slice())?;
        }

        Ok((StatusCode::CREATED, Json(new_artifact)).into_response())
    } else {
        Err(AppError::FailedInsertion)
    }
}

/// List artifacts by project
///
/// List all artifacts that belongs to a project.
#[utoipa::path(
    get,
    tag = "Projects",
    path = "/projects/{project_id}/artifacts",
    params(
        ("project_id" = String, Path, description = "id of the project that the artifact belongs to")
    ),
    responses(
        (status = 200, description = "Found artifacts", body = [Artifact]),
    )
)]
pub(crate) async fn list_project_artifacts(
    State(client): State<Client>,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Artifact> = client
        .database(DB_NAME)
        .collection::<Artifact>(COLLECTION_NAME);

    let pipeline = vec![
        doc! {
            "$match": doc! {
              "projectId": project_id,
            },
        },
        doc! {
          "$project": doc! {
            "_id": 1,
            "type": 1,
            "createdAt": 1,
            "branch": 1,
            "originalFilename": 1,
            "identifier": 1,
            "iosMetadata": 1,
            "qrcode": 1,
          },
        },
        doc! {
          "$group": doc! {
            "_id": "$branch",
            "artifacts": {
              "$push": "$$ROOT",
            },
          },
        },
    ];
    let options = AggregateOptions::default();
    let mut cursor = coll.aggregate(pipeline, options).await?;

    let mut rows = Vec::new();

    while cursor.advance().await? {
        rows.push(cursor.deserialize_current()?);
    }

    Ok((StatusCode::OK, Json(rows)).into_response())
}

/// Download artifact
///
/// Download artifact from server.
#[utoipa::path(
    get,
    path = "/artifacts/{artifact_id}/download",
    tag = "Artifacts",
    params(
        ("artifact_id" = String, Path, description = "id of the artifact")
    ),
    responses(
        (status = 200, description = "Downloaded successfully", body = ArtifactBinary, content_type = "application/octet-stream")
    )
)]
pub(crate) async fn download_artifact(
    State(client): State<Client>,
    Path(artifact_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Artifact> = client
        .database(DB_NAME)
        .collection::<Artifact>(COLLECTION_NAME);

    let options = FindOneOptions::default();
    let oid = ObjectId::parse_str(artifact_id)?;
    let filter = doc! { "_id": oid };
    if let Some(artifact) = coll.find_one(filter, options).await? {
        let path = artifact.get_path();
        let file = tokio::fs::File::open(path).await?;
        let stream = ReaderStream::new(file);
        let body = StreamBody::new(stream);
        let response = Response::builder()
            .status(StatusCode::OK)
            .body(boxed(body))?;

        Ok(response.into_response())
    } else {
        Err(AppError::NotFound)
    }
}

/// Fetch downlaod headers
///
/// Fetch download headers needed to download the artifact.
#[utoipa::path(
    head,
    path = "/artifacts/{artifact_id}/download",
    tag = "Artifacts",
    params(
        ("artifact_id" = String, Path, description = "id of the artifact")
    ),
    responses(
        (status = 200, description = "Fetched download data successfully")
    )
)]
pub(crate) async fn get_download_headers(
    State(client): State<Client>,
    Path(artifact_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Artifact> = client
        .database(DB_NAME)
        .collection::<Artifact>(COLLECTION_NAME);

    let options = FindOneOptions::default();
    let oid = ObjectId::parse_str(artifact_id)?;
    let filter = doc! { "_id": oid };
    if let Some(artifact) = coll.find_one(filter, options).await? {
        let (original_filename, mime_type, size) = artifact.get_download_data();
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime_type)
            .header("content-length", size.to_string())
            .header(
                "Content-Disposition",
                format!("attachment; filename={original_filename}"),
            )
            .body(body::Empty::new())?;

        Ok(response.into_response())
    } else {
        Err(AppError::NotFound)
    }
}

/// Generate artifact iOS plist
///
/// Generate artifact iOS plist and returns it.
#[utoipa::path(
    get,
    path = "/artifacts/{artifact_id}/ios-plist",
    tag = "Artifacts",
    params(
        ("artifact_id" = String, Path, description = "id of the artifact")
    ),
    responses(
        (status = 200, description = "Generated plist successfully", body = String)
    )
)]
pub(crate) async fn get_ios_plist(
    State(client): State<Client>,
    Path(artifact_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let coll: Collection<Artifact> = client
        .database(DB_NAME)
        .collection::<Artifact>(COLLECTION_NAME);

    let options = AggregateOptions::default();
    let oid = ObjectId::parse_str(artifact_id)?;
    let pipeline = vec![
        doc! {
            "$match": doc! { "_id": oid }
        },
        doc! {
            "$addFields": doc! {
                "convertedProjectId": doc! { "$toObjectId": "$projectId" }
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "projects",
                "localField": "convertedProjectId",
                "foreignField": "_id",
                "as": "project"
            }
        },
        doc! {
            "$unwind": "$project"
        },
        doc! {
            "$limit": 1,
        },
    ];

    let artifact = coll
        .aggregate(pipeline, options)
        .await?
        .with_type::<Artifact>()
        .deserialize_current()?;

    let (artifact_id, bundle_identifier, bundle_version, app_name) = artifact.get_plist_data()?;

    let plist = parse_plist_template(&artifact_id, &bundle_identifier, &bundle_version, &app_name);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml")
        .header("Content-Disposition", "attachment; filename=ios.plist")
        .body(plist)?;

    Ok(response.into_response())
}
