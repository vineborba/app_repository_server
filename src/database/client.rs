use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Value;
use surrealdb::{Error, Surreal};

pub trait Creatable: Into<Value> {}
pub trait Patchable: Into<Value> {}

#[derive(Clone)]
pub struct DBClient {
    pub client: Surreal<Client>,
}

impl DBClient {
    pub async fn connect() -> Result<Self, Error> {
        let client = Surreal::new::<Ws>("localhost:8000").await?;

        client
            .signin(Root {
                username: "root",
                password: "root",
            })
            .await?;

        client.use_ns("dev").use_db("app_repository").await?;

        Ok(DBClient { client })
    }

    pub async fn bootstrap(&self) -> Result<(), Error> {
        self.init_user_table().await?;
        self.init_project_table().await?;
        self.init_artifacts_table().await?;
        Ok(())
    }

    async fn init_user_table(&self) -> Result<(), Error> {
        let table_query = "DEFINE TABLE user SCHEMAFULL;";

        let attributes_query = r#"
        DEFINE FIELD name ON TABLE user TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD email ON TABLE user TYPE string
            ASSERT $value != NONE AND is::email($value);
        DEFINE FIELD password ON TABLE user TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD salt ON TABLE user TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD role ON TABLE user TYPE string
            ASSERT $value != NONE OR 'user';
        DEFINE FIELD favoriteProjects ON TABLE user TYPE array;
        DEFINE FIELD favoriteProjects.* ON TABLE user TYPE record (project);
        "#
        .trim();

        let index_query = "DEFINE INDEX userEmailIndex ON TABLE user COLUMNS email UNIQUE;";

        self.client
            .query(table_query)
            .query(attributes_query)
            .query(index_query)
            .await?;
        Ok(())
    }

    async fn init_project_table(&self) -> Result<(), Error> {
        let table_query = "DEFINE TABLE project SCHEMAFULL;";

        let attributes_query = r#"
        DEFINE FIELD name ON TABLE project TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD description ON TABLE project TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD owner ON TABLE project TYPE record (user)
            ASSERT $value != NONE;
        DEFINE FIELD platforms ON TABLE project TYPE array;
        DEFINE FIELD platforms.* ON TABLE project TYPE string;
        DEFINE FIELD key ON TABLE project TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD image ON TABLE project TYPE string;
        "#
        .trim();

        self.client
            .query(table_query)
            .query(attributes_query)
            .await?;
        Ok(())
    }

    async fn init_artifacts_table(&self) -> Result<(), Error> {
        let table_query = "DEFINE TABLE artifact SCHEMAFULL;";

        let attributes_query = r#"
        DEFINE FIELD original_filename ON TABLE artifact TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD branch ON TABLE artifact TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD extension ON TABLE artifact TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD path ON TABLE artifact TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD project ON TABLE artifact TYPE record (project)
            ASSERT $value != NONE;
        DEFINE FIELD mime_type ON TABLE artifact TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD size ON TABLE artifact TYPE int
            ASSERT $value != NONE;
        DEFINE FIELD identifier ON TABLE artifact TYPE string
            ASSERT $value != NONE;
        DEFINE FIELD created_at ON TABLE artifact TYPE number
            ASSERT $value != NONE;
        DEFINE FIELD qrcode ON TABLE artifact TYPE string;
        DEFINE FIELD ios_metadata ON TABLE artifact TYPE object;
        "#
        .trim();

        self.client
            .query(table_query)
            .query(attributes_query)
            .await?;

        Ok(())
    }
}
