//! 工具桥接
//!
//! 提供前端与工具执行器的桥接功能

use super::super::tools::{ToolRegistry, ToolResult, ExecutionContext, ToolConfig};
use crate::tools::executor::ToolExecutionManager;
use crate::tools::{
    FileSystemTool, SearchTool, BashTool, PlanTool, TodoListTool, AgentSkillsTool,
    ToolCreationTool, network::NetworkTool, system::SystemTool,
    rollback::RollbackTool, pubsub_tool::PubSubTool, message_passing::MessagePassingTool,
    iroh_tool::IrohTool, ipfs_archive::IpfsArchiveTool, git_helper::GitHelperTool,
    browser_tool::BrowserTool, agent_creator::AgentCreatorTool, ui_control::UIControlTool,
    spec_tool::SpecTool, agent_wallet::AgentWalletTool, wallet_manager::WalletManagerTool,
    query_blockchain::QueryBlockchainTool, build_transaction::BuildTransactionTool,
    broadcast_transaction::BroadcastTransactionTool, polymarket::PolymarketTool,
    media_tools::{GenerateImageTool, GenerateAudioTool, GenerateVideoTool, GetVideoStatusTool},
};
use crate::agent::providers::ProviderRegistry;
use crate::media_archive::MediaArchiveManager;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

/// 工具桥接
#[derive(Clone)]
pub struct ToolBridge {
    registry: ToolRegistry,
    execution_manager: ToolExecutionManager,
    request_count: Arc<AtomicU64>,
}

