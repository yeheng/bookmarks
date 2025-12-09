//! 资源模型单元测试
//! 测试 ResourceType 和相关数据结构的基本行为

use super::*;

#[cfg(test)]
mod resource_type_tests {
    use super::*;

    #[test]
    fn test_resource_type_from_valid_strings() {
        // 测试有效的资源类型字符串解析
        assert_eq!(ResourceType::from("link").unwrap(), ResourceType::Link);
        assert_eq!(ResourceType::from("note").unwrap(), ResourceType::Note);
        assert_eq!(ResourceType::from("snippet").unwrap(), ResourceType::Snippet);
        assert_eq!(ResourceType::from("file").unwrap(), ResourceType::File);
    }

    #[test]
    fn test_resource_type_from_case_insensitive() {
        // 测试大小写不敏感
        assert_eq!(ResourceType::from("LINK").unwrap(), ResourceType::Link);
        assert_eq!(ResourceType::from("Link").unwrap(), ResourceType::Link);
        assert_eq!(ResourceType::from("NOTE").unwrap(), ResourceType::Note);
        assert_eq!(ResourceType::from("Snippet").unwrap(), ResourceType::Snippet);
    }

    #[test]
    fn test_resource_type_from_invalid_string() {
        // 测试无效的资源类型字符串
        let result = ResourceType::from("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown resource type"));
    }

    #[test]
    fn test_resource_type_as_str() {
        // 测试转换为字符串
        assert_eq!(ResourceType::Link.as_str(), "link");
        assert_eq!(ResourceType::Note.as_str(), "note");
        assert_eq!(ResourceType::Snippet.as_str(), "snippet");
        assert_eq!(ResourceType::File.as_str(), "file");
    }

    #[test]
    fn test_resource_type_serialization() {
        // 测试 JSON 序列化
        let link_type = ResourceType::Link;
        let json = serde_json::to_string(&link_type).unwrap();
        assert_eq!(json, "\"link\"");

        let note_type = ResourceType::Note;
        let json = serde_json::to_string(&note_type).unwrap();
        assert_eq!(json, "\"note\"");
    }

    #[test]
    fn test_resource_type_deserialization() {
        // 测试 JSON 反序列化
        let json = "\"link\"";
        let resource_type: ResourceType = serde_json::from_str(json).unwrap();
        assert_eq!(resource_type, ResourceType::Link);

        let json = "\"note\"";
        let resource_type: ResourceType = serde_json::from_str(json).unwrap();
        assert_eq!(resource_type, ResourceType::Note);
    }

    #[test]
    fn test_resource_type_roundtrip() {
        // 测试往返转换
        let original = ResourceType::Snippet;
        let as_str = original.as_str();
        let parsed = ResourceType::from(as_str).unwrap();
        assert_eq!(original, parsed);
    }
}

#[cfg(test)]
mod resource_model_tests {
    use super::*;

    fn create_test_resource() -> Resource {
        Resource {
            id: 1,
            user_id: 100,
            collection_id: Some(10),
            title: "Test Resource".to_string(),
            url: Some("https://example.com".to_string()),
            description: Some("Test description".to_string()),
            favicon_url: None,
            screenshot_url: None,
            thumbnail_url: None,
            is_favorite: false,
            is_archived: false,
            is_private: false,
            is_read: false,
            visit_count: 0,
            last_visited: None,
            metadata: serde_json::json!({}),
            resource_type: "link".to_string(),
            content: None,
            source: None,
            mime_type: None,
            created_at: 1000000,
            updated_at: 1000000,
        }
    }

    #[test]
    fn test_resource_get_type_valid() {
        // 测试获取有效的资源类型
        let resource = create_test_resource();
        let resource_type = resource.get_type().unwrap();
        assert_eq!(resource_type, ResourceType::Link);
    }

    #[test]
    fn test_resource_get_type_invalid() {
        // 测试获取无效的资源类型
        let mut resource = create_test_resource();
        resource.resource_type = "invalid_type".to_string();
        let result = resource.get_type();
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_serialization() {
        // 测试资源序列化
        let resource = create_test_resource();
        let json = serde_json::to_value(&resource).unwrap();

        assert_eq!(json["id"], 1);
        assert_eq!(json["user_id"], 100);
        assert_eq!(json["title"], "Test Resource");
        assert_eq!(json["resource_type"], "link"); // 使用 resource_type 而不是 type
    }

    #[test]
    fn test_resource_with_all_types() {
        // 测试不同类型的资源创建
        let link = Resource {
            resource_type: "link".to_string(),
            url: Some("https://example.com".to_string()),
            content: None,
            ..create_test_resource()
        };
        assert_eq!(link.get_type().unwrap(), ResourceType::Link);

        let note = Resource {
            resource_type: "note".to_string(),
            url: None,
            content: Some("Note content".to_string()),
            ..create_test_resource()
        };
        assert_eq!(note.get_type().unwrap(), ResourceType::Note);

        let snippet = Resource {
            resource_type: "snippet".to_string(),
            url: None,
            content: Some("Code snippet".to_string()),
            ..create_test_resource()
        };
        assert_eq!(snippet.get_type().unwrap(), ResourceType::Snippet);

        let file = Resource {
            resource_type: "file".to_string(),
            url: None,
            source: Some("/path/to/file".to_string()),
            mime_type: Some("application/pdf".to_string()),
            ..create_test_resource()
        };
        assert_eq!(file.get_type().unwrap(), ResourceType::File);
    }
}

#[cfg(test)]
mod create_resource_tests {
    use super::*;

