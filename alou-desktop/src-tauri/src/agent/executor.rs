//! Ralph Loop 执行器
//!
//! 实现 Ralph Loop 的核心执行逻辑，支持无限迭代、工具调用、流式响应

use super::ai_client::{AiClient, AiMessage, AiTool, AiToolCall as ProviderToolCall};
use super::task::{Task, TaskManager, TaskStatus, TaskEvent, ToolCall, ToolResult, TaskFinalResult};
use super::error::AgentError;
use crate::bridges::{ToolBridge, ToolCallRequest, ToolCallResponse};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use std::path::PathBuf;

/// Ralph Loop 执行器
pub struct RalphLoopExecutor {
    ai_client: Arc<AiClient>,
    task_manager: Arc<TaskManager>,
    tool_bridge: Arc<ToolBridge>,
    tool_registry: Arc<crate::tools::ToolRegistry>,
    app_handle: Option<tauri::AppHandle>,
}

/// 执行结果
#[derive(Debug)]
pub enum ExecutionResult {
    /// 已完成
    Completed(String),
    /// 需要更多迭代
    NeedsMoreIterations,
    /// 失败
    Failed(String),
}

/// 执行器错误
#[derive(Debug)]
pub enum ExecutorError {
    TaskNotFound(String),
    AiError(String),
    ToolError(String),
    InternalError(String),
}

