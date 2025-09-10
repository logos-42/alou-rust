use crate::types::*;
use crate::deepseek_client::DeepseekClient;
use crate::tool_registry::ToolRegistry;
use crate::prompt_registry::PromptRegistry;
use crate::config::Config;
use crate::prompts::get_mcp_system_prompt;
use crate::env_config::{get_deepseek_api_key, get_deepseek_api_endpoint};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::{info, warn, error, debug};
use serde_json;
use chrono::{DateTime, Utc};

/// 文件操作智能体
pub struct FileOperationAgent {
    deepseek_client: Arc<DeepseekClient>,
    debug_mode: bool,
    config: Config,
    tool_registry: Arc<ToolRegistry>,
    prompt_registry: Arc<PromptRegistry>,
}

impl FileOperationAgent {
    /// 创建新的文件操作智能体
    pub fn new(
        config: Config,
        tool_registry: Arc<ToolRegistry>,
        prompt_registry: Arc<PromptRegistry>,
    ) -> Result<Self> {
        let debug_mode = false; // 暂时硬编码
        
        // 创建DeepSeek客户端配置
        let client_config = DeepseekClientConfig {
            api_key: config.deepseek_api.api_key.clone(),
            base_url: Some(config.deepseek_api.api_endpoint.clone()),
            timeout: Some(config.models.request_timeout * 1000), // 转换为毫秒
            max_retries: Some(config.models.max_retries as u32),
            debug_mode: Some(debug_mode),
            target_dir: Some(config.workspace_root.to_string_lossy().to_string()),
        };

        // 创建DeepSeek客户端
        let deepseek_client = Arc::new(DeepseekClient::new(client_config)?);

        Ok(Self {
            deepseek_client,
            debug_mode,
            config,
            tool_registry,
            prompt_registry,
        })
    }

    /// 初始化智能体
    pub async fn initialize(&mut self) -> Result<()> {
        if self.debug_mode {
            info!("开始初始化智能体...");
        }
        
        // 初始化DeepSeek客户端（包含工具发现）
        // TODO: 需要重新设计这个初始化逻辑，因为 Arc 不能直接借用为可变
        // self.deepseek_client.initialize().await?;
        
        if self.debug_mode {
            info!("智能体初始化完成");
        }
        
        Ok(())
    }

    /// 处理用户请求
    pub async fn process_request(&self, user_input: &str) -> Result<AgentResponse> {
        if self.debug_mode {
            info!("处理请求: {}...", &user_input[..user_input.len().min(50)]);
        }
        
        // 使用MCP优化的系统提示
        let system_prompt = get_mcp_system_prompt(&self.config.workspace_root.to_string_lossy());
        
        // 添加记忆上下文到提示中
        let memory_context = self.get_memory_context(user_input).await;
        let full_prompt = format!("{}\n\n{}\n\n用户请求: \"{}\"", system_prompt, memory_context, user_input);
        
        // 直接调用DeepseekClient处理请求
        let response = self.deepseek_client.chat_with_tools(&full_prompt, None, 10).await?;
        
        // 保存重要信息到记忆中
        self.save_to_memory(user_input, &response).await;
        
        Ok(AgentResponse {
            message: response.clone(),
            actions: vec![],
            content: Some(response),
            memory_updates: None, // 可以在这里添加内存更新信息
        })
    }

    /// 获取对话历史（从DeepseekClient获取）
    pub async fn get_conversation_history(&self) -> Vec<serde_json::Value> {
        self.deepseek_client.get_history().await
    }

    /// 获取可用工具信息
    pub async fn get_available_tools(&self) -> Vec<String> {
        self.deepseek_client.get_available_tools().await
    }


    /// 获取记忆上下文
    async fn get_memory_context(&self, user_input: &str) -> String {
        if !self.debug_mode {
            return String::new();
        }

        match self.search_memory(user_input).await {
            Ok(search_response) => {
                if search_response.contains("相关记忆") {
                    return format!("\n## 相关记忆上下文\n{}\n", search_response);
                }
            }
            Err(error) => {
                if self.debug_mode {
                    debug!("获取记忆上下文失败: {}", error);
                }
            }
        }
        
        String::new()
    }

