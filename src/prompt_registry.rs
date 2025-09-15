use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// MCP提示结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<serde_json::Value>>,
}

/// 获取提示结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    pub description: Option<String>,
    pub messages: Vec<serde_json::Value>,
}

/// 发现的MCP提示
pub struct DiscoveredMcpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<serde_json::Value>>,
    pub server_name: String,
    pub invoke: Box<dyn Fn(HashMap<String, serde_json::Value>) -> Result<GetPromptResult> + Send + Sync>,
}

impl DiscoveredMcpPrompt {
    pub fn new(
        name: String,
        description: Option<String>,
        arguments: Option<Vec<serde_json::Value>>,
        server_name: String,
        invoke: Box<dyn Fn(HashMap<String, serde_json::Value>) -> Result<GetPromptResult> + Send + Sync>,
    ) -> Self {
        Self {
            name,
            description,
            arguments,
            server_name,
            invoke,
        }
    }
}

impl Clone for DiscoveredMcpPrompt {
    fn clone(&self) -> Self {
        // 由于invoke函数无法克隆，我们创建一个默认的实现
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            arguments: self.arguments.clone(),
            server_name: self.server_name.clone(),
            invoke: Box::new(|_params: HashMap<String, serde_json::Value>| {
                Ok(GetPromptResult {
                    description: Some("Cloned prompt".to_string()),
                    messages: vec![],
                })
            }),
        }
    }
}

/// 提示注册表
#[derive(Clone)]
pub struct PromptRegistry {
    prompts: HashMap<String, DiscoveredMcpPrompt>,
}

impl PromptRegistry {
    /// 创建新的提示注册表
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
        }
    }

    /// 注册提示定义
    /// 
    /// # Arguments
    /// * `prompt` - 包含模式和调用函数的提示对象
    pub fn register_prompt(&mut self, prompt: DiscoveredMcpPrompt) {
        if self.prompts.contains_key(&prompt.name) {
            let new_name = format!("{}_{}", prompt.server_name, prompt.name);
            tracing::warn!(
                "Prompt with name \"{}\" is already registered. Renaming to \"{}\".",
                prompt.name, new_name
            );
            let mut renamed_prompt = prompt;
            renamed_prompt.name = new_name.clone();
            self.prompts.insert(new_name, renamed_prompt);
        } else {
            self.prompts.insert(prompt.name.clone(), prompt);
        }
    }

    /// 移除特定MCP服务器的所有提示
    /// 
    /// # Arguments
    /// * `server_name` - 要移除提示的服务器名称
    pub fn remove_mcp_prompts_by_server(&mut self, server_name: &str) {
        self.prompts.retain(|_, prompt| prompt.server_name != server_name);
    }

    /// 获取特定提示的定义
    /// 
    /// # Arguments
    /// * `name` - 提示名称
    /// 
    /// # Returns
    /// 提示定义，如果不存在则返回None
    pub fn get_prompt(&self, name: &str) -> Option<&DiscoveredMcpPrompt> {
        self.prompts.get(name)
    }

    /// 返回所有已注册提示实例的数组
    /// 
    /// # Returns
    /// 按名称排序的提示数组
    pub fn get_all_prompts(&self) -> Vec<&DiscoveredMcpPrompt> {
        let mut prompts: Vec<&DiscoveredMcpPrompt> = self.prompts.values().collect();
        prompts.sort_by(|a, b| a.name.cmp(&b.name));
        prompts
    }

    /// 返回从特定MCP服务器注册的提示数组
    /// 
    /// # Arguments
    /// * `server_name` - 服务器名称
    /// 
    /// # Returns
    /// 按名称排序的服务器提示数组
    pub fn get_prompts_by_server(&self, server_name: &str) -> Vec<&DiscoveredMcpPrompt> {
        let mut server_prompts: Vec<&DiscoveredMcpPrompt> = self
            .prompts
            .values()
            .filter(|prompt| prompt.server_name == server_name)
            .collect();
        server_prompts.sort_by(|a, b| a.name.cmp(&b.name));
        server_prompts
    }

    /// 清空注册表中的所有提示
    pub fn clear(&mut self) {
        self.prompts.clear();
    }

    /// 移除特定服务器的所有提示
    /// 
    /// # Arguments
    /// * `server_name` - 服务器名称
    pub fn remove_prompts_by_server(&mut self, server_name: &str) {
        self.prompts.retain(|_, prompt| prompt.server_name != server_name);
    }

    /// 获取提示数量
    pub fn prompt_count(&self) -> usize {
        self.prompts.len()
    }

    /// 检查提示是否存在
    pub fn has_prompt(&self, name: &str) -> bool {
        self.prompts.contains_key(name)
    }

    /// 获取所有提示名称
    pub fn get_prompt_names(&self) -> Vec<String> {
        self.prompts.keys().cloned().collect()
    }

    /// 获取服务器列表
    pub fn get_server_names(&self) -> Vec<String> {
        let mut servers: Vec<String> = self
            .prompts
            .values()
            .map(|p| p.server_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        servers.sort();
        servers
    }
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 提示注册表构建器
pub struct PromptRegistryBuilder {
    registry: PromptRegistry,
}

impl PromptRegistryBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            registry: PromptRegistry::new(),
        }
    }

    /// 添加提示
    pub fn add_prompt(mut self, prompt: DiscoveredMcpPrompt) -> Self {
        self.registry.register_prompt(prompt);
        self
    }

    /// 构建提示注册表
    pub fn build(self) -> PromptRegistry {
        self.registry
    }
}

