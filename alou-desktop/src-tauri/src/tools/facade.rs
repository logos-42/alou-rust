//! 统一工具访问入口（外观模式）
//!
//! 将 ToolRegistry 和 ToolBus 统一到一个接口

use std::sync::Arc;
use serde_json::Value;
use crate::tools::registry::ToolRegistry;
use crate::agent_runtime::tool_bus::ToolBus;

/// 统一工具访问入口
pub struct ToolFacade {
    registry: Arc<ToolRegistry>,
    toolbus: Arc<ToolBus>,
}

impl ToolFacade {
    /// 创建新的统一入口
    pub fn new(registry: Arc<ToolRegistry>, toolbus: Arc<ToolBus>) -> Self {
        Self { registry, toolbus }
    }

    /// 执行工具（统一接口）
    pub async fn execute(
        &self,
        name: &str,
        args: Value,
        context: crate::tools::ExecutionContext,
    ) -> Result<Value, String> {
        // 1. 先尝试 ToolBus（媒体工具，数量少，快速失败）
        if let Ok(result) = self.toolbus.execute(name, args.clone()).await {
            log::debug!("ToolFacade: 工具 '{}' 由 ToolBus 执行", name);
            return Ok(result);
        }

        // 2. Fallback 到 ToolRegistry（核心工具）
        if let Some(tool) = self.registry.get_tool(name).await {
            log::debug!("ToolFacade: 工具 '{}' 由 ToolRegistry 执行", name);
            let result = tool.execute(args, context).await
                .map_err(|e| format!("工具执行失败：{}", e))?;
            return Ok(serde_json::to_value(result).unwrap_or(Value::Null));
        }

        // 3. 工具不存在
        Err(format!("工具不存在：{}", name))
    }

    /// 列出所有可用工具
    pub async fn list_tools(&self) -> Vec<ToolInfo> {
        let mut tools = Vec::new();

        // 添加 ToolRegistry 的工具
        for meta in self.registry.list_all().await {
            tools.push(ToolInfo {
                name: meta.id,
                description: meta.description,
                source: "registry".to_string(),
            });
        }

        // 添加 ToolBus 的工具
        for meta in self.toolbus.list_tools().await {
            tools.push(ToolInfo {
                name: meta.name,
                description: meta.description,
                source: "toolbus".to_string(),
            });
        }

        tools
    }

    /// 获取工具详情
    pub async fn get_tool_info(&self, name: &str) -> Option<ToolInfo> {
        // 先查 ToolBus
        if let Ok(tools) = self.toolbus.list_tools().await {
            if let Some(tool) = tools.iter().find(|t| t.name == name) {
                return Some(ToolInfo {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    source: "toolbus".to_string(),
                });
            }
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
}

/// 工具信息（统一格式）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub source: String,  // "registry" 或 "toolbus"
}
