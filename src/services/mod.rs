use std::collections::HashMap;
use chrono::{DateTime, Utc};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("URL too long")]
    UrlTooLong,
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Clone)]
pub struct ShortenedUrl {
    pub short_code: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
    pub visits: u64,
}

pub struct UrlService {
    // In-memory storage for now, will be replaced with proper storage layer
    urls: HashMap<String, ShortenedUrl>,
}

impl UrlService {
    pub fn new() -> Self {
        Self {
            urls: HashMap::new(),
        }
    }

    pub fn create_short_url(&mut self, original_url: String) -> ServiceResult<ShortenedUrl> {
        // Validate URL
        let url = Url::parse(&original_url)
            .map_err(|_| ServiceError::InvalidUrl(original_url.clone()))?;

        // Check URL length
        if original_url.len() > 2048 {
            return Err(ServiceError::UrlTooLong);
        }

        // Generate short code
        let short_code = self.generate_short_code();

        // Create shortened URL
        let shortened_url = ShortenedUrl {
            short_code: short_code.clone(),
            original_url: url.to_string(),
            created_at: Utc::now(),
            visits: 0,
        };


        // Store the URL
        self.urls.insert(short_code, shortened_url.clone());

        Ok(shortened_url)
    }

    pub fn get_original_url(&mut self, short_code: &str) -> ServiceResult<String> {
        let url = self.urls.get_mut(short_code)
            .ok_or_else(|| ServiceError::Internal("URL not found".to_string()))?;

        // Increment visit count
        url.visits += 1;

        Ok(url.original_url.clone())
    }

    pub fn get_url_stats(&self, short_code: &str) -> ServiceResult<ShortenedUrl> {
        self.urls.get(short_code)
            .cloned()
            .ok_or_else(|| ServiceError::Internal("URL not found".to_string()))
    }

    fn generate_short_code(&self) -> String {
        // Simple implementation for now
        // TODO: Implement a more robust short code generation algorithm
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        let code: String = (0..6)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        code
    }
}

#[cfg(test)]
mod tests; 