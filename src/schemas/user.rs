use chrono::{Duration, Utc};
use data_encoding::HEXUPPER;
use jsonwebtoken::{encode, EncodingKey, Header};
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use serde::{Deserialize, Serialize};
use std::env;
use std::num::NonZeroU32;
use std::ops::Add;
use utoipa::ToSchema;

use crate::error::AppError;
use crate::helpers::uuid::generate_uuid;
// use crate::helpers::uuid::generate_uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateUserInput {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Serialize, ToSchema)]
pub struct AuthOutput {
    token: String,
}

impl AuthOutput {
    pub fn new(user_email: String, user_id: String) -> Result<AuthOutput, AppError> {
        let iat = Utc::now();
        let exp = iat.add(Duration::days(40));
        let claims = Claims {
            user_id,
            sub: user_email,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        };
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET is not set!");
        let key = EncodingKey::from_secret(secret.as_ref());
        match encode(&Header::default(), &claims, &key) {
            Ok(token) => Ok(AuthOutput { token }),
            Err(e) => Err(AppError::Encode(e)),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Manager,
    Admin,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    // pub id: Option<String>,
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub favorite_projects: Vec<String>,
    password: String,
    salt: String,
}

impl User {
    pub fn new(new_user: CreateUserInput) -> Result<User, AppError> {
        let (password, salt) = User::encrypt_password(new_user.password)?;
        Ok(User {
            // TODO: fix this
            id: generate_uuid(true),
            // id: None,
            name: new_user.name,
            email: new_user.email,
            role: UserRole::User,
            favorite_projects: vec![],
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

        const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
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
        let salt = match HEXUPPER.decode(self.salt.as_bytes()) {
            Ok(s) => s,
            Err(e) => return Err(AppError::Decode(e)),
        };
        let decripted_password = match HEXUPPER.decode(self.password.as_bytes()) {
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

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserOutput {
    name: String,
    favorite_projects: Vec<String>,
}

impl UserOutput {
    pub fn new(user: User) -> UserOutput {
        UserOutput {
            name: user.name,
            favorite_projects: user.favorite_projects,
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFavoriteProjectsInput {
    pub project_id: String,
}
