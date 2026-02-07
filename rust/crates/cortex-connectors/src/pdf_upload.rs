use chrono::Utc;
use cortex_common::types::SourceType;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::traits::RawDocument;

/// Create a RawDocument from uploaded PDF content.
/// In Phase 1, we accept pre-extracted text. Full PDF parsing comes next.
pub fn create_from_text(filename: &str, text: String) -> RawDocument {
    let content_hash = {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    RawDocument {
        source_id: Uuid::new_v4().to_string(),
        source_type: SourceType::PdfUpload,
        title: filename.to_string(),
        content: text,
        mime_type: "application/pdf".to_string(),
        metadata: serde_json::json!({ "filename": filename }),
        content_hash,
        fetched_at: Utc::now(),
        source_url: None,
    }
}

/// Compute SHA-256 hash of binary content.
pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}
