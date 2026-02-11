use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use cortex_common::types::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/search", post(search))
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    query: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
    source_filter: Option<String>,
    #[serde(default = "default_alpha")]
    alpha: f32,
    /// Temporary: pass user_id in request until auth is implemented.
    user_id: Uuid,
}

fn default_top_k() -> usize {
    10
}

fn default_alpha() -> f32 {
    0.7
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    results: Vec<SearchResultItem>,
    query: String,
    total: usize,
}

#[derive(Debug, Serialize)]
struct SearchResultItem {
    chunk_id: Uuid,
    document_id: Uuid,
    text: String,
    score: f32,
    document_title: String,
    source_type: SourceType,
    source_url: Option<String>,
    section_title: Option<String>,
}

async fn search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    if req.query.trim().is_empty() {
        return Err(ApiError::BadRequest("query cannot be empty".to_string()));
    }

    // Embed the query
    let embeddings = state
        .ml_client
        .embed_batch(vec![req.query.clone()], None)
        .await
        .map_err(|e| ApiError::ServiceUnavailable(format!("ML service: {e}")))?;

    let query_vector = embeddings
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::Internal("no embedding returned".to_string()))?;

    // Hybrid search in Weaviate
    let user_id = UserId(req.user_id);
    let results = state
        .weaviate
        .hybrid_search(
            &req.query,
            &query_vector,
            user_id,
            req.source_filter.as_deref(),
            req.top_k,
            req.alpha,
        )
        .await
        .map_err(|e| ApiError::Internal(format!("search failed: {e}")))?;

    let total = results.len();
    let items: Vec<SearchResultItem> = results
        .into_iter()
        .map(|r| SearchResultItem {
            chunk_id: r.chunk_id.0,
            document_id: r.document_id.0,
            text: r.text,
            score: r.score,
            document_title: r.document_title,
            source_type: r.source_type,
            source_url: r.source_url,
            section_title: r.section_title,
        })
        .collect();

    Ok(Json(SearchResponse {
        results: items,
        query: req.query,
        total,
    }))
}
