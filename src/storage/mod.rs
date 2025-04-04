use crate::models::ShortenedUrl;
use crate::errors::UrlShortenerResult;
use async_trait::async_trait;
use std::sync::Arc;

mod memory;
pub use memory::MemoryStorage;

/// The main storage trait that defines the interface for all storage backends
#[async_trait]
pub trait Storage: Send + Sync {
    /// Saves a URL entry to storage
    async fn save_url(&self, url: ShortenedUrl) -> UrlShortenerResult<ShortenedUrl>;
    
    /// Retrieves a shortened URL by its short code and increments the visit count
    async fn get_url(&self, short_code: &str) -> UrlShortenerResult<ShortenedUrl>;
    
    /// Gets statistics for a shortened URL without incrementing the visit count
    async fn get_stats(&self, short_code: &str) -> UrlShortenerResult<ShortenedUrl>;
}

/// A type alias for a shared storage reference
pub type StorageRef = Arc<dyn Storage>;

/// Configuration for storage backends
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// The connection string for the database
    pub connection_string: String,
    /// The maximum number of connections in the pool
    pub max_connections: u32,
    /// The minimum number of connections in the pool
    pub min_connections: u32,
    /// The connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            connection_string: "sqlite::memory:".to_string(),
            max_connections: 10,
            min_connections: 2,
            connection_timeout: 30,
        }
    }
} 