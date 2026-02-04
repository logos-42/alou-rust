//! 工具创建、记录和智能体工具跟踪工具（重构版）
//!
//! 允许智能体：
//! 1. 创建新工具并在特定目录中存放
//! 2. 记录工具使用情况到文档
//! 3. 跟踪智能体自己使用的工具
//! 4. 自动生成工具使用文档
//! 5. 动态执行已创建的工具（Python、Shell、JavaScript）
//! 6. 注册动态工具到工具注册表

use crate::tools::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use crate::tools::tool_parts::{
    definitions::{ToolDefinition, ToolType, ParameterDef, ToolUsageRecord, AgentToolUsageRecord, AgentToolRegistry},
    executor::{DynamicToolExecutor, DynamicToolResult},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

/// 工具创建和记录工具
pub struct ToolCreationTool {
    metadata: ToolMetadata,
    /// 智能体工具注册表
    agent_tool_registry: Arc<RwLock<HashMap<String, AgentToolRegistry>>>,
}

impl ToolCreationTool {
    /// 创建新的工具创建和记录工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "tool_creation".to_string(),
                name: "Tool Creation and Documentation Tool".to_string(),
                description: "创建新工具并记录到文档，支持多种语言；跟踪智能体使用的工具，自动生成文档".to_string(),
                category: ToolCategory::Development,
                priority: ToolPriority::Medium,
                status: ToolStatus::Available,
                version: "2.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "execute".to_string()],
            },
            agent_tool_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // 从 creator 模块导入方法
    pub async fn create_tool_file(&self, tool_def: &ToolDefinition, target_dir: &Path) -> Result<PathBuf, ToolError> {
        use crate::tools::tool_parts::creator;
        creator::ToolCreationTool::create_tool_file(self, tool_def, target_dir).await
    }

    pub async fn create_tool_definition_file(&self, tool_def: &ToolDefinition, target_dir: &Path) -> Result<PathBuf, ToolError> {
        use crate::tools::tool_parts::creator;
        creator::ToolCreationTool::create_tool_definition_file(self, tool_def, target_dir).await
    }

    pub async fn record_tool_usage(&self, record: &ToolUsageRecord, docs_dir: &Path) -> Result<PathBuf, ToolError> {
        use crate::tools::tool_parts::creator;
        creator::ToolCreationTool::record_tool_usage(self, record, docs_dir).await
    }

    pub async fn generate_tool_documentation(&self, tool_def: &ToolDefinition, docs_dir: &Path) -> Result<PathBuf, ToolError> {
        use crate::tools::tool_parts::creator;
        creator::ToolCreationTool::generate_tool_documentation(self, tool_def, docs_dir).await
    }

    pub async fn create_tool(&self, tool_def: ToolDefinition, target_dir: &str, docs_dir: &str) -> Result<HashMap<String, String>, ToolError> {
        use crate::tools::tool_parts::creator;
        creator::ToolCreationTool::create_tool(self, tool_def, target_dir, docs_dir).await
    }

    pub async fn create_and_execute_tool(&self, tool_def: ToolDefinition, target_dir: &str, docs_dir: &str) -> Result<HashMap<String, serde_json::Value>, ToolError> {
        use crate::tools::tool_parts::creator;
        creator::ToolCreationTool::create_and_execute_tool(self, tool_def, target_dir, docs_dir).await
    }

    pub async fn log_tool_usage(&self,
                           tool_name: &str,
                           user: &str,
                           input_params: HashMap<String, serde_json::Value>,
                           result: &str,
                           execution_time_ms: u64,
                           docs_dir: &str) -> Result<String, ToolError> {
        use crate::tools::tool_parts::creator;
        creator::ToolCreationTool::log_tool_usage(self, tool_name, user, input_params, result, execution_time_ms, docs_dir).await
    }

    // 从 agent_tracker 模块导入方法
    pub async fn register_agent(&self, agent_id: &str, agent_name: &str) -> Result<(), ToolError> {
        use crate::tools::tool_parts::agent_tracker;
        agent_tracker::ToolCreationTool::register_agent(self, agent_id, agent_name).await
    }

    pub async fn agent_log_tool_usage(&self,
                                   agent_id: &str,
                                   tool_name: &str,
                                   purpose: &str,
                                   input_params: HashMap<String, serde_json::Value>,
                                   result: &str,
                                   success: bool,
                                   execution_time_ms: u64) -> Result<String, ToolError> {
        use crate::tools::tool_parts::agent_tracker;
        agent_tracker::ToolCreationTool::agent_log_tool_usage(self, agent_id, tool_name, purpose, input_params, result, success, execution_time_ms).await
    }

    pub async fn get_agent_used_tools(&self, agent_id: &str) -> Result<Vec<String>, ToolError> {
        use crate::tools::tool_parts::agent_tracker;
        agent_tracker::ToolCreationTool::get_agent_used_tools(self, agent_id).await
    }

    pub async fn get_agent_usage_records(&self, agent_id: &str, limit: Option<usize>) -> Result<Vec<AgentToolUsageRecord>, ToolError> {
        use crate::tools::tool_parts::agent_tracker;
        agent_tracker::ToolCreationTool::get_agent_usage_records(self, agent_id, limit).await
    }

    pub async fn generate_agent_tool_report(&self, agent_id: &str, docs_dir: &str) -> Result<String, ToolError> {
        use crate::tools::tool_parts::agent_tracker;
        agent_tracker::ToolCreationTool::generate_agent_tool_report(self, agent_id, docs_dir).await
    }

    pub async fn export_agent_tool_data(&self, agent_id: &str, docs_dir: &str) -> Result<String, ToolError> {
        use crate::tools::tool_parts::agent_tracker;
        agent_tracker::ToolCreationTool::export_agent_tool_data(self, agent_id, docs_dir).await
    }

    pub async fn discover_tools(&self, dir: &str) -> Result<Vec<ToolDefinition>, ToolError> {
        use crate::tools::tool_parts::agent_tracker;
        agent_tracker::ToolCreationTool::discover_tools(self, dir).await
    }

    pub async fn list_registered_agents(&self) -> Result<Vec<String>, ToolError> {
        use crate::tools::tool_parts::agent_tracker;
        agent_tracker::ToolCreationTool::list_registered_agents(self).await
    }
}

