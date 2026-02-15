use std::collections::HashMap;

/// A parsed search query with extracted metadata.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StructuredQuery {
    /// The original raw query string.
    pub raw: String,
    /// Extracted search terms (excluding scopes and filters).
    pub terms: Vec<String>,
    /// Extracted phrases (quoted strings).
    pub phrases: Vec<String>,
    /// Extracted filters (key:value).
    pub filters: HashMap<String, String>,
    /// Optional scope prefix (e.g., "gh" from "gh:").
    pub scope: Option<String>,
}

impl StructuredQuery {
    /// Parse a raw query string into a structured query.
    pub fn parse(raw: &str) -> Self {
        let mut terms = Vec::new();
        let mut phrases = Vec::new();
        let mut filters = HashMap::new();
        let mut scope = None;

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self {
                raw: raw.to_string(),
                ..Default::default()
            };
        }

        // Check for scope prefix at the start
        // Must be "scope: ..." or "scope:"
        let mut processing_text = trimmed;
        if let Some(idx) = processing_text.find(':') {
            let potential_scope = &processing_text[..idx];
            // Scope must be a single word (no spaces)
            if !potential_scope.contains(char::is_whitespace) {
                // Check if it's a scope (followed by space or end of string)
                // "gh: react" -> scope="gh", rest=" react"
                // "gh:" -> scope="gh", rest=""
                // "type:pdf" -> NOT a scope, it's a filter (usually)
                // Design decision: First token with colon is scope IF it is at the very beginning
                // But wait, "type:pdf" also looks like a scope?
                // The design doc says:
                // "gh: react" -> scope="gh"
                // "type:pdf" -> filter="type", value="pdf"
                // Distinguishing rule: Scopes are usually "provider aliases".
                // However, the doc says: "gh: react" (colon immediately after keyword).
                
                // Let's adopt a convention: The FIRST token ending in ':' is a scope IF it has no value attached directly?
                // No, "gh: react" has a space. "type:pdf" has no space.
                // So if the char after ':' is a space (or end of string), it's a scope.
                // If the char after ':' is NOT a space, it's a key:value filter.
                
                let after_colon = idx + 1;
                let is_scope = if after_colon >= processing_text.len() {
                    true // "gh:" -> scope
                } else {
                    processing_text.chars().nth(after_colon).unwrap().is_whitespace()
                };

                if is_scope {
                    scope = Some(potential_scope.to_string());
                    if after_colon < processing_text.len() {
                        processing_text = &processing_text[after_colon..];
                    } else {
                        processing_text = "";
                    }
                }
            }
        }

        // Simple tokenizer handling quotes and spaces
        let mut chars = processing_text.chars().peekable();
        let mut current_token = String::new();
        let mut in_quote = false;

        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    if in_quote {
                        // End of quoted phrase
                        if !current_token.is_empty() {
                            phrases.push(current_token.clone());
                            // Also add to terms for broad matching if desired? 
                            // Design says: "phrase + term". 
                            // Usually phrases are strictly exact match, but let's keep separate.
                            current_token.clear();
                        }
                        in_quote = false;
                    } else {
                        // Start of quoted phrase
                        // If we have a pending token, push it as a term
                        if !current_token.is_empty() {
                            process_token(current_token.clone(), &mut terms, &mut filters);
                            current_token.clear();
                        }
                        in_quote = true;
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {
                    if in_quote {
                        current_token.push(c);
                    } else if !current_token.is_empty() {
                        process_token(current_token.clone(), &mut terms, &mut filters);
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(c);
                }
            }
        }

        // Flush last token
        if !current_token.is_empty() {
            if in_quote {
                // Unclosed quote, treat as phrase or term? Treat as phrase for robustness
                phrases.push(current_token);
            } else {
                process_token(current_token, &mut terms, &mut filters);
            }
        }

        Self {
            raw: raw.to_string(),
            terms,
            phrases,
            filters,
            scope,
        }
    }
}

fn process_token(token: String, terms: &mut Vec<String>, filters: &mut HashMap<String, String>) {
    // Check for key:value
    if let Some(idx) = token.find(':') {
        // "type:pdf" -> key="type", value="pdf"
        // "http://example.com" -> key="http", value="//example.com" -> wait, heuristic needed.
        // Heuristic: filtered keys usually don't have special chars?
        // Let's assume broad acceptance for now as per design spec "key:value".
        // But for urls like "http:..." we might not want to treat as filter.
        // For simplicity: split at FIRST colon.
        let key = &token[..idx];
        let val = &token[idx + 1..];
        
        if !key.is_empty() && !val.is_empty() {
            filters.insert(key.to_string(), val.to_string());
            return;
        }
    }
    
    // Normal term
    terms.push(token);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_terms() {
        let q = StructuredQuery::parse("hello world");
        assert_eq!(q.terms, vec!["hello", "world"]);
        assert!(q.scope.is_none());
        assert!(q.filters.is_empty());
    }

    #[test]
    fn test_parse_scope() {
        let q = StructuredQuery::parse("gh: react");
        assert_eq!(q.scope, Some("gh".to_string()));
        assert_eq!(q.terms, vec!["react"]);
    }

    #[test]
    fn test_parse_scope_no_terms() {
        let q = StructuredQuery::parse("gh:");
        assert_eq!(q.scope, Some("gh".to_string()));
        assert!(q.terms.is_empty());
    }

    #[test]
    fn test_parse_filter() {
        let q = StructuredQuery::parse("rust type:pdf");
        assert_eq!(q.terms, vec!["rust"]);
        assert_eq!(q.filters.get("type"), Some(&"pdf".to_string()));
        assert!(q.scope.is_none());
    }

    #[test]
    fn test_parse_scope_and_filter() {
        let q = StructuredQuery::parse("gh: rust lang:rust");
        assert_eq!(q.scope, Some("gh".to_string()));
        assert_eq!(q.terms, vec!["rust"]);
        assert_eq!(q.filters.get("lang"), Some(&"rust".to_string()));
    }

    #[test]
    fn test_parse_quotes() {
        let q = StructuredQuery::parse("\"hello world\" span");
        assert_eq!(q.phrases, vec!["hello world"]);
        assert_eq!(q.terms, vec!["span"]);
    }
    
    #[test]
    fn test_colon_in_url_is_filter_for_now() {
        // Current simple logic treats http:foo as filter key=http val=foo
        // This is acceptable for v1 as per spec "key:value"
        let q = StructuredQuery::parse("http:foo");
        assert_eq!(q.filters.get("http"), Some(&"foo".to_string()));
    }
}
