// ============================================
// Claude Agent SDK 适配器
// 保留连接池作为MCP配置源，使用Claude SDK + DeepSeek API处理所有AI交互
// ============================================

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, warn};

use crate::connection_pool::{ConnectionPool, McpServerConfig};
use crate::error::Error;
use crate::prompts;

use super::types::{
    Agent, AgentConfig, AgentContext, AgentState, AgentMessage, MessageType, 
    ToolCall
};

// HTTP客户端用于直接调用DeepSeek API
use reqwest::Client;
use serde_json::json;

/// DeepSeek API 适配器
/// 保留连接池作为MCP配置源，直接使用DeepSeek API处理所有AI交互
pub struct Adapter {
    /// 原始配置
    config: AgentConfig,
    
    /// 保留的连接池（仅用于MCP服务器配置）
    connection_pool: Arc<ConnectionPool>,
    
    /// HTTP客户端
    http_client: Client,
    
    /// 简化的上下文（仅保留必要状态）
    context: Arc<tokio::sync::RwLock<AgentContext>>,
}

impl Adapter {
    /// 创建新的适配器
    pub async fn new(config: AgentConfig) -> Result<Self, Error> {
        Self::with_connection_pool(config, Arc::new(ConnectionPool::new())).await
    }
    
    /// 使用指定的连接池创建适配器
    pub async fn with_connection_pool(config: AgentConfig, connection_pool: Arc<ConnectionPool>) -> Result<Self, Error> {
        // 创建HTTP客户端
        let http_client = Client::new();
        
        // 创建简化的智能体上下文
        let context = Arc::new(tokio::sync::RwLock::new(AgentContext {
            state: AgentState::Idle,
            message_history: Vec::new(),
            workspace_context: crate::workspace_context::WorkspaceContextFactory::create_smart(),
            available_tools: HashMap::new(),
            current_task: None,
        }));
        
        Ok(Self {
            config,
            context,
            connection_pool,
            http_client,
        })
    }
    
    /// 注册服务器配置（保留连接池逻辑，但仅用于配置）
    async fn register_server_configs(&mut self) -> Result<(), Error> {
        // 从mcp.json加载服务器配置
        if std::path::Path::new("mcp.json").exists() {
            let content = std::fs::read_to_string("mcp.json")?;
            let mcp_config: serde_json::Value = serde_json::from_str(&content)?;
            
            if let Some(servers) = mcp_config.get("mcpServers").and_then(|s| s.as_object()) {
                for (name, config) in servers {
                    let server_config = McpServerConfig {
                        command: config.get("command")
                            .and_then(|c| c.as_str())
                            .unwrap_or("")
                            .to_string(),
                        args: config.get("args")
                            .and_then(|a| a.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                            .unwrap_or_default(),
                        env: Some(config.get("env")
                            .and_then(|e| e.as_object())
                            .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect::<std::collections::HashMap<String, String>>())
                            .unwrap_or_default()),
                        directory: config.get("directory")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                    };
                    
                    self.connection_pool.register_server(name.clone(), server_config).await;
                    info!("已注册MCP服务器配置: {}", name);
                }
            }
        } else {
            warn!("未找到mcp.json文件，将使用默认配置");
        }
        
        Ok(())
    }
    
    /// 构建系统提示（简化版）
    async fn build_system_prompt(&self) -> String {
        let context = self.context.read().await;
        // 使用压缩的工作空间信息
        let workspace_root = context.workspace_context.get_compressed_info();
        
        // 使用简化的MCP系统提示词
        prompts::get_mcp_system_prompt(&workspace_root)
    }
    
    /// 直接使用DeepSeek API进行查询（核心功能）
    async fn query_with_deepseek(&self, prompt: &str) -> Result<String, Error> {
        // 构建系统提示
        let system_prompt = self.build_system_prompt().await;
        
        // 构建请求体
        let request_body = json!({
            "model": self.config.deepseek.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user", 
                    "content": prompt
                }
            ],
            "max_tokens": self.config.deepseek.max_tokens,
            "temperature": self.config.deepseek.temperature,
            "stream": false
        });
        
        // 发送请求到DeepSeek API
        let response = self.http_client
            .post(&format!("{}/v1/chat/completions", self.config.deepseek.base_url))
            .header("Authorization", format!("Bearer {}", self.config.deepseek.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Other(format!("HTTP请求失败: {}", e)))?;
        
        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Other(format!("API请求失败: {} - {}", status, error_text)));
        }
        
        // 解析响应
        let response_json: serde_json::Value = response.json().await
            .map_err(|e| Error::Other(format!("响应解析失败: {}", e)))?;
        
        // 提取回复内容
        if let Some(choices) = response_json.get("choices").and_then(|c| c.as_array()) {
            if let Some(first_choice) = choices.first() {
                if let Some(message) = first_choice.get("message") {
                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                        return Ok(content.to_string());
                    }
                }
            }
        }
        
        Err(Error::Other("API响应格式不正确".to_string()))
    }
}

#[async_trait]
impl Agent for Adapter {
    async fn initialize(&mut self) -> Result<(), Error> {
        tracing::info!("初始化DeepSeek API适配器...");
        
        // 注册服务器配置（仅用于配置，不实际连接）
        self.register_server_configs().await?;
        
        // 更新状态
        {
            let mut context = self.context.write().await;
            context.state = AgentState::Idle;
        }
        
        tracing::info!("DeepSeek API适配器初始化完成");
        
        Ok(())
    }
    
    async fn process_input(&mut self, input: &str) -> Result<String, Error> {
        self.process_input_with_iterations(input, 1).await
    }

    async fn process_input_with_iterations(&mut self, input: &str, _max_iterations: usize) -> Result<String, Error> {
        tracing::info!("处理用户输入: {}", input);
        
        // 更新状态
        {
            let mut context = self.context.write().await;
            context.current_task = Some(input.to_string());
            context.state = AgentState::Thinking;
        }
        
        // 使用DeepSeek API进行查询
        let response = self.query_with_deepseek(input).await?;
        
        // 更新状态和消息历史
        {
            let mut context = self.context.write().await;
            context.state = AgentState::Idle;
            
            let agent_message = AgentMessage {
                id: uuid::Uuid::new_v4().to_string(),
                message_type: MessageType::AgentResponse,
                content: response.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                tool_calls: Vec::new(),
            };
            context.message_history.push(agent_message);
        }
        
        Ok(response)
    }
    
    async fn execute_tool(&mut self, _tool_call: &ToolCall) -> Result<serde_json::Value, Error> {
        // 工具调用由连接池处理
        Err(Error::Other("工具调用由连接池处理".to_string()))
    }
    
    fn get_state(&self) -> &AgentState {
        &AgentState::Idle // 临时实现
    }
    
    fn get_context(&self) -> &AgentContext {
        // 由于AgentContext包含Arc<RwLock<>>，这里需要重新设计
        unimplemented!("需要重新设计状态访问方式 - 使用异步方法获取上下文")
    }
    
    async fn reset(&mut self) -> Result<(), Error> {
        let mut context = self.context.write().await;
        context.state = AgentState::Idle;
        context.message_history.clear();
        context.current_task = None;
        Ok(())
    }
}
