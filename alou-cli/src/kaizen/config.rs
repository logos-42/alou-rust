//! Kaizen 配置管理

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

use super::get_kaizen_data_dir;

/// Kaizen 循环配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaizenConfig {
    /// 运行模式
    pub mode: KaizenMode,
    
    /// 最大迭代次数
    pub max_iterations: usize,
    
    /// 是否启用自动提交
    pub auto_commit: bool,
    
    /// 是否启用自动推送
    pub auto_push: bool,
    
    /// 推送间隔（每 N 次迭代）
    pub push_interval: usize,
    
    /// 安全模式（不实际修改文件）
    pub dry_run: bool,
    
    /// 严格模式（测试 100% 通过才接受）
    pub strict: bool,
    
    /// 启用 Web 搜索
    pub enable_web_search: bool,
    
    /// 目标文件列表（自动研究模式）
    pub target_files: Vec<String>,
    
    /// 任务描述（进化引擎模式）
    pub task_description: Option<String>,
    
    /// LLM 提供商
    pub llm_provider: String,
    
    /// LLM 模型
    pub llm_model: String,
    
    /// LLM API Key
    pub llm_api_key: String,
    
    /// LLM Base URL（可选）
    pub llm_base_url: Option<String>,
}

/// Kaizen 运行模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KaizenMode {
    /// 进化引擎模式
    Evolution,
    /// 自动研究模式（Karpathy 风格）
    Research,
}

impl Default for KaizenConfig {
    fn default() -> Self {
        Self {
            mode: KaizenMode::Evolution,
            max_iterations: 5,
            auto_commit: true,
            auto_push: false,
            push_interval: 5,
            dry_run: true, // 默认安全模式
            strict: false,
            enable_web_search: true,
            target_files: vec![],
            task_description: None,
            llm_provider: "openai".to_string(),
            llm_model: "gpt-4o".to_string(),
            llm_api_key: String::new(),
            llm_base_url: None,
        }
    }
}

impl KaizenConfig {
    /// 从文件加载配置
    pub fn load(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: KaizenConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = KaizenConfig::default();
            config.save(path)?;
            Ok(config)
        }
    }
    
    /// 保存配置到文件
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// 从环境变量加载配置（优先级高于文件配置）
    pub fn load_from_env() -> Self {
        let mut config = KaizenConfig::default();
        
        // 从环境变量读取
        if let Ok(iterations) = std::env::var("KAIZEN_ITERATIONS") {
            if let Ok(n) = iterations.parse::<usize>() {
                config.max_iterations = n;
            }
        }
        
        if let Ok(dry_run) = std::env::var("KAIZEN_DRY_RUN") {
            config.dry_run = dry_run.to_lowercase() == "true";
        }
        
        if let Ok(strict) = std::env::var("KAIZEN_STRICT") {
            config.strict = strict.to_lowercase() == "true";
        }
        
        if let Ok(auto_push) = std::env::var("KAIZEN_AUTO_PUSH") {
            config.auto_push = auto_push.to_lowercase() == "true";
        }
        
        if let Ok(web_search) = std::env::var("KAIZEN_WEB") {
            config.enable_web_search = web_search.to_lowercase() == "true";
        }
        
        // LLM 配置
        if let Ok(provider) = std::env::var("LLM_PROVIDER") {
            config.llm_provider = provider;
        }
        
        if let Ok(model) = std::env::var("LLM_MODEL") {
            config.llm_model = model;
        }
        
        if let Ok(api_key) = std::env::var("LLM_API_KEY") {
            config.llm_api_key = api_key;
        }
        
        if let Ok(base_url) = std::env::var("LLM_BASE_URL") {
            config.llm_base_url = Some(base_url);
        }
        
        config
    }
    
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let data_dir = get_kaizen_data_dir()?;
        Ok(data_dir.join("config.json"))
    }
}
