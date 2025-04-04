use super::{Storage, StorageConfig};
use crate::models::ShortenedUrl;
use crate::errors::{UrlShortenerResult, UrlShortenerError, UrlShortenerErrorType};
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory storage implementation using a HashMap
pub struct MemoryStorage {
    urls: RwLock<HashMap<String, ShortenedUrl>>,
}

impl MemoryStorage {
    /// Creates a new in-memory storage instance
    pub fn new(_config: StorageConfig) -> Self {
        Self {
            urls: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl Storage for MemoryStorage {
    async fn save_url(&self, url: ShortenedUrl) -> UrlShortenerResult<ShortenedUrl> {
        let mut urls = self.urls.write().map_err(|_| {
            UrlShortenerError::from(UrlShortenerErrorType::InternalError(
                "Failed to acquire write lock".to_string(),
            ))
        })?;

        urls.insert(url.short_url.clone(), url.clone());
        Ok(url)
    }

    async fn get_url(&self, short_code: &str) -> UrlShortenerResult<ShortenedUrl> {
        let mut urls = self.urls.write().map_err(|_| {
            UrlShortenerError::from(UrlShortenerErrorType::InternalError(
                "Failed to acquire write lock".to_string(),
            ))
        })?;

        if let Some(url) = urls.get_mut(short_code) {
            url.visits += 1;
            Ok(url.clone())
        } else {
            Err(UrlShortenerErrorType::NotFound.into())
        }
    }

    async fn get_stats(&self, short_code: &str) -> UrlShortenerResult<ShortenedUrl> {
        let urls = self.urls.read().map_err(|_| {
            UrlShortenerError::from(UrlShortenerErrorType::InternalError(
                "Failed to acquire read lock".to_string(),
            ))
        })?;

        urls.get(short_code)
            .cloned()
            .ok_or_else(|| UrlShortenerErrorType::NotFound.into())
    }
} 