    /// 搜索记忆
    async fn search_memory(&self, query: &str) -> Result<String> {
        let search_prompt = format!("请使用 mcp_memory_search_nodes 工具搜索与以下内容相关的记忆: \"{}\"", query);
        self.deepseek_client.chat_with_tools(&search_prompt, None, 3).await
    }

    /// 保存重要信息到记忆中
    async fn save_to_memory(&self, user_input: &str, response: &str) {
        if !self.should_save_to_memory(user_input, response) {
            return;
        }

        match self.save_conversation_to_memory(user_input, response).await {
            Ok(_) => {
                if self.debug_mode {
                    info!("已保存对话到记忆中");
                }
            }
            Err(error) => {
                if self.debug_mode {
                    debug!("保存到记忆失败: {}", error);
                }
            }
        }
    }

    /// 保存对话到记忆
    async fn save_conversation_to_memory(&self, user_input: &str, response: &str) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        
        // 通过 DeepseekClient 调用 memory 工具保存对话
        let save_prompt = format!(r#"请使用 mcp_memory_create_entities 工具保存以下对话到记忆中:
        
实体名称: 对话_{}
实体类型: conversation
观察记录:
- 用户输入: {}
- AI响应: {}...
- 时间: {}"#, 
            timestamp, 
            user_input, 
            &response[..response.len().min(200)], 
            timestamp
        );
        
        self.deepseek_client.chat_with_tools(&save_prompt, None, 3).await?;
        Ok(())
    }

    /// 判断是否需要保存到记忆中
    fn should_save_to_memory(&self, user_input: &str, response: &str) -> bool {
        // 保存包含重要信息的对话
        let important_keywords = [
            "项目", "配置", "设置", "偏好", "习惯", "重要", "记住", "保存",
            "项目结构", "文件路径", "工作目录", "开发环境"
        ];
        
        let input_lower = user_input.to_lowercase();
        let response_lower = response.to_lowercase();
        
        important_keywords.iter().any(|keyword| 
            input_lower.contains(keyword) || response_lower.contains(keyword)
        ) || response.len() > 500 // 长响应也保存
    }

    /// 获取智能体状态
    pub async fn get_status(&self) -> AgentStatus {
        let is_initialized = self.deepseek_client.is_initialized().await;
        let tool_count = self.get_available_tools().await.len();
        let memory_count = 0; // 不再使用本地内存上下文

        AgentStatus {
            is_initialized,
            tool_count,
            memory_count,
            debug_mode: self.debug_mode,
        }
    }

    /// 重置智能体状态
    pub async fn reset(&self) -> Result<()> {
        // 清理对话历史
        self.deepseek_client.set_history(Vec::new(), None).await;
        
        info!("智能体状态已重置");
        Ok(())
    }

    /// 设置调试模式
    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
    }

    /// 获取调试模式状态
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }
}

/// 智能体状态
#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub is_initialized: bool,
    pub tool_count: usize,
    pub memory_count: usize,
    pub debug_mode: bool,
}

/// 智能体构建器
pub struct AgentBuilder {
    config: Option<Config>,
    tool_registry: Option<Arc<ToolRegistry>>,
    prompt_registry: Option<Arc<PromptRegistry>>,
}

impl AgentBuilder {
    /// 创建新的智能体构建器
    pub fn new() -> Self {
        Self {
            config: None,
            tool_registry: None,
            prompt_registry: None,
        }
    }

    /// 设置配置
    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// 设置工具注册表
    pub fn tool_registry(mut self, tool_registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(tool_registry);
        self
    }

    /// 设置提示注册表
    pub fn prompt_registry(mut self, prompt_registry: Arc<PromptRegistry>) -> Self {
        self.prompt_registry = Some(prompt_registry);
        self
    }

