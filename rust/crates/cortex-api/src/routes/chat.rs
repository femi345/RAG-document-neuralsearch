use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::routing::post;
use axum::{Json, Router};
use cortex_common::types::*;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/chat", post(chat))
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    query: String,
    #[serde(default = "default_provider")]
    provider: String,
    #[serde(default = "default_model")]
    model: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
    source_filter: Option<String>,
    /// Temporary: pass user_id until auth is implemented.
    user_id: Uuid,
}

fn default_provider() -> String {
    "claude".to_string()
}

fn default_model() -> String {
    "claude-sonnet-4-5-20250929".to_string()
}

fn default_top_k() -> usize {
    8
}

async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    if req.query.trim().is_empty() {
        return Err(ApiError::BadRequest("query cannot be empty".to_string()));
    }

    let user_id = UserId(req.user_id);

    // 1. Embed the query
    let embeddings = state
        .ml_client
        .embed_batch(vec![req.query.clone()], None)
        .await
        .map_err(|e| ApiError::ServiceUnavailable(format!("ML service: {e}")))?;

    let query_vector = embeddings
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::Internal("no embedding returned".to_string()))?;

    // 2. Search for relevant chunks
    let results = state
        .weaviate
        .hybrid_search(
            &req.query,
            &query_vector,
            user_id,
            req.source_filter.as_deref(),
            req.top_k * 3, // Over-fetch for reranking
            0.7,
        )
        .await
        .map_err(|e| ApiError::Internal(format!("search failed: {e}")))?;

    // 3. Rerank
    let docs_for_rerank: Vec<(String, String, f32)> = results
        .iter()
        .map(|r| (r.chunk_id.0.to_string(), r.text.clone(), r.score))
        .collect();

    let reranked = state
        .ml_client
        .rerank(&req.query, docs_for_rerank, req.top_k as i32)
        .await
        .map_err(|e| ApiError::ServiceUnavailable(format!("ML service rerank: {e}")))?;

    // 4. Build context from top-k reranked chunks
    let context = reranked
        .iter()
        .enumerate()
        .map(|(i, (id, text, _score))| {
            let source = results
                .iter()
                .find(|r| r.chunk_id.0.to_string() == *id);
            let title = source.map(|s| s.document_title.as_str()).unwrap_or("Unknown");
            format!("[Source {}] ({})\n{}", i + 1, title, text)
        })
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    let system_prompt = format!(
        "You are a helpful AI assistant. Answer the user's question based on the provided context. \
         Always cite your sources using [Source N] format. If the context doesn't contain enough \
         information to fully answer, say so.\n\nContext:\n{context}"
    );

    // 5. Stream LLM response via SSE
    let ml_client = state.ml_client.clone();
    let query = req.query.clone();
    let provider = req.provider.clone();
    let model = req.model.clone();

    // Build citation data
    let citations: Vec<CitationData> = reranked
        .iter()
        .enumerate()
        .map(|(i, (id, text, _))| {
            let source = results.iter().find(|r| r.chunk_id.0.to_string() == *id);
            CitationData {
                index: i + 1,
                chunk_id: id.clone(),
                document_title: source
                    .map(|s| s.document_title.clone())
                    .unwrap_or_default(),
                source_url: source.and_then(|s| s.source_url.clone()),
                snippet: text.chars().take(200).collect(),
            }
        })
        .collect();

    let stream = async_stream::stream! {
        // Send citations first
        let citations_event = serde_json::to_string(&citations).unwrap_or_default();
        yield Ok(Event::default().event("citations").data(citations_event));

        // Stream LLM response
        match ml_client.generate_stream(&query, &system_prompt, &provider, &model).await {
            Ok(mut stream) => {
                use futures::StreamExt;
                while let Some(Ok(chunk)) = stream.next().await {
                    yield Ok(Event::default().event("text").data(chunk.text));
                    if chunk.is_final {
                        break;
                    }
                }
                yield Ok(Event::default().event("done").data(""));
            }
            Err(e) => {
                yield Ok(Event::default().event("error").data(format!("LLM error: {e}")));
            }
        }
    };

    Ok(Sse::new(stream))
}

#[derive(Debug, Serialize)]
struct CitationData {
    index: usize,
    chunk_id: String,
    document_title: String,
    source_url: Option<String>,
    snippet: String,
}
