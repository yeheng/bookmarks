//! Tantivy schema definitions for bookmarks and files.
//!
//! Uses ngram tokenizer for CJK text support.

use tantivy::schema::*;
use tantivy::tokenizer::{NgramTokenizer, TextAnalyzer};
use tantivy::Index;

/// Ngram tokenizer name for CJK and fuzzy matching support.
pub const NGRAM_TOKENIZER_NAME: &str = "ngram3";

/// Register custom tokenizers on the index.
///
/// This must be called after index creation to enable ngram tokenization.
pub fn register_tokenizers(index: &Index) -> tantivy::Result<()> {
    let tokenizer_manager = index.tokenizers();

    // Ngram tokenizer with min=2, max=3 for CJK and fuzzy matching
    // This breaks "中文搜索" into ["中文", "文搜", "搜索", "中文搜", "文搜索"]
    let ngram_tokenizer = TextAnalyzer::builder(NgramTokenizer::new(2, 3, false).unwrap())
        .filter(tantivy::tokenizer::LowerCaser)
        .build();

    tokenizer_manager.register(NGRAM_TOKENIZER_NAME, ngram_tokenizer);
    Ok(())
}

/// Build the Tantivy schema for bookmark indexing.
///
/// Fields:
/// - id: Stored and fast field for document identification
/// - title, url, description: Full-text searchable fields with ngram tokenizer
/// - tags: Searchable tags field
/// - last_accessed, created_at, updated_at: Fast fields for sorting/filtering
pub fn build_bookmark_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // ID field - stored, fast, and indexed for delete_term support
    schema_builder.add_i64_field("id", STORED | FAST | INDEXED);

    // Text fields with ngram tokenizer for CJK support
    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(NGRAM_TOKENIZER_NAME)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();

    schema_builder.add_text_field("title", text_options.clone());
    schema_builder.add_text_field("url", text_options.clone());
    schema_builder.add_text_field("description", text_options.clone());
    schema_builder.add_text_field("tags", text_options);

    // Numeric fields for sorting/filtering
    schema_builder.add_i64_field("last_accessed", FAST);
    schema_builder.add_i64_field("created_at", FAST);
    schema_builder.add_i64_field("updated_at", FAST);

    // Frecency fields - stored in index to avoid DB queries during search
    // access_count: number of times the bookmark has been accessed
    // access_timestamp: last access time (unix timestamp)
    schema_builder.add_i64_field("access_count", FAST | STORED);
    schema_builder.add_i64_field("access_timestamp", FAST | STORED);

    schema_builder.build()
}

/// Build the Tantivy schema for file indexing.
///
/// Fields:
/// - id: Stored and fast field for document identification
/// - path, name: Full-text searchable with ngram tokenizer for fuzzy matching
/// - extension: Exact match field for filtering
/// - size, modified_at: Fast fields for sorting
/// - directory_id: Fast field for bulk operations
pub fn build_file_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // ID field - stored, fast, and indexed for delete_term support
    schema_builder.add_i64_field("id", STORED | FAST | INDEXED);

    // Text fields with ngram tokenizer for fuzzy file name matching
    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(NGRAM_TOKENIZER_NAME)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();

    schema_builder.add_text_field("path", text_options.clone());
    schema_builder.add_text_field("name", text_options);

    // Extension as exact string match
    schema_builder.add_text_field("extension", STRING | STORED);

    // Numeric fields
    schema_builder.add_i64_field("size", STORED | FAST);
    schema_builder.add_i64_field("modified_at", STORED | FAST);
    schema_builder.add_i64_field("directory_id", FAST | INDEXED);

    // Frecency fields - stored in index to avoid DB queries during search
    // access_count: number of times the file has been accessed
    // access_timestamp: last access time (unix timestamp)
    schema_builder.add_i64_field("access_count", FAST | STORED);
    schema_builder.add_i64_field("access_timestamp", FAST | STORED);

    schema_builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_schema_has_required_fields() {
        let schema = build_bookmark_schema();
        assert!(schema.get_field("id").is_ok());
        assert!(schema.get_field("title").is_ok());
        assert!(schema.get_field("url").is_ok());
        assert!(schema.get_field("description").is_ok());
        assert!(schema.get_field("tags").is_ok());
        assert!(schema.get_field("last_accessed").is_ok());
        assert!(schema.get_field("created_at").is_ok());
        assert!(schema.get_field("updated_at").is_ok());
        // Frecency fields
        assert!(schema.get_field("access_count").is_ok());
        assert!(schema.get_field("access_timestamp").is_ok());
    }

    #[test]
    fn test_file_schema_has_required_fields() {
        let schema = build_file_schema();
        assert!(schema.get_field("id").is_ok());
        assert!(schema.get_field("path").is_ok());
        assert!(schema.get_field("name").is_ok());
        assert!(schema.get_field("extension").is_ok());
        assert!(schema.get_field("size").is_ok());
        assert!(schema.get_field("modified_at").is_ok());
        assert!(schema.get_field("directory_id").is_ok());
        // Frecency fields
        assert!(schema.get_field("access_count").is_ok());
        assert!(schema.get_field("access_timestamp").is_ok());
    }
}
