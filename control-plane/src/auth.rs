use axum::{
    extract::{Request, State},
    http::{header, Method, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::AppState;

pub async fn api_key_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let config = state.config.read().await;
    let api_key = match &config.api_key {
        Some(key) => key.clone(),
        None => {
            drop(config);
            return next.run(request).await;
        }
    };
    drop(config);

    if request.method() == Method::OPTIONS {
        return next.run(request).await;
    }

    if request.uri().path() == "/health" {
        return next.run(request).await;
    }

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
        .and_then(|v: &str| v.strip_prefix("Bearer "));

    if auth_header == Some(&api_key) {
        return next.run(request).await;
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Content-Type", "text/plain")
        .body(axum::body::Body::from("Unauthorized"))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        routing::get,
        Router,
    };
    use http::Request as HttpRequest;
    use tower::util::ServiceExt;

    use crate::config::Config;
    use crate::db::sqlite::Database;

    async fn test_state(api_key: Option<&str>) -> AppState {
        let db = Database::new("sqlite::memory:").await.unwrap();
        AppState {
            db,
            docker_manager: None,
            config: std::sync::Arc::new(tokio::sync::RwLock::new(Config {
                database_url: String::new(),
                server_addr: String::new(),
                agent_image: String::new(),
                api_key: api_key.map(String::from),
                anthropic_api_key: None,
                allowed_origins: Vec::new(),
            })),
            log_secrets: std::sync::Arc::new(Vec::new()),
        }
    }

    fn test_app(state: AppState) -> Router {
        Router::new()
            .route("/health", get(|| async { "ok" }))
            .route("/api/test", get(|| async { "secret" }))
            .layer(axum::middleware::from_fn_with_state(state.clone(), api_key_auth))
            .with_state(state)
    }

    // ---- no API key configured ----

    #[tokio::test]
    async fn no_key_allows_health() {
        let app = test_app(test_state(None).await);
        let resp = app
            .oneshot(HttpRequest::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn no_key_allows_protected() {
        let app = test_app(test_state(None).await);
        let resp = app
            .oneshot(HttpRequest::get("/api/test").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ---- API key configured ----

    #[tokio::test]
    async fn health_skips_auth() {
        let app = test_app(test_state(Some("secret")).await);
        let resp = app
            .oneshot(HttpRequest::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn missing_key_returns_401() {
        let app = test_app(test_state(Some("secret")).await);
        let resp = app
            .oneshot(HttpRequest::get("/api/test").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn wrong_key_returns_401() {
        let app = test_app(test_state(Some("secret")).await);
        let resp = app
            .oneshot(
                HttpRequest::get("/api/test")
                    .header("Authorization", "Bearer wrong")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn correct_key_passes() {
        let app = test_app(test_state(Some("secret")).await);
        let resp = app
            .oneshot(
                HttpRequest::get("/api/test")
                    .header("Authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn wrong_format_key_returns_401() {
        let app = test_app(test_state(Some("secret")).await);
        let resp = app
            .oneshot(
                HttpRequest::get("/api/test")
                    .header("Authorization", "Basic secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn empty_api_key_still_401() {
        // key="secret" but client sends empty Bearer token
        let app = test_app(test_state(Some("secret")).await);
        let resp = app
            .oneshot(
                HttpRequest::get("/api/test")
                    .header("Authorization", "Bearer ")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
