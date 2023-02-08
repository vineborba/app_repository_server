use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{self, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, patch, post},
    Router,
};
use mongodb::Client;
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetRequestHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{
    artifacts::{create_artifact, get_artifacts, list_project_artifacts},
    projects::{
        create_project, get_project, get_projects, remove_project_image, update_project,
        update_project_image,
    },
    users::{create_user, edit_favorite_projects, get_user_data, get_users, login_user},
    SecurityAddon,
};

#[derive(OpenApi)]
#[openapi(
        paths(
            crate::handlers::artifacts::get_artifacts,
            crate::handlers::artifacts::create_artifact,
            crate::handlers::artifacts::list_project_artifacts,
            crate::handlers::projects::create_project,
            crate::handlers::projects::get_projects,
            crate::handlers::projects::get_project,
            crate::handlers::projects::update_project,
            crate::handlers::projects::update_project_image,
            crate::handlers::projects::remove_project_image,
            crate::handlers::users::create_user,
            crate::handlers::users::get_users,
            crate::handlers::users::get_user_data,
            crate::handlers::users::login_user,
            crate::handlers::users::edit_favorite_projects,
        ),
        components(
            schemas(
                crate::models::project::BaseProjectInput,
                crate::models::project::Project,
                crate::models::project::Platforms,
                crate::models::project::EditImageInput,
                crate::models::user::CreateUserInput,
                crate::models::user::User,
                crate::models::user::UserRole,
                crate::models::user::AuthOutput,
                crate::models::user::LoginInput,
                crate::models::user::UserOutput,
                crate::models::user::UpdateFavoriteProjectsInput,
                crate::models::artifact::Artifact,
                crate::models::artifact::ArtifactExtensions,
                crate::models::artifact::CreateArtifactInput,
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "Projects", description = "Projects management API"),
            (name = "Artifacts", description = "Artifacts management API"),
            (name = "Users", description = "Users management API"),
        )
    )]
struct ApiDoc;

pub(super) async fn router(db: Client) -> Router {
    let default_request_body_limit: usize = 2 * 1024 * 1024; // 2MB
    let image_request_body_limit: usize = 5 * 1024 * 1024; // 5MB
    let artifact_request_body_limit: usize = 300 * 1024 * 1024; // 300MB
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
                    Router::new()
                        .route("/", get(get_project).patch(update_project))
                        .route(
                            "/image",
                            patch(update_project_image)
                                .route_layer(DefaultBodyLimit::max(image_request_body_limit))
                                .delete(remove_project_image),
                        )
                        .route(
                            "/artifacts",
                            get(list_project_artifacts)
                                .post(create_artifact)
                                .route_layer(DefaultBodyLimit::max(artifact_request_body_limit)),
                        ),
                ),
        )
        .nest(
            "/users",
            Router::new()
                .route("/", get(get_users).post(create_user))
                .route("/login", post(login_user))
                .route("/me", get(get_user_data))
                .route("/favorite-projects", patch(edit_favorite_projects)),
        )
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::OPTIONS,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_origin(Any)
                .allow_headers([CONTENT_TYPE, AUTHORIZATION]),
        )
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(DefaultBodyLimit::max(default_request_body_limit))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestHeaderLayer::if_not_present(
            header::SERVER,
            server_header,
        ));

    app.with_state(db)
}
