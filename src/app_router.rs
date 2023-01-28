use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{self, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use mongodb::Client;
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    set_header::SetRequestHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{
    artifacts::{create_artifact, get_artifacts, list_project_artifacts},
    projects::{create_project, get_project, get_projects},
    users::{create_user, get_user_data, get_users, login_user},
};

#[derive(OpenApi)]
#[openapi(
        paths(
            crate::handlers::artifacts::get_artifacts,
            crate::handlers::artifacts::create_artifact,
            crate::handlers::projects::create_project,
            crate::handlers::projects::get_projects,
            crate::handlers::users::create_user,
            crate::handlers::users::get_users,
        ),
        components(
            schemas(
                crate::models::project::CreateProject, crate::models::project::Project, crate::models::project::Platforms,
                crate::models::user::CreateUser, crate::models::user::User, crate::models::user::UserRole,
                crate::models::artifact::Artifact,crate::models::artifact::ArtifactExtensions,crate::models::artifact::CreateArtifactInput,
            )
        ),
        // modifiers(&SecurityAddon),
        tags(
            (name = "Projects", description = "Projects management API"),
            (name = "Artifacts", description = "Artifacts management API"),
            (name = "Users", description = "Users management API"),
        )
    )]
struct ApiDoc;

pub(super) async fn router(db: Client) -> Router {
    let body_limit_request: usize = 10240 * 1000 * 1000;
    let server_header = HeaderValue::from_static("open-dist");

    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .nest("/artifacts", Router::new().route("/", get(get_artifacts)))
        .nest(
            "/projects",
            Router::new()
                .route("/", get(get_projects).post(create_project))
                .nest(
                    "/:project_id",
                    Router::new().route("/", get(get_project)).route(
                        "/artifacts",
                        get(list_project_artifacts)
                            .post(create_artifact)
                            .route_layer(DefaultBodyLimit::disable()),
                    ),
                ),
        )
        .nest(
            "/users",
            Router::new()
                .route("/", get(get_users).post(create_user))
                .route("/login", post(login_user))
                .route("/me", get(get_user_data)),
        )
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_origin(Any)
                .allow_headers([CONTENT_TYPE, AUTHORIZATION]),
        )
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyLimitLayer::new(body_limit_request))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestHeaderLayer::if_not_present(
            header::SERVER,
            server_header,
        ));

    app.with_state(db)
}
