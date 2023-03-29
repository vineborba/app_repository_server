use serde_json::json;

use crate::{database::client::DBClient, error::AppError, schemas::user::User};

#[derive(Clone)]
pub struct UserRepository {
    db: DBClient,
}

impl UserRepository {
    pub fn new(db: DBClient) -> Self {
        UserRepository { db }
    }

    pub async fn get(&self, user_id: &str) -> Result<Option<User>, AppError> {
        let query = r#"
        SELECT *
        FROM user:$uid
        FETCH favoriteProjects;
        "#
        .trim();

        let mut res = self.db.client.query(query).bind(("uid", user_id)).await?;

        let user: Option<User> = res.take(0)?;
        Ok(user)
    }

    pub async fn get_all(&self) -> Result<Vec<User>, AppError> {
        let users: Vec<User> = self.db.client.select("user").await?;
        Ok(users)
    }

    pub async fn get_by_email(&self, user_email: String) -> Result<Option<User>, AppError> {
        let query = "SELECT * FROM user WHERE email = $userEmail;";
        let mut response = self
            .db
            .client
            .query(query)
            .bind(("userEmail", user_email))
            .await?;

        let user: Option<User> = response.take(0)?;
        Ok(user)
    }

    pub async fn create(&self, data: User) -> Result<User, AppError> {
        match self
            .db
            .client
            .create(("user", data.id.as_str()))
            .content(data)
            .await
        {
            Ok(user) => Ok(user),
            Err(e) => {
                println!("{}", e.to_string());
                if e.to_string().contains("userEmailIndex") {
                    Err(AppError::UserAlreadyRegistered)
                } else {
                    Err(AppError::SurrealError(e))
                }
            }
        }

        // let user: User = self
        //     .db
        //     .client
        //     .create(("user", data.id.as_str()))
        //     .content(data)
        //     .await?;
        // Ok(user)
    }

    pub async fn insert_favorite_project(
        &self,
        user_id: String,
        project_id: String,
    ) -> Result<(), AppError> {
        let query = "UPDATE person:$pid SET favoriteProjects += $pid;";

        self.db
            .client
            .query(query)
            .bind(json!({ "uid": user_id, "pid": project_id }))
            .await?;
        Ok(())
    }

    pub async fn remove_favorite_project(
        &self,
        user_id: String,
        project_id: String,
    ) -> Result<(), AppError> {
        let query = "UPDATE person:$pid SET favoriteProjects -= $pid;";
        self.db
            .client
            .query(query)
            .bind(json!({ "uid": user_id, "pid": project_id }))
            .await?;
        Ok(())
    }
}
