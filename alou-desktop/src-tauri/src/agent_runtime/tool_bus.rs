//! Tool Bus - 工具总线

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use crate::agent::providers::ProviderRegistry;
use crate::tools::trait_def::Tool;
use crate::media_archive::MediaArchiveManager;

/// 工具总线
pub struct ToolBus {
    tools: RwLock<HashMap<String, Box<dyn Tool>>>,
}

impl ToolBus {
    pub fn new() -> Self {
        let mut bus = Self {
            tools: RwLock::new(HashMap::new()),
        };

        // 注册内置工具
        bus.register_tool("filesystem", Box::new(FilesystemTool));
        bus.register_tool("browser", Box::new(BrowserTool));
        bus.register_tool("search", Box::new(SearchTool));
        bus.register_tool("shell", Box::new(ShellTool));

        bus
    }

    /// 创建并注册媒体工具
    pub fn register_media_tools(
        &self,
        provider_registry: Arc<ProviderRegistry>,
        archive_manager: Arc<MediaArchiveManager>,
    ) {
        use crate::tools::media_tools::{
            GenerateImageTool,
            GenerateAudioTool,
            GenerateVideoTool,
            GetVideoStatusTool,
        };

        log::info!("[ToolBus] 开始注册媒体工具，可用 providers: {:?}", provider_registry.available_providers());

        self.register_tool("generate_image", Box::new(GenerateImageTool::new(
            provider_registry.clone(),
            archive_manager.clone(),
        )));
        self.register_tool("generate_audio", Box::new(GenerateAudioTool::new(
            provider_registry.clone(),
            archive_manager.clone(),
        )));
        self.register_tool("generate_video", Box::new(GenerateVideoTool::new(
            provider_registry.clone(),
            archive_manager.clone(),
        )));
        self.register_tool("get_video_status", Box::new(GetVideoStatusTool::new(
            provider_registry.clone(),
            archive_manager.clone(),
        )));

        log::info!("[ToolBus] 媒体工具注册完成，共注册 4 个工具");
    }
    
    pub fn register_tool(&self, name: &str, tool: Box<dyn Tool>) {
        self.tools.blocking_write().insert(name.to_string(), tool);
        log::info!("工具注册：{}", name);
    }
    
    pub async fn execute(&self, tool_name: &str, args: Value) -> Result<Value, String> {
        let tools = self.tools.read().await;
        let tool = tools.get(tool_name)
            .ok_or_else(|| format!("工具不存在：{}", tool_name))?;

        let context = crate::tools::trait_def::ToolContext::default();
        tool.execute(args, &context).await
    }
    
    pub async fn list_tools(&self) -> Vec<ToolInfo> {
        let tools = self.tools.read().await;
        tools.values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
            })
            .collect()
    }
}

impl Default for ToolBus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

// 内置工具实现（简化版）

struct FilesystemTool;
#[async_trait::async_trait]
impl Tool for FilesystemTool {
    fn name(&self) -> &str { "filesystem" }
    fn description(&self) -> &str { "文件系统操作工具" }
    async fn execute(&self, _args: Value, _context: &crate::tools::trait_def::ToolContext) -> Result<Value, String> {
        Ok(Value::String("filesystem tool not implemented".to_string()))
    }
}

struct BrowserTool;
#[async_trait::async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str { "browser" }
    fn description(&self) -> &str { "浏览器自动化工具" }
    async fn execute(&self, _args: Value, _context: &crate::tools::trait_def::ToolContext) -> Result<Value, String> {
        Ok(Value::String("browser tool not implemented".to_string()))
    }
}

struct SearchTool;
#[async_trait::async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str { "search" }
    fn description(&self) -> &str { "搜索工具" }
    async fn execute(&self, _args: Value, _context: &crate::tools::trait_def::ToolContext) -> Result<Value, String> {
        Ok(Value::String("search tool not implemented".to_string()))
    }
}

struct ShellTool;
#[async_trait::async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str { "shell" }
    fn description(&self) -> &str { "Shell 命令执行工具" }
    async fn execute(&self, _args: Value, _context: &crate::tools::trait_def::ToolContext) -> Result<Value, String> {
        Ok(Value::String("shell tool not implemented".to_string()))
    }
}
