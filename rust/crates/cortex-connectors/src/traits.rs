use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cortex_common::types::SourceType;
use serde::{Deserialize, Serialize};

/// A raw document fetched from a data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawDocument {
    /// Unique ID within the source system (e.g., Notion page ID, Gmail message ID).
    pub source_id: String,
    pub source_type: SourceType,
    pub title: String,
    /// Extracted text content.
    pub content: String,
    pub mime_type: String,
    /// Source-specific metadata.
    pub metadata: serde_json::Value,
    /// SHA-256 hash of content for change detection.
    pub content_hash: String,
    pub fetched_at: DateTime<Utc>,
    /// Link back to the original document.
    pub source_url: Option<String>,
}

/// OAuth2 credentials stored per connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

/// Every data source connector implements this trait.
#[async_trait]
pub trait Connector: Send + Sync {
    /// Fetch all documents (full sync).
    async fn fetch_all(
        &self,
        credentials: &Credentials,
    ) -> Result<Vec<RawDocument>, ConnectorError>;

    /// Fetch only documents changed since `since`.
    async fn fetch_incremental(
        &self,
        credentials: &Credentials,
        since: DateTime<Utc>,
    ) -> Result<Vec<RawDocument>, ConnectorError>;

    /// Validate that credentials are still valid.
    async fn validate_credentials(
        &self,
        credentials: &Credentials,
    ) -> Result<bool, ConnectorError>;

    fn source_type(&self) -> SourceType;
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("authentication failed: {0}")]
    AuthFailed(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
