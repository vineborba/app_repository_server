use core::fmt;
use std::time::{SystemTime, SystemTimeError};

use bson::serde_helpers::{
    deserialize_hex_string_from_object_id, serialize_hex_string_as_object_id, serialize_u64_as_i64,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::AppError, helpers::artifact::create_file_path};

use super::project::Project;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactExtensions {
    Ipa,
    Apk,
    Aab,
}

impl fmt::Display for ArtifactExtensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[derive(Serialize, Deserialize, Default, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IosMetadata {
    pub bundle_identifier: String,
    pub bundle_version: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
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
    project: Option<Project>,
    mime_type: String,
    size: usize,
    identifier: String,
    #[serde(serialize_with = "serialize_u64_as_i64")]
    created_at: u64,
    qrcode: Option<String>,
    ios_metadata: Option<IosMetadata>,
}

impl Artifact {
    pub fn new(data: ArtifactToCreate) -> Result<Artifact, SystemTimeError> {
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        Ok(Artifact {
            id: ObjectId::new().to_string(),
            branch: data.branch,
            extension: data.extension,
            identifier: data.identifier,
            mime_type: data.mime_type,
            original_filename: data.original_filename,
            path: data.path,
            project_id: data.project_id,
            size: data.size,
            ios_metadata: data.ios_metdata,
            created_at: duration.as_secs() * 1000,
            qrcode: None,
            project: None,
        })
    }

    pub fn get_extension(&self) -> &ArtifactExtensions {
        &self.extension
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn get_download_data(&self) -> (&String, &String, &usize) {
        (&self.original_filename, &self.mime_type, &self.size)
    }

    pub fn get_plist_data(&self) -> Result<(String, String, String, String), AppError> {
        if let (Some(ios_metadata), Some(project)) = (&self.ios_metadata, &self.project) {
            Ok((
                self.id.clone(),
                ios_metadata.bundle_identifier.clone(),
                ios_metadata.bundle_version.clone(),
                project.name.clone(),
            ))
        } else {
            Err(AppError::InvalidIosMetadata)
        }
    }
}

#[derive(Deserialize, Default)]
pub struct CreateArtifact {
    pub branch: Option<String>,
    pub identifier: Option<String>,
    pub mime_type: Option<String>,
    pub original_filename: Option<String>,
    pub extension: Option<ArtifactExtensions>,
    pub size: Option<usize>,
    pub metadata: Option<IosMetadata>,
}

#[allow(dead_code)]
#[derive(ToSchema)]
pub struct CreateArtifactInput {
    branch: Option<String>,
    identifier: Option<String>,
    bundle_identifier: Option<String>,
    bundle_version: Option<String>,
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
    ios_metdata: Option<IosMetadata>,
}

impl ArtifactToCreate {
    pub fn new(data: CreateArtifact, project_id: String) -> Result<ArtifactToCreate, AppError> {
        let branch = data.branch.unwrap_or_else(|| "develop".to_string());
        let identifier = data
            .identifier
            .unwrap_or_else(|| "unidentified".to_string());
        let extension = data.extension.unwrap_or(ArtifactExtensions::Apk);
        let path = create_file_path(
            &project_id,
            &branch,
            &identifier,
            &extension.to_string().to_lowercase(),
        )?;
        if let (Some(original_filename), Some(mime_type), Some(size)) =
            (data.original_filename, data.mime_type, data.size)
        {
            Ok(ArtifactToCreate {
                original_filename,
                mime_type,
                size,
                branch,
                extension,
                identifier,
                path,
                project_id,
                ios_metdata: data.metadata,
            })
        } else {
            Err(AppError::Never)
        }
    }
}

#[derive(ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct ArtifactBinary(String);
