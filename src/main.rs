mod app_router;
mod database;
mod error;
mod handlers;
mod models;

use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    BoxError,
};
use axum_server::tls_rustls::RustlsConfig;
use dotenv::dotenv;
use std::{env, net::SocketAddr, path::PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app_router::router;
use error::AppError;

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

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

    let ports = Ports {
        http: env::var("HTTP_PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse()
            .expect("Failed to parse HTTP_PORT"),
        https: env::var("HTTPS_PORT")
            .unwrap_or_else(|_| "3002".to_string())
            .parse()
            .expect("Failed to parse HTTP_PORT"),
    };

    // spawn a second server to redirect http requests to this server
    tokio::spawn(redirect_http_to_https(ports));

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env::var("CERTS_PATH").expect("CERTS_PATH not set")).join("cert.pem"),
        PathBuf::from(env::var("CERTS_PATH").expect("CERTS_PATH not set")).join("key.pem"),
    )
    .await
    .unwrap();

    let db = database::connect().await?;

    let app = router(db).await;

    let addr = SocketAddr::from(([0, 0, 0, 0], ports.https));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .expect("Couldn't initialize server!");

    Ok(())
}

async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    tracing::debug!("http redirect listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(redirect.into_service().into_make_service())
        .await
        .unwrap();
}
