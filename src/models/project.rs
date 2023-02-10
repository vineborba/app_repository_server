use bson::serde_helpers::{
    deserialize_hex_string_from_object_id, serialize_hex_string_as_object_id,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Platforms {
    Android,
    Ios,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(
        rename = "_id",
        serialize_with = "serialize_hex_string_as_object_id",
        deserialize_with = "deserialize_hex_string_from_object_id"
    )]
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(
        serialize_with = "serialize_hex_string_as_object_id",
        deserialize_with = "deserialize_hex_string_from_object_id"
    )]
    pub owner: String,
    pub platforms: Vec<Platforms>,
    key: String,
    pub image: Option<String>,
}

impl Project {
    pub fn new(new_project: BaseProjectInput, owner_id: String) -> Project {
        let id = ObjectId::new().to_string();
        let key = Project::create_project_key();
        Project {
            id,
            key,
            name: new_project.name,
            description: new_project.description,
            owner: owner_id,
            platforms: new_project.platforms,
            image: None,
        }
    }

    fn create_project_key() -> String {
        Uuid::new_v4().to_string()
    }
}

#[derive(Deserialize, ToSchema)]
pub struct BaseProjectInput {
    pub name: String,
    pub description: String,
    pub platforms: Vec<Platforms>,
}

#[allow(dead_code)]
#[derive(ToSchema, Debug)]
pub struct EditImageInput {
    #[schema(value_type = String, format = Binary)]
    file: String,
}
