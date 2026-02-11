use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use cortex_common::types::*;
use cortex_scheduler::jobs::JobPayload;
use cortex_store::models::CreateJob;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ingest/upload", post(upload))
        .route("/ingest/jobs/{job_id}", get(get_job))
}

#[derive(Debug, Deserialize)]
struct UploadRequest {
    filename: String,
    content: String,
    /// Temporary: pass user_id until auth is implemented.
    user_id: Uuid,
}

#[derive(Debug, Serialize)]
struct UploadResponse {
    job_id: Uuid,
    status: String,
}

async fn upload(
    State(state): State<AppState>,
    Json(req): Json<UploadRequest>,
) -> Result<Json<UploadResponse>, ApiError> {
    if req.content.trim().is_empty() {
        return Err(ApiError::BadRequest("content cannot be empty".to_string()));
    }

    let user_id = UserId(req.user_id);

    // Create job record
    let job_id = state
        .postgres
        .create_job(&CreateJob {
            user_id,
            connector_id: None,
            job_type: JobType::FileUpload,
        })
        .await?;

    // Submit to worker pool
    state
        .worker_pool
        .submit(JobPayload::FileUpload {
            job_id,
            user_id,
            filename: req.filename,
            content: req.content,
        })
        .await
        .map_err(|e| ApiError::Internal(format!("failed to submit job: {e}")))?;

    Ok(Json(UploadResponse {
        job_id: job_id.0,
        status: "queued".to_string(),
    }))
}

#[derive(Debug, Serialize)]
struct JobResponse {
    job_id: Uuid,
    status: JobStatus,
    progress: JobProgress,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct JobProgress {
    total: i32,
    processed: i32,
}

async fn get_job(
    State(state): State<AppState>,
    axum::extract::Path(job_id): axum::extract::Path<Uuid>,
) -> Result<Json<JobResponse>, ApiError> {
    let job = state
        .postgres
        .get_job(JobId(job_id))
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(JobResponse {
        job_id: job.id.0,
        status: job.status,
        progress: JobProgress {
            total: job.total_items,
            processed: job.processed_items,
        },
        error: job.error_message,
    }))
}
