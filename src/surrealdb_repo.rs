use std::sync::Arc;
use surrealdb::sql::Value;
use surrealdb::{dbs::Session, kvs::Datastore, Error};

pub trait Creatable: Into<Value> {}
pub trait Patchable: Into<Value> {}

#[derive(Clone)]
pub struct DBClient {
    pub datastore: Arc<Datastore>,
    pub session: Session,
}

impl DBClient {
    pub async fn connect() -> Result<Self, Error> {
        let datastore = Arc::new(Datastore::new("http://localhost:8000").await?);
        let session = Session::for_kv().with_ns("dev").with_db("app_repository");

        Ok(DBClient { datastore, session })
    }
}
