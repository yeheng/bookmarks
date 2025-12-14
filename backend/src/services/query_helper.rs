use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::models::{ResourceWithTags, SearchType};
use crate::utils::error::AppResult;

pub struct QueryOptions<'a> {
    pub user_id: i64,
    pub collection_id: Option<i64>,
    pub resource_type: Option<&'a str>,
    pub tags: &'a [String],
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_private: Option<bool>,
    pub is_read: Option<bool>,
    pub search_term: Option<&'a str>,
    pub search_type: Option<&'a SearchType>, // Default to All if None
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub limit: i64,
    pub offset: i64,
    pub sort_by: &'a str,
    pub sort_order: &'a str,
}

impl Default for QueryOptions<'_> {
    fn default() -> Self {
        Self {
            user_id: 0,
            collection_id: None,
            resource_type: None,
            tags: &[],
            is_favorite: None,
            is_archived: None,
            is_private: None,
            is_read: None,
            search_term: None,
            search_type: None,
            date_from: None,
            date_to: None,
            limit: 50,
            offset: 0,
            sort_by: "created_at",
            sort_order: "desc",
        }
    }
}

pub async fn fetch_resources(
    pool: &SqlitePool,
    options: &QueryOptions<'_>,
) -> AppResult<Vec<ResourceWithTags>> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r#"
        SELECT
            r.id, r.user_id, r.collection_id, r.title, r.url, r.description,
            r.favicon_url, r.screenshot_url, r.thumbnail_url,
            r.is_favorite, r.is_archived, r.is_private, r.is_read,
            r.visit_count, r.last_visited,
            r.metadata, r.type, r.content, r.source, r.mime_type,
            r.created_at, r.updated_at,
            COALESCE(
                CASE
                    WHEN COUNT(t.name) > 0
                    THEN '[' || GROUP_CONCAT('"' || REPLACE(t.name, '"', '""') || '"', ',') || ']'
                    ELSE '[]'
                END,
                '[]'
            ) as tags,
            c.name as collection_name,
            c.color as collection_color,
            COALESCE(
                (SELECT COUNT(*) FROM resource_references rr
                 WHERE rr.source_id = r.id OR rr.target_id = r.id),
                0
            ) as reference_count
        FROM resources r
        LEFT JOIN collections c ON r.collection_id = c.id
        LEFT JOIN resource_tags rt ON r.id = rt.resource_id
        LEFT JOIN tags t ON rt.tag_id = t.id
        "#,
    );

    // If search term is present, join FTS table
    if options.search_term.is_some() {
        query_builder.push(" JOIN resources_fts fts ON r.id = fts.rowid ");
    }

    query_builder.push(" WHERE r.user_id = ");
    query_builder.push_bind(options.user_id);

    // Dynamic filters
    if let Some(collection_id) = options.collection_id {
        query_builder.push(" AND r.collection_id = ");
        query_builder.push_bind(collection_id);
    }

    if let Some(resource_type) = options.resource_type {
        query_builder.push(" AND r.type = ");
        query_builder.push_bind(resource_type);
    }

    if let Some(is_favorite) = options.is_favorite {
        query_builder.push(" AND r.is_favorite = ");
        query_builder.push_bind(is_favorite);
    }

    if let Some(is_archived) = options.is_archived {
        query_builder.push(" AND r.is_archived = ");
        query_builder.push_bind(is_archived);
    }

    if let Some(is_private) = options.is_private {
        query_builder.push(" AND r.is_private = ");
        query_builder.push_bind(is_private);
    }

    if let Some(is_read) = options.is_read {
        query_builder.push(" AND r.is_read = ");
        query_builder.push_bind(is_read);
    }

    if let Some(date_from) = options.date_from {
        query_builder.push(" AND r.created_at >= ");
        query_builder.push_bind(date_from);
    }

    if let Some(date_to) = options.date_to {
        query_builder.push(" AND r.created_at <= ");
        query_builder.push_bind(date_to);
    }

    // FTS Search
    if let Some(search_term) = options.search_term {
        // Use FTS Match
        let fts_match = match options.search_type.unwrap_or(&SearchType::All) {
            SearchType::Title => " AND resources_fts MATCH 'title:' || ",
            SearchType::Content => " AND resources_fts MATCH 'content:' || ",
            SearchType::Url => " AND resources_fts MATCH 'url:' || ",
            SearchType::All => " AND resources_fts MATCH ",
        };
        query_builder.push(fts_match);
        query_builder.push_bind(search_term);
    }

    // Tags Filtering
    if !options.tags.is_empty() {
        query_builder.push(
            " AND r.id IN (
                SELECT resource_id
                FROM resource_tags
                JOIN tags ON resource_tags.tag_id = tags.id
                WHERE tags.name IN (",
        );

        let mut separated = query_builder.separated(", ");
        for tag in options.tags {
            separated.push_bind(tag);
        }

        query_builder.push(") GROUP BY resource_id HAVING COUNT(DISTINCT tags.id) = ");
        query_builder.push_bind(options.tags.len() as i64);
        query_builder.push(")");
    }

    // Grouping
    query_builder.push(" GROUP BY r.id, c.name, c.color");

    // Sorting
    let sort_field = match options.sort_by {
        "title" => "r.title",
        "updated_at" => "r.updated_at",
        "visit_count" => "r.visit_count",
        "last_visited" => "r.last_visited",
        _ => "r.created_at",
    };

    let sort_direction = match options.sort_order.to_lowercase().as_str() {
        "asc" => "ASC",
        _ => "DESC",
    };

    // Support rank sort if searching
    if options.search_term.is_some() && options.sort_by == "rank" {
        query_builder.push(" ORDER BY rank");
    } else {
        query_builder.push(format!(" ORDER BY {} {}", sort_field, sort_direction));
    }

    // Pagination
    query_builder.push(" LIMIT ");
    query_builder.push_bind(options.limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(options.offset);

    let resources = query_builder
        .build_query_as::<ResourceWithTags>()
        .fetch_all(pool)
        .await?;
    Ok(resources)
}

pub async fn count_resources(pool: &SqlitePool, options: &QueryOptions<'_>) -> AppResult<i64> {
    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT COUNT(*) FROM resources r");

    if options.search_term.is_some() {
        query_builder.push(" JOIN resources_fts fts ON r.id = fts.rowid ");
    }

    query_builder.push(" WHERE r.user_id = ");
    query_builder.push_bind(options.user_id);

    // Dynamic filters
    if let Some(collection_id) = options.collection_id {
        query_builder.push(" AND r.collection_id = ");
        query_builder.push_bind(collection_id);
    }
    if let Some(resource_type) = options.resource_type {
        query_builder.push(" AND r.type = ");
        query_builder.push_bind(resource_type);
    }
    if let Some(is_favorite) = options.is_favorite {
        query_builder.push(" AND r.is_favorite = ");
        query_builder.push_bind(is_favorite);
    }
    if let Some(is_archived) = options.is_archived {
        query_builder.push(" AND r.is_archived = ");
        query_builder.push_bind(is_archived);
    }
    if let Some(is_private) = options.is_private {
        query_builder.push(" AND r.is_private = ");
        query_builder.push_bind(is_private);
    }
    if let Some(is_read) = options.is_read {
        query_builder.push(" AND r.is_read = ");
        query_builder.push_bind(is_read);
    }
    if let Some(date_from) = options.date_from {
        query_builder.push(" AND r.created_at >= ");
        query_builder.push_bind(date_from);
    }
    if let Some(date_to) = options.date_to {
        query_builder.push(" AND r.created_at <= ");
        query_builder.push_bind(date_to);
    }

    if let Some(search_term) = options.search_term {
        let fts_match = match options.search_type.unwrap_or(&SearchType::All) {
            SearchType::Title => " AND resources_fts MATCH 'title:' || ",
            SearchType::Content => " AND resources_fts MATCH 'content:' || ",
            SearchType::Url => " AND resources_fts MATCH 'url:' || ",
            SearchType::All => " AND resources_fts MATCH ",
        };
        query_builder.push(fts_match);
        query_builder.push_bind(search_term);
    }

    if !options.tags.is_empty() {
        query_builder.push(
            " AND r.id IN (
                SELECT resource_id
                FROM resource_tags
                JOIN tags ON resource_tags.tag_id = tags.id
                WHERE tags.name IN (",
        );
        let mut separated = query_builder.separated(", ");
        for tag in options.tags {
            separated.push_bind(tag);
        }
        query_builder.push(") GROUP BY resource_id HAVING COUNT(DISTINCT tags.id) = ");
        query_builder.push_bind(options.tags.len() as i64);
        query_builder.push(")");
    }

    let count = query_builder
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await?;
    Ok(count)
}
