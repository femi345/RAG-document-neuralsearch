use cortex_common::types::*;

/// A job to be executed by the worker pool.
#[derive(Debug, Clone)]
pub enum JobPayload {
    /// Ingest a file that was uploaded.
    FileUpload {
        job_id: JobId,
        user_id: UserId,
        filename: String,
        content: String,
    },
    /// Run a full sync for a connector.
    FullSync {
        job_id: JobId,
        user_id: UserId,
        connector_id: ConnectorId,
    },
    /// Run an incremental sync for a connector.
    IncrementalSync {
        job_id: JobId,
        user_id: UserId,
        connector_id: ConnectorId,
    },
}

impl JobPayload {
    pub fn job_id(&self) -> JobId {
        match self {
            JobPayload::FileUpload { job_id, .. } => *job_id,
            JobPayload::FullSync { job_id, .. } => *job_id,
            JobPayload::IncrementalSync { job_id, .. } => *job_id,
        }
    }
}
