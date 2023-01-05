use axum::{
    http::{header, HeaderValue},
    routing::get,
    Router,
};
use mongodb::Client;
use std::time::Duration;
use tower_http::{
    limit::RequestBodyLimitLayer, set_header::SetRequestHeaderLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::handlers::{
    projects::{create_project, get_projects},
    users::{create_user, get_users},
};

pub(super) async fn router(db: Client) -> Router {
    let server_header = HeaderValue::from_static("open-dist");

    let app = Router::new()
        .nest(
            "/projects",
            Router::new().route("/", get(get_projects).post(create_project)),
        )
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

    app.with_state(db)
}
