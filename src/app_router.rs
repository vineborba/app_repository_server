use axum::{
    extract::{DefaultBodyLimit, FromRef},
    http::{
        header::{self, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, patch, post},
    Router,
};
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    set_header::SetRequestHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    database::client::DBClient,
    handlers::{
        artifacts::{
            create_artifact, download_artifact, get_artifacts, get_download_headers, get_ios_plist,
            list_project_artifacts,
        },
        projects::{
            create_project, get_project, get_projects, remove_project_image, update_project,
            update_project_image,
        },
        users::{create_user, edit_favorite_projects, get_user_data, get_users, login_user},
        SecurityAddon,
    },
    repositories::{
        artifact::ArtifactRepository, project::ProjectRepository, user::UserRepository,
    },
};

#[derive(OpenApi)]
#[openapi(
        info(
            title = "App Repository Server",
            description = "App Repository Server OpenAPI definitions",
            contact (
                name = "Vinicius de Borba",
                url = "https://github.com/vineborba"
            )
        ),
        servers (
            (url = "https://localhost:3002", description = "Development server"),
            (url = "http://localhost:3001", description = "Development server"),
        ),
        paths(
            crate::handlers::artifacts::get_artifacts,
            crate::handlers::artifacts::create_artifact,
            crate::handlers::artifacts::list_project_artifacts,
            crate::handlers::artifacts::download_artifact,
            crate::handlers::artifacts::get_download_headers,
            crate::handlers::artifacts::get_ios_plist,
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
                crate::schemas::project::BaseProjectInput,
                crate::schemas::project::Project,
                crate::schemas::project::Platforms,
                crate::schemas::project::EditImageInput,
                crate::schemas::user::CreateUserInput,
                crate::schemas::user::User,
                crate::schemas::user::UserRole,
                crate::schemas::user::AuthOutput,
                crate::schemas::user::LoginInput,
                crate::schemas::user::UserOutput,
                crate::schemas::user::UpdateFavoriteProjectsInput,
                crate::schemas::artifact::Artifact,
                crate::schemas::artifact::ArtifactExtensions,
                crate::schemas::artifact::CreateArtifactInput,
                crate::schemas::artifact::IosMetadata,
                crate::schemas::artifact::ArtifactBinary,
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

#[derive(Clone)]
struct AppState {
    project_repository: ProjectRepository,
    artifact_repository: ArtifactRepository,
    user_repository: UserRepository,
}

impl FromRef<AppState> for ProjectRepository {
    fn from_ref(input: &AppState) -> ProjectRepository {
        input.project_repository.clone()
    }
}

impl FromRef<AppState> for ArtifactRepository {
    fn from_ref(input: &AppState) -> ArtifactRepository {
        input.artifact_repository.clone()
    }
}

impl FromRef<AppState> for UserRepository {
    fn from_ref(input: &AppState) -> UserRepository {
        input.user_repository.clone()
    }
}

pub struct AppRouter {}

impl AppRouter {
    pub fn new(sdb: DBClient) -> Router {
        let default_request_body_limit: usize = 2 * 1024 * 1024; // 2 MiB
        let image_request_body_limit: usize = 5 * 1024 * 1024; // 5 MiB
        let artifact_request_body_limit: usize = 300 * 1024 * 1024; // 300 MiB
        let server_header = HeaderValue::from_static("open-dist");

        let project_repository = ProjectRepository::new(sdb.clone());
        let artifact_repository = ArtifactRepository::new(sdb.clone());
        let user_repository = UserRepository::new(sdb.clone());

        let state = AppState {
            project_repository,
            artifact_repository,
            user_repository,
        };

        let app = Router::new()
            .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
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
                                    .route_layer(DefaultBodyLimit::max(
                                        artifact_request_body_limit,
                                    )),
                            ),
                    )
                    .nest(
                        "/artifacts",
                        Router::new().route("/", get(get_artifacts)).nest(
                            "/:artifact_id",
                            Router::new()
                                .route(
                                    "/download",
                                    get(download_artifact).head(get_download_headers),
                                )
                                .route("/ios-plist", get(get_ios_plist)),
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

        app.with_state(state)
    }
}
