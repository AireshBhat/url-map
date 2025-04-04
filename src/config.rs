use std::env;
use crate::storage::StorageConfig;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub max_connections: Option<u32>,
    pub connection_timeout_secs: Option<u64>,
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://airesh:abcd@localhost:5432/url_shortener".to_string(),
            max_connections: Some(5),
            connection_timeout_secs: Some(30),
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| Self::default().database_url),
            max_connections: env::var("POSTGRES_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .or(Self::default().max_connections),
            connection_timeout_secs: env::var("POSTGRES_CONNECTION_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .or(Self::default().connection_timeout_secs),
            host: env::var("HOST")
                .unwrap_or_else(|_| Self::default().host),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(Self::default().port),
        }
    }

    pub fn to_storage_config(&self) -> StorageConfig {
        StorageConfig {
            connection_string: self.database_url.clone(),
            max_connections: self.max_connections,
            connection_timeout_secs: self.connection_timeout_secs,
        }
    }
} 