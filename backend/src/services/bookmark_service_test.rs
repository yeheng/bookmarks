#[cfg(test)]
mod tests {
    use crate::{models::*, services::BookmarkService};
    use sqlx::SqlitePool;

    async fn create_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        
        // Create tables for testing
        sqlx::query(
            r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                avatar_url TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                email_verified INTEGER NOT NULL DEFAULT 0,
                email_verification_token TEXT,
                password_reset_token TEXT,
                password_reset_expires_at INTEGER,
                last_login_at INTEGER,
                created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER))
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE collections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT,
                icon TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                collection_id INTEGER,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                description TEXT,
                favicon_url TEXT,
                screenshot_url TEXT,
                thumbnail_url TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                is_archived INTEGER NOT NULL DEFAULT 0,
                is_private INTEGER NOT NULL DEFAULT 0,
                is_read INTEGER NOT NULL DEFAULT 0,
                visit_count INTEGER NOT NULL DEFAULT 0,
                last_visited INTEGER,
                reading_time INTEGER,
                difficulty_level INTEGER,
                metadata TEXT DEFAULT '{}',
                created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE bookmark_tags (
                bookmark_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (bookmark_id, tag_id),
                FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // 创建 FTS5 虚拟表用于全文搜索
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
                title,
                description,
                tags,
                url,
                content='',
                tokenize='unicode61'
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert test user
        sqlx::query(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)"
        )
        .bind("testuser")
        .bind("test@example.com")
        .bind("hashed_password")
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_bookmark_success() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let bookmark_data = CreateBookmark {
            collection_id: None,
            title: "Test Bookmark".to_string(),
            url: "https://example.com".to_string(),
            description: Some("Test description".to_string()),
            is_favorite: Some(true),
            is_private: Some(false),
            tags: Some(vec!["test".to_string(), "example".to_string()]),
        };

        let result = BookmarkService::create_bookmark(user_id, bookmark_data, &pool).await;
        if let Err(e) = &result {
            println!("Error creating bookmark: {:?}", e);
            panic!("Failed to create bookmark: {:?}", e);
        }
        assert!(result.is_ok());

        let bookmark = result.unwrap();
        assert_eq!(bookmark.title, "Test Bookmark");
        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.user_id, user_id);
        assert!(bookmark.is_favorite);
        assert!(!bookmark.is_private);
    }

    #[tokio::test]
    async fn test_create_bookmark_invalid_url() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let bookmark_data = CreateBookmark {
            collection_id: None,
            title: "Test Bookmark".to_string(),
            url: "invalid-url".to_string(),
            description: None,
            is_favorite: None,
            is_private: None,
            tags: None,
        };

        let result = BookmarkService::create_bookmark(user_id, bookmark_data, &pool).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_bookmarks_empty() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let query = BookmarkQuery::default();
        let result = BookmarkService::get_bookmarks(user_id, query, &pool).await;
        assert!(result.is_ok());

        let bookmarks = result.unwrap();
        assert!(bookmarks.is_empty());
    }

    #[tokio::test]
    async fn test_get_bookmark_by_id_not_found() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let bookmark_id = 999;
        
        let result = BookmarkService::get_bookmark_by_id(user_id, bookmark_id, &pool).await;
        assert!(result.is_ok());

        let bookmark = result.unwrap();
        assert!(bookmark.is_none());
    }

    #[tokio::test]
    async fn test_update_bookmark_not_found() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let bookmark_id = 999;
        
        let update_data = UpdateBookmark {
            title: Some("Updated Title".to_string()),
            url: None,
            description: None,
            collection_id: None,
            clear_collection_id: None,
            is_favorite: None,
            is_archived: None,
            is_private: None,
            is_read: None,
            reading_time: None,
            difficulty_level: None,
            tags: None,
        };

        let result = BookmarkService::update_bookmark(user_id, bookmark_id, update_data, &pool).await;
        assert!(result.is_ok());

        let bookmark = result.unwrap();
        assert!(bookmark.is_none());
    }

    #[tokio::test]
    async fn test_delete_bookmark_not_found() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let bookmark_id = 999;
        
        let result = BookmarkService::delete_bookmark(user_id, bookmark_id, &pool).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_bookmark_exists_false() {
        let pool = create_test_pool().await;
        let user_id = 1;
        let url = "https://example.com";
        
        let result = BookmarkService::bookmark_exists(user_id, url, &pool).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_batch_process_empty() {
        let pool = create_test_pool().await;
        let user_id = 1;
        
        let request = BookmarkBatchRequest {
            action: BookmarkBatchAction::Delete,
            bookmark_ids: vec![],
            data: None,
        };

        let result = BookmarkService::batch_process(user_id, request, &pool).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.processed, 0);
        assert_eq!(batch_result.failed, 0);
    }

}
