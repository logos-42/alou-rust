use crate::types::*;
use crate::tools::{BaseDeclarativeTool, BaseToolInvocation};
use crate::types::Tool;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json;
use anyhow::{Result, Context};
use tokio_util::sync::CancellationToken;

/// 发现的MCP工具调用实现
pub struct DiscoveredMcpToolInvocation {
    mcp_tool: Arc<dyn CallableTool + Send + Sync>,
    server_name: String,
    server_tool_name: String,
    display_name: String,
    timeout: Option<u64>,
    trust: Option<bool>,
    params: HashMap<String, serde_json::Value>,
    allowlist: Arc<tokio::sync::RwLock<HashMap<String, bool>>>,
}

impl DiscoveredMcpToolInvocation {
    pub fn new(
        mcp_tool: Arc<dyn CallableTool + Send + Sync>,
        server_name: String,
        server_tool_name: String,
        display_name: String,
        timeout: Option<u64>,
        trust: Option<bool>,
        params: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            mcp_tool,
            server_name,
            server_tool_name,
            display_name,
            timeout,
            trust,
            params,
            allowlist: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ToolInvocation for DiscoveredMcpToolInvocation {
    fn name(&self) -> &str {
        &self.server_tool_name
    }

    fn params(&self) -> &HashMap<String, serde_json::Value> {
        &self.params
    }

    async fn should_confirm_execute(&self, _abort_signal: &CancellationToken) -> Result<Option<ToolCallConfirmationDetails>, Box<dyn std::error::Error + Send + Sync>> {
        let server_allowlist_key = &self.server_name;
        let tool_allowlist_key = format!("{}.{}", self.server_name, self.server_tool_name);

        if self.trust.unwrap_or(false) {
            return Ok(None); // 服务器受信任，无需确认
        }

        let allowlist = self.allowlist.read().await;
        if allowlist.contains_key(server_allowlist_key) || allowlist.contains_key(&tool_allowlist_key) {
            return Ok(None); // 服务器和/或工具已在白名单中
        }

        let confirmation_details = ToolCallConfirmationDetails {
            tool_name: self.server_tool_name.clone(),
            params: self.params.clone(),
        };

        Ok(Some(confirmation_details))
    }

    async fn execute(&self) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>> {
        let function_calls = vec![FunctionCall {
            name: self.server_tool_name.clone(),
            args: self.params.clone(),
        }];

        let raw_response_parts = self.mcp_tool.call_tool(function_calls).await?;
        let transformed_parts = transform_mcp_content_to_parts(&raw_response_parts);

        Ok(ToolResultContent {
            content: get_stringified_result_for_display(&raw_response_parts),
            mime_type: None,
            llm_content: Some(serde_json::to_value(transformed_parts)?),
            return_display: Some(get_stringified_result_for_display(&raw_response_parts)),
        })
    }

    fn get_description(&self) -> &str {
        &self.display_name
    }
}

/// 发现的MCP工具
pub struct DiscoveredMcpTool {
    mcp_tool: Arc<dyn CallableTool + Send + Sync>,
    server_name: String,
    server_tool_name: String,
    description: String,
    parameter_schema: serde_json::Value,
    timeout: Option<u64>,
    trust: Option<bool>,
    base_tool: BaseDeclarativeTool,
}

impl DiscoveredMcpTool {
    pub fn new(
        mcp_tool: Arc<dyn CallableTool + Send + Sync>,
        server_name: String,
        server_tool_name: String,
        description: String,
        parameter_schema: serde_json::Value,
        timeout: Option<u64>,
        trust: Option<bool>,
        name_override: Option<String>,
    ) -> Self {
        let name = name_override.unwrap_or_else(|| generate_valid_name(&server_tool_name));
        let display_name = format!("{} ({} MCP Server)", server_tool_name, server_name);
        
        let base_tool = BaseDeclarativeTool::new(
            name,
            display_name,
            description.clone(),
            Kind::Other,
            parameter_schema.clone(),
            true,  // is_output_markdown
            false, // can_update_output
        );

        Self {
            mcp_tool,
            server_name,
            server_tool_name,
            description,
            parameter_schema,
            timeout,
            trust,
            base_tool,
        }
    }

    /// 创建完全限定的工具名称
    pub fn as_fully_qualified_tool(&self) -> Self {
        let qualified_name = format!("{}__{}", self.server_name, self.server_tool_name);
        Self::new(
            self.mcp_tool.clone(),
            self.server_name.clone(),
            self.server_tool_name.clone(),
            self.description.clone(),
            self.parameter_schema.clone(),
            self.timeout,
            self.trust,
            Some(qualified_name),
        )
    }

    /// 获取服务器名称
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// 获取服务器工具名称
    pub fn server_tool_name(&self) -> &str {
        &self.server_tool_name
    }
}

#[async_trait]
impl Tool for DiscoveredMcpTool {
    fn name(&self) -> &str {
        &self.base_tool.name
    }

    fn description(&self) -> &str {
        &self.base_tool.description
    }

    fn display_name(&self) -> &str {
        &self.base_tool.display_name
    }

    fn kind(&self) -> Kind {
        self.base_tool.kind.clone()
    }

    fn parameter_schema(&self) -> &serde_json::Value {
        &self.base_tool.parameter_schema
    }

    fn is_output_markdown(&self) -> bool {
        self.base_tool.is_output_markdown
    }

    fn can_update_output(&self) -> bool {
        self.base_tool.can_update_output
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn should_confirm_execute(&self, abort_signal: &CancellationToken) -> Result<Option<ToolCallConfirmationDetails>, Box<dyn std::error::Error + Send + Sync>> {
        let invocation = self.create_invocation(HashMap::new());
        invocation.should_confirm_execute(abort_signal).await
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>> {
        let invocation = self.create_invocation(params);
        invocation.execute().await
    }

    async fn build_and_execute(&self, params: HashMap<String, serde_json::Value>, abort_signal: Option<&CancellationToken>) -> Result<ToolResultContent, Box<dyn std::error::Error + Send + Sync>> {
        let invocation = self.create_invocation(params);
        if let Some(signal) = abort_signal {
            invocation.should_confirm_execute(signal).await?;
        }
        invocation.execute().await
    }
}

impl DiscoveredMcpTool {
    fn create_invocation(&self, params: HashMap<String, serde_json::Value>) -> DiscoveredMcpToolInvocation {
        DiscoveredMcpToolInvocation::new(
            self.mcp_tool.clone(),
            self.server_name.clone(),
            self.server_tool_name.clone(),
            self.display_name().to_string(),
            self.timeout,
            self.trust,
            params,
        )
    }
}

/// 转换文本块
fn transform_text_block(block: &McpContentBlock) -> Option<Part> {
    if let McpContentBlock::Text { text } = block {
        Some(Part {
            text: Some(text.clone()),
            inline_data: None,
            function_response: None,
        })
    } else {
        None
    }
}

/// 转换图像/音频块
fn transform_image_audio_block(block: &McpContentBlock, tool_name: &str) -> Option<Vec<Part>> {
    match block {
        McpContentBlock::Image { mime_type, data } | McpContentBlock::Audio { mime_type, data } => {
            let block_type = if matches!(block, McpContentBlock::Image { .. }) {
                "image"
            } else {
                "audio"
            };
            
            Some(vec![
                Part {
                    text: Some(format!(
                        "[Tool '{}' provided the following {} data with mime-type: {}]",
                        tool_name, block_type, mime_type
                    )),
                    inline_data: None,
                    function_response: None,
                },
                Part {
                    text: None,
                    inline_data: Some(InlineData {
                        mime_type: mime_type.clone(),
                        data: data.clone(),
                    }),
                    function_response: None,
                },
            ])
        }
        _ => None,
    }
}

/// 转换资源块
fn transform_resource_block(block: &McpContentBlock, tool_name: &str) -> Option<Part> {
    if let McpContentBlock::Resource { resource } = block {
        if let Some(text) = &resource.text {
            return Some(Part {
                text: Some(text.clone()),
                inline_data: None,
                function_response: None,
            });
        }
        
        if let Some(blob) = &resource.blob {
            let mime_type = resource.mime_type.as_deref().unwrap_or("application/octet-stream");
            return Some(Part {
                text: Some(format!(
                    "[Tool '{}' provided the following embedded resource with mime-type: {}]",
                    tool_name, mime_type
                )),
                inline_data: Some(InlineData {
                    mime_type: mime_type.to_string(),
                    data: blob.clone(),
                }),
                function_response: None,
            });
        }
    }
    None
}

/// 转换资源链接块
fn transform_resource_link_block(block: &McpContentBlock) -> Option<Part> {
    if let McpContentBlock::ResourceLink { uri, title, name } = block {
        let unknown = "Unknown".to_string();
        let display_name = title.as_ref().or(name.as_ref()).unwrap_or(&unknown);
        Some(Part {
            text: Some(format!("Resource Link: {} at {}", display_name, uri)),
            inline_data: None,
            function_response: None,
        })
    } else {
        None
    }
}

/// 将原始MCP内容块从SDK响应转换为标准GenAI Part数组
/// 
/// # Arguments
/// * `sdk_response` - 从`mcp_tool.call_tool()`返回的原始Part[]数组
/// 
/// # Returns
/// 准备用于调度器的干净Part[]数组
fn transform_mcp_content_to_parts(sdk_response: &[Part]) -> Vec<Part> {
    if sdk_response.is_empty() {
        return vec![Part {
            text: Some("[Error: Empty tool response]".to_string()),
            inline_data: None,
            function_response: None,
        }];
    }

    let func_response = &sdk_response[0].function_response;
    let tool_name = func_response.as_ref()
        .map(|fr| fr.name.as_str())
        .unwrap_or("unknown tool");

    // 尝试从function_response中提取MCP内容
    if let Some(func_response) = func_response {
        if let Ok(mcp_content) = serde_json::from_value::<Vec<McpContentBlock>>(func_response.response.clone()) {
            let mut transformed = Vec::new();
            
            for block in mcp_content {
                let parts = match &block {
                    McpContentBlock::Text { .. } => {
                        if let Some(part) = transform_text_block(&block) {
                            vec![part]
                        } else {
                            continue;
                        }
                    }
                    McpContentBlock::Image { .. } | McpContentBlock::Audio { .. } => {
                        if let Some(parts) = transform_image_audio_block(&block, tool_name) {
                            parts
                        } else {
                            continue;
                        }
                    }
                    McpContentBlock::Resource { .. } => {
                        if let Some(part) = transform_resource_block(&block, tool_name) {
                            vec![part]
                        } else {
                            continue;
                        }
                    }
                    McpContentBlock::ResourceLink { .. } => {
                        if let Some(part) = transform_resource_link_block(&block) {
                            vec![part]
                        } else {
                            continue;
                        }
                    }
                };
                
                transformed.extend(parts);
            }
            
            return transformed;
        }
    }

    // 如果无法解析MCP内容，返回原始响应
    sdk_response.to_vec()
}

/// 处理MCP工具的原始响应以生成干净的、人类可读的字符串用于CLI显示
/// 它总结非文本内容并直接呈现文本
/// 
/// # Arguments
/// * `raw_response` - 从GenAI SDK返回的原始Part[]数组
/// 
/// # Returns
/// 表示工具输出的格式化字符串
fn get_stringified_result_for_display(raw_response: &[Part]) -> String {
    if raw_response.is_empty() {
        return "[Error: Empty tool response]".to_string();
    }

    let func_response = &raw_response[0].function_response;
    let tool_name = func_response.as_ref()
        .map(|fr| fr.name.as_str())
        .unwrap_or("unknown tool");

    // 尝试从function_response中提取MCP内容
    if let Some(func_response) = func_response {
        if let Ok(mcp_content) = serde_json::from_value::<Vec<McpContentBlock>>(func_response.response.clone()) {
            let display_parts: Vec<String> = mcp_content.iter().map(|block| {
                match block {
                    McpContentBlock::Text { text } => text.clone(),
                    McpContentBlock::Image { mime_type, .. } => format!("[Image: {}]", mime_type),
                    McpContentBlock::Audio { mime_type, .. } => format!("[Audio: {}]", mime_type),
                    McpContentBlock::ResourceLink { uri, title, name } => {
                        let unknown = "Unknown".to_string();
        let display_name = title.as_ref().or(name.as_ref()).unwrap_or(&unknown);
                        format!("[Link to {}: {}]", display_name, uri)
                    }
                    McpContentBlock::Resource { resource } => {
                        if let Some(text) = &resource.text {
                            text.clone()
                        } else {
                            format!("[Embedded Resource: {}]", 
                                resource.mime_type.as_deref().unwrap_or("unknown type"))
                        }
                    }
                }
            }).collect();

            return display_parts.join("\n");
        }
    }

    // 如果无法解析MCP内容，返回JSON格式的原始响应
    format!("```json\n{}\n```", serde_json::to_string_pretty(raw_response).unwrap_or_else(|_| "{}".to_string()))
}

/// 生成有效的工具名称
/// 替换无效字符（基于Gemini API的400错误消息）为下划线
/// 
/// # Arguments
/// * `name` - 原始名称
/// 
/// # Returns
/// 有效的工具名称
pub fn generate_valid_name(name: &str) -> String {
    // 替换无效字符为下划线
    let mut valid_toolname = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    // 如果长度超过63个字符，用'___'替换中间部分
    // (Gemini API说最大长度64，但实际限制似乎是63)
    if valid_toolname.len() > 63 {
        valid_toolname = format!(
            "{}___{}",
            &valid_toolname[..28],
            &valid_toolname[valid_toolname.len() - 32..]
        );
    }

    valid_toolname
}

/// MCP工具工厂
pub struct McpToolFactory;

impl McpToolFactory {
    /// 创建发现的MCP工具
    pub fn create_discovered_tool(
        mcp_tool: Arc<dyn CallableTool + Send + Sync>,
        server_name: String,
        server_tool_name: String,
        description: String,
        parameter_schema: serde_json::Value,
        timeout: Option<u64>,
        trust: Option<bool>,
    ) -> DiscoveredMcpTool {
        DiscoveredMcpTool::new(
            mcp_tool,
            server_name,
            server_tool_name,
            description,
            parameter_schema,
            timeout,
            trust,
            None,
        )
    }

    /// 创建完全限定的MCP工具
    pub fn create_fully_qualified_tool(
        mcp_tool: Arc<dyn CallableTool + Send + Sync>,
        server_name: String,
        server_tool_name: String,
        description: String,
        parameter_schema: serde_json::Value,
        timeout: Option<u64>,
        trust: Option<bool>,
    ) -> DiscoveredMcpTool {
        let tool = DiscoveredMcpTool::new(
            mcp_tool,
            server_name,
            server_tool_name,
            description,
            parameter_schema,
            timeout,
            trust,
            None,
        );
        tool.as_fully_qualified_tool()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // 模拟可调用工具
    struct MockCallableTool;

    #[async_trait]
    impl CallableTool for MockCallableTool {
        async fn call_tool(&self, _function_calls: Vec<FunctionCall>) -> Result<Vec<Part>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![Part {
                text: Some("Mock response".to_string()),
                inline_data: None,
                function_response: None,
            }])
        }
    }

    #[test]
    fn test_generate_valid_name() {
        assert_eq!(generate_valid_name("test-tool"), "test-tool");
        assert_eq!(generate_valid_name("test tool"), "test_tool");
        assert_eq!(generate_valid_name("test@tool#"), "test_tool_");
    }

    #[test]
    fn test_discovered_mcp_tool_creation() {
        let mock_tool = Arc::new(MockCallableTool);
        let tool = McpToolFactory::create_discovered_tool(
            mock_tool,
            "test_server".to_string(),
            "test_tool".to_string(),
            "Test tool description".to_string(),
            serde_json::json!({"type": "object"}),
            Some(30000),
            Some(false),
        );

        assert_eq!(tool.server_name(), "test_server");
        assert_eq!(tool.server_tool_name(), "test_tool");
    }

    #[test]
    fn test_transform_text_block() {
        let block = McpContentBlock::Text { text: "Hello world".to_string() };
        let part = transform_text_block(&block);
        assert!(part.is_some());
        assert_eq!(part.unwrap().text, Some("Hello world".to_string()));
    }

    #[test]
    fn test_get_stringified_result_for_display() {
        let parts = vec![Part {
            text: Some("Test response".to_string()),
            inline_data: None,
            function_response: None,
        }];

        let result = get_stringified_result_for_display(&parts);
        assert!(result.contains("Test response"));
    }
}
