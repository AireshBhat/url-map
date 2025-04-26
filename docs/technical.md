# URL Shortener Service - Technical Architecture
## Overview

This document outlines the technical architecture for a URL shortener service built using Rust, Actix-web, and SQLx. The system follows a layered architecture with trait-based abstractions and asynchronous I/O patterns.

## Technology Stack

* Backend Framework: Actix-web 4.4.0
* Database Access: SQLx 0.7 (PostgreSQL)
* Language: Rust 2021 Edition
* Logging: tracing, tracing-subscriber, tracing-actix-web
* Serialization: serde, serde_json
* ID Generation: nanoid
* Async Runtime: tokio 1.32.0
* Error Handling: anyhow, thiserror
* Metrics: prometheus-client (in progress)

## Core Architecture

The system is designed as a layered architecture with the following components:

### 1. HTTP Layer (`src/main.rs`, `src/routes.rs`)

The HTTP layer is implemented using Actix-web and handles all incoming HTTP requests. Key components:

* **Server Setup**: Configured in `src/main.rs` using `HttpServer` and `App`
* **Route Configuration**: Defined in `src/routes.rs` with endpoints for URL operations
* **Middleware**: Includes:
  * Logging middleware (`src/middleware/`)
  * Error handling middleware
  * Request tracing
  * Compression middleware

### 2. Handler Layer (`src/handlers/`)

The handler layer processes HTTP requests and delegates to the service layer. Key characteristics:

* Handlers are organized by functionality
* Each handler:
  * Validates input parameters
  * Extracts request data
  * Calls appropriate service methods
  * Formats responses
* Uses dependency injection for service access
* Includes handlers for URL creation, redirection, and statistics

### 3. Service Layer (`src/services/`)

The service layer contains business logic and orchestrates data operations:

* Implements core business logic including:
  * URL shortening algorithm
  * Unique ID generation using nanoid
  * URL validation
  * Access tracking
* Communicates with storage layer through traits
* Handles error mapping between layers

### 4. Storage Layer (`src/storage/`)

The storage layer handles data persistence:

* **Storage Traits**: Abstract interfaces for data access
* **PostgreSQL Implementation**: Using SQLx for async database operations
* **In-Memory Implementation**: For testing and development
* Handles:
  * Connection pooling
  * Query execution
  * Error handling
  * Transaction management

### 5. Error Handling (`src/errors/`)

Comprehensive error handling system:

* Custom error types for different failure scenarios
* Integration with Actix-web's error handling
* Structured error propagation across layers
* Error logging and tracing

### 6. Configuration (`src/config.rs`)

* Environment-based configuration
* Database connection settings
* Application settings
* Logging configuration

### 7. Logging (`src/logging.rs`)

* Structured logging using tracing
* Request/response logging
* Error logging
* Performance metrics
* JSON formatting for logs
* Correlation IDs for request tracking

## Key Architectural Patterns

1. **Trait-based Design**: Storage operations defined through traits for flexibility
2. **Dependency Injection**: Services receive storage implementations via constructor
3. **Middleware Pipeline**: Requests processed through a series of middleware
4. **Async/Await**: Leverages Rust's async capabilities for non-blocking I/O
5. **Error Propagation**: Structured error handling across architectural layers

## Data Flow

1. HTTP Request → HTTP Layer
2. HTTP Layer → Handler Layer
3. Handler Layer → Service Layer
4. Service Layer → Storage Layer
5. Storage Layer → Database
6. Response flows back up the chain

## Development Guidelines

1. **Error Handling**:
   * Use custom error types for domain-specific errors
   * Implement proper error propagation
   * Log errors with context
   * Provide meaningful error messages

2. **Logging**:
   * Use structured logging for all operations
   * Include request IDs in logs
   * Log errors with full context
   * Use appropriate log levels

3. **Testing**:
   * Unit tests for business logic
   * Integration tests for API endpoints
   * Database tests for storage layer
   * Error handling tests

4. **Performance**:
   * Use async/await for I/O operations
   * Implement connection pooling
   * Cache frequently accessed data
   * Monitor and optimize database queries

5. **Security**:
   * Validate all inputs
   * Sanitize URLs
   * Implement rate limiting
   * Use secure database connections

## Data Models

### URL Models (`src/models/mod.rs`)

The URL model represents a shortened URL in the system:

```rust
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct ShortenedUrl {
    pub id: i64,  
    pub original_url: String,
    pub short_url: String, 
    pub created_at: DateTime<Utc>,
    pub visits: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUrlRequest {
    pub original_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUrlResponse {
    pub short_url: String,
    pub original_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlStats {
    pub short_url: String,
    pub original_url: String,
    pub visits: i64,
    pub created_at: DateTime<Utc>,
}
```

# URL Shortener Error Handling Mechanism

## Overview

The URL Shortener implements a comprehensive error handling system that provides consistent error reporting across the application. The system is built around custom error types and uses Rust's Result type for error propagation.

## Core Components

