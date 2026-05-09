//! Skills 工具类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 技能定义 (简化版，用于工具接口)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub tags: Vec<String>,
    pub enabled: bool,
}

impl From<crate::skills::SkillManifest> for SkillInfo {
    fn from(manifest: crate::skills::SkillManifest) -> Self {
        Self {
            id: manifest.id,
            display_name: manifest.display_name,
            description: manifest.description,
            category: manifest.category.as_str().to_string(),
            version: manifest.version,
            tags: manifest.tags,
            enabled: manifest.enabled,
        }
    }
}

/// 创建技能请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkillRequest {
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub persona: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub constraints: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// 执行技能请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteSkillRequest {
    pub skill_id: String,
    pub inputs: HashMap<String, serde_json::Value>,
}

/// 搜索技能请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSkillsRequest {
    pub query: String,
    pub category: Option<String>,
}

/// 技能执行响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResponse {
    pub success: bool,
    pub output: serde_json::Value,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

impl From<crate::skills::executor::SkillExecutionResult> for SkillExecutionResponse {
    fn from(result: crate::skills::executor::SkillExecutionResult) -> Self {
        Self {
            success: result.success,
            output: result.output,
            execution_time_ms: result.execution_time_ms,
            error: result.error,
        }
    }
}
