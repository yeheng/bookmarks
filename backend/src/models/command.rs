use serde::{Deserialize, Serialize};

/// 所有支持的命令动作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // 资源管理命令
    GetResources,
    GetResource,
    CreateResource,
    UpdateResource,
    DeleteResource,
    BatchUpdateResources,

    // 资源引用管理命令
    CreateResourceReference,
    DeleteResourceReference,
    GetResourceReferences,

    // 收藏夹管理命令
    GetCollections,
    GetCollection,
    CreateCollection,
    UpdateCollection,
    DeleteCollection,

    // 标签管理命令
    GetTags,
    GetTag,
    CreateTag,
    UpdateTag,
    DeleteTag,

    // 搜索命令
    SearchResources,

    // 统计命令
    GetUserStats,

    // 认证命令
    Login,
    Register,
    Logout,
    GetCurrentUser,
}

/// 命令请求基础结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// 要执行的命令动作
    pub action: Action,
    /// 命令参数，动态结构支持不同命令的参数需求
    pub params: serde_json::Value,
    /// 请求ID，用于追踪和日志
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// 命令响应基础结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// 是否成功执行
    pub success: bool,
    /// 响应数据
    pub data: Option<serde_json::Value>,
    /// 错误信息（如果有）
    pub error: Option<CommandError>,
    /// 对应的请求ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// 响应时间戳
    pub timestamp: i64,
}

/// 命令错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandError {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl CommandRequest {
    /// 创建新的命令请求
    pub fn _new(action: Action, params: serde_json::Value) -> Self {
        Self {
            action,
            params,
            request_id: Some(uuid::Uuid::new_v4().to_string()),
        }
    }

    /// 创建带请求ID的命令请求
    pub fn _with_request_id(action: Action, params: serde_json::Value, request_id: String) -> Self {
        Self {
            action,
            params,
            request_id: Some(request_id),
        }
    }

    /// 从参数中提取特定类型的参数
    pub fn get_param<T>(&self, key: &str) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(value) = self.params.get(key) {
            serde_json::from_value(value.clone())
        } else {
            serde_json::from_value(serde_json::Value::Null)
        }
    }

    /// 获取整个参数结构并反序列化为指定类型
    pub fn get_params<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value(self.params.clone())
    }
}

impl CommandResponse {
    /// 创建成功响应
    pub fn _success(data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            data,
            error: None,
            request_id: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 创建带请求ID的成功响应
    pub fn success_with_request_id(
        data: Option<serde_json::Value>,
        request_id: String,
    ) -> Self {
        Self {
            success: true,
            data,
            error: None,
            request_id: Some(request_id),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 创建错误响应
    pub fn _error(code: String, message: String, details: Option<serde_json::Value>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(CommandError {
                code,
                message,
                details,
            }),
            request_id: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// 创建带请求ID的错误响应
    pub fn error_with_request_id(
        code: String,
        message: String,
        details: Option<serde_json::Value>,
        request_id: String,
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(CommandError {
                code,
                message,
                details,
            }),
            request_id: Some(request_id),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_command_request_creation() {
        let params = json!({
            "id": 1,
            "name": "test"
        });

        let request = CommandRequest::_new(Action::GetResource, params.clone());

        assert_eq!(request.action, Action::GetResource);
        assert_eq!(request.params, params);
        assert!(request.request_id.is_some());
    }

    #[test]
    fn test_command_response_creation() {
        let data = json!({"id": 1, "name": "test"});

        let success_response = CommandResponse::_success(Some(data.clone()));
        assert!(success_response.success);
        assert!(success_response.error.is_none());
        assert_eq!(success_response.data, Some(data));

        let error_response = CommandResponse::_error(
            "NOT_FOUND".to_string(),
            "Resource not found".to_string(),
            None,
        );
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert!(error_response.error.is_some());
    }

    #[test]
    fn test_param_extraction() {
        let params = json!({
            "id": 42,
            "name": "test resource",
            "tags": ["tag1", "tag2"]
        });

        let request = CommandRequest::_new(Action::UpdateResource, params);

        let id: i64 = request.get_param("id").unwrap();
        assert_eq!(id, 42);

        let name: String = request.get_param("name").unwrap();
        assert_eq!(name, "test resource");
    }
}