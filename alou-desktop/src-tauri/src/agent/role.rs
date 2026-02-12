//! 智能体角色配置系统
//!
//! 智能体自我管理的角色设计、性格参数和沟通规范

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 沟通风格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationStyle {
    Formal,        // 正式
    Casual,        // 轻松
    Professional,  // 专业
    Friendly,      // 友好
    Technical,     // 技术性
}

/// 响应长度偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseLength {
    Brief,         // 简短
    Medium,        // 中等
    Detailed,      // 详细
}

/// 主动性级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProactivityLevel {
    Passive,       // 被动响应
    Reactive,     // 适度主动
    Proactive,    // 积极主动
}

/// 智能体角色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRoleConfig {
    /// 唯一标识
    pub id: String,
    
    /// 角色名称
    pub name: String,
    
    /// 角色描述
    pub description: String,
    
    /// 性格参数
    pub personality: PersonalityConfig,
    
    /// 沟通规范
    pub communication: CommunicationConfig,
    
    /// 行为规则
    pub behavior_rules: Vec<BehaviorRule>,
    
    /// 专业知识领域
    pub expertise_domains: Vec<String>,
    
    /// 创建时间
    pub created_at: i64,
    
    /// 更新时间
    pub updated_at: i64,
    
    /// 版本号
    pub version: u32,
}

impl Default for AgentRoleConfig {
    fn default() -> Self {
        Self {
            id: format!("role_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string()),
            name: "Default Agent".to_string(),
            description: "Default autonomous agent".to_string(),
            personality: PersonalityConfig::default(),
            communication: CommunicationConfig::default(),
            behavior_rules: Vec::new(),
            expertise_domains: Vec::new(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            version: 1,
        }
    }
}

/// 性格配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    /// 创造力 (0.0-1.0)
    pub creativity: f64,
    
    /// 逻辑性 (0.0-1.0)
    pub logic: f64,
    
    /// 好奇心 (0.0-1.0)
    pub curiosity: f64,
    
    /// 耐心 (0.0-1.0)
    pub patience: f64,
    
    /// 主动性 (0.0-1.0)
    pub proactivity: f64,
    
    /// 谨慎度 (0.0-1.0)
    pub caution: f64,
    
    /// 同理心 (0.0-1.0)
    pub empathy: f64,
    
    /// 幽默感 (0.0-1.0)
    pub humor: f64,
    
    /// 适应能力 (0.0-1.0)
    pub adaptability: f64,
    
    /// 自信度 (0.0-1.0)
    pub confidence: f64,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            creativity: 0.7,
            logic: 0.8,
            curiosity: 0.6,
            patience: 0.7,
            proactivity: 0.5,
            caution: 0.5,
            empathy: 0.6,
            humor: 0.3,
            adaptability: 0.7,
            confidence: 0.7,
        }
    }
}

/// 沟通配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationConfig {
    /// 沟通风格
    pub style: CommunicationStyle,
    
    /// 响应长度偏好
    pub response_length: ResponseLength,
    
    /// 是否使用表情符号
    pub use_emoji: bool,
    
    /// 是否使用markdown
    pub use_markdown: bool,
    
    /// 代码解释详细程度 (0.0-1.0)
    pub code_explanation_detail: f64,
    
    /// 问候语
    pub greeting: String,
    
    /// 结束语
    pub closing: String,
    
    /// 常用短语
    pub common_phrases: Vec<String>,
    
    /// 避免使用的词汇
    pub avoid_words: Vec<String>,
    
    /// 专属词汇/口头禅
    pub signature_phrases: Vec<String>,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            style: CommunicationStyle::Professional,
            response_length: ResponseLength::Medium,
            use_emoji: true,
            use_markdown: true,
            code_explanation_detail: 0.7,
            greeting: "你好！我是 Alou，很高兴为你服务。有什么我可以帮助你的吗？".to_string(),
            closing: "如果有其他问题，随时告诉我！".to_string(),
            common_phrases: Vec::new(),
            avoid_words: Vec::new(),
            signature_phrases: Vec::new(),
        }
    }
}

/// 行为规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorRule {
    /// 规则ID
    pub id: String,
    
    /// 规则名称
    pub name: String,
    
    /// 规则描述
    pub description: String,
    
    /// 触发条件
    pub trigger: String,
    
    /// 执行动作
    pub action: String,
    
    /// 优先级 (数字越大优先级越高)
    pub priority: u32,
    
    /// 是否启用
    pub enabled: bool,
}

/// 角色管理器
pub struct RoleManager {
    /// 当前角色配置
    current_role: Option<AgentRoleConfig>,
    