### 1. Error Types

The system defines the error types in `UrlShortenerErrorType`:

```rust
// src/errors.rs
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EnumIter, Hash)]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
#[non_exhaustive]
pub enum UrlShortenerErrorType {
    // Input validation errors
    InvalidUrl,
    UrlTooLong,
    
    // Database errors
    DatabaseError(String),
    ConnectionError(String),
    
    // Input/Output errors
    InvalidInput(String),
    
    // HTTP errors 
    NoContentTypeHeader,
    InvalidContentType,
    
    // Security
    BlockedUrl,
    
    // System errors
    InternalError(String),
    
    // Generic
    Unknown(String),
}
```

### 2. Main Error Structure

```rust
// src/errors.rs
use std::{fmt, backtrace::Backtrace};

pub type UrlShortenerResult<T> = Result<T, UrlShortenerError>;

pub struct UrlShortenerError {
    pub error_type: UrlShortenerErrorType,
    pub inner: anyhow::Error,
    pub context: Backtrace,
}

impl<T> From<T> for UrlShortenerError
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        let cause = t.into();
        let error_type = match cause.downcast_ref::<sqlx::Error>() {
            Some(sqlx::Error::RowNotFound) => UrlShortenerErrorType::NotFound,
            Some(db_err) => UrlShortenerErrorType::DatabaseError(format!("{}", db_err)),
            _ => UrlShortenerErrorType::Unknown(format!("{}", &cause))
        };
        UrlShortenerError {
            error_type,
            inner: cause,
            context: Backtrace::capture(),
        }
    }
}

impl Debug for UrlShortenerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UrlShortenerError")
         .field("message", &self.error_type)
         .field("inner", &self.inner)
         .field("context", &self.context)
         .finish()
    }
}

impl fmt::Display for UrlShortenerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", &self.error_type)?;
        writeln!(f, "{}", self.inner)?;
        fmt::Display::fmt(&self.context, f)
    }
}
```

### 3. HTTP Integration

```rust
// src/errors.rs
impl actix_web::error::ResponseError for UrlShortenerError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;
        
        match self.error_type {
            UrlShortenerErrorType::NotFound => StatusCode::NOT_FOUND,
            UrlShortenerErrorType::InvalidUrl | 
            UrlShortenerErrorType::UrlTooLong | 
            UrlShortenerErrorType::InvalidInput(_) => StatusCode::BAD_REQUEST,
            UrlShortenerErrorType::DatabaseError(_) |
            UrlShortenerErrorType::ConnectionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UrlShortenerErrorType::BlockedUrl => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(&self.error_type)
    }
}
```

### 4. Error Type Conversions

```rust
// src/errors.rs
impl From<UrlShortenerErrorType> for UrlShortenerError {
    fn from(error_type: UrlShortenerErrorType) -> Self {
        let inner = anyhow::anyhow!("{}", error_type);
        UrlShortenerError {
            error_type,
            inner,
            context: Backtrace::capture(),
        }
    }
}

// Conversion from database errors
impl From<sqlx::Error> for UrlShortenerError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => UrlShortenerErrorType::NotFound.into(),
            sqlx::Error::Database(db_err) => {
                UrlShortenerErrorType::DatabaseError(db_err.to_string()).into()
            }
            sqlx::Error::PoolTimedOut => {
                UrlShortenerErrorType::ConnectionError("Database connection timed out".to_string()).into()
            }
            _ => UrlShortenerErrorType::Unknown(format!("Database error: {}", error)).into(),
        }
    }
}
```

## Error Validation Helpers

```rust
// src/utils/validators.rs
pub fn is_valid_url(url: &str) -> bool {
    // Simple validation
    if url.is_empty() {
        return false;
    }
    
    match url::Url::parse(url) {
        Ok(parsed_url) => {
            let scheme = parsed_url.scheme();
            scheme == "http" || scheme == "https"
        }
        Err(_) => false,
    }
}

pub fn is_valid_short_id(id: &str) -> bool {
    // Only allow alphanumeric characters
    !id.is_empty() && id.len() <= 10 && id.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn is_url_blocked(url: &str) -> bool {
    // Check against a list of blocked domains
    // This would be more sophisticated in production
    let blocked_domains = ["spam.org", "phishing.net"];
    
    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(domain) = parsed_url.host_str() {
            return blocked_domains.contains(&domain);
        }
    }
    
    false
}
```

## Best Practices

1. **Use Error Types**: Use the most specific `UrlShortenerErrorType` for each error case and add a new error type if needed
2. **Error Propagation**: Use the `?` operator to propagate errors up the call stack
3. **Custom Context**: Use `with_error_type()` to add context to errors from external libraries
4. **Consistent Returns**: Always return `UrlShortenerResult<T>` from functions that can fail
5. **Validation First**: Validate inputs early to prevent deeper errors
6. **Error Testing**: Write tests for error serialization and HTTP status code mapping
7. **Error Logging**: Log errors with context before returning them to the client
```