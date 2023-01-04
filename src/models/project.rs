use bson::serde_helpers::{
    deserialize_hex_string_from_object_id, serialize_hex_string_as_object_id,
    uuid_as_binary::serialize,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum Platforms {
    ANDROID,
    IOS,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize)]
pub struct CreateProject {
    pub name: String,
    pub description: String,
    pub platforms: Vec<Platforms>,
}

impl Project {
    pub fn new(new_project: CreateProject, owner_id: String) -> Project {
        let id = ObjectId::new().to_string();
        let key = Project::create_project_key(&id);
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

    fn create_project_key(project_id: &String) -> String {
        Uuid::new_v4().to_string()
    }
}
