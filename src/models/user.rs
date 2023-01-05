use bson::serde_helpers::{
    deserialize_hex_string_from_object_id, serialize_hex_string_as_object_id,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum UserRole {
    USER,
    MANAGER,
    ADMIN,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(
        rename = "_id",
        serialize_with = "serialize_hex_string_as_object_id",
        deserialize_with = "deserialize_hex_string_from_object_id"
    )]
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
}

impl User {
    pub fn new(new_user: CreateUser) -> User {
        User {
            id: ObjectId::new().to_string(),
            name: new_user.name,
            email: new_user.email,
            role: UserRole::USER,
        }
    }
}
