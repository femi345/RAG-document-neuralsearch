pub mod recursive;
pub mod strategies;

use serde::{Deserialize, Serialize};

/// A text chunk produced by the chunking engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub text: String,
    pub chunk_index: usize,
    pub section_title: Option<String>,
    pub start_char: usize,
    pub end_char: usize,
    pub token_count_estimate: usize,
}

/// Trait for chunking strategies.
pub trait ChunkingStrategy: Send + Sync {
    fn chunk(&self, text: &str, section_title: Option<&str>) -> Vec<TextChunk>;
}

/// Estimate token count using the ~4 chars per token heuristic.
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}
