use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::helpers::uuid::generate_uuid;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Platforms {
    Android,
    Ios,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Project {
    // pub id: Option<String>,
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub platforms: Vec<Platforms>,
    pub key: String,
    pub image: Option<String>,
}

impl Project {
    pub fn new(new_project: BaseProjectInput, owner_id: String) -> Project {
        let key = generate_uuid(true);
        // TODO: fix this
        let id = generate_uuid(true);
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
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
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