#[async_trait]
impl ToolExecutor for ToolCreationTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "create_tool" => {
                // 解析工具定义
                let tool_def_value = args.get("tool_definition")
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_definition' field".to_string()))?;

                let tool_def: ToolDefinition = serde_json::from_value(tool_def_value.clone())
                    .map_err(|e| ToolError::InvalidArguments(format!("Invalid tool definition: {}", e)))?;

                let target_dir = args.get("target_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./tools/custom"); // 默认目录

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/tools"); // 默认文档目录

                let result = self.create_tool(tool_def, target_dir, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(result),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Created tool '{}' and documentation", result.get("tool_name").unwrap_or(&"unknown".to_string()))),
                    warnings: vec![],
                    context: None,
                })
            }

            "log_usage" => {
                let tool_name = args.get("tool_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_name' field".to_string()))?
                    .to_string();

                let user = args.get("user")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'user' field".to_string()))?
                    .to_string();

                let input_params: HashMap<String, serde_json::Value> = args.get("input_params")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let result = args.get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let execution_time_ms = args.get("execution_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/tools"); // 默认文档目录

                let log_path = self.log_tool_usage(&tool_name, &user, input_params, &result, execution_time_ms, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "tool_name": tool_name,
                        "user": user,
                        "log_path": log_path
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Logged usage of tool '{}' by user '{}'", tool_name, user)),
                    warnings: vec![],
                    context: None,
                })
            }

            "create_and_log" => {
                // 结合创建工具和记录使用的操作
                let tool_def_value = args.get("tool_definition")
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_definition' field".to_string()))?;

                let tool_def: ToolDefinition = serde_json::from_value(tool_def_value.clone())
                    .map_err(|e| ToolError::InvalidArguments(format!("Invalid tool definition: {}", e)))?;

                let target_dir = args.get("target_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./tools/custom");

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/tools");

                let creation_result = self.create_tool(tool_def.clone(), target_dir, docs_dir).await?;

                // 记录创建操作
                let input_params = HashMap::from([
                    ("tool_name".to_string(), serde_json::Value::String(tool_def.name.clone())),
                    ("tool_type".to_string(), serde_json::Value::String(format!("{:?}", tool_def.tool_type))),
                ]);

                let _log_result = self.log_tool_usage(
                    &tool_def.name,
                    "system",
                    input_params,
                    "Tool created successfully",
                    0,
                    docs_dir
                ).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(creation_result),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Created and logged tool '{}'", tool_def.name)),
                    warnings: vec![],
                    context: None,
                })
            }

            // ==================== 动态工具执行操作 ====================

            "execute_tool" => {
                let tool_type = args.get("tool_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_type' field".to_string()))?
                    .to_string();

                let script_path = args.get("script_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'script_path' field".to_string()))?
                    .to_string();

                let input_params: HashMap<String, serde_json::Value> = args.get("input_params")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let executor = DynamicToolExecutor::new();
                let result = executor.execute(
                    serde_json::json!({
                        "tool_type": tool_type,
                        "script_path": script_path,
                        "input_params": input_params
                    }),
                    _context
                ).await?;

                Ok(ToolResult {
                    success: result.success,
                    data: result.data,
                    error: result.error,
                    execution_time_ms: result.execution_time_ms,
                    output: result.output,
                    warnings: vec![],
                    context: None,
                })
            }

            "create_and_execute" => {
                // 创建工具并立即执行
                let tool_def_value = args.get("tool_definition")
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_definition' field".to_string()))?;

                let tool_def: ToolDefinition = serde_json::from_value(tool_def_value.clone())
                    .map_err(|e| ToolError::InvalidArguments(format!("Invalid tool definition: {}", e)))?;

                let target_dir = args.get("target_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./tools/custom");

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/tools");

                let result = self.create_and_execute_tool(tool_def, target_dir, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(result),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Created and executed tool")),
                    warnings: vec![],
                    context: None,
                })
            }

            // ==================== 智能体工具跟踪操作 ====================

            "register_agent" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let agent_name = args.get("agent_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&agent_id)
                    .to_string();

                self.register_agent(&agent_id, &agent_name).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "agent_name": agent_name
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Registered agent '{}' ({})", agent_id, agent_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "agent_log_tool" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let tool_name = args.get("tool_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'tool_name' field".to_string()))?
                    .to_string();

                let purpose = args.get("purpose")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let input_params: HashMap<String, serde_json::Value> = args.get("input_params")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let result = args.get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let success = args.get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let execution_time_ms = args.get("execution_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let record_id = self.agent_log_tool_usage(
                    &agent_id, &tool_name, &purpose, input_params, &result, success, execution_time_ms
                ).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "tool_name": tool_name,
                        "record_id": record_id
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Agent '{}' logged tool usage: {}", agent_id, tool_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "get_agent_used_tools" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let tools = self.get_agent_used_tools(&agent_id).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "used_tools": tools,
                        "count": tools.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Agent '{}' has used {} tools", agent_id, tools.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "get_agent_usage_records" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);

                let records = self.get_agent_usage_records(&agent_id, limit).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "usage_records": records,
                        "count": records.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} usage records for agent '{}'", records.len(), agent_id)),
                    warnings: vec![],
                    context: None,
                })
            }

            "generate_agent_tool_report" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/agents");

                let report_path = self.generate_agent_tool_report(&agent_id, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "report_path": report_path
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Generated tool report for agent '{}' at {}", agent_id, report_path)),
                    warnings: vec![],
                    context: None,
                })
            }

            "export_agent_tool_data" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?
                    .to_string();

                let docs_dir = args.get("docs_dir")
                    .and_then(|v| v.as_str())
                    .unwrap_or("./docs/agents");

                let export_path = self.export_agent_tool_data(&agent_id, docs_dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agent_id": agent_id,
                        "export_path": export_path
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Exported agent '{}' tool data to {}", agent_id, export_path)),
                    warnings: vec![],
                    context: None,
                })
            }

            "discover_tools" => {
                let dir = args.get("dir")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'dir' field".to_string()))?
                    .to_string();

                let tools = self.discover_tools(&dir).await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "directory": dir,
                        "tools": tools,
                        "count": tools.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Discovered {} tools in directory '{}'", tools.len(), dir)),
                    warnings: vec![],
                    context: None,
                })
            }

            "list_registered_agents" => {
                let agents = self.list_registered_agents().await?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "agents": agents,
                        "count": agents.len()
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} registered agents", agents.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            _ => Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
        }
    }

    async fn validate_args(&self, args: &serde_json::Value) -> Result<(), ToolError> {
        if !args.is_object() {
            return Err(ToolError::InvalidArguments("Arguments must be an object".to_string()));
        }
        if args.get("action").is_none() {
            return Err(ToolError::InvalidArguments("Missing required field: action".to_string()));
        }
        Ok(())
    }

    fn help(&self) -> String {
        r##"Tool Creation and Documentation Tool (Enhanced)

Create new tools, document usage, and track agent tool usage.

=== Core Actions ===

  - create_tool: Create a new tool with definition and documentation
    params: {
      "tool_definition": {...},
      "target_dir": "Directory to store the tool (default: ./tools/custom)",
      "docs_dir": "Directory to store documentation (default: ./docs/tools)"
    }

  - log_usage: Log tool usage to documentation
    params: {
      "tool_name": "Name of the tool used",
      "user": "User who used the tool",
      "input_params": {...},
      "result": "Execution result",
      "execution_time_ms": Execution time in milliseconds,
      "docs_dir": "Directory to store documentation (default: ./docs/tools)"
    }

  - create_and_log: Create a tool and log its creation
    params: Same as create_tool

  - execute_tool: Execute a dynamically created tool
    params: {
      "tool_type": "Python|Shell|JavaScript",
      "script_path": "Path to the script file",
      "input_params": {"key": "value", ...}
    }

  - create_and_execute: Create a tool and execute it immediately
    params: {
      "tool_definition": {...},
      "target_dir": "Directory to store the tool",
      "docs_dir": "Directory to store documentation"
    }

=== Agent Tool Tracking Actions ===

  - register_agent: Register an agent for tool tracking
    params: {
      "agent_id": "Unique agent identifier",
      "agent_name": "Human-readable agent name (optional, defaults to agent_id)"
    }

  - agent_log_tool: Agent records its own tool usage
    params: {
      "agent_id": "Agent identifier",
      "tool_name": "Name of the tool used",
      "purpose": "Purpose/description of using this tool",
      "input_params": {...},
      "result": "Execution result",
      "success": true|false,
      "execution_time_ms": Execution time in milliseconds
    }

  - get_agent_used_tools: Get list of tools used by an agent
    params: {
      "agent_id": "Agent identifier"
    }

  - get_agent_usage_records: Get detailed usage records for an agent
    params: {
      "agent_id": "Agent identifier",
      "limit": Optional number of records to return
    }

  - generate_agent_tool_report: Generate Markdown report of agent tool usage
    params: {
      "agent_id": "Agent identifier",
      "docs_dir": "Output directory (default: ./docs/agents)"
    }

  - export_agent_tool_data: Export agent tool data as JSON
    params: {
      "agent_id": "Agent identifier",
      "docs_dir": "Output directory (default: ./docs/agents)"
    }

  - discover_tools: Discover all tools in a directory
    params: {
      "dir": "Directory to scan for tools"
    }

  - list_registered_agents: List all registered agents
    params: {}

Examples:

Register an agent:
{
  "action": "register_agent",
  "agent_id": "researcher-agent-001",
  "agent_name": "Research Agent"
}

Agent logs tool usage:
{
  "action": "agent_log_tool",
  "agent_id": "researcher-agent-001",
  "tool_name": "web_search",
  "purpose": "Search for information about AI trends",
  "input_params": {
    "query": "AI development trends 2024"
  },
  "result": "Found 15 relevant articles",
  "success": true,
  "execution_time_ms": 150
}

Generate agent tool usage report:
{
  "action": "generate_agent_tool_report",
  "agent_id": "researcher-agent-001",
  "docs_dir": "./docs/agent-reports"
}

Discover available tools:
{
  "action": "discover_tools",
  "dir": "./tools/custom"
}

=== Dynamic Tool Execution Examples ===

Execute a created Python tool:
{
  "action": "execute_tool",
  "tool_type": "Python",
  "script_path": "./tools/custom/my_python_tool.py",
  "input_params": {
    "input": "data.csv",
    "format": "json"
  }
}

Create and execute a tool in one step:
{
  "action": "create_and_execute",
  "tool_definition": {
    "name": "quick_calculator",
    "description": "A simple calculator tool",
    "tool_type": "Python",
    "content": "#!/usr/bin/env python3\nimport sys\na = int(sys.argv[1])\nb = int(sys.argv[2])\nprint(str(a) + str(b) + str(a + b))",
    "parameters": [
      {
        "name": "a",
        "param_type": "number",
        "required": true,
        "description": "First number"
      },
      {
        "name": "b",
        "param_type": "number",
        "required": true,
        "description": "Second number"
      }
    ],
    "author": "AI Assistant",
    "version": "1.0.0",
    "dependencies": []
  },
  "target_dir": "./tools/custom",
  "docs_dir": "./docs/tools"
}"##.to_string()
    }
}