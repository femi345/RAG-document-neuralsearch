use cortex_common::types::*;
use cortex_connectors::pdf_upload;
use cortex_ingestion::pipeline::IngestionPipeline;
use cortex_store::postgres::PostgresStore;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::jobs::JobPayload;

/// Async worker pool that processes ingestion jobs.
pub struct WorkerPool {
    tx: mpsc::Sender<JobPayload>,
}

impl WorkerPool {
    /// Spawn a worker pool with `concurrency` parallel workers.
    pub fn spawn(
        concurrency: usize,
        pipeline: Arc<IngestionPipeline>,
        postgres: PostgresStore,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<JobPayload>(256);
        let rx = Arc::new(tokio::sync::Mutex::new(rx));

        for worker_id in 0..concurrency {
            let rx = rx.clone();
            let pipeline = pipeline.clone();
            let postgres = postgres.clone();

            tokio::spawn(async move {
                loop {
                    let job = {
                        let mut rx = rx.lock().await;
                        rx.recv().await
                    };

                    let Some(job) = job else {
                        tracing::info!(worker_id, "Worker channel closed, shutting down");
                        break;
                    };

                    let job_id = job.job_id();
                    tracing::info!(worker_id, %job_id, "Processing job");

                    // Mark job as running
                    let _ = postgres
                        .update_job_status(job_id, JobStatus::Running, None)
                        .await;

                    let result = process_job(&pipeline, &postgres, job).await;

                    match result {
                        Ok(()) => {
                            let _ = postgres
                                .update_job_status(job_id, JobStatus::Completed, None)
                                .await;
                            tracing::info!(worker_id, %job_id, "Job completed");
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            let _ = postgres
                                .update_job_status(job_id, JobStatus::Failed, Some(&err_msg))
                                .await;
                            tracing::error!(worker_id, %job_id, error = %e, "Job failed");
                        }
                    }
                }
            });
        }

        Self { tx }
    }

    /// Submit a job to the worker pool.
    pub async fn submit(&self, job: JobPayload) -> Result<(), mpsc::error::SendError<JobPayload>> {
        self.tx.send(job).await
    }
}

async fn process_job(
    pipeline: &IngestionPipeline,
    postgres: &PostgresStore,
    job: JobPayload,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match job {
        JobPayload::FileUpload {
            job_id,
            user_id,
            filename,
            content,
        } => {
            let raw_doc = pdf_upload::create_from_text(&filename, content);
            let _ = postgres.update_job_progress(job_id, 0, 1).await;
            pipeline.ingest(raw_doc, user_id).await?;
            let _ = postgres.update_job_progress(job_id, 1, 1).await;
            Ok(())
        }
        JobPayload::FullSync { .. } | JobPayload::IncrementalSync { .. } => {
            // Connector sync will be implemented in Phase 3
            tracing::warn!("Connector sync not yet implemented");
            Ok(())
        }
    }
}
