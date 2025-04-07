use chrono::{DateTime, Utc};
use tracing::{debug, error, info, instrument, warn};
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
        debug!("Creating new UrlService instance");
        Self { storage }
    }

    #[instrument(skip(self), fields(url_length = original_url.len()))]
    pub async fn create_short_url(&self, original_url: String) -> UrlShortenerResult<ShortenedUrl> {
        debug!("Attempting to create short URL");

        // Validate URL
        let url = match Url::parse(&original_url) {
            Ok(url) => {
                debug!(scheme = %url.scheme(), host = %url.host_str().unwrap_or("unknown"), "URL parsed successfully");
                url
            },
            Err(e) => {
                warn!(error = %e, "Invalid URL format");
                return Err(UrlShortenerErrorType::InvalidUrl(e.to_string()).into());
            }
        };

        // Check URL length
        if original_url.len() > 2048 {
            warn!(length = original_url.len(), "URL exceeds maximum length");
            return Err(UrlShortenerErrorType::UrlTooLong("URL exceeds 2048 characters".to_string()).into());
        }

        // Generate short code
        let short_code = nanoid!(10);
        debug!(short_code = %short_code, "Generated short code");

        // Create shortened URL
        let shortened_url = ShortenedUrl {
            short_code: short_code.clone(),
            original_url: url.to_string(),
            created_at: Utc::now(),
            visits: 0,
        };

        // Store the URL using the storage layer
        let storage_url: StorageShortenedUrl = shortened_url.into();
        match self.storage.save_url(storage_url).await {
            Ok(saved_url) => {
                info!(
                    short_code = %short_code,
                    original_url = %url,
                    "Successfully created short URL"
                );
                Ok(saved_url.into())
            },
            Err(e) => {
                error!(
                    error = %e,
                    short_code = %short_code,
                    original_url = %url,
                    "Failed to save URL"
                );
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn get_original_url(&self, short_code: &str) -> UrlShortenerResult<String> {
        debug!(short_code = %short_code, "Looking up original URL");
        
        match self.storage.get_url(short_code).await {
            Ok(url) => {
                info!(
                    short_code = %short_code,
                    original_url = %url.original_url,
                    "Successfully retrieved original URL"
                );
                Ok(url.original_url)
            },
            Err(e) => {
                warn!(
                    error = %e,
                    short_code = %short_code,
                    "Failed to retrieve original URL"
                );
                Err(e)
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn get_url_stats(&self, short_code: &str) -> UrlShortenerResult<ShortenedUrl> {
        debug!(short_code = %short_code, "Retrieving URL statistics");
        
        match self.storage.get_stats(short_code).await {
            Ok(url) => {
                info!(
                    short_code = %short_code,
                    visits = %url.visits,
                    created_at = %url.created_at,
                    "Successfully retrieved URL statistics"
                );
                Ok(url.into())
            },
            Err(e) => {
                warn!(
                    error = %e,
                    short_code = %short_code,
                    "Failed to retrieve URL statistics"
                );
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests; 