use crate::{estimate_tokens, ChunkingStrategy, TextChunk};

/// Recursive character text splitter — splits on decreasing separator granularity.
pub struct RecursiveChunker {
    target_tokens: usize,
    overlap_tokens: usize,
    separators: Vec<&'static str>,
}

impl RecursiveChunker {
    pub fn new(target_tokens: usize, overlap_tokens: usize) -> Self {
        Self {
            target_tokens,
            overlap_tokens,
            separators: vec!["\n\n", "\n", ". ", " "],
        }
    }

    pub fn default_config() -> Self {
        Self::new(400, 50)
    }

    fn split_text(&self, text: &str) -> Vec<String> {
        self.split_recursive(text, 0)
    }

    fn split_recursive(&self, text: &str, depth: usize) -> Vec<String> {
        if estimate_tokens(text) <= self.target_tokens {
            return vec![text.to_string()];
        }

        let separator = self.separators.get(depth).copied().unwrap_or(" ");

        let splits: Vec<&str> = text.split(separator).collect();
        let mut chunks: Vec<String> = Vec::new();
        let mut current = String::new();

        for split in splits {
            let candidate = if current.is_empty() {
                split.to_string()
            } else {
                format!("{}{}{}", current, separator, split)
            };

            if estimate_tokens(&candidate) > self.target_tokens && !current.is_empty() {
                chunks.push(current.clone());
                // Add overlap from the end of the previous chunk
                let overlap = self.get_overlap(&current);
                current = format!("{}{}{}", overlap, separator, split);
            } else {
                current = candidate;
            }
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        // Recursively split any chunks that are still too large
        let mut result = Vec::new();
        for chunk in chunks {
            if estimate_tokens(&chunk) > self.target_tokens && depth + 1 < self.separators.len() {
                result.extend(self.split_recursive(&chunk, depth + 1));
            } else {
                result.push(chunk);
            }
        }

        result
    }

    fn get_overlap(&self, text: &str) -> String {
        let target_chars = self.overlap_tokens * 4;
        if text.len() <= target_chars {
            return text.to_string();
        }
        text[text.len() - target_chars..].to_string()
    }
}

impl ChunkingStrategy for RecursiveChunker {
    fn chunk(&self, text: &str, section_title: Option<&str>) -> Vec<TextChunk> {
        let raw_chunks = self.split_text(text);
        let mut offset = 0;

        raw_chunks
            .into_iter()
            .enumerate()
            .filter(|(_, chunk)| !chunk.trim().is_empty())
            .map(|(i, chunk)| {
                let start = text[offset..].find(chunk.trim()).unwrap_or(0) + offset;
                let end = start + chunk.len();
                offset = start;

                TextChunk {
                    token_count_estimate: estimate_tokens(&chunk),
                    text: chunk,
                    chunk_index: i,
                    section_title: section_title.map(String::from),
                    start_char: start,
                    end_char: end,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_text_not_split() {
        let chunker = RecursiveChunker::default_config();
        let chunks = chunker.chunk("Hello world.", None);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "Hello world.");
    }

    #[test]
    fn test_long_text_split_on_paragraphs() {
        let chunker = RecursiveChunker::new(20, 5); // Very small target for testing
        let text = "First paragraph with some content here.\n\nSecond paragraph with different content here.\n\nThird paragraph with more content.";
        let chunks = chunker.chunk(text, Some("Test Section"));
        assert!(chunks.len() > 1);
        assert_eq!(chunks[0].section_title, Some("Test Section".to_string()));
    }
}
