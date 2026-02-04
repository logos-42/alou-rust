//! 工具定义模块
//! 
//! 包含工具相关的数据结构定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工具类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Rust,
    Python,
    JavaScript,
    Shell,
    Custom,
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    /// 参数名
    pub name: String,
    /// 参数类型
    pub param_type: String,
    /// 是否必需
    pub required: bool,
    /// 描述
    pub description: String,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 工具类型 (Rust, Python, JavaScript, Shell)
    pub tool_type: ToolType,
    /// 工具代码内容
    pub content: String,
    /// 参数定义
    pub parameters: Vec<ParameterDef>,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 作者
    pub author: String,
    /// 版本
    pub version: String,
    /// 依赖项
    pub dependencies: Vec<String>,
}

/// 工具使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageRecord {
    /// 记录ID
    pub id: String,
    /// 工具名称
    pub tool_name: String,
    /// 使用者（智能体ID或用户ID）
    pub user: String,
    /// 使用时间
    pub timestamp: i64,
    /// 输入参数
    pub input_params: HashMap<String, serde_json::Value>,
    /// 执行结果
    pub result: String,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
}

/// 智能体工具使用记录 - 专门用于跟踪智能体自己使用的工具
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolUsageRecord {
    /// 记录ID
    pub id: String,
    /// 智能体ID
    pub agent_id: String,
    /// 工具名称
    pub tool_name: String,
    /// 调用时间
    pub timestamp: i64,
    /// 目的/用途描述
    pub purpose: String,
    /// 输入参数
    pub input_params: HashMap<String, serde_json::Value>,
    /// 执行结果
    pub result: String,
    /// 是否成功
    pub success: bool,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
}

/// 智能体工具注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolRegistry {
    /// 智能体ID
    pub agent_id: String,
    /// 智能体名称
    pub agent_name: String,
    /// 已使用的工具列表
    pub used_tools: Vec<String>,
    /// 工具使用记录
    pub usage_records: Vec<AgentToolUsageRecord>,
    /// 注册时间
    pub registered_at: i64,
    /// 最后更新
    pub last_updated: i64,
}