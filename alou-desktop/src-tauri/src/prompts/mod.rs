//! Prompt 管理模块
//!
//! 统一管理所有系统提示词，支持动态生成和缓存

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Prompt 上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptContext {
    /// 迭代次数
    pub iteration: Option<u32>,
    /// 历史摘要
    pub history: Option<String>,
    /// 学习进度
    pub learning: Option<String>,
    /// 当前结果
    pub result: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 调研统计
    pub findings: Option<usize>,
    pub prompts: Option<usize>,
    pub skills: Option<usize>,
}

impl Default for PromptContext {
    fn default() -> Self {
        Self {
            iteration: None,
            history: None,
            learning: None,
            result: None,
            error: None,
            findings: None,
            prompts: None,
            skills: None,
        }
    }
}

/// Prompt 管理器
pub struct PromptManager {
    /// Prompt 模板缓存
    templates: Arc<RwLock<HashMap<String, String>>>,
    /// 生成的 Prompt 缓存
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl PromptManager {
    /// 创建新的 Prompt 管理器
    pub fn new() -> Self {
        let manager = Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // 初始化基础模板（同步方式）
        let templates = manager.templates.clone();
        let cache = manager.cache.clone();
        tokio::spawn(async move {
            let _ = Self::initialize_templates_static(templates, cache).await;
        });

        manager
    }

    /// 静态初始化方法
    async fn initialize_templates_static(
        templates: Arc<RwLock<HashMap<String, String>>>,
        _cache: Arc<RwLock<HashMap<String, String>>>,
    ) {
        let mut templates = templates.write().await;

        // 工作流决策 Prompt
        templates.insert("workflow_decision".to_string(), r#"
你是智能工作流协调器。基于以下信息决策下一步：

迭代: {iteration}
历史摘要: {history}
学习进度: {learning}
当前结果: {result}

决策选项（简洁回复）：
1. COMPLETED - 任务完成
2. RETRY:<原因> - 重试
3. RESEARCH:<查询> - 调研
4. ADJUST:<策略> - 调整
5. CONTINUE - 继续

考虑历史和学习，选择最优行动。
"#.to_string());

        // 错误分析 Prompt
        templates.insert("error_analysis".to_string(), r#"
你是一个错误分析专家。请分析以下工作流执行错误并提供修复建议：

迭代次数: {iteration}
错误信息: {error}

请分析错误的严重程度并提供建议：
1. 如果是临时性错误（网络超时、资源暂不可用等），返回 "RETRYABLE: <描述>"
2. 如果是配置错误或参数问题，返回 "CONFIG_ERROR: <修复建议>"
3. 如果是致命错误（权限、依赖缺失等），返回 "CRITICAL: <描述>"
4. 如果是逻辑错误，返回 "LOGIC_ERROR: <修复建议>"
5. 如果是未知错误，返回 "UNKNOWN: <分析结果>"

请只返回上述格式之一，简洁明了。
"#.to_string());

        // 调研总结 Prompt
        templates.insert("research_summary".to_string(), r#"
简要总结以下调研结果（不超过200字）：

发现: {findings} 个文档
提示: {prompts} 个
技能: {skills} 个

请提供：
1. 关键发现（3-5个要点）
2. 建议行动（1-2个）

格式：简洁要点列表
"#.to_string());
    }
    
    /// 初始化基础 Prompt 模板
    async fn initialize_templates(&self) {
        let mut templates = self.templates.write().await;
        
        // 工作流决策 Prompt
        templates.insert("workflow_decision".to_string(), r#"
你是智能工作流协调器。基于以下信息决策下一步：

迭代: {iteration}
历史摘要: {history}
学习进度: {learning}
当前结果: {result}

决策选项（简洁回复）：
1. COMPLETED - 任务完成
2. RETRY:<原因> - 重试
3. RESEARCH:<查询> - 调研
4. ADJUST:<策略> - 调整
5. CONTINUE - 继续

考虑历史和学习，选择最优行动。
"#.to_string());
        
        // 错误分析 Prompt
        templates.insert("error_analysis".to_string(), r#"
你是一个错误分析专家。请分析以下工作流执行错误并提供修复建议：

迭代次数: {iteration}
错误信息: {error}

请分析错误的严重程度并提供建议：
1. 如果是临时性错误（网络超时、资源暂不可用等），返回 "RETRYABLE: <描述>"
2. 如果是配置错误或参数问题，返回 "CONFIG_ERROR: <修复建议>"
3. 如果是致命错误（权限、依赖缺失等），返回 "CRITICAL: <描述>"
4. 如果是逻辑错误，返回 "LOGIC_ERROR: <修复建议>"
5. 如果是未知错误，返回 "UNKNOWN: <分析结果>"

请只返回上述格式之一，简洁明了。
"#.to_string());
        
        // 调研总结 Prompt
        templates.insert("research_summary".to_string(), r#"
简要总结以下调研结果（不超过200字）：

发现: {findings} 个文档
提示: {prompts} 个
技能: {skills} 个

请提供：
1. 关键发现（3-5个要点）
2. 建议行动（1-2个）

格式：简洁要点列表
"#.to_string());
    }
    
    /// 生成工作流 Prompt
    pub async fn generate_workflow_prompt(&self, prompt_type: &str, context: &PromptContext) -> String {
        let cache_key = format!("workflow_{}_{}", prompt_type, serde_json::to_string(context).unwrap_or_default());
        
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached_prompt) = cache.get(&cache_key) {
                return cached_prompt.clone();
            }
        }
        
        // 生成新 Prompt
        let templates = self.templates.read().await;
        let template = templates.get(prompt_type)
            .map(|s| s.as_str())
            .unwrap_or("Prompt template not found");

        let mut prompt = template.to_string();
        
        // 变量替换
        if let Some(iteration) = context.iteration {
            prompt = prompt.replace("{iteration}", &iteration.to_string());
        }
        
        if let Some(history) = &context.history {
            prompt = prompt.replace("{history}", history);
        }
        
        if let Some(learning) = &context.learning {
            prompt = prompt.replace("{learning}", learning);
        }
        
        if let Some(result) = &context.result {
            prompt = prompt.replace("{result}", result);
        }
        
        if let Some(error) = &context.error {
            prompt = prompt.replace("{error}", error);
        }
        
        if let Some(findings) = context.findings {
            prompt = prompt.replace("{findings}", &findings.to_string());
        }
        
        if let Some(prompts) = context.prompts {
            prompt = prompt.replace("{prompts}", &prompts.to_string());
        }
        
        if let Some(skills) = context.skills {
            prompt = prompt.replace("{skills}", &skills.to_string());
        }
        
        // 缓存结果
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, prompt.clone());
        }
        
        prompt
    }
    
    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
    
    /// 获取模板列表
    pub async fn list_templates(&self) -> Vec<String> {
        let templates = self.templates.read().await;
        templates.keys().cloned().collect()
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局 Prompt 管理器实例
pub static PROMPT_MANAGER: std::sync::LazyLock<PromptManager> = std::sync::LazyLock::new(PromptManager::new);
