mod memory;
mod postgres;

pub use memory::MemoryStorage;
pub use postgres::PostgresStorage;

use async_trait::async_trait;
use std::sync::Arc;
use crate::errors::UrlShortenerResult;
use crate::models::ShortenedUrl;

/// The main storage trait that defines the interface for all storage backends
#[async_trait]
pub trait Storage: Send + Sync {
    /// Saves a shortened URL to storage
    async fn save_url(&self, url: ShortenedUrl) -> UrlShortenerResult<ShortenedUrl>;
    
    /// Retrieves a shortened URL by its short code and increments the visit count
    async fn get_url(&self, short_code: &str) -> UrlShortenerResult<ShortenedUrl>;
    
    /// Gets statistics for a shortened URL without incrementing the visit count
    async fn get_stats(&self, short_code: &str) -> UrlShortenerResult<ShortenedUrl>;
}

/// A type alias for a shared storage reference
pub type StorageRef = Arc<dyn Storage>;

/// Configuration for storage backends
#[derive(Clone, Debug)]
pub struct StorageConfig {
    /// The connection string for the database
    pub connection_string: String,
    /// The maximum number of connections in the pool
    pub max_connections: Option<u32>,
    /// The connection timeout in seconds
    pub connection_timeout_secs: Option<u64>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            connection_string: "memory".to_string(),
            max_connections: None,
            connection_timeout_secs: None,
        }
    }
} 