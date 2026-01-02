use serde::{Deserialize, Serialize};

/// Parse result for agent creation command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationParseResult {
    pub name: String,
    pub role_description: String,
}

/// Parse agent creation command using AI
pub async fn parse_creation_command(command: &str) -> Result<CreationParseResult, String> {
    // Simple parsing logic for now - can be enhanced with AI later
    let command_lower = command.to_lowercase();
    
    // Extract name (find "name:" or just use first word)
    let name = if let Some(idx) = command_lower.find("name:") {
        let after_name = &command[idx + 5..];
        let name_end = after_name.find(',').or_else(|| after_name.find('\n'))
            .unwrap_or(after_name.len());
        after_name[..name_end].trim().to_string()
    } else {
        // Extract first meaningful word as name
        command
            .split_whitespace()
            .find(|w| w.len() > 2)
            .unwrap_or("智能体")
            .to_string()
    };

    // Extract role description
    let role_description = if let Some(idx) = command_lower.find("描述:") {
        Some(idx)
    } else if let Some(idx) = command_lower.find("description:") {
        Some(idx)
    } else {
        command_lower.find("role:")
    };

    let role_description = if let Some(idx) = role_description {
        let start = idx + 3;
        if let Some(end) = command[start..].find(['.', '!', '\n']) {
            command[start..start + end].trim().to_string()
        } else {
            command[start..].trim().to_string()
        }
    } else {
        format!("一个名为{}的智能助手", name)
    };

    Ok(CreationParseResult {
        name: if name.is_empty() { "智能体".to_string() } else { name },
        role_description: if role_description.is_empty() { 
            "一个智能助手".to_string() 
        } else { 
            role_description 
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_command() {
        let result = parse_creation_command("创建一个客服机器人").await;
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.name, "客服");
    }

    #[tokio::test]
    async fn test_parse_with_name() {
        let result = parse_creation_command("name: 助手 描述: 帮助用户解决问题").await;
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.name, "助手");
        assert!(parsed.role_description.contains("帮助"));
    }
}
