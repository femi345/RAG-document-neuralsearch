pub mod plaintext;

/// A parsed document ready for chunking.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: String,
    pub sections: Vec<Section>,
    pub full_text: String,
}

/// A section within a parsed document.
#[derive(Debug, Clone)]
pub struct Section {
    pub title: Option<String>,
    pub content: String,
    pub start_offset: usize,
}

/// Parse raw text content into a structured document.
/// Detects headings (markdown-style # or ALL-CAPS lines) and splits into sections.
pub fn parse_text(title: &str, content: &str) -> ParsedDocument {
    let mut sections = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_content = String::new();
    let mut current_start = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect markdown headings
        if trimmed.starts_with('#') {
            // Save previous section
            if !current_content.trim().is_empty() {
                sections.push(Section {
                    title: current_title.take(),
                    content: current_content.trim().to_string(),
                    start_offset: current_start,
                });
            }
            current_title = Some(trimmed.trim_start_matches('#').trim().to_string());
            current_content = String::new();
            current_start = content.find(line).unwrap_or(0);
        } else {
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
        }
    }

    // Don't forget the last section
    if !current_content.trim().is_empty() {
        sections.push(Section {
            title: current_title,
            content: current_content.trim().to_string(),
            start_offset: current_start,
        });
    }

    // If no sections were found, treat the whole content as one section
    if sections.is_empty() {
        sections.push(Section {
            title: None,
            content: content.to_string(),
            start_offset: 0,
        });
    }

    ParsedDocument {
        title: title.to_string(),
        sections,
        full_text: content.to_string(),
    }
}
