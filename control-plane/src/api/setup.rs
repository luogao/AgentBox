use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct SetupStatus {
    pub docker_connected: bool,
    pub agent_image_ready: bool,
    pub agent_image_name: String,
    pub api_key_configured: bool,
    pub all_ready: bool,
    pub project_root: String,
}

pub async fn system_status(
    State(state): State<AppState>,
) -> Json<SetupStatus> {
    let (docker_connected, agent_image_ready, agent_image_name) = match &state.docker_manager {
        Some(dm) => {
            let config = state.config.read().await;
            let image = config.agent_image.clone();
            drop(config);
            let ping = dm.ping().await;
            let exists = dm.image_exists(&image).await;
            (ping, exists, image)
        }
        None => (false, false, String::new()),
    };

    let config = state.config.read().await;
    let api_key_configured = config.api_key.is_some();
    drop(config);

    let all_ready = docker_connected && agent_image_ready && api_key_configured;

    let project_root = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    Json(SetupStatus {
        docker_connected,
        agent_image_ready,
        agent_image_name,
        api_key_configured,
        all_ready,
        project_root,
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateConfigResponse {
    pub api_key_updated: bool,
    pub new_api_key: Option<String>,
}

pub async fn update_config(
    State(state): State<AppState>,
    Json(payload): Json<UpdateConfigRequest>,
) -> Json<UpdateConfigResponse> {
    let mut api_key_updated = false;
    let mut new_api_key = None;

    if let Some(ref key) = payload.api_key {
        if !key.is_empty() {
            let _ = state.db.set_config("API_KEY", key).await;
            state.config.write().await.api_key = Some(key.clone());
            api_key_updated = true;
            new_api_key = Some(key.clone());
            tracing::info!("API_KEY updated via setup wizard");
        }
    }

    Json(UpdateConfigResponse {
        api_key_updated,
        new_api_key,
    })
}
