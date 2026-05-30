use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerStatus {
    Creating,
    Running,
    Idle,
    Stopping,
    Stopped,
    Failed,
}

impl fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerStatus::Creating => write!(f, "Creating"),
            ContainerStatus::Running => write!(f, "Running"),
            ContainerStatus::Idle => write!(f, "Idle"),
            ContainerStatus::Stopping => write!(f, "Stopping"),
            ContainerStatus::Stopped => write!(f, "Stopped"),
            ContainerStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl ContainerStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Creating" => ContainerStatus::Creating,
            "Running" => ContainerStatus::Running,
            "Idle" => ContainerStatus::Idle,
            "Stopping" => ContainerStatus::Stopping,
            "Stopped" => ContainerStatus::Stopped,
            "Failed" => ContainerStatus::Failed,
            _ => ContainerStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Container {
    pub id: String,
    pub task: String,
    pub status: String,
    pub docker_id: Option<String>,
    pub skill_repos: String,
    pub cpu_limit: String,
    pub memory_limit: String,
    pub idle_timeout: i64,
    pub max_lifetime: i64,
    pub created_at: String,
    pub last_activity: String,
}

impl Container {
    pub fn get_status(&self) -> ContainerStatus {
        ContainerStatus::from_str(&self.status)
    }

    pub fn set_status(&mut self, status: ContainerStatus) {
        self.status = status.to_string();
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateContainerRequest {
    pub task: String,
    pub skill_repos: Option<Vec<String>>,
    pub skill_branch: Option<String>,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub idle_timeout: Option<i64>,
    pub max_lifetime: Option<i64>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct ContainerResponse {
    pub id: String,
    pub status: ContainerStatus,
    pub created_at: String,
    pub docker_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StatusReport {
    pub status: String,
    pub progress: f32,
    pub current_step: String,
    pub logs: Vec<String>,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct ListContainersQuery {
    pub status: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total: i64,
    pub by_status: std::collections::HashMap<String, i64>,
}
