use mongodb::{options::ClientOptions, Client};
use std::env;

pub async fn connect() -> Result<Client, mongodb::error::Error> {
    let mongo_uri = env::var("MONGO_URI").expect("Failed to load MONGO_URI");
    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("AppDist".to_string());

    let client = Client::with_options(client_options)?;

    Ok(client)
}
