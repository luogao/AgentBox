use axum::{
    extract::{Path, Query, State},
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

    let skill_repos = payload.skill_repos.unwrap_or_default();
    let skill_repos_json =
        serde_json::to_string(&skill_repos).unwrap_or_else(|_| "[]".to_string());

    let mut env_vars = vec![
        format!("TASK={}", payload.task),
        format!("CONTAINER_ID={}", container_id),
        format!("CONTROL_PLANE_URL=http://host.docker.internal:8080"),
        format!("SKILL_REPOS={}", skill_repos.join(",")),
    ];

    {
        let config = state.config.read().await;
        if let Some(ref api_key) = config.anthropic_api_key {
            let already_set = payload.env.as_ref().map_or(false, |e| e.contains_key("ANTHROPIC_API_KEY"));
            if !already_set {
                env_vars.push(format!("ANTHROPIC_API_KEY={}", api_key));
            }
        }
    }

    if let Some(extra_env) = &payload.env {
        for (k, v) in extra_env {
            if !env_vars.iter().any(|e| e.starts_with(&format!("{}=", k))) {
                env_vars.push(format!("{}={}", k, v));
            }
        }
    }

    let cpu_limit = payload.cpu_limit.unwrap_or_else(|| "2".to_string());
    let memory_limit = payload.memory_limit.unwrap_or_else(|| "4Gi".to_string());

    let docker_manager = state
        .docker_manager
        .as_ref()
        .ok_or_else(|| AppError::DockerError("Docker not configured".to_string()))?;

    let agent_image = state.config.read().await.agent_image.clone();
    let docker_id = docker_manager
        .create_container(
            &docker_name,
            &agent_image,
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
            if let Err(e) = docker_manager.stop_container(docker_id).await {
                tracing::warn!(
                    "stop_container({}) failed during delete of {}: {}",
                    docker_id,
                    id,
                    e
                );
            }
            if let Err(e) = docker_manager.remove_container(docker_id).await {
                // 容器可能已被生命周期管理器清理；记录后继续删除 DB 记录
                tracing::warn!(
                    "remove_container({}) failed during delete of {}: {}",
                    docker_id,
                    id,
                    e
                );
            }
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

pub async fn list_containers(
    State(state): State<AppState>,
    Query(params): Query<ListContainersQuery>,
) -> Result<Json<PaginatedResponse<Container>>, AppError> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("desc");
    let status = params.status.as_deref();
    let search = params.search.as_deref();

    let (data, total) = state
        .db
        .list_containers(status, search, sort_by, sort_order, page, per_page)
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i64;

    Ok(Json(PaginatedResponse {
        data,
        total,
        page,
        per_page,
        total_pages,
    }))
}

pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>, AppError> {
    let stats = state.db.get_stats().await?;
    Ok(Json(stats))
}