    /// 构建智能体
    pub fn build(self) -> Result<FileOperationAgent> {
        let config = self.config.ok_or_else(|| anyhow::anyhow!("Config not provided"))?;
        let tool_registry = self.tool_registry.ok_or_else(|| anyhow::anyhow!("ToolRegistry not provided"))?;
        let prompt_registry = self.prompt_registry.ok_or_else(|| anyhow::anyhow!("PromptRegistry not provided"))?;

        FileOperationAgent::new(config, tool_registry, prompt_registry)
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 智能体工厂
pub struct AgentFactory;

impl AgentFactory {
    /// 创建默认智能体
    pub fn create_default() -> Result<FileOperationAgent> {
        let config = Config::new()?;
        let tool_registry = Arc::new(ToolRegistry::new());
        let prompt_registry = Arc::new(PromptRegistry::new());
        
        AgentBuilder::new()
            .config(config)
            .tool_registry(tool_registry)
            .prompt_registry(prompt_registry)
            .build()
    }

    /// 创建调试模式智能体
    pub fn create_debug() -> Result<FileOperationAgent> {
        let mut config = Config::new()?;
        // config.deepseek_api.debug_mode = Some(true); // 暂时注释掉
        let tool_registry = Arc::new(ToolRegistry::new());
        let prompt_registry = Arc::new(PromptRegistry::new());
        
        AgentBuilder::new()
            .config(config)
            .tool_registry(tool_registry)
            .prompt_registry(prompt_registry)
            .build()
    }

    /// 从环境变量创建智能体
    pub fn from_env() -> Result<FileOperationAgent> {
        Self::create_default()
    }

    /// 创建高性能智能体
    pub fn create_high_performance() -> Result<FileOperationAgent> {
        let mut config = Config::new()?;
        config.models.request_timeout = 10; // 10秒超时
        config.models.max_retries = 5; // 更多重试次数
        let tool_registry = Arc::new(ToolRegistry::new());
        let prompt_registry = Arc::new(PromptRegistry::new());
        
        AgentBuilder::new()
            .config(config)
            .tool_registry(tool_registry)
            .prompt_registry(prompt_registry)
            .build()
    }
}

/// 智能体管理器
pub struct AgentManager {
    agents: Arc<tokio::sync::RwLock<HashMap<String, Arc<FileOperationAgent>>>>,
}

impl AgentManager {
    /// 创建新的智能体管理器
    pub fn new() -> Self {
        Self {
            agents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 添加智能体
    pub async fn add_agent(&self, name: String, agent: Arc<FileOperationAgent>) {
        let mut agents = self.agents.write().await;
        agents.insert(name, agent);
    }

    /// 获取智能体
    pub async fn get_agent(&self, name: &str) -> Option<Arc<FileOperationAgent>> {
        let agents = self.agents.read().await;
        agents.get(name).cloned()
    }

    /// 移除智能体
    pub async fn remove_agent(&self, name: &str) -> Option<Arc<FileOperationAgent>> {
        let mut agents = self.agents.write().await;
        agents.remove(name)
    }

    /// 列出所有智能体
    pub async fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// 获取智能体数量
    pub async fn agent_count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_builder() {
        // 这个测试需要有效的API密钥，所以跳过
        // let agent = AgentBuilder::new()
        //     .debug_mode(true)
        //     .api_key("test_key".to_string())
        //     .build()
        //     .await;
        // assert!(agent.is_ok());
    }

    #[test]
    fn test_should_save_to_memory() {
        // 创建一个测试用的智能体
        let config = Config::default();
        let tool_registry = Arc::new(ToolRegistry::new(&config).unwrap());
        let prompt_registry = Arc::new(PromptRegistry::new());
        let agent = FileOperationAgent::new(config, tool_registry, prompt_registry).unwrap();

        // 测试重要关键词
        assert!(agent.should_save_to_memory("这是一个重要的项目配置", ""));
        assert!(agent.should_save_to_memory("", "这是项目结构信息"));
        
        // 测试长响应
        let long_response = "a".repeat(600);
        assert!(agent.should_save_to_memory("普通问题", &long_response));
        
        // 测试普通对话
        assert!(!agent.should_save_to_memory("你好", "你好！"));
    }

    #[tokio::test]
    async fn test_agent_manager() {
        let manager = AgentManager::new();
        assert_eq!(manager.agent_count().await, 0);
        
        let config = Config::default();
        let tool_registry = Arc::new(ToolRegistry::new(&config).unwrap());
        let prompt_registry = Arc::new(PromptRegistry::new());
        let agent = Arc::new(FileOperationAgent::new(config, tool_registry, prompt_registry).unwrap());
        
        manager.add_agent("test_agent".to_string(), agent).await;
        assert_eq!(manager.agent_count().await, 1);
        
        let retrieved = manager.get_agent("test_agent").await;
        assert!(retrieved.is_some());
        
        let removed = manager.remove_agent("test_agent").await;
        assert!(removed.is_some());
        assert_eq!(manager.agent_count().await, 0);
    }
}
