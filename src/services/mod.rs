use chrono::{DateTime, Utc};
use url::Url;
use crate::errors::{UrlShortenerResult, UrlShortenerErrorType};
use crate::models::ShortenedUrl as StorageShortenedUrl;
use crate::storage::StorageRef;
use nanoid::nanoid;

#[derive(Debug, Clone)]
pub struct ShortenedUrl {
    pub short_code: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
    pub visits: u64,
}

impl From<ShortenedUrl> for StorageShortenedUrl {
    fn from(url: ShortenedUrl) -> Self {
        Self {
            id: 0, // Will be set by storage layer
            original_url: url.original_url,
            short_url: url.short_code,
            created_at: url.created_at,
            visits: url.visits as i64,
        }
    }
}

impl From<StorageShortenedUrl> for ShortenedUrl {
    fn from(url: StorageShortenedUrl) -> Self {
        Self {
            short_code: url.short_url,
            original_url: url.original_url,
            created_at: url.created_at,
            visits: url.visits as u64,
        }
    }
}

pub struct UrlService {
    storage: StorageRef,
}

impl UrlService {
    pub fn new(storage: StorageRef) -> Self {
        Self { storage }
    }

    pub async fn create_short_url(&self, original_url: String) -> UrlShortenerResult<ShortenedUrl> {
        // Validate URL
        let url = Url::parse(&original_url)
            .map_err(|e| UrlShortenerErrorType::InvalidUrl(e.to_string()))?;

        // Check URL length
        if original_url.len() > 2048 {
            return Err(UrlShortenerErrorType::UrlTooLong("URL exceeds 2048 characters".to_string()).into());
        }

        // Generate short code
        let short_code = nanoid!(10);

        // Create shortened URL
        let shortened_url = ShortenedUrl {
            short_code,
            original_url: url.to_string(),
            created_at: Utc::now(),
            visits: 0,
        };

        // Store the URL using the storage layer
        let storage_url: StorageShortenedUrl = shortened_url.into();
        let saved_url = self.storage.save_url(storage_url).await?;

        Ok(saved_url.into())
    }

    pub async fn get_original_url(&self, short_code: &str) -> UrlShortenerResult<String> {
        let url = self.storage.get_url(short_code).await?;
        Ok(url.original_url)
    }

    pub async fn get_url_stats(&self, short_code: &str) -> UrlShortenerResult<ShortenedUrl> {
        let url = self.storage.get_stats(short_code).await?;
        Ok(url.into())
    }
}

#[cfg(test)]
mod tests; 