impl From<AgentError> for ExecutorError {
    fn from(err: AgentError) -> Self {
        match err {
            AgentError::TaskError(msg) => ExecutorError::TaskNotFound(msg),
            AgentError::AiError(msg) => ExecutorError::AiError(msg),
            AgentError::ToolError(msg) => ExecutorError::ToolError(msg),
            _ => ExecutorError::InternalError(err.to_string()),
        }
    }
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorError::TaskNotFound(msg) => write!(f, "任务未找到: {}", msg),
            ExecutorError::AiError(msg) => write!(f, "AI 错误: {}", msg),
            ExecutorError::ToolError(msg) => write!(f, "工具错误: {}", msg),
            ExecutorError::InternalError(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl RalphLoopExecutor {
    pub fn new(
        ai_client: Arc<AiClient>,
        task_manager: Arc<TaskManager>,
        tool_bridge: Arc<ToolBridge>,
        tool_registry: Arc<crate::tools::ToolRegistry>,
    ) -> Self {
        Self {
            ai_client,
            task_manager,
            tool_bridge,
            tool_registry,
            app_handle: None,
        }
    }

    /// 设置 AppHandle（用于向前端发送进度事件）
    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    /// 生成初始文档内容
    fn generate_initial_document_content(document_type: &str, agent_id: &str) -> String {
        let now = chrono::Utc::now().to_rfc3339();
        match document_type.to_lowercase().as_str() {
            "memory" => format!(r#"# 长期记忆

**智能体 ID**: {}

## 用户偏好
（暂无记录）

## 项目信息
（暂无记录）

## 学到的知识
（暂无记录）

## 重要对话
（暂无记录）

---
最后更新: {}
"#, agent_id, now),
            "soul" => format!(r#"# 核心身份

**智能体 ID**: {}

## 角色定位
专业的AI助手

## 核心价值观
- 准确性：提供准确可靠的信息
- 效率：快速完成任务
- 安全性：注重操作安全
- 学习性：从每次交互中学习和改进

## 个性特点
- 友好且专业
- 注重细节
- 善于沟通

---
最后更新: {}
"#, agent_id, now),
            "identity" => format!(r#"# 身份定义

**智能体 ID**: {}

## 名称
智能体

## 角色
专业的AI助手

## 专长领域
（根据实际使用情况更新）

## 工作方式
- 理解用户需求
- 选择合适工具
- 执行任务
- 反馈结果

---
最后更新: {}
"#, agent_id, now),
            "capabilities" => format!(r#"# 能力清单

## 核心能力
- 文件操作：读取、写入、编辑、搜索文件
- 终端命令：执行系统命令
- 网络操作：搜索信息、获取网页内容
- 任务规划：制定和管理任务计划
- 代码理解：分析和修改代码

## 工具使用
- 熟练使用所有可用工具
- 能够组合多个工具完成复杂任务
- 理解工具的限制和最佳实践

## 学习能力
- 从用户反馈中学习
- 记录成功的解决方案
- 避免重复错误

---
最后更新: {}
"#, now),
            "constraints" => format!(r#"# 约束和限制

## 操作限制
- 不执行危险命令
- 不访问敏感文件
- 不进行未经授权的网络操作

## 行为准则
- 始终征求用户确认重要操作
- 清晰解释操作步骤
- 提供操作结果反馈

## 安全原则
- 保护用户数据安全
- 遵守系统安全策略
- 及时报告异常情况

---
最后更新: {}
"#, now),
            "tools" => format!(r#"# 工具使用记录

## 常用工具
（根据实际使用情况更新）

## 工具组合
（记录有效的工具组合方案）

## 最佳实践
（记录工具使用的最佳实践）

---
最后更新: {}
"#, now),
            "agents" => format!(r#"# 协作智能体

## 已知智能体
（暂无记录）

## 协作经验
（暂无记录）

## 协作模式
（暂无记录）

---
最后更新: {}
"#, now),
            _ => format!(r#"# {}

---
最后更新: {}
"#, document_type, now),
        }
    }

    /// 执行完整的 Ralph Loop
    pub async fn execute(&self, task_id: &str) -> std::result::Result<TaskFinalResult, ExecutorError> {
        log::info!("[RalphLoop] 开始执行任务: {}", task_id);

        // 更新任务状态为运行中
        self.task_manager
            .update_task(task_id, |task| {
                task.status = TaskStatus::Running;
            })
            .await?;

        self.emit_event(task_id, TaskEvent::TaskStarted {
            task_id: task_id.to_string(),
        })
        .await;

        loop {
            // 获取当前任务状态
            let task = self
                .task_manager
                .get_task(task_id)
                .await
                .ok_or_else(|| ExecutorError::TaskNotFound(task_id.to_string()))?;

            log::info!(
                "[RalphLoop] 任务 {} 迭代 {}/{}",
                task_id,
                task.metadata.iteration_count,
                task.metadata.max_iterations
            );

            // 1. 调用 AI
            let ai_response = match self.call_ai(&task).await {
                Ok(response) => response,
                Err(e) => {
                    log::error!("[RalphLoop] AI 调用失败: {}", e);
                    self.handle_error(task_id, &e.to_string()).await?;
                    return Err(ExecutorError::AiError(e.to_string()));
                }
            };

            // 发送 AI 响应事件
            if !ai_response.content.is_empty() {
                self.emit_event(task_id, TaskEvent::AiResponse {
                    task_id: task_id.to_string(),
                    content: ai_response.content.clone(),
                })
                .await;
            }

            // 2. 检查是否有工具调用
            if ai_response.tool_calls.is_empty() {
                // 没有工具调用，任务完成
                let final_content = ai_response.content;
                self.handle_completion(task_id, &final_content).await?;
                return Ok(TaskFinalResult {
                    task_id: task_id.to_string(),
                    success: true,
                    result: final_content,
                    error: None,
                    iteration_count: self
                        .task_manager
                        .get_task(task_id)
                        .await
                        .map(|t| t.metadata.iteration_count)
                        .unwrap_or(0),
                });
            }

            // 3. 执行工具调用
            self.task_manager
                .update_task(task_id, |task| {
                    task.status = TaskStatus::ProcessingTools;
                    task.pending_tools = ai_response
                        .tool_calls
                        .iter()
                        .map(|tc| ToolCall {
                            id: tc.id.clone(),
                            name: tc.name.clone(),
                            arguments: tc.arguments.clone(),
                        })
                        .collect();
                })
                .await?;

            self.emit_event(task_id, TaskEvent::ToolCallsPending {
                task_id: task_id.to_string(),
                count: ai_response.tool_calls.len(),
            })
            .await;

            // 执行所有工具调用
            let tool_results = match self.execute_tools(task_id, &ai_response.tool_calls).await {
                Ok(results) => results,
                Err(e) => {
                    log::error!("[RalphLoop] 工具执行失败: {}", e);
                    self.handle_error(task_id, &e.to_string()).await?;
                    return Err(ExecutorError::ToolError(e.to_string()));
                }
            };

            // 4. 将工具结果添加到消息历史
            self.add_tool_results_to_task(task_id, &ai_response.tool_calls, &tool_results)
                .await?;

            // 5. 增加迭代计数
            let _ = self.task_manager
                .update_task(task_id, |task| {
                    task.metadata.iteration_count += 1;
                })
                .await;

            // 继续下一次迭代（Ralph Loop 的核心）
            log::info!("[RalphLoop] 继续下一次迭代");
        }
    }

    /// 调用 AI
    async fn call_ai(&self, task: &Task) -> super::error::Result<super::ai_client::AiResponse> {
        log::info!("[RalphLoop] 调用 AI，消息数: {}", task.messages.len());

        let messages = self.build_messages(task);
        let tools = self.get_available_tools().await;

        self.ai_client.send_message(messages, Some(tools)).await
    }

    /// 构建消息列表
    fn build_messages(&self, task: &Task) -> Vec<AiMessage> {
        task.messages.clone()
    }

    /// 获取可用工具（从 ToolRegistry 异步获取）
    async fn get_available_tools(&self) -> Vec<AiTool> {
        // 从 ToolRegistry 获取工具列表
        let metadatas = self.tool_registry.list_all().await;
        
        // 如果 registry 为空，返回默认工具
        if metadatas.is_empty() {
            log::warn!("[RalphLoop] ToolRegistry 为空，使用默认工具列表");
            return Self::get_default_tools();
        }
        
        // 将 ToolMetadata 转换为 AiTool
        metadatas.into_iter().map(Self::metadata_to_ai_tool).collect()
    }

    /// 将 ToolMetadata 转换为 AiTool
    fn metadata_to_ai_tool(metadata: crate::tools::ToolMetadata) -> AiTool {
        AiTool {
            name: metadata.id,
            description: metadata.description,
            parameters: Self::get_tool_parameters(&metadata.name),
        }
    }

    /// 获取工具的参数定义
    fn get_tool_parameters(tool_name: &str) -> serde_json::Value {
        match tool_name {
            "filesystem" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["read", "write", "edit", "delete", "list", "copy", "move"], "description": "文件操作类型" },
                    "path": { "type": "string", "description": "文件或目录路径" },
                    "content": { "type": "string", "description": "写入的内容" },
                    "old_text": { "type": "string", "description": "要替换的文本" },
                    "new_text": { "type": "string", "description": "新文本" },
                    "destination": { "type": "string", "description": "目标路径" }
                },
                "required": ["operation", "path"]
            }),
            "bash" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["execute"], "description": "操作类型，固定为 execute", "default": "execute" },
                    "shell": { "type": "string", "enum": ["bash", "cmd", "powershell", "python", "node"], "description": "Shell类型", "default": "bash" },
                    "command": { "type": "string", "description": "要执行的命令" },
                    "working_dir": { "type": "string", "description": "工作目录（可选）" },
                    "environment": { "type": "array", "items": { "type": "array", "items": { "type": "string" }, "minItems": 2, "maxItems": 2 }, "description": "环境变量数组，格式: [[\"KEY\", \"VALUE\"]]", "default": [] },
                    "timeout_seconds": { "type": "integer", "description": "超时时间（秒）", "default": 30 }
                },
                "required": ["operation", "shell", "command"]
            }),
            "search" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["grep", "glob", "find"], "description": "搜索操作类型：grep=文本搜索，glob=文件模式匹配，find=高级查找" },
                    "pattern": { "type": "string", "description": "搜索模式或文件匹配模式" },
                    "directory": { "type": "string", "description": "搜索目录" },
                    "file_pattern": { "type": "string", "description": "文件匹配模式（grep操作可选）" },
                    "case_sensitive": { "type": "boolean", "description": "是否区分大小写（grep操作，默认false）", "default": false },
                    "max_results": { "type": "integer", "description": "最大结果数（grep操作可选）" },
                    "recursive": { "type": "boolean", "description": "是否递归搜索（glob操作，默认false）", "default": false },
                    "options": { "type": "object", "description": "高级查找选项（find操作）" }
                },
                "required": ["operation", "pattern", "directory"]
            }),
            "git_helper" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["execute", "smart_commit", "create_feature_branch", "safe_merge", "status_check", "diff_summary", "log_history", "stash_management", "get_prompt", "batch_operation", "undo_operation", "remote_sync", "init_repository", "config_management"], "description": "Git操作类型" },
                    "subcommand": { "type": "string", "description": "Git子命令（execute需要）" },
                    "args": { "type": "array", "items": { "type": "string" }, "description": "命令参数（execute可选）" },
                    "working_dir": { "type": "string", "description": "工作目录（可选）" },
                    "message": { "type": "string", "description": "提交信息（smart_commit/stash_management需要）" },
                    "add_all": { "type": "boolean", "description": "是否添加所有文件（smart_commit可选）" },
                    "allow_empty": { "type": "boolean", "description": "是否允许空提交（smart_commit可选）" },
                    "branch_name": { "type": "string", "description": "分支名称（create_feature_branch需要）" },
                    "base_branch": { "type": "string", "description": "基础分支（create_feature_branch可选）" },
                    "source_branch": { "type": "string", "description": "源分支（safe_merge需要）" },
                    "strategy": { "type": "string", "description": "合并策略（safe_merge可选，默认merge）" },
                    "detailed": { "type": "boolean", "description": "是否详细输出（status_check可选）" },
                    "target": { "type": "string", "description": "目标（diff_summary/undo_operation可选）" },
                    "stat_only": { "type": "boolean", "description": "仅统计（diff_summary可选）" },
                    "count": { "type": "integer", "description": "日志数量（log_history可选，默认10）" },
                    "branch": { "type": "string", "description": "分支名称（log_history/remote_sync可选）" },
                    "format": { "type": "string", "description": "日志格式（log_history可选，默认oneline）" },
                    "operation": { "type": "string", "description": "操作类型（stash_management/remote_sync/config_management需要）" },
                    "scenario": { "type": "string", "description": "场景（get_prompt需要）" },
                    "context": { "type": "object", "description": "上下文（get_prompt可选）" },
                    "operations": { "type": "array", "description": "批量操作列表（batch_operation需要）" },
                    "undo_type": { "type": "string", "description": "撤销类型（undo_operation需要）" },
                    "force": { "type": "boolean", "description": "是否强制（undo_operation可选）" },
                    "remote": { "type": "string", "description": "远程仓库名（remote_sync可选，默认origin）" },
                    "path": { "type": "string", "description": "仓库路径（init_repository需要）" },
                    "initial_branch": { "type": "string", "description": "初始分支名（init_repository可选，默认main）" },
                    "key": { "type": "string", "description": "配置键（config_management需要）" },
                    "value": { "type": "string", "description": "配置值（config_management可选）" },
                    "global": { "type": "boolean", "description": "是否全局配置（config_management可选）" }
                },
                "required": ["action"]
            }),
            "network" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["HttpRequest", "DnsLookup", "Ping"], "description": "网络操作类型" },
                    "method": { "type": "string", "description": "HTTP方法（HttpRequest需要）：GET, POST, PUT, DELETE等" },
                    "url": { "type": "string", "description": "URL地址（HttpRequest需要）" },
                    "headers": { "type": "object", "description": "请求头（HttpRequest可选）" },
                    "body": { "type": "string", "description": "请求体（HttpRequest可选）" },
                    "timeout": { "type": "integer", "description": "超时时间秒数（HttpRequest可选）" },
                    "domain": { "type": "string", "description": "域名（DnsLookup需要）" },
                    "host": { "type": "string", "description": "主机地址（Ping需要）" },
                    "count": { "type": "integer", "description": "Ping次数（Ping可选，默认4）" }
                },
                "required": ["operation"]
            }),
            "system" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["Info", "Processes", "Cpu", "Memory", "Disks", "Environment"], "description": "系统操作类型" }
                },
                "required": ["operation"]
            }),
            "plan" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create_plan", "add_step", "update_step_status", "get_plan", "list_plans", "delete_plan", "analyze_dependencies", "create_todo", "update_todo_status", "list_todos", "delete_todo"], "description": "计划操作类型" },
                    "name": { "type": "string", "description": "计划名称（create_plan需要）" },
                    "description": { "type": "string", "description": "描述（create_plan/create_todo需要）" },
                    "goal": { "type": "string", "description": "目标（create_plan需要）" },
                    "steps": { "type": "array", "description": "步骤列表（create_plan需要）" },
                    "plan_id": { "type": "string", "description": "计划ID（add_step/update_step_status/get_plan/delete_plan/analyze_dependencies需要）" },
                    "step_name": { "type": "string", "description": "步骤名称（add_step需要）" },
                    "step_description": { "type": "string", "description": "步骤描述（add_step需要）" },
                    "dependencies": { "type": "array", "items": { "type": "string" }, "description": "依赖步骤ID列表（add_step可选）" },
                    "estimated_duration": { "type": "integer", "description": "预计时长分钟数（add_step可选）" },
                    "resources": { "type": "array", "items": { "type": "string" }, "description": "资源列表（add_step可选）" },
                    "step_id": { "type": "string", "description": "步骤ID（update_step_status需要）" },
                    "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "paused", "cancelled", "failed"], "description": "状态（update_step_status/update_todo_status需要）" },
                    "title": { "type": "string", "description": "标题（create_todo需要）" },
                    "priority": { "type": "string", "enum": ["low", "medium", "high", "urgent"], "description": "优先级（create_todo可选）" },
                    "due_date": { "type": "integer", "description": "截止日期时间戳（create_todo可选）" },
                    "todo_id": { "type": "string", "description": "待办ID（update_todo_status/delete_todo需要）" },
                    "status_filter": { "type": "string", "description": "状态过滤（list_todos可选）" }
                },
                "required": ["action"]
            }),
            "todolist" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["Create", "Update", "Delete", "Get", "List", "Clear"], "description": "待办操作类型" },
                    "id": { "type": "string", "description": "待办事项ID（Update/Delete/Get需要）" },
                    "title": { "type": "string", "description": "标题（Create需要，Update可选）" },
                    "description": { "type": "string", "description": "描述（Create/Update可选）" },
                    "priority": { "type": "string", "enum": ["low", "medium", "high"], "description": "优先级（Create/Update可选）" },
                    "status": { "type": "string", "description": "状态（Update可选）" },
                    "due_date": { "type": "integer", "description": "截止日期时间戳（Create/Update可选）" },
                    "status_filter": { "type": "string", "description": "状态过滤（List/Clear可选）" },
                    "priority_filter": { "type": "string", "description": "优先级过滤（List可选）" }
                },
                "required": ["operation"]
            }),
            "agent_skills" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["discover", "list", "load", "execute", "search"], "description": "技能操作：discover=扫描可用技能，list=列出已发现的技能，load=加载技能完整内容，execute=执行技能，search=搜索技能" },
                    "skill_name": { "type": "string", "description": "技能名称（load/execute/search 操作需要）" },
                    "inputs": { "type": "object", "description": "执行技能时的输入参数（execute 操作需要）" },
                    "query": { "type": "string", "description": "搜索关键词（search 操作需要）" }
                },
                "required": ["action"]
            }),
            "agent_creator" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create", "list", "get", "delete"], "description": "操作类型：create=创建新Agent，list=列出所有Agent，get=获取指定Agent，delete=删除Agent" },
                    "agent_id": { "type": "string", "description": "Agent ID（get/delete 操作需要）" },
                    "display_name": { "type": "string", "description": "Agent名称（create 操作需要）" },
                    "description": { "type": "string", "description": "Agent描述（create 操作可选）" },
                    "skills": { "type": "array", "items": { "type": "string" }, "description": "技能列表（create 操作可选）" },
                    "config": { "type": "object", "description": "额外配置（可选）" }
                },
                "required": ["action"]
            }),
            "agent_document" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["read", "update"], "description": "read=读取文档内容（从系统提示词中提取），update=更新文档内容（会触发前端同步到持久化存储）" },
                    "document_type": { "type": "string", "enum": ["soul", "identity", "capabilities", "constraints", "tools", "memory", "agents"], "description": "要读取/更新的文档类型" },
                    "new_content": { "type": "string", "description": "新文档内容（Markdown格式，update时必填）" },
                    "reason": { "type": "string", "description": "更新原因（建议填写，有助于调试和理解）" }
                },
                "required": ["action", "document_type"]
            }),
            "tool_creation" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create", "update", "delete", "get", "list", "execute"], "description": "工具创建操作类型" },
                    "tool_id": { "type": "string", "description": "工具ID（update/delete/get/execute需要）" },
                    "name": { "type": "string", "description": "工具名称（create需要）" },
                    "description": { "type": "string", "description": "工具描述（create需要）" },
                    "code": { "type": "string", "description": "工具代码（create/update需要）" },
                    "parameters": { "type": "object", "description": "工具参数schema（create/update可选）" },
                    "args": { "type": "object", "description": "执行参数（execute需要）" }
                },
                "required": ["action"]
            }),
            "rollback" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create_snapshot", "create_batch_snapshot", "list_snapshots", "restore_snapshot", "compare_snapshot", "delete_snapshot", "cleanup_snapshots"], "description": "回滚操作类型" },
                    "target_path": { "type": "string", "description": "目标路径（create_snapshot需要）" },
                    "name": { "type": "string", "description": "快照名称（create_snapshot需要）" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "标签（create_snapshot/list_snapshots可选）" },
                    "target_paths": { "type": "array", "items": { "type": "string" }, "description": "多个目标路径（create_batch_snapshot需要）" },
                    "name_prefix": { "type": "string", "description": "快照名称前缀（create_batch_snapshot需要）" },
                    "session_id": { "type": "string", "description": "会话ID过滤（list_snapshots可选）" },
                    "snapshot_id": { "type": "string", "description": "快照ID（restore_snapshot/compare_snapshot/delete_snapshot需要）" },
                    "restore_path": { "type": "string", "description": "恢复路径（restore_snapshot可选）" },
                    "force": { "type": "boolean", "description": "是否强制覆盖（restore_snapshot可选）" },
                    "keep_last": { "type": "integer", "description": "保留最近N个快照（cleanup_snapshots可选）" },
                    "older_than_days": { "type": "integer", "description": "删除N天前的快照（cleanup_snapshots可选）" }
                },
                "required": ["action"]
            }),
            "pubsub" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["publish", "subscribe", "subscriber_count", "list_topics", "get_history", "create_persistent_topic", "create_group", "join_group", "leave_group", "send_group_message", "get_group_info", "list_groups"], "description": "PubSub 操作类型" },
                    "topic": { "type": "string", "description": "主题名称（除list_topics外都需要）" },
                    "message": { "type": "string", "description": "消息内容（publish/send_group_message 需要）" },
                    "message_type": { "type": "string", "description": "消息类型（publish可选）" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "消息标签（publish可选）" },
                    "limit": { "type": "integer", "description": "消息数量限制（subscribe/get_history可选，默认10）" },
                    "description": { "type": "string", "description": "主题/群聊描述（create_persistent_topic/create_group 可选）" },
                    "persistent": { "type": "boolean", "description": "是否持久化（create_persistent_topic可选，默认true）" },
                    "group_name": { "type": "string", "description": "群聊名称（create_group 需要）" },
                    "members": { "type": "array", "items": { "type": "string" }, "description": "初始成员列表（create_group 可选）" },
                    "group_id": { "type": "string", "description": "群聊 ID（join_group/leave_group/send_group_message/get_group_info 需要）" }
                },
                "required": ["action"]
            }),
            "message_passing" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["send_message", "subscribe_topic", "list_topics", "get_message_history", "create_topic"], "description": "消息传递操作类型" },
                    "topic": { "type": "string", "description": "主题名称（除list_topics外都需要）" },
                    "content": { "type": "string", "description": "消息内容（send_message需要）" },
                    "message_type": { "type": "string", "description": "消息类型（send_message可选，默认text）" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "消息标签（send_message可选）" },
                    "latest_only": { "type": "boolean", "description": "是否仅获取最新消息（subscribe_topic可选，默认false）" },
                    "limit": { "type": "integer", "description": "消息数量限制（subscribe_topic/get_message_history可选，默认10）" },
                    "description": { "type": "string", "description": "主题描述（create_topic可选）" }
                },
                "required": ["action"]
            }),
            "iroh" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create_doc", "open_doc", "set", "get", "list_entries", "get_node_id", "connect_to_node", "share_doc_ticket", "create_group", "join_group", "leave_group", "send_group_message", "get_group_info", "list_groups"], "description": "Iroh操作类型" },
                    "name": { "type": "string", "description": "文档名称（create_doc可选）" },
                    "doc_id": { "type": "string", "description": "文档ID（open_doc/set/get/list_entries/share_doc_ticket需要）" },
                    "key": { "type": "string", "description": "键（set/get需要）" },
                    "value": { "type": "string", "description": "值（set需要）" },
                    "peer_id": { "type": "string", "description": "对等节点ID（connect_to_node需要）" },
                    "addr": { "type": "string", "description": "节点地址（connect_to_node可选）" },
                    "group_name": { "type": "string", "description": "群聊名称（create_group 需要）" },
                    "group_id": { "type": "string", "description": "群聊 ID（join_group/leave_group/send_group_message/get_group_info 需要）" },
                    "description": { "type": "string", "description": "群聊描述（create_group 可选）" },
                    "members": { "type": "array", "items": { "type": "string" }, "description": "成员列表（create_group 可选）" },
                    "message": { "type": "string", "description": "消息内容（send_group_message 需要）" }
                },
                "required": ["action"]
            }),
            "ipfs_archive" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["add", "get", "pin", "unpin", "list_pins", "cat"], "description": "IPFS归档操作类型" },
                    "path": { "type": "string", "description": "文件路径（add需要）" },
                    "cid": { "type": "string", "description": "内容ID（get/pin/unpin/cat需要）" },
                    "output_path": { "type": "string", "description": "输出路径（get可选）" },
                    "recursive": { "type": "boolean", "description": "是否递归（add可选）" }
                },
                "required": ["action"]
            }),
            "browser" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["open_page", "close_page", "navigate", "refresh", "click_element", "input_text", "get_element_text", "get_page_title", "get_page_url", "take_screenshot", "wait_for_element", "scroll_to_element", "execute_script", "get_page_source", "set_window_size", "open_new_tab", "switch_tab"], "description": "浏览器操作类型" },
                    "url": { "type": "string", "description": "页面URL（open_page/navigate/open_new_tab需要）" },
                    "wait_time": { "type": "integer", "description": "等待时间秒数（open_page/click_element可选）" },
                    "selector": { "type": "string", "description": "元素选择器（click_element/input_text/get_element_text/wait_for_element/scroll_to_element需要）" },
                    "text": { "type": "string", "description": "输入文本（input_text需要）" },
                    "clear_first": { "type": "boolean", "description": "是否先清空（input_text可选，默认true）" },
                    "path": { "type": "string", "description": "截图保存路径（take_screenshot可选）" },
                    "timeout": { "type": "integer", "description": "超时时间秒数（wait_for_element可选，默认10）" },
                    "script": { "type": "string", "description": "JavaScript代码（execute_script需要）" },
                    "width": { "type": "integer", "description": "窗口宽度（set_window_size需要）" },
                    "height": { "type": "integer", "description": "窗口高度（set_window_size需要）" },
                    "index": { "type": "integer", "description": "标签页索引（switch_tab需要）" }
                },
                "required": ["action"]
            }),
            "ui_control" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["click_button", "set_input_text", "get_input_text", "select_dropdown_option", "toggle_checkbox", "select_radio_button", "show_notification", "show_modal", "get_window_state", "set_window_state"], "description": "UI控制操作类型" },
                    "button_id": { "type": "string", "description": "按钮ID（click_button需要）" },
                    "params": { "type": "object", "description": "额外参数（click_button可选）" },
                    "input_id": { "type": "string", "description": "输入框ID（set_input_text/get_input_text需要）" },
                    "text": { "type": "string", "description": "文本内容（set_input_text需要）" },
                    "dropdown_id": { "type": "string", "description": "下拉菜单ID（select_dropdown_option需要）" },
                    "option_value": { "type": "string", "description": "选项值（select_dropdown_option/select_radio_button需要）" },
                    "checkbox_id": { "type": "string", "description": "复选框ID（toggle_checkbox需要）" },
                    "radio_group_id": { "type": "string", "description": "单选按钮组ID（select_radio_button需要）" },
                    "title": { "type": "string", "description": "标题（show_notification/show_modal需要）" },
                    "message": { "type": "string", "description": "消息内容（show_notification需要）" },
                    "notification_type": { "type": "string", "enum": ["info", "warning", "error", "success"], "description": "通知类型（show_notification可选）" },
                    "content": { "type": "string", "description": "对话框内容（show_modal需要）" },
                    "buttons": { "type": "array", "items": { "type": "string" }, "description": "按钮配置（show_modal可选）" },
                    "state": { "type": "string", "enum": ["minimized", "maximized", "fullscreen", "normal"], "description": "窗口状态（set_window_state需要）" }
                },
                "required": ["action"]
            }),
            _ => serde_json::json!({
                "type": "object",
                "properties": {},
                "description": format!("工具 '{}' 的参数定义", tool_name)
            }),
        }
    }

    /// 获取默认工具列表（20个标准工具）
    fn get_default_tools() -> Vec<AiTool> {
        vec![
            AiTool { name: "filesystem".to_string(), description: "文件系统操作：读取、写入、编辑、删除、复制、移动文件和目录".to_string(), parameters: Self::get_tool_parameters("filesystem") },
            AiTool { name: "search".to_string(), description: "在文件中搜索内容，支持正则表达式".to_string(), parameters: Self::get_tool_parameters("search") },
            AiTool { name: "bash".to_string(), description: "执行终端命令（Bash/PowerShell）".to_string(), parameters: Self::get_tool_parameters("bash") },
            AiTool { name: "git_helper".to_string(), description: "Git版本控制操作：提交、推送、拉取、分支管理等".to_string(), parameters: Self::get_tool_parameters("git_helper") },
            AiTool { name: "network".to_string(), description: "网络请求操作：HTTP GET/POST/PUT/DELETE、文件下载上传".to_string(), parameters: Self::get_tool_parameters("network") },
            AiTool { name: "system".to_string(), description: "系统信息获取：CPU、内存、磁盘、进程、环境变量".to_string(), parameters: Self::get_tool_parameters("system") },
            AiTool { name: "plan".to_string(), description: "任务计划管理：创建计划、添加步骤、追踪进度".to_string(), parameters: Self::get_tool_parameters("plan") },
            AiTool { name: "todolist".to_string(), description: "待办事项管理：添加、完成、删除、更新待办".to_string(), parameters: Self::get_tool_parameters("todolist") },
            AiTool { name: "agent_skills".to_string(), description: "Agent技能管理：discover=扫描可用技能，list=列出已发现的技能，load=加载技能完整内容，execute=执行技能，search=搜索技能。技能是可复用的代码模块，可以自动发现和调用。你应该主动使用discover发现新技能，并在合适的时候execute执行它们。".to_string(), parameters: Self::get_tool_parameters("agent_skills") },
            AiTool { name: "agent_creator".to_string(), description: "Agent创建管理：创建、更新、删除、克隆Agent".to_string(), parameters: Self::get_tool_parameters("agent_creator") },
            AiTool { name: "agent_document".to_string(), description: "读取或更新自己的身份文档（SOUL.md, MEMORY.md, AGENTS.md 等）。用 update 更新 MEMORY.md 来跨会话记忆重要信息。这是你的长期记忆系统，可以记录重要的用户偏好、项目信息、学到的知识等。".to_string(), parameters: Self::get_tool_parameters("agent_document") },
            AiTool { name: "tool_creation".to_string(), description: "动态工具创建：创建、更新、删除自定义工具".to_string(), parameters: Self::get_tool_parameters("tool_creation") },
            AiTool { name: "rollback".to_string(), description: "文件快照与回滚：创建快照、恢复到之前状态".to_string(), parameters: Self::get_tool_parameters("rollback") },
            AiTool { name: "pubsub".to_string(), description: "发布订阅消息系统：发布消息、订阅主题、创建和管理群聊".to_string(), parameters: Self::get_tool_parameters("pubsub") },
            AiTool { name: "message_passing".to_string(), description: "消息队列：发送和接收异步消息".to_string(), parameters: Self::get_tool_parameters("message_passing") },
            AiTool { name: "iroh".to_string(), description: "Iroh P2P文件传输和群聊：分享文件、创建P2P群聊、管理成员".to_string(), parameters: Self::get_tool_parameters("iroh") },
            AiTool { name: "ipfs_archive".to_string(), description: "IPFS归档管理：归档文件到IPFS、提取、固定".to_string(), parameters: Self::get_tool_parameters("ipfs_archive") },
            AiTool { name: "browser".to_string(), description: "浏览器自动化：导航、点击、输入、截图、执行JS".to_string(), parameters: Self::get_tool_parameters("browser") },
            AiTool { name: "ui_control".to_string(), description: "UI控制：显示通知、更新状态、打开对话框".to_string(), parameters: Self::get_tool_parameters("ui_control") },
        ]
    }

    /// 从系统提示词中提取指定文档段落
    fn extract_section_from_prompt(prompt: &str, doc_type: &str) -> Option<String> {
        let marker = format!("=== {} ===", doc_type.to_uppercase());
        let start = prompt.find(&marker)?;
        let content_start = start + marker.len();
        let remaining = &prompt[content_start..];
        // 下一个 === 段落开始（或字符串末尾）
        let end = remaining.find("\n=== ").unwrap_or(remaining.len());
        let content = remaining[..end].trim().to_string();
        if content.is_empty() { None } else { Some(content) }
    }

    /// 处理 agent_document 工具调用（内联，不经过 ToolBridge）
    async fn execute_agent_document(&self, task_id: &str, tool_call: &ProviderToolCall) -> super::error::Result<ToolResult> {
        let action = tool_call.arguments.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let doc_type = tool_call.arguments.get("document_type").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "read" => {
                // 从文件读取文档（而不是从系统提示词提取）
                let task = self.task_manager.get_task(task_id).await;
                let agent_id = task.map(|t| t.metadata.agent_id.clone()).unwrap_or_default();
                
                if agent_id.is_empty() {
                    return Ok(ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        success: false,
                        data: None,
                        error: Some("无法获取 agent_id".to_string()),
                    });
                }
                
                // 直接从文件读取文档
                let mut content: Option<String> = None;
                if let Some(app) = &self.app_handle {
                    if let Ok(app_data_dir) = app.path().app_data_dir() {
                        let doc_path = std::path::PathBuf::from(&app_data_dir)
                            .join("agent-documents")
                            .join(&agent_id)
                            .join(format!("{}.md", doc_type.to_uppercase()));
                        
                        log::info!("[RalphLoop] 尝试读取文档: {:?}", doc_path);
                        
                        // 自动创建目录和初始文档（如果不存在）
                        if !doc_path.exists() {
                            if let Some(parent) = doc_path.parent() {
                                if let Err(e) = std::fs::create_dir_all(parent) {
                                    log::warn!("[RalphLoop] 创建文档目录失败: {}", e);
                                } else {
                                    // 写入初始内容
                                    let initial_content = Self::generate_initial_document_content(doc_type, &agent_id);
                                    if let Err(e) = std::fs::write(&doc_path, &initial_content) {
                                        log::warn!("[RalphLoop] 创建初始文档失败: {}", e);
                                    } else {
                                        log::info!("[RalphLoop] 自动创建文档: {:?}", doc_path);
                                        content = Some(initial_content);
                                    }
                                }
                            }
                        } else {
                            // 文件存在，读取内容
                            match tokio::fs::read_to_string(&doc_path).await {
                                Ok(text) => {
                                    content = Some(text);
                                },
                                Err(e) => {
                                    log::warn!("[RalphLoop] 读取文档失败: {}", e);
                                }
                            }
                        }
                    }
                };

                match content {
                    Some(text) => {
                        let len = text.len();
                        log::info!("[RalphLoop] agent_document read: 已从文件读取 {} 文档，长度: {}", doc_type, len);
                        Ok(ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            success: true,
                            data: Some(serde_json::json!({ "document_type": doc_type, "content": text })),
                            error: None,
                        })
                    }
                    None => {
                        log::warn!("[RalphLoop] agent_document read: 未找到文档 '{}' 或读取失败", doc_type);
                        Ok(ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            success: false,
                            data: None,
                            error: Some(format!("文档 '{}' 未找到或读取失败", doc_type)),
                        })
                    }
                }
            }
            "update" => {
                let new_content = tool_call.arguments.get("new_content")
                    .and_then(|v| v.as_str()).unwrap_or("");
                let reason = tool_call.arguments.get("reason")
                    .and_then(|v| v.as_str()).unwrap_or("");

                if new_content.is_empty() {
                    return Ok(ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        success: false,
                        data: None,
                        error: Some("update 操作需要提供 new_content 字段".to_string()),
                    });
                }

                // 发送 document:updated 事件到前端，前端负责持久化到 agentStore
                if let Some(app) = &self.app_handle {
                    let payload = serde_json::json!({
                        "document_type": doc_type,
                        "new_content": new_content,
                        "reason": reason,
                    });
                    match app.emit("document:updated", &payload) {
                        Ok(_) => log::info!("[RalphLoop] 已发送 document:updated 事件，文档类型: {}, 原因: {}", doc_type, reason),
                        Err(e) => log::warn!("[RalphLoop] 发送 document:updated 事件失败: {}", e),
                    }
                }

                Ok(ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "document_type": doc_type,
                        "status": "updated",
                        "message": format!("文档 '{}' 已发送更新请求，将由前端持久化", doc_type),
                        "reason": reason,
                    })),
                    error: None,
                })
            }
            _ => Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: false,
                data: None,
                error: Some(format!("agent_document: 未知的 action '{}'，支持 read / update", action)),
            }),
        }
    }

    /// 执行工具调用
    async fn execute_tools(
        &self,
        task_id: &str,
        tool_calls: &[ProviderToolCall],
    ) -> super::error::Result<Vec<ToolResult>> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            // 发送工具执行开始事件（包含参数）
            self.emit_event(task_id, TaskEvent::ToolExecuting {
                task_id: task_id.to_string(),
                tool_name: tool_call.name.clone(),
                arguments: Some(tool_call.arguments.clone()),
            })
            .await;

            // agent_document 工具内联处理，不经过 ToolBridge
            let result = if tool_call.name == "agent_document" {
                self.execute_agent_document(task_id, tool_call).await?
            } else {
                self.execute_single_tool(tool_call).await?
            };

            // 发送工具执行完成事件
            self.emit_event(task_id, TaskEvent::ToolCompleted {
                task_id: task_id.to_string(),
                tool_name: tool_call.name.clone(),
                result: result.summary(),
            })
            .await;

            results.push(result);
        }

        Ok(results)
    }

    /// 执行单个工具
    async fn execute_single_tool(&self, tool_call: &ProviderToolCall) -> super::error::Result<ToolResult> {
        // 添加详细日志
        log::info!("[RalphLoop] 执行工具：{}", tool_call.name);
        log::info!("[RalphLoop] 工具参数：{}", serde_json::to_string(&tool_call.arguments).unwrap_or_default());
        
        let request = ToolCallRequest {
            session_id: "ralph_loop".to_string(),
            user_id: None,
            tool_id: tool_call.name.clone(),
            args: tool_call.arguments.clone(),
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            environment: std::env::vars().collect(),
            timeout_seconds: Some(300), // 5 分钟超时
            permissions: vec![
                "read".to_string(),
                "write".to_string(),
                "execute".to_string(),
            ],
        };

        match self.tool_bridge.handle_request(request).await {
            Ok(ToolCallResponse { success, result, error }) => {
                // ── 如果是 agent_creator create 成功，通知前端更新侧边栏 ──
                if success && tool_call.name == "agent_creator" {
                    let action = tool_call.arguments
                        .get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if action == "create" {
                        let display_name = tool_call.arguments
                            .get("display_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("New Agent")
                            .to_string();
                        let description = tool_call.arguments
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if let Some(app) = &self.app_handle {
                            let payload = serde_json::json!({
                                "name": display_name,
                                "role_description": description,
                            });
                            if let Err(e) = app.emit("agent:created", &payload) {
                                log::warn!("[RalphLoop] 发送 agent:created 事件失败: {}", e);
                            } else {
                                log::info!("[RalphLoop] 已发送 agent:created 事件，名称: {}", display_name);
                            }
                        }
                    }
                }
                Ok(ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success,
                    data: result.and_then(|r| Some(r.data)),
                    error,
                })
            }
            Err(e) => Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: false,
                data: None,
                error: Some(e.to_string()),
            }),
        }
    }

    /// 添加工具结果到任务
    async fn add_tool_results_to_task(
        &self,
        task_id: &str,
        tool_calls: &[ProviderToolCall],
        results: &[ToolResult],
    ) -> std::result::Result<(), ExecutorError> {
        self.task_manager
            .update_task(task_id, |task| {
                // 保存工具结果
                for result in results {
                    task.tool_results.push(result.clone());
                }

                // 将 AI 的工具调用消息添加到历史
                let tool_calls_message = AiMessage::assistant_with_tools(
                    "".to_string(),
                    tool_calls
                        .iter()
                        .map(|tc| ProviderToolCall {
                            id: tc.id.clone(),
                            name: tc.name.clone(),
                            arguments: tc.arguments.clone(),
                        })
                        .collect(),
                );
                task.messages.push(tool_calls_message);

                // 将工具结果消息添加到历史
                for (tool_call, result) in tool_calls.iter().zip(results.iter()) {
                    let content = if result.success {
                        serde_json::to_string(&result.data).unwrap_or_else(|_| String::new())
                    } else {
                        result.error.clone().unwrap_or_else(|| String::new())
                    };
                    task.messages.push(AiMessage::tool_result(
                        tool_call.id.clone(),
                        content,
                    ));
                }
            })
            .await
            .map_err(|e| ExecutorError::InternalError(e.to_string()))
    }

    /// 处理任务完成
    async fn handle_completion(&self, task_id: &str, result: &str) -> std::result::Result<(), ExecutorError> {
        self.task_manager
            .update_task(task_id, |task| {
                task.status = TaskStatus::Completed;
                task.final_response = Some(result.to_string());
            })
            .await
            .map_err(|e| ExecutorError::InternalError(e.to_string()))?;

        self.emit_event(task_id, TaskEvent::TaskCompleted {
            task_id: task_id.to_string(),
            result: result.to_string(),
        })
        .await;

        Ok(())
    }

    /// 处理错误
    async fn handle_error(&self, task_id: &str, error: &str) -> std::result::Result<(), ExecutorError> {
        self.task_manager
            .update_task(task_id, |task| {
                task.status = TaskStatus::Failed;
                task.error = Some(error.to_string());
            })
            .await
            .map_err(|e| ExecutorError::InternalError(e.to_string()))?;

        self.emit_event(task_id, TaskEvent::TaskFailed {
            task_id: task_id.to_string(),
            error: error.to_string(),
        })
        .await;

        Ok(())
    }

    /// 发送事件（同时到内部 broadcast channel 和 Tauri 前端）
    async fn emit_event(&self, task_id: &str, event: TaskEvent) {
        // 通过 task_manager 的事件通道发送（内部订阅者使用）
        self.task_manager.emit_event(event.clone()).await;

        // 如果有 AppHandle，将事件转换为前端格式并发送
        if let Some(app) = &self.app_handle {
            let payload = self.task_event_to_progress(task_id, event);
            if let Some(p) = payload {
                if let Err(e) = app.emit("agent:progress", &p) {
                    log::warn!("[RalphLoop] 发送 Tauri 事件失败: {}", e);
                }
            }
        }
    }

    /// 将内部 TaskEvent 转换为前端可用的进度事件
    fn task_event_to_progress(&self, task_id: &str, event: TaskEvent) -> Option<serde_json::Value> {
        let task_id = task_id.to_string();
        match event {
            TaskEvent::TaskStarted { .. } => Some(serde_json::json!({
                "type": "started",
                "task_id": task_id,
            })),
            TaskEvent::AiResponse { content, .. } => {
                // 获取当前迭代数（用于前端显示）
                Some(serde_json::json!({
                    "type": "thinking",
                    "task_id": task_id,
                    "content": content,
                }))
            }
            TaskEvent::ToolExecuting { tool_name, .. } => Some(serde_json::json!({
                "type": "tool_calling",
                "task_id": task_id,
                "tool_name": tool_name,
            })),
            TaskEvent::ToolCompleted { tool_name, result, .. } => Some(serde_json::json!({
                "type": "tool_done",
                "task_id": task_id,
                "tool_name": tool_name,
                "success": result.success,
                "preview": result.data_preview,
                "error": result.error,
            })),
            TaskEvent::TaskCompleted { result, .. } => Some(serde_json::json!({
                "type": "completed",
                "task_id": task_id,
                "result": result,
            })),
            TaskEvent::TaskFailed { error, .. } => Some(serde_json::json!({
                "type": "failed",
                "task_id": task_id,
                "error": error,
            })),
            TaskEvent::ToolCallsPending { count, .. } => Some(serde_json::json!({
                "type": "tools_pending",
                "task_id": task_id,
                "count": count,
            })),
            _ => None,
        }
    }
}
