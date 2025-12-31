//! Durable Objects测试套件

use crate::compatibility::models::{CompatibleRequest, CompatibleResponse, TaskStatusResponse};
use serde_json::json;

/// 测试Durable Objects基础功能
#[tokio::test]
async fn test_durable_objects_basics() {
    // 测试任务状态枚举
    use crate::compatibility::models::TaskStatus;
    
    assert_eq!(TaskStatus::Queued.to_string(), "queued");
    assert_eq!(TaskStatus::Running.to_string(), "running");
    assert_eq!(TaskStatus::Completed.to_string(), "completed");
    assert_eq!(TaskStatus::Failed.to_string(), "failed");
    
    // 测试任务状态响应
    let status_response = TaskStatusResponse::new(
        "test_task_123".to_string(),
        "running".to_string(),
    );
    
    assert_eq!(status_response.task_id, "test_task_123");
    assert_eq!(status_response.status, "running");
    assert!(status_response.progress.is_none());
    assert!(status_response.current_step.is_none());
    assert!(status_response.result.is_none());
    assert!(status_response.error.is_none());
    assert!(status_response.created_at > 0);
    assert!(status_response.updated_at > 0);
}

/// 测试异步任务决策逻辑
#[tokio::test]
async fn test_async_task_decision() {
    use crate::compatibility::models::should_use_async;
    
    // 测试明确指定异步
    let explicit_async = CompatibleRequest {
        task_type: Some("async".to_string()),
        ..Default::default()
    };
    assert!(should_use_async(&explicit_async));
    
    // 测试明确指定同步
    let explicit_sync = CompatibleRequest {
        task_type: Some("sync".to_string()),
        ..Default::default()
    };
    assert!(!should_use_async(&explicit_sync));
    
    // 测试长模型自动使用异步
    let long_model = CompatibleRequest {
        model: "claude-3-5-sonnet".to_string(),
        ..Default::default()
    };
    assert!(should_use_async(&long_model));
    
    // 测试短模型使用同步
    let short_model = CompatibleRequest {
        model: "deepseek-chat".to_string(),
        ..Default::default()
    };
    assert!(!should_use_async(&short_model));
}

/// 测试执行时间估计
#[tokio::test]
async fn test_execution_time_estimation() {
    use crate::compatibility::models::estimate_execution_time;
    
    // 测试短任务
    let short_request = CompatibleRequest {
        prompt: "Short prompt".to_string(),
        model: "deepseek-chat".to_string(),
        history: vec![],
        tools: vec![],
        ..Default::default()
    };
    
    let short_time = estimate_execution_time(&short_request);
    assert!(short_time > 0);
    assert!(short_time < 30); // 短任务应该小于30秒
}

/// 测试工具调用框架
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
    
    // 测试工具参数模式
    for tool in available_tools {
        assert!(!tool.name.is_empty());
        if let Some(parameters) = tool.parameters {
            assert!(parameters.is_object());
        }
    }
}

/// 测试监控指标
#[tokio::test]
async fn test_compatibility_metrics() {
    use crate::compatibility::metrics::CompatibilityMetrics;
    
    let metrics = CompatibilityMetrics::new();
    
    // 记录一些指标
    metrics.record_legacy_api_call(true);
    metrics.record_legacy_api_call(false);
    metrics.record_task_created();
    metrics.record_task_completed(true);
    metrics.record_task_completed(false);
    metrics.record_sync_response_time(100_000); // 100ms
    metrics.record_async_response_time(500_000); // 500ms
    metrics.record_tool_call(true, false);
    metrics.record_tool_call(false, true);
    
    // 获取快照
    let snapshot = metrics.snapshot();
    
    // 验证指标
    assert_eq!(snapshot.legacy_api_calls, 2);
    assert_eq!(snapshot.legacy_api_errors, 1);
    assert!(snapshot.legacy_api_error_rate > 0.0);
    
    assert_eq!(snapshot.tasks_created, 1);
    assert_eq!(snapshot.tasks_completed, 1);
    assert_eq!(snapshot.tasks_failed, 1);
    assert!(snapshot.task_success_rate > 0.0);
    
    assert!(snapshot.avg_sync_response_time_ms > 0.0);
    assert!(snapshot.avg_async_response_time_ms > 0.0);
    
    assert_eq!(snapshot.tool_calls, 2);
    assert_eq!(snapshot.tool_call_errors, 1);
    assert_eq!(snapshot.tool_calls_requires_local, 1);
    assert!(snapshot.tool_call_success_rate > 0.0);
}
