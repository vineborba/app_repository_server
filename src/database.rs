use bson::doc;
use mongodb::{
    options::{ClientOptions, IndexOptions},
    Client, IndexModel,
};
use std::env;

use crate::models::user::User;

pub async fn connect() -> Result<Client, mongodb::error::Error> {
    let mongo_uri = env::var("MONGO_URI").expect("Failed to load MONGO_URI");
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("AppDist".to_string());

    let client = Client::with_options(client_options)?;

    let user_collection = client.database("appdist").collection::<User>("users");

    let options = IndexOptions::builder().unique(true).build();
    let unique_email_index = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(options)
        .build();
    user_collection
        .create_index(unique_email_index, None)
        .await?;
    Ok(client)
}
