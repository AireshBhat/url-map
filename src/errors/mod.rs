use std::fmt;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;

/// Result type alias for URL Shortener operations
pub type UrlShortenerResult<T> = Result<T, UrlShortenerError>;

/// Main error types for the URL Shortener service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "error", content = "message")]
pub enum UrlShortenerErrorType {
    /// URL validation errors
    #[serde(rename = "invalid_url")]
    InvalidUrl(String),
    
    /// URL is too long
    #[serde(rename = "url_too_long")]
    UrlTooLong(String),
    
    /// Resource not found
    #[serde(rename = "not_found")]
    NotFound,
    
    /// Database errors
    #[serde(rename = "database_error")]
    DatabaseError(String),
    
    /// Connection errors
    #[serde(rename = "connection_error")]
    ConnectionError(String),
    
    /// Input validation errors
    #[serde(rename = "invalid_input")]
    InvalidInput(String),
    
    /// Rate limiting errors
    #[serde(rename = "rate_limit_exceeded")]
    RateLimitExceeded,
    
    /// Security related errors
    #[serde(rename = "blocked_url")]
    BlockedUrl(String),
    
    /// Internal server errors
    #[serde(rename = "internal_error")]
    InternalError(String),
}

/// Main error structure that includes context and backtrace
pub struct UrlShortenerError {
    /// The type of error that occurred
    pub error_type: UrlShortenerErrorType,
    /// The underlying error that caused this error
    pub source: Option<anyhow::Error>,
    /// Backtrace for debugging
    pub backtrace: Backtrace,
}

impl UrlShortenerError {
    /// Creates a new error with the given type
    pub fn new(error_type: UrlShortenerErrorType) -> Self {
        Self {
            error_type,
            source: None,
            backtrace: Backtrace::capture(),
        }
    }

    /// Creates a new error with the given type and source error
    pub fn with_source<E>(error_type: UrlShortenerErrorType, source: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self {
            error_type,
            source: Some(source.into()),
            backtrace: Backtrace::capture(),
        }
    }
}

impl fmt::Debug for UrlShortenerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("UrlShortenerError");
        builder.field("type", &self.error_type);
        
        if let Some(source) = &self.source {
            builder.field("source", source);
        }
        
        builder.field("backtrace", &self.backtrace);
        builder.finish()
    }
}

impl fmt::Display for UrlShortenerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.error_type)?;
        if let Some(source) = &self.source {
            write!(f, ": {}", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for UrlShortenerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

impl actix_web::ResponseError for UrlShortenerError {
    fn status_code(&self) -> StatusCode {
        match &self.error_type {
            UrlShortenerErrorType::NotFound => StatusCode::NOT_FOUND,
            UrlShortenerErrorType::InvalidUrl(_) |
            UrlShortenerErrorType::UrlTooLong(_) |
            UrlShortenerErrorType::InvalidInput(_) => StatusCode::BAD_REQUEST,
            UrlShortenerErrorType::BlockedUrl(_) => StatusCode::FORBIDDEN,
            UrlShortenerErrorType::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            UrlShortenerErrorType::DatabaseError(_) |
            UrlShortenerErrorType::ConnectionError(_) |
            UrlShortenerErrorType::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let json = serde_json::json!({
            "error": self.error_type,
            "status": self.status_code().as_u16(),
        });

        actix_web::HttpResponse::build(self.status_code())
            .json(json)
    }
}

// Implement From for common error types
impl From<sqlx::Error> for UrlShortenerError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::new(UrlShortenerErrorType::NotFound),
            sqlx::Error::Database(db_err) => {
                Self::with_source(
                    UrlShortenerErrorType::DatabaseError(db_err.to_string()),
                    db_err,
                )
            }
            sqlx::Error::PoolTimedOut => {
                Self::new(UrlShortenerErrorType::ConnectionError(
                    "Database connection pool timeout".to_string(),
                ))
            }
            _ => Self::with_source(
                UrlShortenerErrorType::DatabaseError("Database error".to_string()),
                err,
            ),
        }
    }
}

impl From<url::ParseError> for UrlShortenerError {
    fn from(err: url::ParseError) -> Self {
        Self::with_source(
            UrlShortenerErrorType::InvalidUrl(err.to_string()),
            err,
        )
    }
}

impl From<std::io::Error> for UrlShortenerError {
    fn from(err: std::io::Error) -> Self {
        Self::with_source(
            UrlShortenerErrorType::InternalError(err.to_string()),
            err,
        )
    }
}

// Convenience constructor for NotFound errors
impl From<UrlShortenerErrorType> for UrlShortenerError {
    fn from(error_type: UrlShortenerErrorType) -> Self {
        Self::new(error_type)
    }
} 