//! Ralph Loop 执行器
//!
//! 实现 Ralph Loop 的核心执行逻辑，支持无限迭代、工具调用、流式响应

use super::ai_client::{AiClient, AiMessage, AiTool, AiToolCall, AiToolCall as ProviderToolCall};
use super::task::{Task, TaskManager, TaskStatus, TaskEvent, ToolCall, ToolResult, TaskFinalResult};
use super::error::AgentError;
use crate::bridges::{ToolBridge, ToolCallRequest, ToolCallResponse};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Ralph Loop 执行器
pub struct RalphLoopExecutor {
    ai_client: Arc<AiClient>,
    task_manager: Arc<TaskManager>,
    tool_bridge: Arc<ToolBridge>,
    tool_registry: Arc<crate::tools::ToolRegistry>,
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
                    "command": { "type": "string", "description": "要执行的命令" },
                    "working_dir": { "type": "string", "description": "工作目录" },
                    "timeout": { "type": "integer", "description": "超时时间（秒）", "default": 300 }
                },
                "required": ["command"]
            }),
            "search" => serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "搜索模式" },
                    "path": { "type": "string", "description": "搜索路径" },
                    "file_pattern": { "type": "string", "description": "文件匹配模式" }
                },
                "required": ["pattern", "path"]
            }),
            "git_helper" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["status", "add", "commit", "push", "pull", "branch", "checkout", "log", "diff"], "description": "Git操作" },
                    "message": { "type": "string", "description": "提交信息" },
                    "branch_name": { "type": "string", "description": "分支名称" },
                    "files": { "type": "array", "items": { "type": "string" }, "description": "文件列表" }
                },
                "required": ["operation"]
            }),
            "network" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["http_get", "http_post", "http_put", "http_delete", "download", "upload"], "description": "网络操作" },
                    "url": { "type": "string", "description": "URL地址" },
                    "headers": { "type": "object", "description": "请求头" },
                    "body": { "type": "string", "description": "请求体" },
                    "save_path": { "type": "string", "description": "保存路径" }
                },
                "required": ["operation", "url"]
            }),
            "system" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["info", "memory", "disk", "cpu", "processes", "env"], "description": "系统操作" },
                    "process_name": { "type": "string", "description": "进程名称" }
                },
                "required": ["operation"]
            }),
            "plan" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["create", "add_step", "update_step", "complete_step", "get_plan", "list_plans"], "description": "计划操作" },
                    "plan_id": { "type": "string", "description": "计划ID" },
                    "title": { "type": "string", "description": "计划标题" },
                    "description": { "type": "string", "description": "计划描述" },
                    "step_id": { "type": "string", "description": "步骤ID" },
                    "step_description": { "type": "string", "description": "步骤描述" }
                },
                "required": ["operation"]
            }),
            "todolist" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["add", "complete", "remove", "list", "update"], "description": "待办操作" },
                    "item_id": { "type": "string", "description": "项目ID" },
                    "title": { "type": "string", "description": "标题" },
                    "description": { "type": "string", "description": "描述" },
                    "priority": { "type": "string", "enum": ["low", "medium", "high"], "description": "优先级" }
                },
                "required": ["operation"]
            }),
            "agent_skills" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["list", "execute", "get_info"], "description": "技能操作" },
                    "skill_name": { "type": "string", "description": "技能名称" },
                    "parameters": { "type": "object", "description": "执行参数" }
                },
                "required": ["operation"]
            }),
            "agent_collaboration" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["create_session", "join_session", "leave_session", "send_message", "get_messages", "list_sessions"], "description": "协作操作" },
                    "session_id": { "type": "string", "description": "会话ID" },
                    "agent_id": { "type": "string", "description": "Agent ID" },
                    "message": { "type": "string", "description": "消息内容" }
                },
                "required": ["operation"]
            }),
            "agent_creator" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["create", "update", "delete", "get", "list", "clone"], "description": "创建器操作" },
                    "agent_id": { "type": "string", "description": "Agent ID" },
                    "name": { "type": "string", "description": "名称" },
                    "description": { "type": "string", "description": "描述" },
                    "skills": { "type": "array", "items": { "type": "string" }, "description": "技能列表" },
                    "config": { "type": "object", "description": "配置" }
                },
                "required": ["operation"]
            }),
            "tool_creation" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["create_tool", "update_tool", "delete_tool", "get_tool", "list_tools", "execute_tool"], "description": "工具创建操作" },
                    "tool_id": { "type": "string", "description": "工具ID" },
                    "name": { "type": "string", "description": "名称" },
                    "description": { "type": "string", "description": "描述" },
                    "code": { "type": "string", "description": "代码" },
                    "parameters": { "type": "object", "description": "参数" }
                },
                "required": ["operation"]
            }),
            "rollback" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["create_snapshot", "list_snapshots", "rollback", "delete_snapshot"], "description": "回滚操作" },
                    "snapshot_id": { "type": "string", "description": "快照ID" },
                    "path": { "type": "string", "description": "路径" },
                    "description": { "type": "string", "description": "描述" }
                },
                "required": ["operation"]
            }),
            "pubsub" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["publish", "subscribe", "unsubscribe", "list_topics"], "description": "PubSub操作" },
                    "topic": { "type": "string", "description": "主题" },
                    "message": { "type": "string", "description": "消息内容" }
                },
                "required": ["operation"]
            }),
            "message_passing" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["send", "receive", "list_queues"], "description": "消息操作" },
                    "queue": { "type": "string", "description": "队列名称" },
                    "message": { "type": "string", "description": "消息内容" }
                },
                "required": ["operation"]
            }),
            "iroh" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["share_file", "get_file", "list_shares"], "description": "Iroh操作" },
                    "ticket": { "type": "string", "description": "票据" },
                    "path": { "type": "string", "description": "文件路径" }
                },
                "required": ["operation"]
            }),
            "ipfs_archive" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["archive", "extract", "list", "pin"], "description": "IPFS归档操作" },
                    "cid": { "type": "string", "description": "CID" },
                    "path": { "type": "string", "description": "路径" }
                },
                "required": ["operation"]
            }),
            "browser" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["navigate", "click", "type", "screenshot", "get_content", "execute_script"], "description": "浏览器操作" },
                    "url": { "type": "string", "description": "URL" },
                    "selector": { "type": "string", "description": "CSS选择器" },
                    "text": { "type": "string", "description": "文本" },
                    "script": { "type": "string", "description": "JavaScript代码" }
                },
                "required": ["operation"]
            }),
            "ui_control" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["show_notification", "update_status", "open_dialog", "close_dialog"], "description": "UI控制操作" },
                    "title": { "type": "string", "description": "标题" },
                    "message": { "type": "string", "description": "消息" },
                    "type": { "type": "string", "enum": ["info", "success", "warning", "error"], "description": "类型" }
                },
                "required": ["operation"]
            }),
            _ => serde_json::json!({
                "type": "object",
                "properties": {},
                "description": format!("工具 '{}' 的参数定义", tool_name)
            }),
        }
    }

    /// 获取默认工具列表（19个标准工具）
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
            AiTool { name: "agent_skills".to_string(), description: "Agent技能管理：查看可用技能、执行技能".to_string(), parameters: Self::get_tool_parameters("agent_skills") },
            AiTool { name: "agent_collaboration".to_string(), description: "多Agent协作：创建会话、发送消息、协同工作".to_string(), parameters: Self::get_tool_parameters("agent_collaboration") },
            AiTool { name: "agent_creator".to_string(), description: "Agent创建管理：创建、更新、删除、克隆Agent".to_string(), parameters: Self::get_tool_parameters("agent_creator") },
            AiTool { name: "tool_creation".to_string(), description: "动态工具创建：创建、更新、删除自定义工具".to_string(), parameters: Self::get_tool_parameters("tool_creation") },
            AiTool { name: "rollback".to_string(), description: "文件快照与回滚：创建快照、恢复到之前状态".to_string(), parameters: Self::get_tool_parameters("rollback") },
            AiTool { name: "pubsub".to_string(), description: "发布订阅消息系统：发布消息、订阅主题".to_string(), parameters: Self::get_tool_parameters("pubsub") },
            AiTool { name: "message_passing".to_string(), description: "消息队列：发送和接收异步消息".to_string(), parameters: Self::get_tool_parameters("message_passing") },
            AiTool { name: "iroh".to_string(), description: "Iroh P2P文件传输：分享和获取文件".to_string(), parameters: Self::get_tool_parameters("iroh") },
            AiTool { name: "ipfs_archive".to_string(), description: "IPFS归档管理：归档文件到IPFS、提取、固定".to_string(), parameters: Self::get_tool_parameters("ipfs_archive") },
            AiTool { name: "browser".to_string(), description: "浏览器自动化：导航、点击、输入、截图、执行JS".to_string(), parameters: Self::get_tool_parameters("browser") },
            AiTool { name: "ui_control".to_string(), description: "UI控制：显示通知、更新状态、打开对话框".to_string(), parameters: Self::get_tool_parameters("ui_control") },
        ]
    }

    /// 执行工具调用
    async fn execute_tools(
        &self,
        task_id: &str,
        tool_calls: &[ProviderToolCall],
    ) -> super::error::Result<Vec<ToolResult>> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            // 发送工具执行开始事件
            self.emit_event(task_id, TaskEvent::ToolExecuting {
                task_id: task_id.to_string(),
                tool_name: tool_call.name.clone(),
            })
            .await;

            // 执行工具
            let result = self.execute_single_tool(tool_call).await?;

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

    /// 发送事件
    async fn emit_event(&self, _task_id: &str, event: TaskEvent) {
        // 通过 task_manager 的事件通道发送
        self.task_manager.emit_event(event).await;
    }
}
