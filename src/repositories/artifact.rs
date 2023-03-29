use serde_json::json;

use crate::{database::client::DBClient, error::AppError, schemas::artifact::Artifact};

#[derive(Clone)]
pub struct ArtifactRepository {
    db: DBClient,
}

impl ArtifactRepository {
    pub fn new(db: DBClient) -> Self {
        ArtifactRepository { db }
    }

    pub async fn get(&self, artifact_id: String) -> Result<Option<Artifact>, AppError> {
        let aid = format!("artifact:{}", artifact_id);

        let mut artifacts: Vec<Artifact> = self.db.client.select(aid.as_str()).await?;
        Ok(artifacts.pop())
    }

    pub async fn get_all(&self) -> Result<Vec<Artifact>, AppError> {
        let artifacts: Vec<Artifact> = self.db.client.select("artifact").await?;
        Ok(artifacts)
    }

    pub async fn get_by_project(&self, project_id: String) -> Result<Vec<Artifact>, AppError> {
        let query = r#"
        SELECT id, type, created_at, branch, original_filename, identifier, ios_metadata, qrcode
        FROM artifact
        WHERE ->belongs->project:$pid
        SPLIT branch
        "#;
        let mut response = self
            .db
            .client
            .query(query)
            .bind(("pid", project_id))
            .await?;
        let artifacts: Vec<Artifact> = response.take(0)?;
        Ok(artifacts)
    }

    pub async fn get_with_project(
        &self,
        artifact_id: String,
    ) -> Result<Option<Artifact>, AppError> {
        let query = r#"
        SELECT *, ->belongs->project AS project FROM artifacts:$aid FETCH project
        "#;
        let mut response = self
            .db
            .client
            .query(query)
            .bind(("aid", artifact_id))
            .await?;
        let artifact: Option<Artifact> = response.take(0)?;
        Ok(artifact)
    }

    pub async fn create(&self, data: Artifact, project_id: String) -> Result<Artifact, AppError> {
        let artifact: Artifact = self
            .db
            .client
            .create(("artifact", data.id.as_str()))
            .content(data)
            .await?;

        // self.db
        //     .client
        //     .query("RELATE artifact:$aid->belongs->project:$pid")
        //     .bind(json!({ "aid": artifact.id, "pid": project_id }))
        //     .await?;

        Ok(artifact)
    }

    pub async fn update_qrcode(
        self,
        artifact_id: String,
        qrcode: String,
    ) -> Result<Artifact, AppError> {
        let artifact = self
            .db
            .client
            .update(("artifact", artifact_id))
            .merge(json!({ "qrcode": qrcode }))
            .await?;
        Ok(artifact)
    }

    /* pub async fn delete(&self, artifact_id: String) -> Result<(), AppError> {
        let aid = format!("artifact:{}", artifact_id);
        self.db.client.delete(aid).await?;
        Ok(())
    } */
}
