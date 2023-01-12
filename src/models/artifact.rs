use core::fmt;

use bson::serde_helpers::{
    deserialize_hex_string_from_object_id, serialize_hex_string_as_object_id,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use app_dist_server::create_file_path;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub enum ArtifactExtensions {
    IPA,
    APK,
    AAB,
}

impl fmt::Display for ArtifactExtensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    #[serde(
        rename = "_id",
        serialize_with = "serialize_hex_string_as_object_id",
        deserialize_with = "deserialize_hex_string_from_object_id"
    )]
    id: String,
    original_filename: String,
    branch: String,
    extension: ArtifactExtensions,
    path: String,
    project_id: String,
    mime_type: String,
    size: usize,
    identifier: String,
    qrcode: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
pub struct CreateArtifact {
    pub branch: Option<String>,
    pub identifier: Option<String>,
    pub mime_type: Option<String>,
    pub original_filename: Option<String>,
    pub extension: Option<ArtifactExtensions>,
    pub size: Option<usize>,
}

#[allow(dead_code)]
#[derive(ToSchema, Debug)]
pub struct CreateArtifactInput {
    branch: Option<String>,
    identifier: Option<String>,
    #[schema(value_type = String, format = Binary)]
    file: String,
}

pub struct ArtifactToCreate {
    extension: ArtifactExtensions,
    path: String,
    original_filename: String,
    mime_type: String,
    size: usize,
    project_id: String,
    branch: String,
    identifier: String,
}

impl ArtifactToCreate {
    pub fn new(data: CreateArtifact, project_id: String) -> ArtifactToCreate {
        let branch = data.branch.unwrap();
        let identifier = data.identifier.unwrap();
        let extension = data.extension.unwrap();
        let path = create_file_path(
            &project_id,
            &branch,
            &identifier,
            &extension.to_string().to_lowercase(),
        )
        .unwrap();
        ArtifactToCreate {
            original_filename: data.original_filename.unwrap(),
            mime_type: data.mime_type.unwrap(),
            size: data.size.unwrap(),
            branch,
            extension,
            identifier,
            path,
            project_id,
        }
    }
}

impl Artifact {
    pub fn new(data: ArtifactToCreate) -> Artifact {
        Artifact {
            id: ObjectId::new().to_string(),
            branch: data.branch,
            extension: data.extension,
            identifier: data.identifier,
            mime_type: data.mime_type,
            original_filename: data.original_filename,
            path: data.path,
            project_id: data.project_id,
            size: data.size,
            qrcode: None,
        }
    }
}
