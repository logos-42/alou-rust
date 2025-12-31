//! 兼容性测试套件

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, HistoryMessage, Tool};
use serde_json::json;

/// 测试现有SDK的请求格式兼容性
#[tokio::test]
async fn test_legacy_api_compatibility() {
    // 测试现有SDK的请求格式
    let request = CompatibleRequest {
        api_key: Some("test_key".to_string()),
        prompt: "Hello, how are you?".to_string(),
        system_prompt: Some("You are a helpful assistant.".to_string()),
        history: vec![
            HistoryMessage {
                role: "user".to_string(),
                content: "Previous message".to_string(),
                timestamp: Some(1234567890),
            },
            HistoryMessage {
                role: "assistant".to_string(),
                content: "Previous response".to_string(),
                timestamp: Some(1234567891),
            },
        ],
        agent_info: Some(json!({
            "name": "test_agent",
            "version": "1.0.0"
        })),
        tools: vec![
            Tool {
                name: "get_current_time".to_string(),
                description: Some("Get current time".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "enum": ["timestamp", "iso8601", "human"]
                        }
                    }
                })),
            },
        ],
        model: "deepseek-chat".to_string(),
        max_tokens: Some(4096),
        temperature: 0.7,
        task_type: Some("sync".to_string()),
        timeout: Some(30),
        callback_url: None,
        session_id: Some("test_session".to_string()),
        wallet_address: Some("0x1234567890abcdef".to_string()),
        chain: Some("ethereum".to_string()),
        context_events: vec![],
    };

    // 验证请求可以正确序列化和反序列化
    let json_str = serde_json::to_string(&request).unwrap();
    let deserialized: CompatibleRequest = serde_json::from_str(&json_str).unwrap();
    
    assert_eq!(deserialized.prompt, "Hello, how are you?");
    assert_eq!(deserialized.model, "deepseek-chat");
    assert_eq!(deserialized.task_type, Some("sync".to_string()));
}

/// 测试工具执行框架
#[tokio::test]
async fn test_tool_execution_framework() {
    use crate::compatibility::tools::{ToolExecutor, CompatibleTool};
    
    // 创建工具执行器
    let executor = ToolExecutor::new();
    
    // 测试获取可用工具
    let available_tools = executor.get_available_tools();
    assert!(!available_tools.is_empty());
    
    // 验证包含基础工具
    let tool_names: Vec<String> = available_tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"get_current_time".to_string()));
    assert!(tool_names.contains(&"calculate".to_string()));
    assert!(tool_names.contains(&"search_web".to_string()));
}

/// 测试任务状态响应
#[tokio::test]
async fn test_task_status_response() {
    use crate::compatibility::models::TaskStatusResponse;
    
    let status = TaskStatusResponse::new(
        "test_task_123".to_string(),
        "running".to_string(),
    );
    
    assert_eq!(status.task_id, "test_task_123");
    assert_eq!(status.status, "running");
    assert!(status.progress.is_none());
    assert!(status.current_step.is_none());
    assert!(status.result.is_none());
    assert!(status.error.is_none());
    assert!(status.created_at > 0);
    assert!(status.updated_at > 0);
}
