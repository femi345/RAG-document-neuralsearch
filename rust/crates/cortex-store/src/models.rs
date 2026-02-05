use chrono::{DateTime, Utc};
use cortex_common::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub user_id: UserId,
    pub source_type: SourceType,
    pub source_id: String,
    pub title: String,
    pub source_url: Option<String>,
    pub content_hash: String,
    pub chunk_count: i32,
    pub mime_type: Option<String>,
    pub metadata: serde_json::Value,
    pub indexed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connector {
    pub id: ConnectorId,
    pub user_id: UserId,
    pub source_type: SourceType,
    pub status: ConnectorStatus,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_cursor: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub user_id: UserId,
    pub connector_id: Option<ConnectorId>,
    pub job_type: JobType,
    pub status: JobStatus,
    pub total_items: i32,
    pub processed_items: i32,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// A chunk stored in Weaviate with its embedding and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: ChunkId,
    pub document_id: DocumentId,
    pub user_id: UserId,
    pub text: String,
    pub source_type: SourceType,
    pub document_title: String,
    pub source_url: Option<String>,
    pub chunk_index: i32,
    pub section_title: Option<String>,
    pub metadata: serde_json::Value,
}

/// A search result returned from Weaviate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: ChunkId,
    pub document_id: DocumentId,
    pub text: String,
    pub score: f32,
    pub document_title: String,
    pub source_type: SourceType,
    pub source_url: Option<String>,
    pub section_title: Option<String>,
}

/// Parameters for creating a new document record.
#[derive(Debug, Clone)]
pub struct CreateDocument {
    pub user_id: UserId,
    pub source_type: SourceType,
    pub source_id: String,
    pub title: String,
    pub source_url: Option<String>,
    pub content_hash: String,
    pub chunk_count: i32,
    pub mime_type: Option<String>,
    pub metadata: serde_json::Value,
}

/// Parameters for creating a new job.
#[derive(Debug, Clone)]
pub struct CreateJob {
    pub user_id: UserId,
    pub connector_id: Option<ConnectorId>,
    pub job_type: JobType,
}
