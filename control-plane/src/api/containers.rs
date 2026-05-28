use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;

use crate::error::AppError;
use crate::models::container::*;
use crate::AppState;

pub async fn create_container(
    State(state): State<AppState>,
    Json(payload): Json<CreateContainerRequest>,
) -> Result<(StatusCode, Json<ContainerResponse>), AppError> {
    let container_id = uuid::Uuid::new_v4().to_string();
    let docker_name = format!("agent-{}", &container_id);

    let skill_repos_json =
        serde_json::to_string(&payload.skill_repos).unwrap_or_else(|_| "[]".to_string());

    let mut env_vars = vec![
        format!("TASK={}", payload.task),
        format!("CONTAINER_ID={}", container_id),
        format!("CONTROL_PLANE_URL=http://host.docker.internal:8080"),
        format!("SKILL_REPOS={}", payload.skill_repos.join(",")),
    ];

    if let Some(extra_env) = &payload.env {
        for (k, v) in extra_env {
            env_vars.push(format!("{}={}", k, v));
        }
    }

    let cpu_limit = payload.cpu_limit.unwrap_or_else(|| "2".to_string());
    let memory_limit = payload.memory_limit.unwrap_or_else(|| "4Gi".to_string());

    let docker_manager = state
        .docker_manager
        .as_ref()
        .ok_or_else(|| AppError::DockerError("Docker not configured".to_string()))?;

    let docker_id = docker_manager
        .create_container(
            &docker_name,
            &state.config.agent_image,
            env_vars,
            &cpu_limit,
            &memory_limit,
        )
        .await?;

    let now = Utc::now().to_rfc3339();
    let container = Container {
        id: container_id.clone(),
        task: payload.task,
        status: "Running".to_string(),
        docker_id: Some(docker_id),
        skill_repos: skill_repos_json,
        cpu_limit,
        memory_limit,
        idle_timeout: payload.idle_timeout.unwrap_or(300),
        max_lifetime: payload.max_lifetime.unwrap_or(3600),
        created_at: now.clone(),
        last_activity: now,
    };

    let response = ContainerResponse {
        id: container.id.clone(),
        status: container.get_status(),
        created_at: container.created_at.clone(),
        docker_id: container.docker_id.clone(),
    };

    state.db.create_container(&container).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Container>, AppError> {
    let container = state.db.get_container(&id).await?;
    Ok(Json(container))
}

pub async fn delete_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let container = state.db.get_container(&id).await?;

    if let Some(docker_id) = &container.docker_id {
        if let Some(docker_manager) = &state.docker_manager {
            let _ = docker_manager.stop_container(docker_id).await;
            let _ = docker_manager.remove_container(docker_id).await;
        }
    }

    state.db.delete_container(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn report_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<StatusReport>,
) -> Result<StatusCode, AppError> {
    state.db.update_last_activity(&id).await?;

    match payload.status.as_str() {
        "completed" => {
            state.db.update_status(&id, "Stopped").await?;
        }
        "failed" => {
            state.db.update_status(&id, "Failed").await?;
        }
        _ => {}
    }

    Ok(StatusCode::OK)
}
