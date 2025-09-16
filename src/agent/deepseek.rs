use serde::{Deserialize, Serialize};
use crate::error::Error;

/// DeepSeek API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    /// API基础URL
    pub base_url: String,
    /// API密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// 最大token数
    pub max_tokens: u32,
    /// 温度参数
    pub temperature: f32,
}

/// DeepSeek API客户端
pub struct DeepSeekClient {
    /// HTTP客户端
    client: reqwest::Client,
    /// 配置
    config: DeepSeekConfig,
}

impl DeepSeekClient {
    /// 创建新的DeepSeek客户端
    pub fn new(config: DeepSeekConfig) -> Self {
        let client = reqwest::Client::new();
        Self { client, config }
    }
    
    /// 发送聊天请求
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse, Error> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: Some(false),
            tools: None,
        };
        
        let response = self.client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Other(format!("API请求失败: {}", error_text)));
        }
        
        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
            
        Ok(chat_response)
    }
    
    /// 发送带工具的聊天请求
    pub async fn chat_with_tools(&self, messages: Vec<ChatMessage>, tools: Vec<serde_json::Value>) -> Result<ChatResponse, Error> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: Some(false),
            tools: Some(tools),
        };
        
        let response = self.client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Other(format!("API请求失败: {}", error_text)));
        }
        
        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
            
        Ok(chat_response)
    }
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 角色
    pub role: String,
    /// 内容
    pub content: String,
    /// 工具调用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallMessage>>,
    /// 工具结果（用于tool角色）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// 工具调用消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallMessage {
    /// 工具调用ID
    pub id: Option<String>,
    /// 工具调用类型
    #[serde(rename = "type")]
    pub call_type: String,
    /// 函数调用
    pub function: FunctionCall,
}

/// 函数调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// 函数名称
    pub name: String,
    /// 函数参数
    pub arguments: String,
}

/// 聊天请求
#[derive(Debug, Serialize)]
struct ChatRequest {
    /// 模型名称
    model: String,
    /// 消息列表
    messages: Vec<ChatMessage>,
    /// 最大token数
    max_tokens: Option<u32>,
    /// 温度参数
    temperature: Option<f32>,
    /// 是否流式响应
    stream: Option<bool>,
    /// 工具定义
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

/// 聊天响应
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// 选择列表
    pub choices: Vec<Choice>,
    /// 使用情况
    pub usage: Option<Usage>,
}

/// 选择
#[derive(Debug, Deserialize)]
pub struct Choice {
    /// 消息
    pub message: ChatMessage,
    /// 完成原因
    pub finish_reason: Option<String>,
}

/// 使用情况
#[derive(Debug, Deserialize)]
pub struct Usage {
    /// 提示token数
    pub prompt_tokens: u32,
    /// 完成token数
    pub completion_tokens: u32,
    /// 总token数
    pub total_tokens: u32,
}
