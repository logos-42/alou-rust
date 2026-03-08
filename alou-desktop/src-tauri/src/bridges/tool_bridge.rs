//! 工具桥接
//!
//! 提供前端与工具执行器的桥接功能

use super::super::tools::{ToolRegistry, ToolResult, ExecutionContext, ToolConfig};
use crate::tools::executor::ToolExecutionManager;
use crate::tools::{
    FileSystemTool, SearchTool, BashTool, PlanTool, TodoListTool, AgentSkillsTool,
    AgentCollaborationTool, ToolCreationTool, network::NetworkTool, system::SystemTool,
    rollback::RollbackTool, pubsub_tool::PubSubTool, message_passing::MessagePassingTool,
    iroh_tool::IrohTool, ipfs_archive::IpfsArchiveTool, git_helper::GitHelperTool,
    browser_tool::BrowserTool, agent_creator::AgentCreatorTool, ui_control::UIControlTool,
    spec_tool::SpecTool, agent_wallet::AgentWalletTool, wallet_manager::WalletManagerTool,
    query_blockchain::QueryBlockchainTool, build_transaction::BuildTransactionTool,
    broadcast_transaction::BroadcastTransactionTool,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// 工具桥接
#[derive(Clone)]
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
        
        // 注册Git助手工具
        let git_tool = Arc::new(GitHelperTool::new());
        self.register_tool(git_tool).await?;
        
        // 注册网络工具
        let network_tool = Arc::new(NetworkTool::new());
        self.register_tool(network_tool).await?;
        
        // 注册系统工具
        let system_tool = Arc::new(SystemTool::new());
        self.register_tool(system_tool).await?;
        
        // 注册计划工具
        let plan_tool = Arc::new(PlanTool::new());
        self.register_tool(plan_tool).await?;
        
        // 注册待办事项工具
        let todo_tool = Arc::new(TodoListTool::new());
        self.register_tool(todo_tool).await?;
        
        // 注册Agent Skills工具
        let skills_tool = Arc::new(AgentSkillsTool::new()?);
        self.register_tool(skills_tool).await?;

        // 注册Agent协作工具
        let ipfs_api_url = std::env::var("IPFS_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".to_string());
        let collab_tool = Arc::new(AgentCollaborationTool::new(ipfs_api_url));
        self.register_tool(collab_tool).await?;
        
        // 注册Agent创建工具
        let agent_creator = Arc::new(AgentCreatorTool::new());
        self.register_tool(agent_creator).await?;
        
        // 注册工具创建工具
        let tool_creation = Arc::new(ToolCreationTool::new());
        self.register_tool(tool_creation).await?;
        
        // 注册回滚工具
        let rollback_tool = Arc::new(RollbackTool::new());
        self.register_tool(rollback_tool).await?;
        
        // 注册PubSub工具
        let pubsub_tool = Arc::new(PubSubTool::new());
        self.register_tool(pubsub_tool).await?;
        
        // 注册消息传递工具
        let msg_tool = Arc::new(MessagePassingTool::new());
        self.register_tool(msg_tool).await?;
        
        // 注册Iroh工具
        let iroh_tool = Arc::new(IrohTool::new());
        self.register_tool(iroh_tool).await?;
        
        // 注册IPFS归档工具
        let ipfs_archive_tool = Arc::new(IpfsArchiveTool::new());
        self.register_tool(ipfs_archive_tool).await?;
        
        // 注册浏览器工具
        let browser_tool = Arc::new(BrowserTool::new());
        self.register_tool(browser_tool).await?;

        // 注册UI控制工具
        let ui_tool = Arc::new(UIControlTool::new(None));
        self.register_tool(ui_tool).await?;
        
        // 注册 Spec 工具
        let spec_tool = Arc::new(SpecTool::new());
        self.register_tool(spec_tool).await?;
        
        // 注册 Agent 钱包工具
        let agent_wallet_tool = Arc::new(AgentWalletTool::new());
        self.register_tool(agent_wallet_tool).await?;
        
        // 注册钱包管理器工具
        let wallet_manager_tool = Arc::new(WalletManagerTool::new());
        self.register_tool(wallet_manager_tool).await?;
        
        // 注册区块链查询工具
        let query_blockchain_tool = Arc::new(QueryBlockchainTool::new());
        self.register_tool(query_blockchain_tool).await?;
        
        // 注册交易构建工具
        let build_transaction_tool = Arc::new(BuildTransactionTool::new());
        self.register_tool(build_transaction_tool).await?;
        
        // 注册交易广播工具
        let broadcast_transaction_tool = Arc::new(BroadcastTransactionTool::new());
        self.register_tool(broadcast_transaction_tool).await?;
        
        println!("✅ All {} tools registered successfully in ToolBridge", self.registry.count().await);
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
    pub fn update_config(&mut self, config: ToolBridgeConfig) {
        println!("[ToolBridge] Updating configuration");
        // 更新工具执行管理器的配置
        self.execution_manager.update_config(config.tool_config.clone());
        // 更新缓存配置（当前仅打印日志，未来可以实现缓存管理）
        if config.enable_cache {
            println!("[ToolBridge] Cache enabled (size: {})", config.cache_size);
        } else {
            println!("[ToolBridge] Cache disabled");
        }
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

    /// 获取所有工具列表
    pub async fn list_tools(&self) -> Vec<serde_json::Value> {
        let metadata_list = self.registry.list_all().await;

        metadata_list.into_iter().map(|metadata| {
            serde_json::json!({
                "id": metadata.id,
                "name": metadata.name,
                "category": format!("{:?}", metadata.category),
                "description": metadata.description,
                "version": metadata.version,
                "status": format!("{:?}", metadata.status),
            })
        }).collect()
    }

    /// 取消工具执行
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.execution_manager.cancel_execution(execution_id).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// 获取执行历史
    pub async fn get_execution_history(&self, limit: usize) -> Vec<serde_json::Value> {
        self.execution_manager.get_all_execution_contexts(limit).await
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