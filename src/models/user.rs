use bson::serde_helpers::{
    deserialize_hex_string_from_object_id, serialize_hex_string_as_object_id,
};
use chrono::{Duration, Utc};
use data_encoding::HEXUPPER;
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::bson::oid::ObjectId;
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::ops::Add;
use utoipa::ToSchema;

use crate::error::AppError;

const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;

#[derive(Deserialize, ToSchema)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Claims {
    sub: String,
    user_id: String,
    iat: i64,
    exp: i64,
}

#[derive(Serialize, ToSchema)]
pub struct LoginOutput {
    token: String,
}

impl LoginOutput {
    pub fn new(user_email: String, user_id: String) -> Result<LoginOutput, AppError> {
        let iat = Utc::now();
        let exp = iat.clone().add(Duration::days(40));
        let claims = Claims {
            user_id,
            sub: user_email,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        };
        let key = EncodingKey::from_secret("secret".as_ref());
        match encode(&Header::default(), &claims, &key) {
            Ok(token) => Ok(LoginOutput { token }),
            Err(e) => Err(AppError::Encode(e)),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum UserRole {
    USER,
    MANAGER,
    ADMIN,
}

#[derive(Serialize, Deserialize, ToSchema)]
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
    password: String,
    salt: String,
}

impl User {
    pub fn new(new_user: CreateUser) -> Result<User, AppError> {
        let (password, salt) = User::encrypt_password(new_user.password)?;
        Ok(User {
            id: ObjectId::new().to_string(),
            name: new_user.name,
            email: new_user.email,
            role: UserRole::USER,
            password,
            salt,
        })
    }

    fn encrypt_password(password: String) -> Result<(String, String), AppError> {
        let n_iter = match NonZeroU32::new(100_000) {
            Some(v) => v,
            None => return Err(AppError::Never),
        };
        let rng = rand::SystemRandom::new();

        let mut salt = [0u8; CREDENTIAL_LEN];
        rng.fill(&mut salt)?;

        let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA512,
            n_iter,
            &salt,
            password.as_bytes(),
            &mut pbkdf2_hash,
        );

        Ok((HEXUPPER.encode(&pbkdf2_hash), HEXUPPER.encode(&salt)))
    }

    pub fn validate_password(&self, password: String) -> Result<(), AppError> {
        let salt = match HEXUPPER.decode(&self.salt.as_bytes()) {
            Ok(s) => s,
            Err(e) => return Err(AppError::Decode(e)),
        };
        let decripted_password = match HEXUPPER.decode(&self.password.as_bytes()) {
            Ok(s) => s,
            Err(e) => return Err(AppError::Decode(e)),
        };
        let n_iter = match NonZeroU32::new(100_000) {
            Some(v) => v,
            None => return Err(AppError::Never),
        };
        match pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA512,
            n_iter,
            &salt,
            password.as_bytes(),
            &decripted_password,
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::InvalidCredentials),
        }
    }
}
