//! 统一工具访问入口（外观模式）
//!
//! 提供统一的工具执行接口

use std::sync::Arc;
use serde_json::Value;
use crate::tools::registry::ToolRegistry;
use crate::tools::executor::ToolExecutionManager;
use crate::tools::{ExecutionContext, ToolResult, ToolError};

/// 统一工具访问入口
pub struct ToolFacade {
    registry: Arc<ToolRegistry>,
    execution_manager: Arc<ToolExecutionManager>,
}

impl ToolFacade {
    /// 创建新的统一入口
    pub fn new(registry: Arc<ToolRegistry>, execution_manager: Arc<ToolExecutionManager>) -> Self {
        Self { registry, execution_manager }
    }

    /// 从 BridgeManager 创建
    pub fn from_bridge_manager(bridge_manager: &crate::bridges::BridgeManager) -> Self {
        // 获取 ToolBridge 的内部组件
        // 注意：这里需要 BridgeManager 暴露相关方法
        // 暂时使用空实现
        Self {
            registry: Arc::new(ToolRegistry::new()),
            execution_manager: Arc::new(ToolExecutionManager::new(crate::tools::ToolConfig::default())),
        }
    }

    /// 执行工具（统一接口）
    pub async fn execute(
        &self,
        name: &str,
        args: Value,
        context: ExecutionContext,
    ) -> Result<Value, String> {
        // 使用 ToolExecutionManager 执行
        match self.execution_manager.execute_tool(name, args, context).await {
            Ok(result) => {
                if result.success {
                    Ok(result.data)
                } else {
                    Err(result.error.unwrap_or_else(|| "执行失败".to_string()))
                }
            }
            Err(e) => Err(format!("{:?}", e))
        }
    }

    /// 列出所有可用工具
    pub async fn list_tools(&self) -> Vec<ToolInfo> {
        let mut tools = Vec::new();

        // 添加 ToolExecutionManager 的工具
        for meta in self.execution_manager.list_tools() {
            tools.push(ToolInfo {
                name: meta.id.clone(),
                description: meta.description.clone(),
                source: "execution_manager".to_string(),
            });
        }

        tools
    }

    /// 获取工具详情
    pub async fn get_tool_info(&self, name: &str) -> Option<ToolInfo> {
        if let Some(_executor) = self.execution_manager.get_executor(name) {
            let meta = self.execution_manager.get_executor(name)?.metadata();
            return Some(ToolInfo {
                name: meta.id.clone(),
                description: meta.description.clone(),
                source: "execution_manager".to_string(),
            });
        }

        // 再查 ToolRegistry
        if let Some(tool) = self.registry.get_tool(name).await {
            let meta = tool.metadata();
            return Some(ToolInfo {
                name: meta.id.clone(),
                description: meta.description.clone(),
                source: "registry".to_string(),
            });
        }

        None
    }

    /// 检查工具是否存在
    pub fn has_tool(&self, name: &str) -> bool {
        self.execution_manager.has_tool(name)
    }
}

/// 工具信息（统一格式）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub source: String,
}
