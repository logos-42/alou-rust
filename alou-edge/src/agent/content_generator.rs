use crate::agent::ai_client::{AiClient, AiMessage};
use crate::utils::error::Result;

/// 内容生成器，用于生成智能体的名字和prompt
pub struct ContentGenerator {
    ai_client: AiClient,
}

impl ContentGenerator {
    pub fn new(ai_client: AiClient) -> Self {
        Self { ai_client }
    }

    /// 生成智能体名字
    /// 
    /// # Arguments
    /// * `description` - 智能体描述，用于生成名字
    /// 
    /// # Returns
    /// 生成的智能体名字（2-6个字）
    pub async fn generate_name(&self, description: &str) -> Result<String> {
        let prompt = format!(
            "请为以下描述的智能体生成一个简洁、有创意的中文名字（2-6个字）：\n{}\n\n只返回名字，不要其他说明文字。",
            description
        );
        let messages = vec![AiMessage::text("user", prompt)];
        let response = self.ai_client.send_message(messages, None).await?;
        // 清理响应内容，提取名字（去除可能的引号、标点等）
        let name = response.content.trim();
        let name = name.trim_matches(|c| c == '"' || c == '\'' || c == '。' || c == '，');
        Ok(name.to_string())
    }

    /// 生成智能体的role_description（prompt）
    /// 
    /// # Arguments
    /// * `description` - 智能体描述，用于生成prompt
    /// * `category` - 可选的类别信息
    /// 
    /// # Returns
    /// 生成的详细角色描述（100-200字）
    pub async fn generate_prompt(&self, description: &str, category: Option<&str>) -> Result<String> {
        let category_part = category
            .map(|c| format!("类别：{}\n", c))
            .unwrap_or_default();
        let prompt = format!(
            "请为以下描述的智能体生成详细且专业的角色描述（role_description），用于定义智能体的行为和能力：\n{}{}\n\n要求：\n1. 清晰描述智能体的主要职责\n2. 说明智能体的特性和优势\n3. 100-200字左右\n4. 只返回角色描述内容，不要其他说明文字。",
            category_part, description
        );
        let messages = vec![AiMessage::text("user", prompt)];
        let response = self.ai_client.send_message(messages, None).await?;
        Ok(response.content.trim().to_string())
    }

    // 图像生成功能暂不实现
    // 如需要在后续版本中添加，可以考虑：
    // - OpenAI DALL-E API
    // - Cloudflare Workers AI (Stable Diffusion)
    // - 其他图像生成服务
}

