use crate::types::*;
use crate::mcp_tool::DiscoveredMcpTool;
use crate::mcp_config::McpConfigLoader;
use crate::types::McpServerConfig;
use crate::workspace_context::{WorkspaceContext, BasicWorkspaceContext};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Result, Context};
use serde_json;
use tokio::sync::RwLock;

/// 工具注册表
/// 管理所有可用工具的注册和发现
#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Box<dyn Tool + Send + Sync>>>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册工具定义
    /// 
    /// # Arguments
    /// * `tool` - 包含模式和执行逻辑的工具对象
    pub async fn register_tool(&self, tool: Box<dyn Tool + Send + Sync>) -> Result<()> {
        let tool_name = tool.name().to_string();
        let mut tools = self.tools.write().await;
        
        if tools.contains_key(&tool_name) {
            // 如果是MCP工具，转换为完全限定名称
            if let Some(mcp_tool) = tool.as_any().downcast_ref::<DiscoveredMcpTool>() {
                let qualified_tool = mcp_tool.as_fully_qualified_tool();
                tools.insert(qualified_tool.name().to_string(), Box::new(qualified_tool));
            } else {
                // 决定行为：记录警告或允许覆盖
                tracing::warn!(
                    "Tool with name \"{}\" is already registered. Overwriting.",
                    tool_name
                );
                tools.insert(tool_name, tool);
            }
        } else {
            tools.insert(tool_name, tool);
        }
        
        Ok(())
    }

    /// 移除所有发现的工具
    async fn remove_discovered_tools(&self) {
        let mut tools = self.tools.write().await;
        tools.retain(|_, tool| {
            // 检查是否为MCP工具
            tool.as_any().downcast_ref::<DiscoveredMcpTool>().is_none()
        });
    }

    /// 移除特定MCP服务器的所有工具
    /// 
    /// # Arguments
    /// * `server_name` - 要移除工具的服务器名称
    pub async fn remove_mcp_tools_by_server(&self, server_name: &str) {
        let mut tools = self.tools.write().await;
        tools.retain(|_, tool| {
            if let Some(mcp_tool) = tool.as_any().downcast_ref::<DiscoveredMcpTool>() {
                mcp_tool.server_name() != server_name
            } else {
                true
            }
        });
    }

    /// 发现项目中的所有工具（如果可用且已配置）
    /// 可以多次调用以更新发现的工具
    /// 这将从命令行和MCP服务器发现工具
    pub async fn discover_all_tools(&self, debug_mode: bool) -> Result<()> {
        // 移除任何先前发现的工具
        self.remove_discovered_tools().await;

        // 创建所需依赖的实例
        let workspace_context = Arc::new(BasicWorkspaceContext::new());

        // 使用MCP服务器发现工具（如果已配置）
        // 从mcp.json加载MCP服务器配置
        let mcp_servers = self.load_mcp_servers().await?;
        
        if !mcp_servers.is_empty() {
            // 处理环境变量替换
            let processed_servers = self.process_environment_variables(mcp_servers).await?;
            
            // 发现MCP工具
            self.discover_mcp_tools_internal(processed_servers, debug_mode, workspace_context).await?;
        }

        Ok(())
    }

    /// 仅从MCP服务器发现工具
    /// 这将不会从命令行发现工具，仅从MCP服务器发现
    pub async fn discover_mcp_tools(&self, debug_mode: bool) -> Result<()> {
        // 移除任何先前发现的工具
        self.remove_discovered_tools().await;

        // 创建所需依赖的实例
        let workspace_context = Arc::new(BasicWorkspaceContext::new());

        // 使用MCP服务器发现工具（如果已配置）
        let mcp_servers = self.load_mcp_servers().await?;
        
        if !mcp_servers.is_empty() {
            // 处理环境变量替换
            let processed_servers = self.process_environment_variables(mcp_servers).await?;
            
            // 发现MCP工具
            self.discover_mcp_tools_internal(processed_servers, debug_mode, workspace_context).await?;
        }

        Ok(())
    }

    /// 发现或重新发现单个MCP服务器的工具
    /// 
    /// # Arguments
    /// * `server_name` - 要发现工具的服务器名称
    /// * `debug_mode` - 是否启用调试模式
    pub async fn discover_tools_for_server(&self, server_name: &str, debug_mode: bool) -> Result<()> {
        // 移除此服务器先前发现的任何工具
        self.remove_mcp_tools_by_server(server_name).await;

        // 创建所需依赖的实例
        let workspace_context = Arc::new(BasicWorkspaceContext::new());

        // 从mcp.json加载MCP服务器配置
        let mcp_servers = self.load_mcp_servers().await?;
        
        // 过滤到仅指定的服务器
        let server_config = mcp_servers.get(server_name);
        let filtered_mcp_servers = if let Some(config) = server_config {
            let mut filtered = HashMap::new();
            filtered.insert(server_name.to_string(), config.clone());
            filtered
        } else {
            HashMap::new()
        };
        
        if !filtered_mcp_servers.is_empty() {
            // 处理环境变量替换
            let processed_servers = self.process_environment_variables(filtered_mcp_servers).await?;
            
            // 发现MCP工具
            self.discover_mcp_tools_internal(processed_servers, debug_mode, workspace_context).await?;
        }

        Ok(())
    }

    /// 获取工具模式列表（FunctionDeclaration数组）
    /// 从ToolListUnion结构中提取声明
    /// 如果已配置，包括发现的（相对于注册的）工具
    /// 
    /// # Returns
    /// FunctionDeclaration数组
    pub async fn get_function_declarations(&self) -> Vec<serde_json::Value> {
        let tools = self.tools.read().await;
        let mut declarations = Vec::new();
        
        for tool in tools.values() {
            // 确保我们有正确的工具名称和参数模式
            if !tool.name().is_empty() {
                let declaration = serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameter_schema()
                    }
                });
                declarations.push(declaration);
            }
        }
        
        declarations
    }

    /// 基于工具名称列表获取过滤的工具模式列表
    /// 
    /// # Arguments
    /// * `tool_names` - 要包含的工具名称数组
    /// 
    /// # Returns
    /// 指定工具的FunctionDeclaration数组
    pub async fn get_function_declarations_filtered(&self, tool_names: &[String]) -> Vec<serde_json::Value> {
        let tools = self.tools.read().await;
        let mut declarations = Vec::new();
        
        for name in tool_names {
            if let Some(tool) = tools.get(name) {
                let declaration = serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameter_schema()
                    }
                });
                declarations.push(declaration);
            }
        }
        
        declarations
    }

    /// 返回所有已注册和发现的工具实例数组
    pub async fn get_all_tools(&self) -> Vec<Box<dyn Tool + Send + Sync>> {
        let tools = self.tools.read().await;
        let mut tool_list: Vec<Box<dyn Tool + Send + Sync>> = Vec::new();
        
        // 由于 dyn Tool 不能直接 clone，我们需要重新创建工具
        // 这里暂时返回空列表，实际实现需要根据具体工具类型来重新创建
        for (name, _tool) in tools.iter() {
            // TODO: 根据工具类型重新创建工具实例
            // 这需要更复杂的实现来支持工具的重新创建
        }
        
        tool_list
    }

    /// 返回从特定MCP服务器注册的工具数组
    pub async fn get_tools_by_server(&self, server_name: &str) -> Vec<Box<dyn Tool + Send + Sync>> {
        let tools = self.tools.read().await;
        let mut server_tools = Vec::new();
        
        for tool in tools.values() {
            if let Some(mcp_tool) = tool.as_any().downcast_ref::<DiscoveredMcpTool>() {
                if mcp_tool.server_name() == server_name {
                    // TODO: 重新创建工具实例而不是 clone
                    // server_tools.push(tool.clone());
                }
            }
        }
        
        server_tools
    }

    /// 获取特定工具的定义
    pub async fn get_tool(&self, name: &str) -> Option<Box<dyn Tool + Send + Sync>> {
        let tools = self.tools.read().await;
        
        // 首先尝试精确匹配
        if tools.contains_key(name) {
            return None; // TODO: 需要重新创建工具实例
        }
        
        // 尝试其他可能的格式
        // 将连字符转换为下划线
        let underscore_name = name.replace('-', "_");
        if tools.contains_key(&underscore_name) {
            return None; // TODO: 需要重新创建工具实例
        }
        
        // 将下划线转换为连字符
        let dash_name = name.replace('_', "-");
        if tools.contains_key(&dash_name) {
            return None; // TODO: 需要重新创建工具实例
        }
        
        // 尝试查找MCP工具的完全限定名称
        for (key, tool) in tools.iter() {
            if let Some(mcp_tool) = tool.as_any().downcast_ref::<DiscoveredMcpTool>() {
                // 检查是否是完全限定名称
                if key == &format!("{}__{}", mcp_tool.server_name(), name) {
                    return None; // TODO: 需要重新创建工具实例
                }
                // 检查原始名称
                if mcp_tool.server_tool_name() == name {
                    return None; // TODO: 需要重新创建工具实例
                }
                // 检查原始名称的不同格式
                if mcp_tool.server_tool_name() == underscore_name {
                    return None; // TODO: 需要重新创建工具实例
                }
                if mcp_tool.server_tool_name() == dash_name {
                    return None; // TODO: 需要重新创建工具实例
                }
            }
        }
        
        None
    }

    /// 获取工具数量
    pub async fn tool_count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    /// 检查工具是否存在
    pub async fn has_tool(&self, name: &str) -> bool {
        self.get_tool(name).await.is_some()
    }

    /// 获取所有工具名称
    pub async fn get_tool_names(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// 清空所有工具
    pub async fn clear_all_tools(&self) {
        let mut tools = self.tools.write().await;
        tools.clear();
    }

    /// 加载MCP服务器配置
    async fn load_mcp_servers(&self) -> Result<HashMap<String, McpServerConfig>> {
        // 尝试多个可能的路径
        let possible_paths = vec![
            PathBuf::from("mcp.json"),                    // 当前工作目录
            PathBuf::from("dist/mcp.json"),               // 生产环境
            PathBuf::from("../mcp.json"),                 // 开发环境
        ];
        
        for path in possible_paths {
            if path.exists() {
                if let Some(parent) = path.parent() {
                    if let Some(_server) = McpConfigLoader::load_mcp_config()? {
                        // TODO: 需要重新实现这个逻辑，因为现在返回的是单个配置而不是HashMap
                        // return Ok(servers);
                    }
                }
            }
        }
        
        Ok(HashMap::new())
    }

    /// 处理MCP服务器配置中的环境变量
    async fn process_environment_variables(&self, mcp_servers: HashMap<String, McpServerConfig>) -> Result<HashMap<String, McpServerConfig>> {
        let mut processed_servers = HashMap::new();
        
        for (server_name, mut server_config) in mcp_servers {
            // 处理args数组中的环境变量
            if let Some(args) = &mut server_config.args {
                let mut new_args = Vec::new();
                
                for arg in args.iter() {
                    // 处理 ALOU_INSTALL_DIR
                    if arg.contains("${ALOU_INSTALL_DIR}") {
                        let install_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                        let resolved_path = self.resolve_fetch_path(arg, &install_dir, &server_name);
                        new_args.push(resolved_path);
                    }
                    // 处理 OS_FILESYSTEM_PATHS_ARRAY - 展开为多个参数
                    else if arg.contains("${OS_FILESYSTEM_PATHS_ARRAY}") {
                        let paths = self.get_os_filesystem_paths();
                        // 将逗号分隔的字符串转换为数组并展开
                        let path_array: Vec<String> = paths.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        
                        // 在Mac上添加调试信息
                        if cfg!(target_os = "macos") && !path_array.is_empty() {
                            tracing::debug!("Filesystem paths for Mac: {}", path_array.join(", "));
                        }
                        
                        new_args.extend(path_array);
                    }
                    else {
                        new_args.push(arg.clone());
                    }
                }
                
                server_config.args = Some(new_args);
            }
            
            processed_servers.insert(server_name, server_config);
        }
        
        Ok(processed_servers)
    }

    /// 解析fetch工具的路径
    fn resolve_fetch_path(&self, arg: &str, install_dir: &PathBuf, server_name: &str) -> String {
        // 智能路径解析：尝试多个可能的路径
        let possible_paths = vec![
            install_dir.join("dist/src/fetch.js"),           // 生产环境：npm包安装目录
            install_dir.join("../dist/src/fetch.js"),        // 开发环境：从src目录向上查找
            PathBuf::from("dist/src/fetch.js"),              // 当前工作目录
            PathBuf::from("src/fetch.ts"),                   // 开发环境：直接使用源码
        ];
        
        // 查找第一个存在的文件
        for test_path in possible_paths {
            if test_path.exists() {
                return test_path.to_string_lossy().to_string();
            }
        }
        
        // 如果找到了文件，使用绝对路径；否则使用原始路径
        tracing::warn!("Warning: Could not find fetch.js in any expected location for server {}", server_name);
        arg.replace("${ALOU_INSTALL_DIR}", &install_dir.to_string_lossy())
    }

    /// 获取跨平台的文件系统路径
    fn get_os_filesystem_paths(&self) -> String {
        if cfg!(target_os = "windows") {
            // Windows: 检测存在的盘符
            self.get_windows_drives()
        } else if cfg!(target_os = "macos") {
            // macOS: 只使用用户目录，避免根目录权限问题
            "/Users".to_string()
        } else if cfg!(target_os = "linux") {
            // Linux
            "/,/home".to_string()
        } else {
            "/".to_string()
        }
    }

    /// 获取Windows系统的可用盘符
    fn get_windows_drives(&self) -> String {
        let mut drives = Vec::new();
        
        // 检测常见的Windows盘符
        let common_drives = vec!['C', 'D', 'E', 'F'];
        for drive in common_drives {
            let drive_path = format!("{}:\\", drive);
            if PathBuf::from(&drive_path).exists() {
                drives.push(drive_path);
            }
        }
        
        // 如果没有找到任何盘符，至少返回C盘
        if drives.is_empty() {
            "C:\\".to_string()
        } else {
            drives.join(",")
        }
    }

    /// 内部MCP工具发现方法
    async fn discover_mcp_tools_internal(
        &self,
        mcp_servers: HashMap<String, McpServerConfig>,
        debug_mode: bool,
        workspace_context: Arc<dyn WorkspaceContext + Send + Sync>,
    ) -> Result<()> {
        // 这里应该实现实际的MCP工具发现逻辑
        // 由于MCP客户端还没有实现，这里先留空
        // 在实际实现中，这里会：
        // 1. 连接到每个MCP服务器
        // 2. 发现可用的工具
        // 3. 将工具注册到注册表中
        
        if debug_mode {
            tracing::debug!("Discovering MCP tools from {} servers", mcp_servers.len());
        }
        
        // TODO: 实现实际的MCP工具发现
        // 这需要与mcp_client模块集成
        
        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具注册表构建器
pub struct ToolRegistryBuilder {
    registry: ToolRegistry,
}

impl ToolRegistryBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// 添加工具
    pub async fn add_tool(mut self, tool: Box<dyn Tool + Send + Sync>) -> Result<Self> {
        self.registry.register_tool(tool).await?;
        Ok(self)
    }

    /// 发现所有工具
    pub async fn discover_all_tools(mut self, debug_mode: bool) -> Result<Self> {
        self.registry.discover_all_tools(debug_mode).await?;
        Ok(self)
    }

    /// 构建工具注册表
    pub fn build(self) -> ToolRegistry {
        self.registry
    }
}

impl Default for ToolRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{BaseDeclarativeTool, Kind};

    // 模拟工具实现
    struct MockTool {
        name: String,
        description: String,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str { &self.name }
        fn description(&self) -> &str { &self.description }
        fn display_name(&self) -> &str { &self.name }
        fn kind(&self) -> Kind { Kind::Other }
        fn parameter_schema(&self) -> &serde_json::Value { &serde_json::json!({}) }
        fn is_output_markdown(&self) -> bool { true }
        fn can_update_output(&self) -> bool { false }
        
        async fn execute(&self, _params: HashMap<String, serde_json::Value>) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>> {
            Ok(ToolResultContent {
                content: "Mock response".to_string(),
                mime_type: None,
                llm_content: None,
                return_display: None,
            })
        }
    }

    #[tokio::test]
    async fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.tool_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_tool() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockTool {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
        });
        
        registry.register_tool(tool).await.unwrap();
        assert_eq!(registry.tool_count().await, 1);
        assert!(registry.has_tool("test_tool").await);
    }

    #[tokio::test]
    async fn test_get_tool() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockTool {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
        });
        
        registry.register_tool(tool).await.unwrap();
        
        let retrieved_tool = registry.get_tool("test_tool").await;
        assert!(retrieved_tool.is_some());
        assert_eq!(retrieved_tool.unwrap().name(), "test_tool");
    }

    #[tokio::test]
    async fn test_get_function_declarations() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockTool {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
        });
        
        registry.register_tool(tool).await.unwrap();
        
        let declarations = registry.get_function_declarations().await;
        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0]["function"]["name"], "test_tool");
    }

    #[tokio::test]
    async fn test_tool_registry_builder() {
        let registry = ToolRegistryBuilder::new()
            .add_tool(Box::new(MockTool {
                name: "test_tool".to_string(),
                description: "Test tool".to_string(),
            }))
            .await
            .unwrap()
            .build();
        
        assert_eq!(registry.tool_count().await, 1);
    }
}
