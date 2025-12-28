//! Agent creation command parser
//! 智能体创建指令解析器

use crate::ts_agent::claude_agent::{ClaudeAgent, AgentQueryOptions};
use crate::utils::error::Result;
use serde::{Deserialize, Serialize};
use std::env;

/// Parsed agent creation information
/// 解析后的智能体创建信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedAgentInfo {
    pub name: String,
    pub role_description: String,
}

/// Parse agent creation command using AI
/// 使用AI解析智能体创建指令
pub async fn parse_creation_command(command: &str) -> Result<ParsedAgentInfo> {
    // 获取DeepSeek API Key
    let api_key = env::var("DEEPSEEK_API_KEY")
        .or_else(|_| env::var("CLAUDE_API_KEY"))
        .map_err(|_| "No API key found for command parsing")?;

    let cleaned_command = clean_command(command);
    
    // 如果指令为空，返回默认值
    if cleaned_command.is_empty() {
        return Ok(ParsedAgentInfo {
            name: format!("智能体_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default().abs().to_string().chars().rev().take(6).collect::<String>()),
            role_description: "这是一个自动创建的智能体，可以帮助您处理各种任务。".to_string(),
        });
    }

    // 构建AI提示词
    let prompt = build_creation_prompt(&cleaned_command);
    
    // 创建ClaudeAgent实例并查询
    let agent = ClaudeAgent::new(api_key);
    
    let result = agent.query(AgentQueryOptions {
        prompt,
        systemPrompt: Some("你是一个智能体创建助手，负责解析用户指令并生成智能体信息。请严格按照要求返回JSON格式的结果。".to_string()),
        model: Some("deepseek-chat".to_string()),
        maxTokens: Some(500),
        temperature: Some(0.7),
        ..Default::default()
    }).await.map_err(|e| format!("AI query failed: {}", e))?;

    // 解析AI返回的结果
    parse_ai_response(&result.response)
}

/// Clean the creation command by removing keywords
/// 清理创建指令，移除关键词
fn clean_command(command: &str) -> String {
    let create_keywords = [
        "创建智能体", "新建智能体", "create agent", "new agent", 
        "/create", "/new", "创建", "新建", "agent"
    ];
    
    let mut cleaned = command.to_lowercase().trim().to_string();
    
    for keyword in &create_keywords {
        cleaned = cleaned.replace(&keyword.to_lowercase(), "").trim().to_string();
    }
    
    cleaned
}

/// Build the AI prompt for agent creation
/// 构建智能体创建的AI提示词
fn build_creation_prompt(command: &str) -> String {
    format!(
        r#"用户想要创建一个智能体，指令是："{}"

请解析这个指令并生成智能体信息：
1. 智能体名字（2-6个中文字符，有创意且相关）
2. 角色描述（100-200字，描述智能体的职责和能力）

请以JSON格式返回，格式如下：
{{
  "name": "智能体名字",
  "roleDescription": "详细的角色描述"
}}

只返回JSON，不要其他内容。"#,
        command
    )
}

/// Parse AI response to extract agent information
/// 解析AI响应以提取智能体信息
fn parse_ai_response(response: &str) -> Result<ParsedAgentInfo> {
    // 尝试直接解析JSON
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
        if let (Some(name), Some(role_desc)) = (
            parsed.get("name").and_then(|v| v.as_str()),
            parsed.get("roleDescription").and_then(|v| v.as_str())
        ) {
            return Ok(ParsedAgentInfo {
                name: name.to_string(),
                role_description: role_desc.to_string(),
            });
        }
    }

    // 如果直接JSON解析失败，尝试从文本中提取信息
    extract_from_text(response)
}

/// Extract agent information from text response
/// 从文本响应中提取智能体信息
fn extract_from_text(text: &str) -> Result<ParsedAgentInfo> {
    let lines: Vec<&str> = text.lines().collect();
    let mut name = None;
    let mut role_description = None;
    
    for line in lines {
        let trimmed = line.trim();
        
        // 尝试提取名字
        if name.is_none() && (trimmed.contains("名字") || trimmed.contains("name") || trimmed.contains("：")) {
            if let Some(captures) = regex::Regex::new(r"[：:]\s*(.+)").unwrap().captures(trimmed) {
                if let Some(name_match) = captures.get(1) {
                    let extracted_name = name_match.as_str().trim().replace(['"', '\''], "");
                    if !extracted_name.is_empty() && extracted_name.len() <= 10 {
                        name = Some(extracted_name);
                    }
                }
            }
        }
        
        // 尝试提取角色描述（长度较长的行）
        if role_description.is_none() && trimmed.len() > 20 && !trimmed.contains('{') && !trimmed.contains('}') {
            role_description = Some(trimmed.to_string());
        }
    }
    
    // 如果仍然没有找到信息，使用默认值
    let name = name.unwrap_or_else(|| {
        format!("智能体_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default().abs().to_string().chars().rev().take(6).collect::<String>())
    });
    
    let role_description = role_description.unwrap_or_else(|| {
        "这是一个自动创建的智能体，可以帮助您处理各种任务。".to_string()
    });
    
    Ok(ParsedAgentInfo {
        name,
        role_description,
    })
}