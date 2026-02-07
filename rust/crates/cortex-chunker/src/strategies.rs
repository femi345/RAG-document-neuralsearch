use cortex_common::types::SourceType;

use crate::recursive::RecursiveChunker;
use crate::ChunkingStrategy;

/// Select the appropriate chunking strategy based on content characteristics.
pub fn select_strategy(source_type: SourceType, token_count: usize) -> Box<dyn ChunkingStrategy> {
    match source_type {
        // Long-form content: use larger chunks
        SourceType::Notion | SourceType::PdfUpload => {
            if token_count > 500 {
                // In Phase 2, this will use SemanticChunker for long content
                Box::new(RecursiveChunker::new(400, 50))
            } else {
                Box::new(RecursiveChunker::new(300, 40))
            }
        }
        // Slack messages are already short
        SourceType::Slack => Box::new(RecursiveChunker::new(200, 30)),
        // Gmail: moderate chunk size
        SourceType::Gmail => Box::new(RecursiveChunker::new(350, 50)),
    }
}
