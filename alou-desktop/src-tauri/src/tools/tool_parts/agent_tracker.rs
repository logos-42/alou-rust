//! 智能体跟踪模块
//! 
//! 负责跟踪和记录智能体使用的工具

use crate::tools::ToolError;
use crate::tools::tool_parts::definitions::{AgentToolRegistry, AgentToolUsageRecord, ToolDefinition};
use std::collections::HashMap;
use tokio::fs;
use std::path::Path;
use serde_json;
use chrono::Utc;

use crate::tools::tool_creation::ToolCreationTool;

impl ToolCreationTool {
    /// 注册智能体（初始化智能体的工具跟踪）
    pub async fn register_agent(&self, agent_id: &str, agent_name: &str) -> Result<(), ToolError> {
        let mut registry = self.agent_tool_registry.write().await;

        if registry.contains_key(agent_id) {
            return Ok(()); // 已注册
        }

        let agent_registry = AgentToolRegistry {
            agent_id: agent_id.to_string(),
            agent_name: agent_name.to_string(),
            used_tools: Vec::new(),
            usage_records: Vec::new(),
            registered_at: Utc::now().timestamp(),
            last_updated: Utc::now().timestamp(),
        };

        registry.insert(agent_id.to_string(), agent_registry);
        Ok(())
    }

