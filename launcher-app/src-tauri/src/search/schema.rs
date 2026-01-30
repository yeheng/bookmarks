//! Tantivy schema definitions for bookmarks and files.

use tantivy::schema::*;

/// Build the Tantivy schema for bookmark indexing.
///
/// Fields:
/// - id: Stored and fast field for document identification
/// - title, url, description: Full-text searchable fields
/// - tags: Searchable tags field
/// - last_accessed, created_at, updated_at: Fast fields for sorting/filtering
pub fn build_bookmark_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // ID field - stored, fast, and indexed for delete_term support
    schema_builder.add_i64_field("id", STORED | FAST | INDEXED);

    // Text fields for display (stored but not indexed)
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("url", TEXT | STORED);
    schema_builder.add_text_field("description", TEXT | STORED);
    schema_builder.add_text_field("tags", TEXT | STORED);

    // Numeric fields for sorting/filtering
    schema_builder.add_i64_field("last_accessed", FAST);
    schema_builder.add_i64_field("created_at", FAST);
    schema_builder.add_i64_field("updated_at", FAST);

    schema_builder.build()
}

/// Build the Tantivy schema for file indexing.
///
/// Fields:
/// - id: Stored and fast field for document identification
/// - path, name: Full-text searchable with stored values
/// - extension: Exact match field for filtering
/// - size, modified_at: Fast fields for sorting
/// - directory_id: Fast field for bulk operations
pub fn build_file_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // ID field - stored, fast, and indexed for delete_term support
    schema_builder.add_i64_field("id", STORED | FAST | INDEXED);

    // Text fields - searchable and stored
    schema_builder.add_text_field("path", TEXT | STORED);
    schema_builder.add_text_field("name", TEXT | STORED);

    // Extension as exact string match
    schema_builder.add_text_field("extension", STRING | STORED);

    // Numeric fields
    schema_builder.add_i64_field("size", STORED | FAST);
    schema_builder.add_i64_field("modified_at", STORED | FAST);
    schema_builder.add_i64_field("directory_id", FAST | INDEXED);

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
    }
}
