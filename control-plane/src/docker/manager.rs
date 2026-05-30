use bollard::Docker;
use bollard::query_parameters::CreateContainerOptions;
use bollard::models::{ContainerCreateBody, HostConfig};

use crate::error::AppError;

pub struct DockerManager {
    docker: Docker,
}

impl DockerManager {
    pub async fn new() -> Result<Self, AppError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| AppError::DockerError(e.to_string()))?;
        Ok(Self { docker })
    }

    pub async fn create_container(
        &self,
        name: &str,
        image: &str,
        env_vars: Vec<String>,
        cpu_limit: &str,
        memory_limit: &str,
    ) -> Result<String, AppError> {
        if self.docker.inspect_image(image).await.is_err() {
            return Err(AppError::BadRequest(format!(
                "Image '{}' not found locally. Run: docker build -t {} -f agent-image/Dockerfile .",
                image, image
            )));
        }

        let config = ContainerCreateBody {
            image: Some(image.to_string()),
            env: Some(env_vars),
            host_config: Some(HostConfig {
                memory: Some(parse_memory(memory_limit)),
                cpu_quota: Some(parse_cpu(cpu_limit)),
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: Some(name.to_string()),
            ..Default::default()
        };

        self.docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| AppError::DockerError(e.to_string()))?;

        self.docker
            .start_container(name, None)
            .await
            .map_err(|e| AppError::DockerError(e.to_string()))?;

        Ok(name.to_string())
    }

    pub async fn stop_container(&self, name: &str) -> Result<(), AppError> {
        self.docker
            .stop_container(name, None)
            .await
            .map_err(|e| AppError::DockerError(e.to_string()))?;
        Ok(())
    }

    pub async fn remove_container(&self, name: &str) -> Result<(), AppError> {
        self.docker
            .remove_container(name, None)
            .await
            .map_err(|e| AppError::DockerError(e.to_string()))?;
        Ok(())
    }

    pub fn client(&self) -> &Docker {
        &self.docker
    }

    pub async fn ping(&self) -> bool {
        self.docker.ping().await.is_ok()
    }

    pub async fn image_exists(&self, image: &str) -> bool {
        self.docker.inspect_image(image).await.is_ok()
    }

    pub async fn container_exists(&self, name: &str) -> bool {
        self.docker.inspect_container(name, None).await.is_ok()
    }

    pub async fn is_container_running(&self, name: &str) -> bool {
        match self.docker.inspect_container(name, None).await {
            Ok(info) => info
                .state
                .and_then(|s| s.running)
                .unwrap_or(false),
            Err(_) => false,
        }
    }
}

fn parse_memory(s: &str) -> i64 {
    let s = s.trim();
    if let Some(val) = s.strip_suffix("Gi") {
        val.parse::<i64>().unwrap_or(4) * 1024 * 1024 * 1024
    } else if let Some(val) = s.strip_suffix("Mi") {
        val.parse::<i64>().unwrap_or(4096) * 1024 * 1024
    } else {
        s.parse::<i64>().unwrap_or(4294967296)
    }
}

fn parse_cpu(s: &str) -> i64 {
    s.parse::<i64>().unwrap_or(2) * 100000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory() {
        assert_eq!(parse_memory("4Gi"), 4 * 1024 * 1024 * 1024);
        assert_eq!(parse_memory("512Mi"), 512 * 1024 * 1024);
        assert_eq!(parse_memory("1024"), 1024);
        assert_eq!(parse_memory(" 2Gi "), 2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_cpu() {
        assert_eq!(parse_cpu("1"), 100000);
        assert_eq!(parse_cpu("4"), 400000);
        assert_eq!(parse_cpu("2"), 200000);
    }
}
