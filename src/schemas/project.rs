use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use surrealdb::sql::{thing, Array, Object, Value};
use uuid::Uuid;

use crate::error::AppError;
use crate::helpers::{macros::map, wrapper::W};
use crate::surrealdb_repo::{Creatable, DBClient, Patchable};

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub platforms: Vec<String>,
    key: String,
    pub image: Option<String>,
}

impl Project {
    pub fn new(new_project: BaseProjectInput, owner_id: String) -> Project {
        let key = Project::create_project_key();
        Project {
            id: None,
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

impl From<Project> for Value {
    fn from(value: Project) -> Self {
        match value.id {
            Some(v) => map![
                "id".into() => v.into(),
                "name".into() => value.name.into(),
                "description".into() => value.description.into(),
                "owner".into() => value.owner.into(),
                "platforms".into() => value.platforms.into(),
                "image".into() => value.image.into(),
            ]
            .into(),
            None => map![
                "name".into() => value.name.into(),
                "description".into() => value.description.into(),
                "owner".into() => value.owner.into(),
                "platforms".into() => value.platforms.into(),
                "image".into() => value.image.into(),
            ]
            .into(),
        }
    }
}

impl Creatable for Project {}

#[derive(Deserialize)]
pub struct BaseProjectInput {
    pub name: String,
    pub description: String,
    pub platforms: Vec<String>,
}

impl From<BaseProjectInput> for Value {
    fn from(value: BaseProjectInput) -> Self {
        let mut val: BTreeMap<String, Value> = BTreeMap::new();

        val.insert("name".into(), value.name.into());
        val.insert("description".into(), value.description.into());
        val.insert("platforms".into(), value.platforms.into());

        Value::from(val)
    }
}

impl Patchable for BaseProjectInput {}

#[derive(Clone)]
pub struct ProjectRepository {
    db: DBClient,
}

impl ProjectRepository {
    pub fn new(db: DBClient) -> Self {
        ProjectRepository { db }
    }

    pub async fn get(self, project_id: &str) -> Result<Object, AppError> {
        let query = "SELECT * FROM $pid";

        let pid = format!("project:{}", project_id);
        let vars: BTreeMap<String, Value> = map!["pid".into() => thing(&pid)?.into()];

        let res = self
            .db
            .datastore
            .execute(query, &self.db.session, Some(vars), true)
            .await?;
        let first_res = res.into_iter().next().expect("msg");

        W(first_res.result?.first()).try_into()
    }

    pub async fn get_all(self) -> Result<Vec<Object>, AppError> {
        let query = "SELECT * FROM project";

        let res = self
            .db
            .datastore
            .execute(query, &self.db.session, None, true)
            .await?;
        let first_res = res.into_iter().next().expect("msg");
        let array: Array = W(first_res.result?).try_into()?;
        array.into_iter().map(|value| W(value).try_into()).collect()
    }

    pub async fn create<T: Creatable>(self, data: T) -> Result<Object, AppError> {
        let sql = "CREATE type::table(project) CONTENT $data RETURN *";

        let data: Object = W(data.into()).try_into()?;
        let vars: BTreeMap<String, Value> = map![
            "data".into() => Value::from(data),
        ];
        let res = self
            .db
            .datastore
            .execute(sql, &self.db.session, Some(vars), false)
            .await?;

        let first_val = res
            .into_iter()
            .next()
            .map(|r| r.result)
            .expect("id not returned")?;

        W(first_val.first()).try_into()
    }

    pub async fn update<T: Patchable>(self, project_id: &str, data: T) -> Result<Object, AppError> {
        let query = "UPDATE $pid MERGE $data RETURN *";

        let pid = format!("project:{}", project_id);

        let vars = map![
            "pid".into() => thing(&pid)?.into(),
            "data".into() => data.into()
        ];

        let res = self
            .db
            .datastore
            .execute(query, &self.db.session, Some(vars), true)
            .await?;

        let first_res = res.into_iter().next().expect("msg");

        let result = first_res.result?;

        W(result.first()).try_into()
    }
}
