//! 工具桥接
//!
//! 提供前端与工具执行器的桥接功能

use super::super::tools::{ToolRegistry, ToolResult, ToolError, ExecutionContext, ToolConfig};
use crate::tools::executor::ToolExecutionManager;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// 工具桥接
pub struct ToolBridge {
    registry: ToolRegistry,
    execution_manager: ToolExecutionManager,
    request_count: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl ToolBridge {
    /// 创建新的工具桥接
    pub fn new(config: ToolBridgeConfig) -> Self {
        Self {
            registry: ToolRegistry::new(),
            execution_manager: ToolExecutionManager::new(config.tool_config),
            request_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// 处理工具调用请求
    pub async fn handle_request(&self, request: ToolCallRequest) -> Result<ToolCallResponse, Box<dyn std::error::Error>> {
        // 先增加计数器，避免跨越await点
        {
            let mut count = self.request_count.lock().unwrap();
            *count += 1;
        }

        // 创建执行上下文
        let context = ExecutionContext {
            session_id: request.session_id,
            user_id: request.user_id,
            working_directory: request.working_directory,
            environment: request.environment,
            timeout_seconds: request.timeout_seconds,
            permissions: request.permissions,
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 执行工具
        match self.execution_manager.execute_tool(&request.tool_id, request.args, context).await {
            Ok(result) => Ok(ToolCallResponse {
                success: true,
                result: Some(result),
                error: None,
            }),
            Err(e) => Ok(ToolCallResponse {
                success: false,
                result: None,
                error: Some(format!("{:?}", e)),
            }),
        }
    }

    /// 获取请求计数
    pub async fn get_request_count(&self) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(*self.request_count.lock().unwrap())
    }

    /// 注册工具
    pub async fn register_tool(&mut self, tool: Arc<dyn super::super::tools::ToolExecutor>) -> Result<(), Box<dyn std::error::Error>> {
        self.registry.register(tool).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// 更新配置
    pub fn update_config(&mut self, _config: ToolBridgeConfig) {
        // TODO: 实现配置更新逻辑
    }

    /// 健康检查
    pub async fn health_check(&self) -> super::ComponentHealthStatus {
        super::ComponentHealthStatus {
            is_healthy: true,
            message: "Tool bridge is healthy".to_string(),
            last_check: chrono::Utc::now().timestamp(),
            error_details: None,
        }
    }
}

/// 工具桥接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolBridgeConfig {
    /// 工具配置
    pub tool_config: ToolConfig,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存大小
    pub cache_size: usize,
}

impl Default for ToolBridgeConfig {
    fn default() -> Self {
        Self {
            tool_config: ToolConfig::default(),
            enable_cache: true,
            cache_size: 100,
        }
    }
}

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// 会话ID
    pub session_id: String,
    /// 用户ID
    pub user_id: Option<String>,
    /// 工具ID
    pub tool_id: String,
    /// 工具参数
    pub args: serde_json::Value,
    /// 工作目录
    pub working_directory: Option<String>,
    /// 环境变量
    pub environment: std::collections::HashMap<String, String>,
    /// 超时时间
    pub timeout_seconds: Option<u64>,
    /// 权限
    pub permissions: Vec<String>,
}

/// 工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    /// 是否成功
    pub success: bool,
    /// 结果
    pub result: Option<ToolResult>,
    /// 错误信息
    pub error: Option<String>,
}