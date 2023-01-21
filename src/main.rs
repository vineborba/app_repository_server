mod app_router;
mod database;
mod error;
mod handlers;
mod models;

use axum;
use std::env;
use dotenv::dotenv;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app_router::router;
use error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| {
                "rust_axum=debug,axum=debug,tower_http=debug,mongodb=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = database::connect().await?;

    let app = router(db).await;

    let port = env::var("PORT").unwrap_or("3001".to_string());
    let port = port.parse::<u16>().expect("Couldn't parse PORT as an integer!");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
