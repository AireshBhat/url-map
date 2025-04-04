use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a shortened URL in the system
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShortenedUrl {
    /// Database ID (optional, may not be used in all storage backends)
    pub id: i64,
    /// The original URL that was shortened
    pub original_url: String,
    /// The generated short code for the URL
    pub short_url: String,
    /// When the URL was created
    pub created_at: DateTime<Utc>,
    /// Number of times the URL has been visited
    pub visits: i64,
}

/// Request payload for creating a new shortened URL
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUrlRequest {
    pub original_url: String,
}

/// Response payload for a created shortened URL
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUrlResponse {
    pub short_url: String,
    pub original_url: String,
}

/// Response payload for URL statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct UrlStats {
    pub short_url: String,
    pub original_url: String,
    pub visits: i64,
    pub created_at: DateTime<Utc>,
} 