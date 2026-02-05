use chrono::Utc;
use cortex_common::types::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::*;

#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        let migrations = [
            include_str!("migrations/001_init.sql"),
            include_str!("migrations/002_connectors.sql"),
            include_str!("migrations/003_jobs.sql"),
        ];

        for (i, sql) in migrations.iter().enumerate() {
            tracing::info!("Running migration {}", i + 1);
            sqlx::raw_sql(sql).execute(&self.pool).await?;
        }

        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // ── Documents ──

    pub async fn create_document(&self, doc: &CreateDocument) -> Result<DocumentId, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO documents (id, user_id, source_type, source_id, title, source_url, content_hash, chunk_count, mime_type, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (user_id, source_type, source_id)
            DO UPDATE SET
                title = EXCLUDED.title,
                content_hash = EXCLUDED.content_hash,
                chunk_count = EXCLUDED.chunk_count,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            "#,
        )
        .bind(id)
        .bind(doc.user_id.0)
        .bind(doc.source_type.to_string())
        .bind(&doc.source_id)
        .bind(&doc.title)
        .bind(&doc.source_url)
        .bind(&doc.content_hash)
        .bind(doc.chunk_count)
        .bind(&doc.mime_type)
        .bind(&doc.metadata)
        .execute(&self.pool)
        .await?;

        Ok(DocumentId(id))
    }

    pub async fn get_document(&self, id: DocumentId) -> Result<Option<Document>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, source_type, source_id, title, source_url,
                   content_hash, chunk_count, mime_type, metadata, indexed_at, updated_at
            FROM documents WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| document_from_row(&r)))
    }

    pub async fn list_documents(
        &self,
        user_id: UserId,
        source_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Document>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, source_type, source_id, title, source_url,
                   content_hash, chunk_count, mime_type, metadata, indexed_at, updated_at
            FROM documents
            WHERE user_id = $1 AND ($2::text IS NULL OR source_type = $2)
            ORDER BY updated_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_id.0)
        .bind(source_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(document_from_row).collect())
    }

    pub async fn delete_document(&self, id: DocumentId) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn has_content_hash(
        &self,
        user_id: UserId,
        source_type: &str,
        content_hash: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE user_id = $1 AND source_type = $2 AND content_hash = $3) as exists",
        )
        .bind(user_id.0)
        .bind(source_type)
        .bind(content_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get::<bool, _>("exists"))
    }

    // ── Jobs ──

    pub async fn create_job(&self, job: &CreateJob) -> Result<JobId, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO jobs (id, user_id, connector_id, job_type, status)
            VALUES ($1, $2, $3, $4, 'queued')
            "#,
        )
        .bind(id)
        .bind(job.user_id.0)
        .bind(job.connector_id.map(|c| c.0))
        .bind(job.job_type.to_string())
        .execute(&self.pool)
        .await?;

        Ok(JobId(id))
    }

    pub async fn update_job_status(
        &self,
        id: JobId,
        status: JobStatus,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        let (started, completed) = match status {
            JobStatus::Running => (Some(now), None),
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => (None, Some(now)),
            _ => (None, None),
        };

        sqlx::query(
            r#"
            UPDATE jobs SET
                status = $2,
                error_message = COALESCE($3, error_message),
                started_at = COALESCE($4, started_at),
                completed_at = COALESCE($5, completed_at)
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .bind(status.to_string())
        .bind(error)
        .bind(started)
        .bind(completed)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_job_progress(
        &self,
        id: JobId,
        processed: i32,
        total: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE jobs SET processed_items = $2, total_items = $3 WHERE id = $1")
            .bind(id.0)
            .bind(processed)
            .bind(total)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_job(&self, id: JobId) -> Result<Option<Job>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, connector_id, job_type, status,
                   total_items, processed_items, error_message,
                   started_at, completed_at, created_at
            FROM jobs WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| job_from_row(&r)))
    }
}

fn document_from_row(row: &sqlx::postgres::PgRow) -> Document {
    Document {
        id: DocumentId(row.get("id")),
        user_id: UserId(row.get("user_id")),
        source_type: row
            .get::<String, _>("source_type")
            .parse()
            .unwrap_or(SourceType::PdfUpload),
        source_id: row.get("source_id"),
        title: row.get("title"),
        source_url: row.get("source_url"),
        content_hash: row.get("content_hash"),
        chunk_count: row.get("chunk_count"),
        mime_type: row.get("mime_type"),
        metadata: row.get("metadata"),
        indexed_at: row.get("indexed_at"),
        updated_at: row.get("updated_at"),
    }
}

fn job_from_row(row: &sqlx::postgres::PgRow) -> Job {
    let job_type_str: String = row.get("job_type");
    let status_str: String = row.get("status");

    Job {
        id: JobId(row.get("id")),
        user_id: UserId(row.get("user_id")),
        connector_id: row.get::<Option<Uuid>, _>("connector_id").map(ConnectorId),
        job_type: serde_json::from_str(&format!("\"{}\"", job_type_str))
            .unwrap_or(JobType::FileUpload),
        status: serde_json::from_str(&format!("\"{}\"", status_str)).unwrap_or(JobStatus::Failed),
        total_items: row.get("total_items"),
        processed_items: row.get("processed_items"),
        error_message: row.get("error_message"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        created_at: row.get("created_at"),
    }
}
