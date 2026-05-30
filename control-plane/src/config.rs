use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub agent_image: String,
    pub api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .ok()
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:8080".to_string(),
                    "http://127.0.0.1:3000".to_string(),
                    "http://127.0.0.1:8080".to_string(),
                ]
            });

        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:agent_sandbox.db?mode=rwc".to_string()),
            server_addr: env::var("SERVER_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            agent_image: env::var("AGENT_IMAGE")
                .unwrap_or_else(|_| "agent-sandbox:latest".to_string()),
            api_key: env::var("API_KEY").ok().filter(|k| !k.is_empty()),
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.is_empty()),
            allowed_origins,
        }
    }
}
