//! Skills自动选择器 Tauri命令工具
//!
//! 提供AI自动选择工具的接口

use crate::tools::skill_auto_selector::{SkillAutoSelector, AutoSelectConfig};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 自动选择器工具状态
pub struct SkillAutoSelectorTool {
    selector: Arc<Mutex<SkillAutoSelector>>,
    config: AutoSelectConfig,
}

impl SkillAutoSelectorTool {
    /// 创建新的自动选择器工具
    pub fn new() -> Self {
        Self {
            selector: Arc::new(Mutex::new(SkillAutoSelector::new())),
            config: AutoSelectConfig::default(),
        }
    }

    /// 获取选择器实例
    pub fn get_selector(&self) -> Arc<Mutex<SkillAutoSelector>> {
        self.selector.clone()
    }
}

/// 分析消息并选择工具参数
#[derive(Serialize, Deserialize)]
pub struct AnalyzeParams {
    /// 用户消息
    message: String,
    /// 最大选择数量 (可选)
    max_selections: Option<usize>,
    /// 最小置信度 (可选)
    min_confidence: Option<f64>,
}

/// 分析结果
#[derive(Serialize, Deserialize)]
pub struct AnalyzeResult {
    /// 是否成功
    success: bool,
    /// 匹配的工具列表
    tools: Vec<ToolMatchResult>,
    /// 主要意图
    primary_intent: Option<String>,
    /// 总工具数
    total_count: usize,
    /// 消息
    message: String,
}

/// 单个工具匹配结果
#[derive(Serialize, Deserialize)]
pub struct ToolMatchResult {
    /// 工具ID
    tool_id: String,
    /// 工具名称
    name: String,
    /// 匹配置信度
    confidence: f64,
    /// 匹配原因
    reason: String,
}

/// 配置参数
#[derive(Serialize, Deserialize)]
pub struct ConfigParams {
    /// 是否启用
    enabled: Option<bool>,
    /// 最小置信度
    min_confidence: Option<f64>,
    /// 最大选择数
    max_selections: Option<usize>,
    /// 允许工具链
    allow_tool_chain: Option<bool>,
}

/// Tauri命令：分析消息选择工具
#[tauri::command]
pub async fn analyze_and_select_tools(
    message: String,
    max_selections: Option<usize>,
    min_confidence: Option<f64>,
) -> Result<AnalyzeResult, String> {
    let mut selector = SkillAutoSelector::new();
    
    // 临时调整配置
    if let Some(max) = max_selections {
        selector.config.max_selections = max;
    }
    if let Some(min) = min_confidence {
        selector.config.min_confidence = min;
    }

    // 分析消息
    let matches = selector.analyze_and_select(&message);

    // 构建结果
    let tools_count = matches.len();
    let primary_intent = matches.first().map(|m| m.tool_id.clone());
    let primary_tool_name = matches.first().map(|m| m.tool_id.clone()).unwrap_or_default();

    let tools: Vec<ToolMatchResult> = matches.into_iter().map(|m| ToolMatchResult {
        tool_id: m.tool_id,
        name: m.name,
        confidence: m.confidence,
        reason: m.reason,
    }).collect();

    Ok(AnalyzeResult {
        success: !tools.is_empty(),
        tools,
        primary_intent,
        total_count: tools_count,
        message: if tools_count == 0 {
            "未检测到需要使用的工具".to_string()
        } else {
            format!("建议使用 {} 个工具，主要意图: {}", tools_count, primary_tool_name)
        },
    })
}

/// Tauri命令：生成执行计划
#[tauri::command]
pub async fn generate_execution_plan(message: String) -> Result<serde_json::Value, String> {
    let selector = SkillAutoSelector::new();
    let plan = selector.generate_execution_plan(&message);
    Ok(plan)
}

/// Tauri命令：获取可用工具列表
#[tauri::command]
pub async fn get_available_tools() -> Result<serde_json::Value, String> {
    let selector = SkillAutoSelector::new();
    
    let tools: Vec<serde_json::Value> = selector.tool_keywords.iter().map(|(id, keywords)| {
        json!({
            "id": id,
            "description": selector.tool_descriptions.get(id).unwrap_or(&id.clone()),
            "keywords": keywords,
            "count": keywords.len()
        })
    }).collect();

    Ok(json!({
        "success": true,
        "tools": tools,
        "total": tools.len()
    }))
}

/// Tauri命令：更新配置
#[tauri::command]
pub async fn update_auto_select_config(
    enabled: Option<bool>,
    min_confidence: Option<f64>,
    max_selections: Option<usize>,
    allow_tool_chain: Option<bool>,
) -> Result<serde_json::Value, String> {
    let mut config = AutoSelectConfig::default();
    
    if let Some(e) = enabled {
        config.enabled = e;
    }
    if let Some(mc) = min_confidence {
        config.min_confidence = mc;
    }
    if let Some(ms) = max_selections {
        config.max_selections = ms;
    }
    if let Some(ac) = allow_tool_chain {
        config.allow_tool_chain = ac;
    }

    Ok(json!({
        "success": true,
        "config": {
            "enabled": config.enabled,
            "min_confidence": config.min_confidence,
            "max_selections": config.max_selections,
            "allow_tool_chain": config.allow_tool_chain
        },
        "message": "配置更新成功"
    }))
}

/// Tauri命令：获取当前配置
#[tauri::command]
pub async fn get_auto_select_config() -> Result<serde_json::Value, String> {
    let config = AutoSelectConfig::default();
    
    Ok(json!({
        "success": true,
        "config": {
            "enabled": config.enabled,
            "min_confidence": config.min_confidence,
            "max_selections": config.max_selections,
            "allow_tool_chain": config.allow_tool_chain
        }
    }))
}

/// 初始化自动选择器工具
pub async fn initialize_skill_auto_selector_tool() -> Result<Arc<Mutex<SkillAutoSelectorTool>>, Box<dyn std::error::Error>> {
    Ok(Arc::new(Mutex::new(SkillAutoSelectorTool::new())))
}
