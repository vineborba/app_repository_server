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
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{
    projects::{create_project, get_projects},
    users::{create_user, get_users},
};

#[derive(OpenApi)]
#[openapi(
        paths(
            crate::handlers::projects::create_project,
            crate::handlers::projects::get_projects,
            crate::handlers::users::create_user,
            crate::handlers::users::get_users,
        ),
        components(
            schemas(
                crate::models::project::CreateProject, crate::models::project::Project, crate::models::project::Platforms,
                crate::models::user::CreateUser, crate::models::user::User, crate::models::user::UserRole,
            )
        ),
        // modifiers(&SecurityAddon),
        tags(
            (name = "Projects", description = "Projects management API"),
            (name = "Users", description = "Users management API"),
        )
    )]
struct ApiDoc;

pub(super) async fn router(db: Client) -> Router {
    let server_header = HeaderValue::from_static("open-dist");

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
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