    /// 智能体记录自己使用的工具
    pub async fn agent_log_tool_usage(&self,
                                   agent_id: &str,
                                   tool_name: &str,
                                   purpose: &str,
                                   input_params: HashMap<String, serde_json::Value>,
                                   result: &str,
                                   success: bool,
                                   execution_time_ms: u64) -> Result<String, ToolError> {
        let mut registry = self.agent_tool_registry.write().await;

        if let Some(agent_registry) = registry.get_mut(agent_id) {
            let record = AgentToolUsageRecord {
                id: format!("agent_usage_{}", uuid::Uuid::new_v4()),
                agent_id: agent_id.to_string(),
                tool_name: tool_name.to_string(),
                timestamp: Utc::now().timestamp(),
                purpose: purpose.to_string(),
                input_params,
                result: result.to_string(),
                success,
                execution_time_ms,
            };

            // 如果是新工具，添加到已使用工具列表
            if !agent_registry.used_tools.contains(&tool_name.to_string()) {
                agent_registry.used_tools.push(tool_name.to_string());
            }

            agent_registry.usage_records.push(record.clone());
            agent_registry.last_updated = Utc::now().timestamp();

            Ok(record.id)
        } else {
            Err(ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))
        }
    }

    /// 获取智能体使用的所有工具
    pub async fn get_agent_used_tools(&self, agent_id: &str) -> Result<Vec<String>, ToolError> {
        let registry = self.agent_tool_registry.read().await;

        if let Some(agent_registry) = registry.get(agent_id) {
            Ok(agent_registry.used_tools.clone())
        } else {
            Err(ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))
        }
    }

    /// 获取智能体的工具使用记录
    pub async fn get_agent_usage_records(&self, agent_id: &str, limit: Option<usize>) -> Result<Vec<AgentToolUsageRecord>, ToolError> {
        let registry = self.agent_tool_registry.read().await;

        if let Some(agent_registry) = registry.get(agent_id) {
            let mut records = agent_registry.usage_records.clone();

            if let Some(limit_val) = limit {
                records = records.into_iter().rev().take(limit_val).collect();
                records.reverse();
            }

            Ok(records)
        } else {
            Err(ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))
        }
    }

    /// 生成智能体工具使用报告（Markdown格式）
    #[allow(dead_code)]
    pub async fn generate_agent_tool_report(&self, agent_id: &str, docs_dir: &str) -> Result<String, ToolError> {
        let registry = self.agent_tool_registry.read().await;

        let agent_registry = registry.get(agent_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))?;

        let docs_path = Path::new(docs_dir);
        fs::create_dir_all(docs_path).await
            .map_err(|e| ToolError::InternalError(format!("Failed to create docs directory: {}", e)))?;

        let mut report = format!("# 智能体工具使用报告\n\n");
        report.push_str(&format!("**智能体ID**: {}\n\n", agent_registry.agent_id));
        report.push_str(&format!("**智能体名称**: {}\n\n", agent_registry.agent_name));
        report.push_str(&format!("**注册时间**: {}\n\n", chrono::DateTime::from_timestamp(agent_registry.registered_at, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "未知".to_string())));
        report.push_str(&format!("**最后更新**: {}\n\n", chrono::DateTime::from_timestamp(agent_registry.last_updated, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "未知".to_string())));
        report.push_str(&format!("**已使用工具数量**: {}\n\n", agent_registry.used_tools.len()));
        report.push_str(&format!("**工具调用总次数**: {}\n\n", agent_registry.usage_records.len()));

        report.push_str("## 已使用的工具列表\n\n");
        if agent_registry.used_tools.is_empty() {
            report.push_str("*暂无工具使用记录*\n\n");
        } else {
            for (index, tool) in agent_registry.used_tools.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", index + 1, tool));
            }
            report.push('\n');
        }

        report.push_str("## 工具使用详细记录\n\n");
        if agent_registry.usage_records.is_empty() {
            report.push_str("*暂无使用记录*\n\n");
        } else {
            for record in &agent_registry.usage_records {
                report.push_str(&format!("### {}\n\n", record.tool_name));
                report.push_str(&format!("- **时间**: {}\n", chrono::DateTime::from_timestamp(record.timestamp, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "未知".to_string())));
                report.push_str(&format!("- **目的**: {}\n", record.purpose));
                report.push_str(&format!("- **结果**: {}\n", record.result));
                report.push_str(&format!("- **耗时**: {}ms\n", record.execution_time_ms));
                report.push_str(&format!("- **成功**: {}\n\n", if record.success { "是" } else { "否" }));
            }
        }

        let filename = format!("agent_{}_tool_report.md", agent_id);
        let filepath = docs_path.join(&filename);

        fs::write(&filepath, report).await
            .map_err(|e| ToolError::InternalError(format!("Failed to write agent tool report: {}", e)))?;

        Ok(filepath.to_string_lossy().to_string())
    }

    /// 导出智能体工具使用数据为JSON
    #[allow(dead_code)]
    pub async fn export_agent_tool_data(&self, agent_id: &str, docs_dir: &str) -> Result<String, ToolError> {
        let registry = self.agent_tool_registry.read().await;

        let agent_registry = registry.get(agent_id)
            .ok_or_else(|| ToolError::InvalidArguments(format!("Agent '{}' not registered", agent_id)))?;

        let docs_path = Path::new(docs_dir);
        fs::create_dir_all(docs_path).await
            .map_err(|e| ToolError::InternalError(format!("Failed to create docs directory: {}", e)))?;

        let data = serde_json::to_string_pretty(agent_registry)
            .map_err(|e| ToolError::InternalError(format!("Failed to serialize agent tool data: {}", e)))?;

        let filename = format!("agent_{}_tool_data.json", agent_id);
        let filepath = docs_path.join(&filename);

        fs::write(&filepath, data).await
            .map_err(|e| ToolError::InternalError(format!("Failed to export agent tool data: {}", e)))?;

        Ok(filepath.to_string_lossy().to_string())
    }

    /// 发现指定目录下的所有工具
    #[allow(dead_code)]
    pub async fn discover_tools(&self, dir: &str) -> Result<Vec<ToolDefinition>, ToolError> {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            return Ok(Vec::new());
        }

        let mut tools = Vec::new();
        let mut entries = fs::read_dir(dir_path).await
            .map_err(|e| ToolError::InternalError(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| ToolError::InternalError(format!("Failed to read directory entry: {}", e)))? {

            let path = entry.path();
            if path.is_file() && path.to_string_lossy().ends_with("-def.json") {
                let content = fs::read_to_string(&path).await
                    .map_err(|e| ToolError::InternalError(format!("Failed to read tool definition: {}", e)))?;

                let tool_def: ToolDefinition = serde_json::from_str(&content)
                    .map_err(|e| ToolError::InternalError(format!("Failed to parse tool definition: {}", e)))?;

                tools.push(tool_def);
            }
        }

        Ok(tools)
    }

    /// 列出所有已注册的智能体
    #[allow(dead_code)]
    pub async fn list_registered_agents(&self) -> Result<Vec<String>, ToolError> {
        let registry = self.agent_tool_registry.read().await;
        Ok(registry.keys().cloned().collect())
    }
}