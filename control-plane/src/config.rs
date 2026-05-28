use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub agent_image: String,
    pub api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:agent_sandbox.db?mode=rwc".to_string()),
            server_addr: env::var("SERVER_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            agent_image: env::var("AGENT_IMAGE")
                .unwrap_or_else(|_| "agent-sandbox:latest".to_string()),
            api_key: env::var("API_KEY").ok().filter(|k| !k.is_empty()),
        }
    }
}
