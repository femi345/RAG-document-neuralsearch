use cortex_chunker::{estimate_tokens, strategies};
use cortex_common::types::*;
use cortex_connectors::traits::RawDocument;
use cortex_ml_client::MlClient;
use cortex_store::models::{Chunk, CreateDocument};
use cortex_store::postgres::PostgresStore;
use cortex_store::weaviate::WeaviateStore;

#[derive(Debug)]
pub enum IngestResult {
    Indexed { chunk_count: usize },
    Skipped,
}

pub struct IngestionPipeline {
    postgres: PostgresStore,
    weaviate: WeaviateStore,
    ml_client: MlClient,
}

impl IngestionPipeline {
    pub fn new(postgres: PostgresStore, weaviate: WeaviateStore, ml_client: MlClient) -> Self {
        Self {
            postgres,
            weaviate,
            ml_client,
        }
    }

    /// Process a single document through the full ingestion pipeline.
    pub async fn ingest(
        &self,
        doc: RawDocument,
        user_id: UserId,
    ) -> Result<IngestResult, IngestionError> {
        // 1. Check content hash — skip if unchanged
        let already_indexed = self
            .postgres
            .has_content_hash(user_id, &doc.source_type.to_string(), &doc.content_hash)
            .await
            .map_err(|e| IngestionError::Database(e.to_string()))?;

        if already_indexed {
            tracing::debug!(source_id = %doc.source_id, "Document unchanged, skipping");
            return Ok(IngestResult::Skipped);
        }

        // 2. Parse into sections
        let parsed = crate::parser::parse_text(&doc.title, &doc.content);

        // 3. Select chunking strategy and chunk
        let token_count = estimate_tokens(&doc.content);
        let chunker = strategies::select_strategy(doc.source_type, token_count);

        let mut all_chunks = Vec::new();
        for section in &parsed.sections {
            let text_chunks = chunker.chunk(&section.content, section.title.as_deref());
            all_chunks.extend(text_chunks);
        }

        if all_chunks.is_empty() {
            tracing::warn!(source_id = %doc.source_id, "No chunks produced, skipping");
            return Ok(IngestResult::Skipped);
        }

        // 4. Generate embeddings via ML service
        let texts: Vec<String> = all_chunks.iter().map(|c| c.text.clone()).collect();
        let embeddings = self
            .ml_client
            .embed_batch(texts, None)
            .await
            ?;

        // 5. Create document record in Postgres
        let doc_id = self
            .postgres
            .create_document(&CreateDocument {
                user_id,
                source_type: doc.source_type,
                source_id: doc.source_id.clone(),
                title: doc.title.clone(),
                source_url: doc.source_url.clone(),
                content_hash: doc.content_hash.clone(),
                chunk_count: all_chunks.len() as i32,
                mime_type: Some(doc.mime_type.clone()),
                metadata: doc.metadata.clone(),
            })
            .await
            .map_err(|e| IngestionError::Database(e.to_string()))?;

        // 6. Delete any old chunks for this document
        let _ = self.weaviate.delete_chunks_by_document(doc_id).await;

        // 7. Build chunk models and batch upsert to Weaviate
        let chunks: Vec<Chunk> = all_chunks
            .iter()
            .enumerate()
            .map(|(i, tc)| Chunk {
                id: ChunkId::new(),
                document_id: doc_id,
                user_id,
                text: tc.text.clone(),
                source_type: doc.source_type,
                document_title: doc.title.clone(),
                source_url: doc.source_url.clone(),
                chunk_index: i as i32,
                section_title: tc.section_title.clone(),
                metadata: doc.metadata.clone(),
            })
            .collect();

        self.weaviate
            .batch_upsert_chunks(&chunks, &embeddings)
            .await
            ?;

        let chunk_count = chunks.len();
        tracing::info!(
            source_id = %doc.source_id,
            chunk_count,
            "Document ingested successfully"
        );

        Ok(IngestResult::Indexed { chunk_count })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IngestionError {
    #[error("database error: {0}")]
    Database(String),
    #[error("vector store error: {0}")]
    VectorStore(#[from] cortex_store::weaviate::WeaviateError),
    #[error("ML service error: {0}")]
    MlService(#[from] cortex_ml_client::MlClientError),
    #[error("parse error: {0}")]
    Parse(String),
}
