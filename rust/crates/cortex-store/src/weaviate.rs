use cortex_common::types::*;
use reqwest::Client;
use serde_json::json;

use crate::models::{Chunk, SearchResult};

#[derive(Clone)]
pub struct WeaviateStore {
    client: Client,
    base_url: String,
}

const CHUNK_CLASS: &str = "Chunk";

impl WeaviateStore {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Create the Chunk schema in Weaviate if it doesn't exist.
    pub async fn ensure_schema(&self) -> Result<(), WeaviateError> {
        let url = format!("{}/v1/schema/{}", self.base_url, CHUNK_CLASS);
        let resp = self.client.get(&url).send().await?;

        if resp.status().is_success() {
            tracing::info!("Weaviate Chunk schema already exists");
            return Ok(());
        }

        let schema = json!({
            "class": CHUNK_CLASS,
            "description": "A semantically coherent text chunk from an indexed document",
            "vectorizer": "none",
            "vectorIndexType": "hnsw",
            "vectorIndexConfig": {
                "distance": "cosine",
                "ef": 256,
                "efConstruction": 128,
                "maxConnections": 64
            },
            "properties": [
                {
                    "name": "text",
                    "dataType": ["text"],
                    "tokenization": "word",
                    "indexFilterable": true,
                    "indexSearchable": true
                },
                {
                    "name": "documentId",
                    "dataType": ["text"],
                    "tokenization": "field",
                    "indexFilterable": true,
                    "indexSearchable": false
                },
                {
                    "name": "userId",
                    "dataType": ["text"],
                    "tokenization": "field",
                    "indexFilterable": true,
                    "indexSearchable": false
                },
                {
                    "name": "sourceType",
                    "dataType": ["text"],
                    "tokenization": "field",
                    "indexFilterable": true
                },
                {
                    "name": "documentTitle",
                    "dataType": ["text"],
                    "tokenization": "word",
                    "indexSearchable": true
                },
                {
                    "name": "sourceUrl",
                    "dataType": ["text"],
                    "tokenization": "field",
                    "indexFilterable": false,
                    "indexSearchable": false
                },
                {
                    "name": "chunkIndex",
                    "dataType": ["int"]
                },
                {
                    "name": "sectionTitle",
                    "dataType": ["text"],
                    "tokenization": "word",
                    "indexSearchable": true
                },
                {
                    "name": "metadata",
                    "dataType": ["text"],
                    "tokenization": "field",
                    "indexSearchable": false
                }
            ]
        });

        let url = format!("{}/v1/schema", self.base_url);
        let resp = self.client.post(&url).json(&schema).send().await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WeaviateError::SchemaCreation(body));
        }

        tracing::info!("Created Weaviate Chunk schema");
        Ok(())
    }

    /// Insert a chunk with its embedding vector.
    pub async fn upsert_chunk(
        &self,
        chunk: &Chunk,
        vector: &[f32],
    ) -> Result<(), WeaviateError> {
        let object = json!({
            "class": CHUNK_CLASS,
            "id": chunk.id.0.to_string(),
            "vector": vector,
            "properties": {
                "text": chunk.text,
                "documentId": chunk.document_id.0.to_string(),
                "userId": chunk.user_id.0.to_string(),
                "sourceType": chunk.source_type.to_string(),
                "documentTitle": chunk.document_title,
                "sourceUrl": chunk.source_url,
                "chunkIndex": chunk.chunk_index,
                "sectionTitle": chunk.section_title,
                "metadata": serde_json::to_string(&chunk.metadata).unwrap_or_default(),
            }
        });

        let url = format!("{}/v1/objects", self.base_url);
        let resp = self.client.post(&url).json(&object).send().await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WeaviateError::Insert(body));
        }

        Ok(())
    }

    /// Batch insert multiple chunks with their embeddings.
    pub async fn batch_upsert_chunks(
        &self,
        chunks: &[Chunk],
        vectors: &[Vec<f32>],
    ) -> Result<(), WeaviateError> {
        let objects: Vec<_> = chunks
            .iter()
            .zip(vectors.iter())
            .map(|(chunk, vector)| {
                json!({
                    "class": CHUNK_CLASS,
                    "id": chunk.id.0.to_string(),
                    "vector": vector,
                    "properties": {
                        "text": chunk.text,
                        "documentId": chunk.document_id.0.to_string(),
                        "userId": chunk.user_id.0.to_string(),
                        "sourceType": chunk.source_type.to_string(),
                        "documentTitle": chunk.document_title,
                        "sourceUrl": chunk.source_url,
                        "chunkIndex": chunk.chunk_index,
                        "sectionTitle": chunk.section_title,
                        "metadata": serde_json::to_string(&chunk.metadata).unwrap_or_default(),
                    }
                })
            })
            .collect();

        let batch = json!({ "objects": objects });
        let url = format!("{}/v1/batch/objects", self.base_url);
        let resp = self.client.post(&url).json(&batch).send().await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WeaviateError::Insert(body));
        }

        Ok(())
    }

    /// Hybrid search combining BM25 keyword search and vector similarity.
    pub async fn hybrid_search(
        &self,
        query: &str,
        vector: &[f32],
        user_id: UserId,
        source_filter: Option<&str>,
        limit: usize,
        alpha: f32,
    ) -> Result<Vec<SearchResult>, WeaviateError> {
        let mut where_filter = json!({
            "path": ["userId"],
            "operator": "Equal",
            "valueText": user_id.0.to_string()
        });

        if let Some(source) = source_filter {
            where_filter = json!({
                "operator": "And",
                "operands": [
                    {
                        "path": ["userId"],
                        "operator": "Equal",
                        "valueText": user_id.0.to_string()
                    },
                    {
                        "path": ["sourceType"],
                        "operator": "Equal",
                        "valueText": source
                    }
                ]
            });
        }

        let vector_str: Vec<String> = vector.iter().map(|v| v.to_string()).collect();
        let vector_csv = vector_str.join(", ");

        let graphql = format!(
            r#"{{
                Get {{
                    {class}(
                        hybrid: {{
                            query: {query}
                            vector: [{vector}]
                            alpha: {alpha}
                        }}
                        where: {where_filter}
                        limit: {limit}
                    ) {{
                        text
                        documentId
                        documentTitle
                        sourceType
                        sourceUrl
                        sectionTitle
                        chunkIndex
                        _additional {{
                            id
                            score
                        }}
                    }}
                }}
            }}"#,
            class = CHUNK_CLASS,
            query = serde_json::to_string(query).unwrap(),
            vector = vector_csv,
            alpha = alpha,
            where_filter = serde_json::to_string(&where_filter).unwrap(),
            limit = limit,
        );

        let url = format!("{}/v1/graphql", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&json!({ "query": graphql }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WeaviateError::Query(body));
        }

        let body: serde_json::Value = resp.json().await?;
        let chunks = body["data"]["Get"][CHUNK_CLASS]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let results = chunks
            .into_iter()
            .filter_map(|c| {
                let chunk_id = c["_additional"]["id"].as_str()?.parse().ok()?;
                let doc_id = c["documentId"].as_str()?.parse().ok()?;
                let score = c["_additional"]["score"]
                    .as_str()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);

                Some(SearchResult {
                    chunk_id: ChunkId(chunk_id),
                    document_id: DocumentId(doc_id),
                    text: c["text"].as_str().unwrap_or_default().to_string(),
                    score,
                    document_title: c["documentTitle"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    source_type: c["sourceType"]
                        .as_str()
                        .unwrap_or("pdf_upload")
                        .parse()
                        .unwrap_or(SourceType::PdfUpload),
                    source_url: c["sourceUrl"].as_str().map(String::from),
                    section_title: c["sectionTitle"].as_str().map(String::from),
                })
            })
            .collect();

        Ok(results)
    }

    /// Delete all chunks belonging to a document.
    pub async fn delete_chunks_by_document(
        &self,
        document_id: DocumentId,
    ) -> Result<(), WeaviateError> {
        let batch_delete = json!({
            "match": {
                "class": CHUNK_CLASS,
                "where": {
                    "path": ["documentId"],
                    "operator": "Equal",
                    "valueText": document_id.0.to_string()
                }
            }
        });

        let url = format!("{}/v1/batch/objects", self.base_url);
        let resp = self
            .client
            .delete(&url)
            .json(&batch_delete)
            .send()
            .await?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WeaviateError::Delete(body));
        }

        Ok(())
    }

    /// Check if Weaviate is healthy.
    pub async fn health_check(&self) -> Result<bool, WeaviateError> {
        let url = format!("{}/v1/.well-known/ready", self.base_url);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.status().is_success())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WeaviateError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("schema creation failed: {0}")]
    SchemaCreation(String),
    #[error("insert failed: {0}")]
    Insert(String),
    #[error("query failed: {0}")]
    Query(String),
    #[error("delete failed: {0}")]
    Delete(String),
}
