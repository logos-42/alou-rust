//! 工具桥接
//!
//! 提供前端与工具执行器的桥接功能

use super::super::tools::{ToolRegistry, ToolResult, ExecutionContext, ToolConfig};
use crate::tools::executor::ToolExecutionManager;
use crate::tools::{FileSystemTool, SearchTool, BashTool, PlanTool, TodoListTool, AgentSkillsTool};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// 工具桥接
pub struct ToolBridge {
    registry: ToolRegistry,
    execution_manager: ToolExecutionManager,
    request_count: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl ToolBridge {
    /// 创建新的工具桥接（同步版本，用于Tauri setup）
    pub fn new_sync(config: ToolBridgeConfig) -> Self {
        let mut bridge = Self {
            registry: ToolRegistry::new(),
            execution_manager: ToolExecutionManager::new(config.tool_config.clone()),
            request_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        };
        
        // 在同步上下文中注册工具
        // 注意：这里使用blocking_register来避免异步问题
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = bridge.register_all_tools().await {
                eprintln!("Failed to register tools: {}", e);
            }
        });
        
        bridge
    }

    /// 创建新的工具桥接
    pub async fn new(config: ToolBridgeConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut bridge = Self {
            registry: ToolRegistry::new(),
            execution_manager: ToolExecutionManager::new(config.tool_config.clone()),
            request_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        };
        
        // 注册所有工具
        bridge.register_all_tools().await?;
        
        Ok(bridge)
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

    /// 注册所有工具
    async fn register_all_tools(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 注册文件系统工具
        let fs_tool = Arc::new(FileSystemTool::new());
        self.register_tool(fs_tool).await?;
        
        // 注册搜索工具
        let search_tool = Arc::new(SearchTool::new());
        self.register_tool(search_tool).await?;
        
        // 注册Bash工具
        let bash_tool = Arc::new(BashTool::new());
        self.register_tool(bash_tool).await?;
        
        // 注册计划工具
        let plan_tool = Arc::new(PlanTool::new());
        self.register_tool(plan_tool).await?;
        
        // 注册待办事项工具
        let todo_tool = Arc::new(TodoListTool::new());
        self.register_tool(todo_tool).await?;
        
        // 注册Agent Skills工具
        let skills_tool = Arc::new(AgentSkillsTool::new()?);
        self.register_tool(skills_tool).await?;
        
        println!("✅ All tools registered successfully in ToolBridge");
        Ok(())
    }

    /// 注册工具
    async fn register_tool(&mut self, tool: Arc<dyn super::super::tools::ToolExecutor>) -> Result<(), Box<dyn std::error::Error>> {
        let tool_id = tool.metadata().id.clone();
        
        // 注册到ToolRegistry
        self.registry.register(tool.clone()).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        // 注册到ToolExecutionManager
        self.execution_manager.register_executor(tool_id.clone(), tool);
        
        println!("✅ Tool '{}' registered successfully", tool_id);
        Ok(())
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