mod db;
mod handlers;
mod models;

use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    http::{header, HeaderValue},
    routing::get,
    Router,
};
use dotenv::dotenv;
use tower_http::{
    limit::RequestBodyLimitLayer, set_header::SetRequestHeaderLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use db::client;
use handlers::users::{create_user, get_users};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "rust_axum=debug,axum=debug,tower_http=debug,mongodb=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = client::init_db().await.unwrap();

    let server_header = HeaderValue::from_static("open-dist");

    let app = Router::new()
        .nest(
            "/users",
            Router::new().route("/", get(get_users).post(create_user)),
        )
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyLimitLayer::new(10240))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestHeaderLayer::if_not_present(
            header::SERVER,
            server_header,
        ));

    let app = app.with_state(db);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
