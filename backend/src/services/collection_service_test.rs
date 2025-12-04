use sqlx::SqlitePool;

use crate::models::{CollectionQuery, CreateCollection, UpdateCollection};
use crate::services::collection_service::CollectionService;

async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

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
            sort_order INTEGER DEFAULT 0,
            is_default INTEGER NOT NULL DEFAULT 0,
            is_public INTEGER NOT NULL DEFAULT 0,
            parent_id INTEGER,
            bookmark_count INTEGER DEFAULT 0,
            created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
            updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_id) REFERENCES collections(id) ON DELETE SET NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn create_test_user(pool: &SqlitePool) -> i64 {
    sqlx::query(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ('testuser', 'test@example.com', 'hashed_password')
        "#,
    )
    .execute(pool)
    .await
    .unwrap();

    let user: (i64,) = sqlx::query_as("SELECT id FROM users WHERE username = 'testuser'")
        .fetch_one(pool)
        .await
        .unwrap();

    user.0
}

#[tokio::test]
async fn test_create_collection_success() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let collection_data = CreateCollection {
        name: "Test Collection".to_string(),
        description: Some("Test description".to_string()),
        color: Some("#3b82f6".to_string()),
        icon: Some("folder".to_string()),
        parent_id: None,
    };

    let result = CollectionService::create_collection(user_id, collection_data, &pool).await;
    assert!(result.is_ok());

    let collection = result.unwrap();
    assert_eq!(collection.name, "Test Collection");
    assert_eq!(collection.user_id, user_id);
    assert_eq!(collection.description.unwrap(), "Test description");
    assert_eq!(collection.color, "#3b82f6");
    assert_eq!(collection.icon, "folder");
    assert!(!collection.is_default);
    assert!(!collection.is_public);
    assert_eq!(collection.bookmark_count, 0);
}

#[tokio::test]
async fn test_create_collection_with_defaults() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let collection_data = CreateCollection {
        name: "Default Collection".to_string(),
        description: None,
        color: None,
        icon: None,
        parent_id: None,
    };

    let result = CollectionService::create_collection(user_id, collection_data, &pool).await;
    assert!(result.is_ok());

    let collection = result.unwrap();
    assert_eq!(collection.name, "Default Collection");
    assert_eq!(collection.color, "#3b82f6");
    assert_eq!(collection.icon, "folder");
}

#[tokio::test]
async fn test_get_collections_empty() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let query = CollectionQuery {
        parent_id: None,
        limit: None,
        offset: None,
        is_public: None,
    };

    let result = CollectionService::get_collections(user_id, query, &pool).await;
    assert!(result.is_ok());

    let collections = result.unwrap();
    assert!(collections.is_empty());
}

#[tokio::test]
async fn test_get_collections_with_data() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let collection_data = CreateCollection {
        name: "Collection 1".to_string(),
        description: None,
        color: None,
        icon: None,
        parent_id: None,
    };

    CollectionService::create_collection(user_id, collection_data.clone(), &pool)
        .await
        .unwrap();

    let collection_data2 = CreateCollection {
        name: "Collection 2".to_string(),
        description: None,
        color: None,
        icon: None,
        parent_id: None,
    };

    CollectionService::create_collection(user_id, collection_data2, &pool)
        .await
        .unwrap();

    let query = CollectionQuery {
        parent_id: None,
        limit: Some(10),
        offset: None,
        is_public: None,
    };

    let result = CollectionService::get_collections(user_id, query, &pool).await;
    assert!(result.is_ok());

    let collections = result.unwrap();
    assert_eq!(collections.len(), 2);
    assert_eq!(collections[0].name, "Collection 1");
    assert_eq!(collections[1].name, "Collection 2");
}

