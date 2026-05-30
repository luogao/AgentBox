mod api;
mod auth;
mod config;
mod db;
mod docker;
mod error;
mod models;
mod redact;

use std::sync::Arc;

use axum::{
    http::{HeaderValue, Method},
    middleware,
    routing::{delete, get, post},
    Router,
};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use config::Config;
use db::sqlite::Database;
use docker::lifecycle::LifecycleManager;
use docker::manager::DockerManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub docker_manager: Option<Arc<DockerManager>>,
    pub config: Arc<RwLock<Config>>,
    pub log_secrets: Arc<Vec<String>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::from_env();
    tracing::info!("Starting Control Plane on {}", config.server_addr);

    let db = Database::new(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Load persisted config from DB (overrides env vars)
    let mut config = config;
    if let Ok(Some(api_key)) = db.get_config("API_KEY").await {
        config.api_key = Some(api_key);
    }
    if let Ok(Some(anthropic_key)) = db.get_config("ANTHROPIC_API_KEY").await {
        config.anthropic_api_key = Some(anthropic_key);
    }

    let docker_manager = Arc::new(
        DockerManager::new()
            .await
            .expect("Failed to connect to Docker"),
    );

    let log_secrets = Arc::new(redact::collect_secret_values());
    if !log_secrets.is_empty() {
        tracing::info!(
            "Log redaction enabled for {} secret value(s)",
            log_secrets.len()
        );
    }

    let server_addr = config.server_addr.clone();
    let cors = build_cors(&config);

    let app_state = AppState {
        db: db.clone(),
        docker_manager: Some(docker_manager.clone()),
        config: Arc::new(RwLock::new(config)),
        log_secrets,
    };

    let lifecycle_manager = LifecycleManager::new(db.clone(), Some(docker_manager.clone()));
    tokio::spawn(async move {
        lifecycle_manager.start().await;
    });

    let app = Router::new()
        .route("/health", get(api::health::health_check))
        .route(
            "/api/containers",
            get(api::containers::list_containers).post(api::containers::create_container),
        )
        .route(
            "/api/containers/{id}",
            get(api::containers::get_container),
        )
        .route(
            "/api/containers/{id}",
            delete(api::containers::delete_container),
        )
        .route(
            "/api/containers/{id}/status",
            post(api::containers::report_status),
        )
        .route("/api/stats", get(api::containers::get_stats))
        .route("/api/setup/status", get(api::setup::system_status))
        .route("/api/setup/config", post(api::setup::update_config))
        .route(
            "/api/containers/{id}/logs",
            get(api::ws::container_logs_ws),
        )
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::api_key_auth,
        ))
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&server_addr)
        .await
        .expect("Failed to bind");

    tracing::info!("Listening on {}", server_addr);
    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

fn build_cors(config: &Config) -> CorsLayer {
    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
    ];

    if config.allowed_origins.iter().any(|o| o == "*") {
        tracing::warn!("CORS: ALLOWED_ORIGINS=* — wildcard origin enabled");
        return CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(methods)
            .allow_headers(tower_http::cors::Any);
    }

    let origins: Vec<HeaderValue> = config
        .allowed_origins
        .iter()
        .filter_map(|o| match HeaderValue::from_str(o) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!("Invalid origin '{}' ignored: {}", o, e);
                None
            }
        })
        .collect();

    tracing::info!("CORS: allowed origins = {:?}", config.allowed_origins);
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(methods)
        .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE])
        .allow_credentials(true)
}
