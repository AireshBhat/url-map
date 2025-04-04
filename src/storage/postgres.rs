use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, postgres::PgPoolOptions, Transaction, Postgres};
use std::time::Duration;

use crate::errors::{UrlShortenerError, UrlShortenerErrorType, UrlShortenerResult};
use crate::models::ShortenedUrl;
use super::{Storage, StorageConfig};

pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub async fn new(config: StorageConfig) -> UrlShortenerResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections.unwrap_or(5))
            .acquire_timeout(Duration::from_secs(config.connection_timeout_secs.unwrap_or(30)))
            .connect(&config.connection_string)
            .await
            .map_err(|e| UrlShortenerError::from(UrlShortenerErrorType::ConnectionError(e.to_string())))?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(e.to_string())))?;

        Ok(Self { pool })
    }

    /// Helper function to handle database errors consistently
    fn handle_error(error: sqlx::Error) -> UrlShortenerError {
        match error {
            sqlx::Error::RowNotFound => {
                UrlShortenerError::from(UrlShortenerErrorType::NotFound)
            }
            sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
                // Unique violation error code
                UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(
                    "Short URL already exists".to_string(),
                ))
            }
            _ => UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(error.to_string())),
        }
    }

    /// Helper function to begin a transaction
    async fn begin_tx(&self) -> UrlShortenerResult<Transaction<'_, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(e.to_string())))
    }

    /// Helper function to save URL within a transaction
    async fn save_url_tx(
        tx: &mut Transaction<'_, Postgres>,
        url: &ShortenedUrl,
    ) -> UrlShortenerResult<ShortenedUrl> {
        sqlx::query_as!(
            ShortenedUrl,
            r#"
            INSERT INTO shortened_urls (original_url, short_url, created_at, visits)
            VALUES ($1, $2, $3, $4)
            RETURNING id, original_url, short_url, created_at, visits
            "#,
            url.original_url,
            url.short_url,
            Utc::now(),
            0i64
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(Self::handle_error)
    }

    /// Helper function to get URL within a transaction
    async fn get_url_tx(
        tx: &mut Transaction<'_, Postgres>,
        short_url: &str,
        increment_visits: bool,
    ) -> UrlShortenerResult<ShortenedUrl> {
        if increment_visits {
            sqlx::query_as!(
                ShortenedUrl,
                r#"
                UPDATE shortened_urls 
                SET visits = visits + 1
                WHERE short_url = $1
                RETURNING id, original_url, short_url, created_at, visits
                "#,
                short_url
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(Self::handle_error)
        } else {
            sqlx::query_as!(
                ShortenedUrl,
                r#"
                SELECT id, original_url, short_url, created_at, visits
                FROM shortened_urls
                WHERE short_url = $1
                "#,
                short_url
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(Self::handle_error)
        }
    }
}

#[async_trait]
impl Storage for PostgresStorage {
    async fn save_url(&self, url: ShortenedUrl) -> UrlShortenerResult<ShortenedUrl> {
        let mut tx = self.begin_tx().await?;
        
        let result = Self::save_url_tx(&mut tx, &url).await;
        
        match result {
            Ok(saved_url) => {
                tx.commit().await.map_err(|e| {
                    UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(e.to_string()))
                })?;
                Ok(saved_url)
            }
            Err(e) => {
                tx.rollback().await.map_err(|rollback_err| {
                    UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(format!(
                        "Error: {}. Rollback failed: {}",
                        e,
                        rollback_err
                    )))
                })?;
                Err(e)
            }
        }
    }

    async fn get_url(&self, short_url: &str) -> UrlShortenerResult<ShortenedUrl> {
        let mut tx = self.begin_tx().await?;
        
        let result = Self::get_url_tx(&mut tx, short_url, true).await;
        
        match result {
            Ok(url) => {
                tx.commit().await.map_err(|e| {
                    UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(e.to_string()))
                })?;
                Ok(url)
            }
            Err(e) => {
                tx.rollback().await.map_err(|rollback_err| {
                    UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(format!(
                        "Error: {}. Rollback failed: {}",
                        e,
                        rollback_err
                    )))
                })?;
                Err(e)
            }
        }
    }

    async fn get_stats(&self, short_url: &str) -> UrlShortenerResult<ShortenedUrl> {
        let mut tx = self.begin_tx().await?;
        
        let result = Self::get_url_tx(&mut tx, short_url, false).await;
        
        match result {
            Ok(url) => {
                tx.commit().await.map_err(|e| {
                    UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(e.to_string()))
                })?;
                Ok(url)
            }
            Err(e) => {
                tx.rollback().await.map_err(|rollback_err| {
                    UrlShortenerError::from(UrlShortenerErrorType::DatabaseError(format!(
                        "Error: {}. Rollback failed: {}",
                        e,
                        rollback_err
                    )))
                })?;
                Err(e)
            }
        }
    }
} 