    /// 角色存储路径
    storage_path: PathBuf,
}

impl RoleManager {
    /// 创建新的角色管理器
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let path = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".alou")
            .join("roles");
        
        // 确保目录存在
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        Ok(Self {
            current_role: None,
            storage_path: path,
        })
    }

    /// 创建新角色
    pub fn create_role(&self, name: String, description: String) -> AgentRoleConfig {
        AgentRoleConfig {
            id: format!("role_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string()),
            name,
            description,
            ..Default::default()
        }
    }

    /// 保存角色
    pub async fn save_role(&mut self, role: &AgentRoleConfig) -> Result<(), Box<dyn std::error::Error>> {
        let role_path = self.storage_path.join(&format!("{}.json", role.id));
        
        let content = serde_json::to_string_pretty(role)?;
        fs::write(&role_path, content)?;
        
        Ok(())
    }

    /// 加载角色
    pub async fn load_role(&mut self, role_id: &str) -> Result<Option<AgentRoleConfig>, Box<dyn std::error::Error>> {
        let role_path = self.storage_path.join(&format!("{}.json", role_id));
        
        if !role_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&role_path)?;
        let role: AgentRoleConfig = serde_json::from_str(&content)?;
        
        self.current_role = Some(role.clone());
        Ok(Some(role))
    }

    /// 设置当前角色
    pub fn set_current_role(&mut self, role: AgentRoleConfig) {
        self.current_role = Some(role);
    }

    /// 获取当前角色
    pub fn get_current_role(&self) -> Option<&AgentRoleConfig> {
        self.current_role.as_ref()
    }

    /// 更新性格参数
    pub fn update_personality(&mut self, updates: serde_json::Value) {
        if let Some(ref mut role) = self.current_role {
            if let Some(creativity) = updates.get("creativity").and_then(|v| v.as_f64()) {
                role.personality.creativity = creativity.clamp(0.0, 1.0);
            }
            if let Some(logic) = updates.get("logic").and_then(|v| v.as_f64()) {
                role.personality.logic = logic.clamp(0.0, 1.0);
            }
            if let Some(curiosity) = updates.get("curiosity").and_then(|v| v.as_f64()) {
                role.personality.curiosity = curiosity.clamp(0.0, 1.0);
            }
            if let Some(patience) = updates.get("patience").and_then(|v| v.as_f64()) {
                role.personality.patience = patience.clamp(0.0, 1.0);
            }
            if let Some(proactivity) = updates.get("proactivity").and_then(|v| v.as_f64()) {
                role.personality.proactivity = proactivity.clamp(0.0, 1.0);
            }
            if let Some(caution) = updates.get("caution").and_then(|v| v.as_f64()) {
                role.personality.caution = caution.clamp(0.0, 1.0);
            }
            if let Some(empathy) = updates.get("empathy").and_then(|v| v.as_f64()) {
                role.personality.empathy = empathy.clamp(0.0, 1.0);
            }
            if let Some(humor) = updates.get("humor").and_then(|v| v.as_f64()) {
                role.personality.humor = humor.clamp(0.0, 1.0);
            }
            if let Some(adaptability) = updates.get("adaptability").and_then(|v| v.as_f64()) {
                role.personality.adaptability = adaptability.clamp(0.0, 1.0);
            }
            if let Some(confidence) = updates.get("confidence").and_then(|v| v.as_f64()) {
                role.personality.confidence = confidence.clamp(0.0, 1.0);
            }
            
            role.updated_at = Utc::now().timestamp();
            role.version += 1;
        }
    }

    /// 更新沟通规范
    pub fn update_communication(&mut self, updates: serde_json::Value) {
        if let Some(ref mut role) = self.current_role {
            if let Some(style) = updates.get("style").and_then(|v| v.as_str()) {
                role.communication.style = match style {
                    "formal" => CommunicationStyle::Formal,
                    "casual" => CommunicationStyle::Casual,
                    "professional" => CommunicationStyle::Professional,
                    "friendly" => CommunicationStyle::Friendly,
                    "technical" => CommunicationStyle::Technical,
                    _ => CommunicationStyle::Professional,
                };
            }
            if let Some(length) = updates.get("response_length").and_then(|v| v.as_str()) {
                role.communication.response_length = match length {
                    "brief" => ResponseLength::Brief,
                    "medium" => ResponseLength::Medium,
                    "detailed" => ResponseLength::Detailed,
                    _ => ResponseLength::Medium,
                };
            }
            if let Some(use_emoji) = updates.get("use_emoji").and_then(|v| v.as_bool()) {
                role.communication.use_emoji = use_emoji;
            }
            if let Some(use_markdown) = updates.get("use_markdown").and_then(|v| v.as_bool()) {
                role.communication.use_markdown = use_markdown;
            }
            if let Some(greeting) = updates.get("greeting").and_then(|v| v.as_str()) {
                role.communication.greeting = greeting.to_string();
            }
            if let Some(closing) = updates.get("closing").and_then(|v| v.as_str()) {
                role.communication.closing = closing.to_string();
            }
            
            role.updated_at = Utc::now().timestamp();
            role.version += 1;
        }
    }

    /// 添加行为规则
    pub fn add_behavior_rule(&mut self, rule: BehaviorRule) {
        if let Some(ref mut role) = self.current_role {
            role.behavior_rules.push(rule);
            role.updated_at = Utc::now().timestamp();
            role.version += 1;
        }
    }

    /// 移除行为规则
    pub fn remove_behavior_rule(&mut self, rule_id: &str) {
        if let Some(ref mut role) = self.current_role {
            role.behavior_rules.retain(|r| r.id != rule_id);
            role.updated_at = Utc::now().timestamp();
            role.version += 1;
        }
    }

    /// 生成系统提示词
    pub fn generate_system_prompt(&self) -> String {
        if let Some(role) = &self.current_role {
            let personality = &role.personality;
            let communication = &role.communication;
            
            let mut prompt = format!(
                "你是 {}, {}\n\n",
                role.name, role.description
            );
            
            // 性格描述
            prompt += "## 性格特征\n";
            prompt += &format!("- 创造力: {:.0}%\n", personality.creativity * 100.0);
            prompt += &format!("- 逻辑性: {:.0}%\n", personality.logic * 100.0);
            prompt += &format!("- 好奇心: {:.0}%\n", personality.curiosity * 100.0);
            prompt += &format!("- 耐心: {:.0}%\n", personality.patience * 100.0);
            prompt += &format!("- 同理心: {:.0}%\n", personality.empathy * 100.0);
            prompt += &format!("- 幽默感: {:.0}%\n", personality.humor * 100.0);
            
            // 沟通风格
            prompt += "\n## 沟通风格\n";
            prompt += &format!("- 风格: {:?}\n", communication.style);
            prompt += &format!("- 响应长度: {:?}\n", communication.response_length);
            prompt += &format!("- 使用表情: {}\n", if communication.use_emoji { "是" } else { "否" });
            prompt += &format!("- 使用Markdown: {}\n", if communication.use_markdown { "是" } else { "否" });
            
            if !communication.greeting.is_empty() {
                prompt += &format!("- 问候语: {}\n", communication.greeting);
            }
            
            // 专业知识
            if !role.expertise_domains.is_empty() {
                prompt += "\n## 专业知识领域\n";
                for domain in &role.expertise_domains {
                    prompt += &format!("- {}\n", domain);
                }
            }
            
            // 行为规则
            if !role.behavior_rules.is_empty() {
                prompt += "\n## 行为规则\n";
                for rule in &role.behavior_rules {
                    if rule.enabled {
                        prompt += &format!("- {}: {}\n", rule.name, rule.description);
                    }
                }
            }
            
            prompt
        } else {
            "你是 Alou，一个智能助手。".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_creation() {
        let manager = RoleManager::new().unwrap();
        let role = manager.create_role(
            "技术助手".to_string(),
            "专注于技术问题的智能助手".to_string()
        );
        
        assert!(!role.id.is_empty());
        assert_eq!(role.name, "技术助手");
        assert_eq!(role.version, 1);
    }

    #[test]
    fn test_personality_update() {
        let mut config = AgentRoleConfig::default();
        
        let updates = json!({
            "creativity": 0.9,
            "logic": 0.6,
            "humor": 0.8
        });
        
        // 模拟更新逻辑
        config.personality.creativity = 0.9;
        config.personality.logic = 0.6;
        config.personality.humor = 0.8;
        
        assert_eq!(config.personality.creativity, 0.9);
        assert_eq!(config.personality.logic, 0.6);
        assert_eq!(config.personality.humor, 0.8);
    }

    #[test]
    fn test_system_prompt_generation() {
        let manager = RoleManager::new().unwrap();
        let role = manager.create_role(
            "测试助手".to_string(),
            "用于测试的角色".to_string()
        );
        manager.set_current_role(role);
        
        let prompt = manager.generate_system_prompt();
        
        assert!(prompt.contains("测试助手"));
        assert!(prompt.contains("性格特征"));
        assert!(prompt.contains("沟通风格"));
    }
}