#[tokio::test]
async fn test_get_collection_by_id_success() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let collection_data = CreateCollection {
        name: "Test Collection".to_string(),
        description: None,
        color: None,
        icon: None,
        parent_id: None,
    };

    let collection = CollectionService::create_collection(user_id, collection_data, &pool)
        .await
        .unwrap();

    let result = CollectionService::get_collection_by_id(user_id, collection.id, &pool).await;
    assert!(result.is_ok());

    let found_collection = result.unwrap().unwrap();
    assert_eq!(found_collection.id, collection.id);
    assert_eq!(found_collection.name, "Test Collection");
}

#[tokio::test]
async fn test_get_collection_by_id_not_found() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let result = CollectionService::get_collection_by_id(user_id, 999, &pool).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none())
}

#[tokio::test]
async fn test_update_collection_success() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let collection_data = CreateCollection {
        name: "Old Name".to_string(),
        description: Some("Old description".to_string()),
        color: Some("#000000".to_string()),
        icon: Some("old-icon".to_string()),
        parent_id: None,
    };

    let collection = CollectionService::create_collection(user_id, collection_data, &pool)
        .await
        .unwrap();

    let update_data = UpdateCollection {
        name: Some("New Name".to_string()),
        description: Some("New description".to_string()),
        color: Some("#ffffff".to_string()),
        icon: Some("new-icon".to_string()),
        parent_id: None,
        clear_parent_id: None,
        sort_order: None,
    };

    let result =
        CollectionService::update_collection(user_id, collection.id, update_data, &pool).await;
    assert!(result.is_ok());

    let updated_collection = result.unwrap().unwrap();
    assert_eq!(updated_collection.name, "New Name");
    assert_eq!(updated_collection.description.unwrap(), "New description");
    assert_eq!(updated_collection.color, "#ffffff");
    assert_eq!(updated_collection.icon, "new-icon");
}

#[tokio::test]
async fn test_update_collection_partial() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let collection_data = CreateCollection {
        name: "Original Name".to_string(),
        description: Some("Original description".to_string()),
        color: Some("#000000".to_string()),
        icon: Some("original-icon".to_string()),
        parent_id: None,
    };

    let collection = CollectionService::create_collection(user_id, collection_data, &pool)
        .await
        .unwrap();

    let update_data = UpdateCollection {
        name: Some("Updated Name".to_string()),
        description: None,
        color: None,
        icon: None,
        parent_id: None,
        clear_parent_id: None,
        sort_order: None,
    };

    let result =
        CollectionService::update_collection(user_id, collection.id, update_data, &pool).await;
    assert!(result.is_ok());

    let updated_collection = result.unwrap().unwrap();
    assert_eq!(updated_collection.name, "Updated Name");
    assert_eq!(
        updated_collection.description.unwrap(),
        "Original description"
    );
    assert_eq!(updated_collection.color, "#000000");
    assert_eq!(updated_collection.icon, "original-icon");
}

#[tokio::test]
async fn test_update_collection_not_found() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let update_data = UpdateCollection {
        name: Some("New Name".to_string()),
        description: None,
        color: None,
        icon: None,
        parent_id: None,
        clear_parent_id: None,
        sort_order: None,
    };

    let result = CollectionService::update_collection(user_id, 999, update_data, &pool).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_delete_collection_success() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let collection_data = CreateCollection {
        name: "To Delete".to_string(),
        description: None,
        color: None,
        icon: None,
        parent_id: None,
    };

    let collection = CollectionService::create_collection(user_id, collection_data, &pool)
        .await
        .unwrap();

    let result = CollectionService::delete_collection(user_id, collection.id, &pool).await;
    assert!(result.is_ok());

    let deleted = result.unwrap();
    assert!(deleted);

    let check_result = CollectionService::get_collection_by_id(user_id, collection.id, &pool).await;
    assert!(check_result.is_ok());
    assert!(check_result.unwrap().is_none());
}

#[tokio::test]
async fn test_delete_collection_not_found() {
    let pool = create_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let result = CollectionService::delete_collection(user_id, 999, &pool).await;
    assert!(result.is_ok());

    let deleted = result.unwrap();
    assert!(!deleted);
}
