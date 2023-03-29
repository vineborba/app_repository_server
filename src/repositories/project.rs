use serde_json::json;

use crate::{
    database::client::DBClient,
    error::AppError,
    schemas::project::{BaseProjectInput, Project},
};

#[derive(Clone)]
pub struct ProjectRepository {
    db: DBClient,
}

impl ProjectRepository {
    pub fn new(db: DBClient) -> Self {
        ProjectRepository { db }
    }

    pub async fn get(&self, project_id: String) -> Result<Option<Project>, AppError> {
        let pid = format!("project:{}", project_id);

        let mut projects: Vec<Project> = self.db.client.select(pid.as_str()).await?;
        Ok(projects.pop())
    }

    pub async fn get_all(&self) -> Result<Vec<Project>, AppError> {
        let projects: Vec<Project> = self.db.client.select("project").await?;
        Ok(projects)
    }

    pub async fn create(&self, data: Project) -> Result<Project, AppError> {
        let project: Project = self
            .db
            .client
            .create(("project", data.id.as_str()))
            .content(data)
            .await?;
        Ok(project)
    }

    pub async fn update(
        &self,
        project_id: String,
        data: BaseProjectInput,
    ) -> Result<Project, AppError> {
        let project = self
            .db
            .client
            .update(("project", project_id))
            .merge(data)
            .await?;

        Ok(project)
    }

    pub async fn update_image(
        &self,
        project_id: String,
        image: Option<String>,
    ) -> Result<Project, AppError> {
        let project = self
            .db
            .client
            .update(("project", project_id))
            .merge(json!({ "image": image }))
            .await?;

        Ok(project)
    }
}