impl ToolBridge {
    /// 创建新的工具桥接（同步版本，用于 Tauri setup）
    pub fn new_sync(config: ToolBridgeConfig) -> Self {
        let mut bridge = Self {
            registry: ToolRegistry::new(),
            execution_manager: ToolExecutionManager::new(config.tool_config.clone()),
            request_count: Arc::new(AtomicU64::new(0)),
        };

        // 在同步上下文中注册工具
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = bridge.register_all_tools().await {
                eprintln!("Failed to register tools: {}", e);
            }
        });

        bridge
    }

    /// 创建带媒体工具的工具桥接
    pub fn new_with_media_tools(
        config: ToolBridgeConfig,
        provider_registry: Arc<ProviderRegistry>,
        archive_manager: Arc<MediaArchiveManager>,
    ) -> Self {
        let mut bridge = Self {
            registry: ToolRegistry::new(),
            execution_manager: ToolExecutionManager::new(config.tool_config.clone()),
            request_count: Arc::new(AtomicU64::new(0)),
        };

        // 在同步上下文中注册工具
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = bridge.register_all_tools().await {
                eprintln!("Failed to register tools: {}", e);
            }
            // 注册媒体工具（异步版本）
            bridge.register_media_tools_async(provider_registry, archive_manager).await;
        });

        bridge
    }

    /// 创建新的工具桥接
    pub async fn new(config: ToolBridgeConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut bridge = Self {
            registry: ToolRegistry::new(),
            execution_manager: ToolExecutionManager::new(config.tool_config.clone()),
            request_count: Arc::new(AtomicU64::new(0)),
        };

        // 注册所有工具
        bridge.register_all_tools().await?;

        Ok(bridge)
    }

    /// 注册媒体工具（异步版本）
    pub async fn register_media_tools_async(
        &mut self,
        provider_registry: Arc<ProviderRegistry>,
        archive_manager: Arc<MediaArchiveManager>,
    ) {
        log::info!("[ToolBridge] 注册媒体工具...");

        let image_tool = Arc::new(GenerateImageTool::new(provider_registry.clone(), archive_manager.clone()));
        let audio_tool = Arc::new(GenerateAudioTool::new(provider_registry.clone(), archive_manager.clone()));
        let video_tool = Arc::new(GenerateVideoTool::new(provider_registry.clone(), archive_manager.clone()));
        let status_tool = Arc::new(GetVideoStatusTool::new(provider_registry, archive_manager));

        // 注册到 ToolRegistry（用于工具列表）和 ToolExecutionManager（用于执行）
        if let Err(e) = self.register_tool(image_tool.clone()).await {
            log::warn!("[ToolBridge] 注册 generate_image 到 ToolRegistry 失败: {}", e);
        }
        if let Err(e) = self.register_tool(audio_tool.clone()).await {
            log::warn!("[ToolBridge] 注册 generate_audio 到 ToolRegistry 失败: {}", e);
        }
        if let Err(e) = self.register_tool(video_tool.clone()).await {
            log::warn!("[ToolBridge] 注册 generate_video 到 ToolRegistry 失败: {}", e);
        }
        if let Err(e) = self.register_tool(status_tool.clone()).await {
            log::warn!("[ToolBridge] 注册 get_video_status 到 ToolRegistry 失败: {}", e);
        }

        log::info!("[ToolBridge] ✅ 媒体工具注册完成 (4 个工具)");
    }

    /// 注册媒体工具（同步版本 - 仅用于同步上下文）
    pub fn register_media_tools(
        &mut self,
        provider_registry: Arc<ProviderRegistry>,
        archive_manager: Arc<MediaArchiveManager>,
    ) {
        log::info!("[ToolBridge] 注册媒体工具（同步模式）...");

        let image_tool = Arc::new(GenerateImageTool::new(provider_registry.clone(), archive_manager.clone()));
        let audio_tool = Arc::new(GenerateAudioTool::new(provider_registry.clone(), archive_manager.clone()));
        let video_tool = Arc::new(GenerateVideoTool::new(provider_registry.clone(), archive_manager.clone()));
        let status_tool = Arc::new(GetVideoStatusTool::new(provider_registry, archive_manager));

        // 同步上下文下只注册到 execution_manager
        // ToolRegistry 的注册需要异步，由调用者确保在正确的上下文中调用
        self.execution_manager.register_executor("generate_image".to_string(), image_tool.clone());
        self.execution_manager.register_executor("generate_audio".to_string(), audio_tool.clone());
        self.execution_manager.register_executor("generate_video".to_string(), video_tool.clone());
        self.execution_manager.register_executor("get_video_status".to_string(), status_tool.clone());
        
        log::info!("[ToolBridge] ✅ 媒体工具已注册到 ExecutionManager");
        log::info!("[ToolBridge] ⚠️  注意：ToolRegistry 注册需要在异步上下文中调用 register_media_tools_async");
    }

    /// 处理工具调用请求
    pub async fn handle_request(&self, request: ToolCallRequest) -> Result<ToolCallResponse, Box<dyn std::error::Error + Send + Sync>> {
        // 无锁原子操作增加计数器
        self.request_count.fetch_add(1, Ordering::Relaxed);

        // 创建执行上下文
        let context = ExecutionContext {
            session_id: request.session_id.clone(),
            user_id: request.user_id.clone(),
            working_directory: request.working_directory.clone(),
            environment: request.environment.clone(),
            timeout_seconds: request.timeout_seconds,
            permissions: request.permissions.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        // 直接使用 ToolExecutionManager 执行
        match self.execution_manager.execute_tool(&request.tool_id, request.args, context).await {
            Ok(result) => {
                // 🔥 修复：检查 ToolResult.success，而非外层 Result
                // execute_with_context 会把工具执行错误转为 ToolResult { success: false, data: Null }
                // 而不是返回 Err，所以这里必须检查内部 success 字段
                if result.success {
                    Ok(ToolCallResponse {
                        success: true,
                        result: Some(result),
                        error: None,
                    })
                } else {
                    let error_msg = result.error.clone()
                        .unwrap_or_else(|| format!("工具执行失败，data={}", result.data));
                    log::warn!("[ToolBridge] 工具 {} 执行失败: {}", request.tool_id, error_msg);
                    Ok(ToolCallResponse {
                        success: false,
                        result: Some(result),
                        error: Some(error_msg),
                    })
                }
            }
            Err(e) => Ok(ToolCallResponse {
                success: false,
                result: None,
                error: Some(format!("{:?}", e)),
            }),
        }
    }

    /// 获取请求计数
    pub async fn get_request_count(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.request_count.load(Ordering::Relaxed))
    }

    /// 注册所有工具
    async fn register_all_tools(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 注册文件系统工具
        let fs_tool = Arc::new(FileSystemTool::new());
        self.register_tool(fs_tool).await?;

        // 注册搜索工具
        let search_tool = Arc::new(SearchTool::new());
        self.register_tool(search_tool).await?;

        // 注册 Bash 工具
        let bash_tool = Arc::new(BashTool::new());
        self.register_tool(bash_tool).await?;

        // 注册 Git 助手工具
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

        // 注册 Agent Skills 工具
        let skills_tool = Arc::new(AgentSkillsTool::new()?);
        self.register_tool(skills_tool).await?;

        // 注册 Agent 创建工具
        let agent_creator = Arc::new(AgentCreatorTool::new());
        self.register_tool(agent_creator).await?;

        // 注册工具创建工具
        let tool_creation = Arc::new(ToolCreationTool::new());
        self.register_tool(tool_creation).await?;

        // 注册回滚工具
        let rollback_tool = Arc::new(RollbackTool::new());
        self.register_tool(rollback_tool).await?;

        // 注册 PubSub 工具
        let pubsub_tool = Arc::new(PubSubTool::new());
        self.register_tool(pubsub_tool).await?;

        // 注册消息传递工具
        let msg_tool = Arc::new(MessagePassingTool::new());
        self.register_tool(msg_tool).await?;

        // 注册 Iroh 工具
        let iroh_tool = Arc::new(IrohTool::new());
        self.register_tool(iroh_tool).await?;

        // 注册 IPFS 归档工具
        let ipfs_archive_tool = Arc::new(IpfsArchiveTool::new());
        self.register_tool(ipfs_archive_tool).await?;

        // 注册浏览器工具
        let browser_tool = Arc::new(BrowserTool::new());
        self.register_tool(browser_tool).await?;

        // 注册 UI 控制工具
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

        // 注册 Polymarket 预测市场工具
        let polymarket_tool = Arc::new(PolymarketTool::new());
        self.register_tool(polymarket_tool).await?;

        println!("✅ All {} tools registered successfully in ToolBridge", self.registry.count().await);
        Ok(())
    }

    /// 注册工具
    async fn register_tool(&mut self, tool: Arc<dyn super::super::tools::ToolExecutor>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tool_id = tool.metadata().id.clone();

        // 注册到 ToolRegistry
        self.registry.register(tool.clone()).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // 注册到 ToolExecutionManager
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
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.execution_manager.cancel_execution(execution_id).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
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
    /// 会话 ID
    pub session_id: String,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 工具 ID
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
