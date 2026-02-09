/// Extract text from plain text content (passthrough).
pub fn extract(content: &str) -> String {
    content.to_string()
}

/// Basic HTML-to-text extraction (strip tags).
pub fn extract_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut prev_was_space = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            _ if !in_tag => {
                if ch.is_whitespace() {
                    if !prev_was_space {
                        result.push(' ');
                        prev_was_space = true;
                    }
                } else {
                    result.push(ch);
                    prev_was_space = false;
                }
            }
            _ => {}
        }
    }

    result.trim().to_string()
}