impl Default for PromptRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 提示工厂
pub struct PromptFactory;

impl PromptFactory {
    /// 创建发现的MCP提示
    pub fn create_discovered_prompt(
        name: String,
        description: Option<String>,
        arguments: Option<Vec<serde_json::Value>>,
        server_name: String,
        invoke: Box<dyn Fn(HashMap<String, serde_json::Value>) -> Result<GetPromptResult> + Send + Sync>,
    ) -> DiscoveredMcpPrompt {
        DiscoveredMcpPrompt::new(name, description, arguments, server_name, invoke)
    }

    /// 创建简单的提示
    pub fn create_simple_prompt(
        name: String,
        description: String,
        server_name: String,
    ) -> DiscoveredMcpPrompt {
        let name_clone = name.clone();
        let description_clone = description.clone();
        let invoke = Box::new(move |_params: HashMap<String, serde_json::Value>| {
            Ok(GetPromptResult {
                description: Some(description_clone.clone()),
                messages: vec![serde_json::json!({
                    "role": "user",
                    "content": format!("执行提示: {}", name_clone)
                })],
            })
        });

        DiscoveredMcpPrompt::new(
            name,
            Some(description),
            None,
            server_name,
            invoke,
        )
    }
}

/// 提示工具函数
pub struct PromptUtils;

impl PromptUtils {
    /// 验证提示名称
    pub fn validate_prompt_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(anyhow::anyhow!("提示名称不能为空"));
        }
        
        if name.len() > 100 {
            return Err(anyhow::anyhow!("提示名称长度不能超过100个字符"));
        }
        
        // 检查是否包含无效字符
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
            return Err(anyhow::anyhow!("提示名称包含无效字符"));
        }
        
        Ok(())
    }

    /// 生成有效的提示名称
    pub fn generate_valid_name(name: &str) -> String {
        let mut valid_name = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        // 如果长度超过100个字符，截断
        if valid_name.len() > 100 {
            valid_name.truncate(100);
        }

        valid_name
    }

    /// 合并提示参数
    pub fn merge_arguments(
        base: Option<Vec<serde_json::Value>>,
        override_args: Option<Vec<serde_json::Value>>,
    ) -> Option<Vec<serde_json::Value>> {
        match (base, override_args) {
            (Some(mut base_args), Some(override_args)) => {
                base_args.extend(override_args);
                Some(base_args)
            }
            (Some(base_args), None) => Some(base_args),
            (None, Some(override_args)) => Some(override_args),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_registry_creation() {
        let registry = PromptRegistry::new();
        assert_eq!(registry.prompt_count(), 0);
    }

    #[test]
    fn test_register_prompt() {
        let mut registry = PromptRegistry::new();
        let prompt = PromptFactory::create_simple_prompt(
            "test_prompt".to_string(),
            "Test prompt".to_string(),
            "test_server".to_string(),
        );
        
        registry.register_prompt(prompt);
        assert_eq!(registry.prompt_count(), 1);
        assert!(registry.has_prompt("test_prompt"));
    }

    #[test]
    fn test_get_prompt() {
        let mut registry = PromptRegistry::new();
        let prompt = PromptFactory::create_simple_prompt(
            "test_prompt".to_string(),
            "Test prompt".to_string(),
            "test_server".to_string(),
        );
        
        registry.register_prompt(prompt);
        
        let retrieved_prompt = registry.get_prompt("test_prompt");
        assert!(retrieved_prompt.is_some());
        assert_eq!(retrieved_prompt.unwrap().name, "test_prompt");
    }

    #[test]
    fn test_remove_prompts_by_server() {
        let mut registry = PromptRegistry::new();
        let prompt1 = PromptFactory::create_simple_prompt(
            "prompt1".to_string(),
            "Prompt 1".to_string(),
            "server1".to_string(),
        );
        let prompt2 = PromptFactory::create_simple_prompt(
            "prompt2".to_string(),
            "Prompt 2".to_string(),
            "server2".to_string(),
        );
        
        registry.register_prompt(prompt1);
        registry.register_prompt(prompt2);
        
        assert_eq!(registry.prompt_count(), 2);
        
        registry.remove_prompts_by_server("server1");
        assert_eq!(registry.prompt_count(), 1);
        assert!(!registry.has_prompt("prompt1"));
        assert!(registry.has_prompt("prompt2"));
    }

    #[test]
    fn test_prompt_registry_builder() {
        let registry = PromptRegistryBuilder::new()
            .add_prompt(PromptFactory::create_simple_prompt(
                "test_prompt".to_string(),
                "Test prompt".to_string(),
                "test_server".to_string(),
            ))
            .build();
        
        assert_eq!(registry.prompt_count(), 1);
    }

    #[test]
    fn test_prompt_utils() {
        assert!(PromptUtils::validate_prompt_name("valid_name").is_ok());
        assert!(PromptUtils::validate_prompt_name("").is_err());
        assert!(PromptUtils::validate_prompt_name("name with spaces").is_err());
        
        assert_eq!(PromptUtils::generate_valid_name("valid-name"), "valid-name");
        assert_eq!(PromptUtils::generate_valid_name("name with spaces"), "name_with_spaces");
    }
}