    #[test]
    fn test_create_resource_deserialization_link() {
        // 测试 Link 类型的反序列化
        let json = r#"{
            "title": "Example Link",
            "url": "https://example.com",
            "type": "link",
            "description": "An example link"
        }"#;

        let create_resource: CreateResource = serde_json::from_str(json).unwrap();
        assert_eq!(create_resource.title, "Example Link");
        assert_eq!(create_resource.url.unwrap(), "https://example.com");
        assert_eq!(create_resource.resource_type, "link");
    }

    #[test]
    fn test_create_resource_deserialization_note() {
        // 测试 Note 类型的反序列化
        let json = r#"{
            "title": "My Note",
            "type": "note",
            "content": "This is a note content"
        }"#;

        let create_resource: CreateResource = serde_json::from_str(json).unwrap();
        assert_eq!(create_resource.title, "My Note");
        assert_eq!(create_resource.resource_type, "note");
        assert_eq!(create_resource.content.unwrap(), "This is a note content");
        assert!(create_resource.url.is_none());
    }

    #[test]
    fn test_create_resource_with_tags() {
        // 测试带标签的资源创建
        let json = r#"{
            "title": "Tagged Resource",
            "url": "https://example.com",
            "type": "link",
            "tags": ["rust", "programming", "web"]
        }"#;

        let create_resource: CreateResource = serde_json::from_str(json).unwrap();
        let tags = create_resource.tags.unwrap();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"rust".to_string()));
    }
}

#[cfg(test)]
mod batch_operation_tests {
    use super::*;

    #[test]
    fn test_batch_action_deserialization() {
        // 测试批量操作动作的反序列化
        let json = r#""delete""#;
        let action: ResourceBatchAction = serde_json::from_str(json).unwrap();
        assert!(matches!(action, ResourceBatchAction::Delete));

        let json = r#""move""#;
        let action: ResourceBatchAction = serde_json::from_str(json).unwrap();
        assert!(matches!(action, ResourceBatchAction::Move));
    }

    #[test]
    fn test_batch_request_deserialization() {
        // 测试批量请求的反序列化
        let json = r#"{
            "action": "delete",
            "resource_ids": [1, 2, 3]
        }"#;

        let request: ResourceBatchRequest = serde_json::from_str(json).unwrap();
        assert!(matches!(request.action, ResourceBatchAction::Delete));
        assert_eq!(request.resource_ids.len(), 3);
    }

    #[test]
    fn test_batch_result_serialization() {
        // 测试批量操作结果的序列化
        let result = ResourceBatchResult {
            processed: 5,
            failed: 2,
            errors: vec![
                ResourceBatchError {
                    resource_id: 10,
                    reason: "Not found".to_string(),
                },
            ],
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["processed"], 5);
        assert_eq!(json["failed"], 2);
        assert_eq!(json["errors"].as_array().unwrap().len(), 1);
    }
}

#[cfg(test)]
mod resource_query_tests {
    use super::*;

    #[test]
    fn test_resource_query_default() {
        // 测试查询参数的默认值
        let query = ResourceQuery::default();
        assert!(query.collection_id.is_none());
        assert!(query.tags.is_none());
        assert!(query.search.is_none());
    }

    #[test]
    fn test_resource_query_deserialization() {
        // 测试查询参数的反序列化
        let json = r#"{
            "collection_id": 10,
            "is_favorite": true,
            "limit": 20,
            "offset": 0,
            "type": "link"
        }"#;

        let query: ResourceQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.collection_id.unwrap(), 10);
        assert_eq!(query.is_favorite.unwrap(), true);
        assert_eq!(query.limit.unwrap(), 20);
        assert_eq!(query.resource_type.unwrap(), "link");
    }

    #[test]
    fn test_resource_query_with_sorting() {
        // 测试排序参数
        let json = r#"{
            "sort_by": "created_at",
            "sort_order": "desc"
        }"#;

        let query: ResourceQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.sort_by.unwrap(), "created_at");
        assert_eq!(query.sort_order.unwrap(), "desc");
    }
}

#[cfg(test)]
mod resource_reference_tests {
    use super::*;

    #[test]
    fn test_create_resource_reference_deserialization() {
        // 测试创建引用的反序列化
        let json = r#"{
            "target_id": 42,
            "type": "related"
        }"#;

        let create_ref: CreateResourceReference = serde_json::from_str(json).unwrap();
        assert_eq!(create_ref.target_id, 42);
        assert_eq!(create_ref.reference_type.unwrap(), "related");
    }

    #[test]
    fn test_resource_reference_query_deserialization() {
        // 测试引用查询的反序列化
        let json = r#"{
            "limit": 10,
            "offset": 0,
            "type": "depends_on",
            "direction": "source"
        }"#;

        let query: ResourceReferenceQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit.unwrap(), 10);
        assert_eq!(query.reference_type.unwrap(), "depends_on");
        assert_eq!(query.direction.unwrap(), "source");
    }

    #[test]
    fn test_resource_reference_list_serialization() {
        // 测试引用列表的序列化
        let list = ResourceReferenceList {
            items: vec![],
            limit: 10,
            offset: 0,
            has_more: false,
        };

        let json = serde_json::to_value(&list).unwrap();
        assert_eq!(json["limit"], 10);
        assert_eq!(json["offset"], 0);
        assert_eq!(json["has_more"], false);
    }
}
