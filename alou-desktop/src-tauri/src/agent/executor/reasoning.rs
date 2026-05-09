//! 推理层模块
//!
//! 负责：
//! - 构建增强提示词（结合 Perception 上下文）
//! - 调用 LLM 进行推理
//! - 解析输出为 Thought

use std::sync::Arc;

use serde_json::Value;

use crate::agent::ai_client::{AiClient, AiMessage, AiTool};
use crate::agent::executor::types::{
    EnvironmentState, Thought, Action, Reflection, InformationAssessment, TaskProgress,
};
use crate::agent::perception::Intent;
use crate::tools::facade::ToolFacade;

/// 推理层
pub struct ReasoningLayer {
    ai_client: Arc<AiClient>,
    /// 🔥 统一工具访问入口（包含 ToolRegistry + ToolBus）
    tool_facade: Option<Arc<ToolFacade>>,
}

impl ReasoningLayer {
    /// 创建推理层实例
    pub fn new(ai_client: Arc<AiClient>) -> Self {
        Self { ai_client, tool_facade: None }
    }

    /// 🔥 创建带 ToolFacade 的推理层实例
    pub fn new_with_tool_facade(ai_client: Arc<AiClient>, tool_facade: Arc<ToolFacade>) -> Self {
        Self { 
            ai_client, 
            tool_facade: Some(tool_facade),
        }
    }

    /// 推理（完整流程）
    pub async fn reason(
        &self,
        state: &EnvironmentState,
        intent: Intent,
        tool_registry: Arc<crate::tools::ToolRegistry>,
    ) -> Result<Thought, crate::agent::executor::types::ExecutorError> {
        // 🔥 1. 优先使用前端传入的系统提示，否则构建增强提示词
        let system_prompt = if let Some(ref custom_prompt) = state.system_prompt {
            log::info!("[ReasoningLayer] 使用前端传入的系统提示，长度: {}", custom_prompt.len());
            custom_prompt.clone()
        } else {
            log::info!("[ReasoningLayer] 使用后端构建的系统提示");
            self.build_reasoning_prompt(state, intent, tool_registry.clone()).await
        };

        // 2. 准备消息 - system prompt 作为第一条消息
        let mut messages = vec![AiMessage {
            role: "system".to_string(),
            content: system_prompt,
            tool_call_id: None,
            tool_calls: None,
        }];
        messages.extend(state.messages.clone());

        // 3. 获取工具定义
        let tools = self.get_tools_for_llm(&state.available_tools, tool_registry).await;

        // 4. 调用 LLM
        let response = self.ai_client
            .send_message(messages, if tools.is_empty() { None } else { Some(tools) })
            .await
            .map_err(|e| crate::agent::executor::types::ExecutorError::AiError(e.to_string()))?;

        // 5. 解析响应
        let thought = self.parse_response(&response.content, &response.tool_calls)?;

        Ok(thought)
    }

    /// 快速推理（用于简单场景）
    pub async fn reason_simple(
        &self,
        state: &EnvironmentState,
    ) -> Result<Thought, crate::agent::executor::types::ExecutorError> {
        let system_prompt = r#"你是一个 AI 助手。请分析对话并提供简洁的回复。"#;

        let mut messages = vec![AiMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
            tool_call_id: None,
            tool_calls: None,
        }];
        messages.extend(state.messages.clone());

        let response = self.ai_client
            .send_message(messages, None)
            .await
            .map_err(|e| crate::agent::executor::types::ExecutorError::AiError(e.to_string()))?;

        // 简单响应转 Thought
        Ok(Thought {
            analysis: "简单响应".to_string(),
            reflection: None,
            action: Action::Complete(response.content),
        })
    }

    /// 构建推理提示词（增强版）
    async fn build_reasoning_prompt(
        &self,
        state: &EnvironmentState,
        intent: Intent,
        tool_registry: Arc<crate::tools::ToolRegistry>,
    ) -> String {
        let base_prompt = self.get_base_prompt();
        let tools_desc = self.get_tools_description(&state.available_tools, tool_registry).await;
        let context_desc = self.get_context_description(state);
        let decision_guidelines = self.get_decision_guidelines(intent);

        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            base_prompt, tools_desc, context_desc, decision_guidelines
        )
    }

    /// 获取基础提示词
    fn get_base_prompt(&self) -> String {
        r#"
# AI Agent 推理系统

你是一个智能 Agent，需要完成用户任务。请遵循以下决策流程：

## 决策流程

1. **信息充分性评估**（必须）
   - 判断是否有足够信息执行下一步
   - 如果信息不足，选择 ask_user 或 manage_goal

2. **任务进度评估**（必须）
   - 评估当前完成进度 (0.0-1.0)，不显示，自己记录
   - 列出已完成和待处理的步骤

3. **决策置信度**（必须）
   - 低于 0.6 时建议 gather_more 或 ask_user

4. **选择行动**
   - 如果信息充足且任务可完成 → 使用工具或完成
   - 如果需要规划复杂任务 → 创建目标
   - 如果需要更多信息 → 询问用户

## 🔥 自修复能力（核心特性）

你是 Alou —— 一个**可以修改自身代码**的智能体。你可以读取、编辑、构建和重启自己的源代码。

### 自修复循环
当需要修复自身 bug 或改进功能时，按以下循环操作：

1. **检测问题**：调用 `self_repair_check` 获取编译错误
2. **定位代码**：使用 `filesystem` (read) 读取出错文件，或 `self_repair_project_info` 了解项目结构
3. **修复代码**：使用 `filesystem` (edit/write) 修改源文件
4. **验证修复**：再次调用 `self_repair_check` 确认错误已消除
5. **构建项目**：调用 `self_repair_build` 编译项目
6. **重启生效**：调用 `self_repair_restart` 重启应用加载新代码

### 自修复规则
- 修改代码前，先用 `rollback` 工具创建快照（保护性措施）
- 每次只修一个错误，修完立即验证
- 如果修复引入新错误，用 `rollback` 恢复
- 构建成功后，必须通过 `self_repair_restart` 才能生效
- 前端文件 (.tsx/.ts) 修改后 Vite 会自动热重载，无需重启
- Rust 文件 (.rs) 修改后必须 rebuild + restart 才能生效

### 可修改的项目
- Rust 后端：`src-tauri/src/` 下所有 .rs 文件
- 前端 UI：`src/` 下所有 .tsx/.ts/.css 文件
- 工具定义：`src-tauri/src/tools/` 下的工具实现
- Agent 逻辑：`src-tauri/src/agent/` 下的推理/感知/行动层

## 响应格式

请直接返回你的分析和决策，如果需要调用工具请使用 function calling。
"#.to_string()
    }

    /// 获取工具定义（用于 LLM function calling）
    async fn get_tools_for_llm(
        &self,
        _tool_names: &[String],
        tool_registry: Arc<crate::tools::ToolRegistry>,
    ) -> Vec<AiTool> {
        // 🔥 优先使用 ToolFacade（统一入口，包含 ToolRegistry + ToolBus）
        if let Some(ref facade) = self.tool_facade {
            let all_tools = facade.list_tools().await;
            return all_tools.into_iter().map(|tool| {
                let parameters = Self::get_tool_parameters(&tool.name);
                AiTool {
                    name: tool.name,
                    description: tool.description,
                    parameters,
                }
            }).collect();
        }

        // Fallback: 从 ToolRegistry 获取工具
        let mut tools = Vec::new();
        let registry_tools = tool_registry.list_all().await;
        for tool in registry_tools {
            tools.push(AiTool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: Self::get_tool_parameters(&tool.name),
            });
        }

        // 如果 registry 为空，使用默认工具列表
        if tools.is_empty() {
            tools = Self::get_default_tools();
        }

        tools
    }

    /// 获取默认工具列表（20+ 标准工具）
    fn get_default_tools() -> Vec<AiTool> {
        vec![
            AiTool {
                name: "filesystem".to_string(),
                description: "文件系统操作：读取、写入、编辑、删除、复制、移动文件和目录".to_string(),
                parameters: Self::get_tool_parameters("filesystem"),
            },
            AiTool {
                name: "bash".to_string(),
                description: "执行终端命令（Bash/PowerShell）".to_string(),
                parameters: Self::get_tool_parameters("bash"),
            },
            AiTool {
                name: "search".to_string(),
                description: "在文件中搜索内容，支持正则表达式".to_string(),
                parameters: Self::get_tool_parameters("search"),
            },
            AiTool {
                name: "git_helper".to_string(),
                description: "Git 版本控制操作：提交、推送、拉取、分支管理等".to_string(),
                parameters: Self::get_tool_parameters("git_helper"),
            },
            AiTool {
                name: "network".to_string(),
                description: "网络请求操作：HTTP GET/POST/PUT/DELETE、文件下载上传".to_string(),
                parameters: Self::get_tool_parameters("network"),
            },
            AiTool {
                name: "system".to_string(),
                description: "系统信息获取：CPU、内存、磁盘、进程、环境变量".to_string(),
                parameters: Self::get_tool_parameters("system"),
            },
            AiTool {
                name: "plan".to_string(),
                description: "任务计划管理：创建计划、添加步骤、追踪进度".to_string(),
                parameters: Self::get_tool_parameters("plan"),
            },
            AiTool {
                name: "todolist".to_string(),
                description: "待办事项管理：添加、完成、删除、更新待办".to_string(),
                parameters: Self::get_tool_parameters("todolist"),
            },
            AiTool {
                name: "agent_skills".to_string(),
                description: "Agent 技能管理：discover=扫描可用技能，list=列出已发现的技能，load=加载技能完整内容，execute=执行技能，search=搜索技能".to_string(),
                parameters: Self::get_tool_parameters("agent_skills"),
            },
            AiTool {
                name: "agent_creator".to_string(),
                description: "Agent 创建管理：创建、更新、删除、克隆 Agent".to_string(),
                parameters: Self::get_tool_parameters("agent_creator"),
            },
            AiTool {
                name: "agent_document".to_string(),
                description: "读取或更新自己的身份文档（SOUL.md, MEMORY.md, AGENTS.md 等）".to_string(),
                parameters: Self::get_tool_parameters("agent_document"),
            },
            AiTool {
                name: "tool_creation".to_string(),
                description: "动态工具创建：创建、更新、删除自定义工具".to_string(),
                parameters: Self::get_tool_parameters("tool_creation"),
            },
            AiTool {
                name: "rollback".to_string(),
                description: "文件快照与回滚：创建快照、恢复到之前状态".to_string(),
                parameters: Self::get_tool_parameters("rollback"),
            },
            AiTool {
                name: "pubsub".to_string(),
                description: "发布订阅消息系统：发布消息、订阅主题、创建和管理群聊".to_string(),
                parameters: Self::get_tool_parameters("pubsub"),
            },
            AiTool {
                name: "message_passing".to_string(),
                description: "消息队列：发送和接收异步消息".to_string(),
                parameters: Self::get_tool_parameters("message_passing"),
            },
            AiTool {
                name: "iroh".to_string(),
                description: "Iroh P2P 文件传输和群聊：分享文件、创建 P2P 群聊、管理成员".to_string(),
                parameters: Self::get_tool_parameters("iroh"),
            },
            AiTool {
                name: "ipfs_archive".to_string(),
                description: "IPFS 归档管理：归档文件到 IPFS、提取、固定".to_string(),
                parameters: Self::get_tool_parameters("ipfs_archive"),
            },
            AiTool {
                name: "browser".to_string(),
                description: "浏览器自动化：导航、点击、输入、截图、执行 JS".to_string(),
                parameters: Self::get_tool_parameters("browser"),
            },
            AiTool {
                name: "ui_control".to_string(),
                description: "UI 控制：显示通知、更新状态、打开对话框".to_string(),
                parameters: Self::get_tool_parameters("ui_control"),
            },
            AiTool {
                name: "generate_image".to_string(),
                description: "根据文字描述生成图片，支持风景、人物、艺术创作等".to_string(),
                parameters: Self::get_tool_parameters("generate_image"),
            },
            AiTool {
                name: "generate_audio".to_string(),
                description: "将文字转换为语音 (TTS)，支持多种音色和语言".to_string(),
                parameters: Self::get_tool_parameters("generate_audio"),
            },
            AiTool {
                name: "generate_video".to_string(),
                description: "根据文字描述生成视频，支持动画、实景等风格".to_string(),
                parameters: Self::get_tool_parameters("generate_video"),
            },
            AiTool {
                name: "get_video_status".to_string(),
                description: "查询异步视频生成任务的状态".to_string(),
                parameters: Self::get_tool_parameters("get_video_status"),
            },
            // 🔥 自修复工具 — Alou 修改自身代码的完整闭环
            AiTool {
                name: "self_repair".to_string(),
                description: "自修复系统：检测编译错误、构建项目、重启应用。让 Alou 修改并验证自身代码的完整闭环".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["check", "build", "restart", "project_info", "full_cycle"],
                            "description": "操作类型：check=检测编译错误，build=构建项目，restart=请求重启，project_info=获取项目结构，full_cycle=完整修复循环"
                        },
                        "profile": { "type": "string", "enum": ["debug", "release"], "description": "构建 profile (build 操作，默认 debug)" },
                        "auto_restart": { "type": "boolean", "description": "full_cycle 中是否自动重启 (默认 false)" }
                    },
                    "required": ["action"]
                }),
            },
        ]
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
                    "shell": { "type": "string", "enum": ["bash", "cmd", "powershell", "python", "node"], "description": "Shell 类型", "default": "bash" },
                    "command": { "type": "string", "description": "要执行的命令" },
                    "working_dir": { "type": "string", "description": "工作目录（可选）" },
                    "environment": { "type": "array", "items": { "type": "array", "items": { "type": "string" }, "minItems": 2, "maxItems": 2 }, "description": "环境变量数组，格式：[[\"KEY\", \"VALUE\"]]", "default": [] },
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
                    "file_pattern": { "type": "string", "description": "文件匹配模式（grep 操作可选）" },
                    "case_sensitive": { "type": "boolean", "description": "是否区分大小写（grep 操作，默认 false）", "default": false },
                    "max_results": { "type": "integer", "description": "最大结果数（grep 操作可选）" },
                    "recursive": { "type": "boolean", "description": "是否递归搜索（glob 操作，默认 false）", "default": false },
                    "options": { "type": "object", "description": "高级查找选项（find 操作）" }
                },
                "required": ["operation", "pattern", "directory"]
            }),
            "git_helper" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["execute", "smart_commit", "create_feature_branch", "safe_merge", "status_check", "diff_summary", "log_history", "stash_management", "get_prompt", "batch_operation", "undo_operation", "remote_sync", "init_repository", "config_management"], "description": "Git 操作类型" },
                    "subcommand": { "type": "string", "description": "Git 子命令（execute 需要）" },
                    "args": { "type": "array", "items": { "type": "string" }, "description": "命令参数（execute 可选）" },
                    "working_dir": { "type": "string", "description": "工作目录（可选）" },
                    "message": { "type": "string", "description": "提交信息（smart_commit/stash_management 需要）" },
                    "add_all": { "type": "boolean", "description": "是否添加所有文件（smart_commit 可选）" },
                    "allow_empty": { "type": "boolean", "description": "是否允许空提交（smart_commit 可选）" },
                    "branch_name": { "type": "string", "description": "分支名称（create_feature_branch 需要）" },
                    "base_branch": { "type": "string", "description": "基础分支（create_feature_branch 可选）" },
                    "source_branch": { "type": "string", "description": "源分支（safe_merge 需要）" },
                    "strategy": { "type": "string", "description": "合并策略（safe_merge 可选，默认 merge）" },
                    "detailed": { "type": "boolean", "description": "是否详细输出（status_check 可选）" },
                    "target": { "type": "string", "description": "目标（diff_summary/undo_operation 可选）" },
                    "stat_only": { "type": "boolean", "description": "仅统计（diff_summary 可选）" },
                    "count": { "type": "integer", "description": "日志数量（log_history 可选，默认 10）" },
                    "branch": { "type": "string", "description": "分支名称（log_history/remote_sync 可选）" },
                    "format": { "type": "string", "description": "日志格式（log_history 可选，默认 oneline）" },
                    "operation": { "type": "string", "description": "操作类型（stash_management/remote_sync/config_management 需要）" },
                    "scenario": { "type": "string", "description": "场景（get_prompt 需要）" },
                    "context": { "type": "object", "description": "上下文（get_prompt 可选）" },
                    "operations": { "type": "array", "description": "批量操作列表（batch_operation 需要）" },
                    "undo_type": { "type": "string", "description": "撤销类型（undo_operation 需要）" },
                    "force": { "type": "boolean", "description": "是否强制（undo_operation 可选）" },
                    "remote": { "type": "string", "description": "远程仓库名（remote_sync 可选，默认 origin）" },
                    "path": { "type": "string", "description": "仓库路径（init_repository 需要）" },
                    "initial_branch": { "type": "string", "description": "初始分支名（init_repository 可选，默认 main）" },
                    "key": { "type": "string", "description": "配置键（config_management 需要）" },
                    "value": { "type": "string", "description": "配置值（config_management 可选）" },
                    "global": { "type": "boolean", "description": "是否全局配置（config_management 可选）" }
                },
                "required": ["action"]
            }),
            "network" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["HttpRequest", "DnsLookup", "Ping"], "description": "网络操作类型" },
                    "method": { "type": "string", "description": "HTTP 方法（HttpRequest 需要）：GET, POST, PUT, DELETE 等" },
                    "url": { "type": "string", "description": "URL 地址（HttpRequest 需要）" },
                    "headers": { "type": "object", "description": "请求头（HttpRequest 可选）" },
                    "body": { "type": "string", "description": "请求体（HttpRequest 可选）" },
                    "timeout": { "type": "integer", "description": "超时时间秒数（HttpRequest 可选）" },
                    "domain": { "type": "string", "description": "域名（DnsLookup 需要）" },
                    "host": { "type": "string", "description": "主机地址（Ping 需要）" },
                    "count": { "type": "integer", "description": "Ping 次数（Ping 可选，默认 4）" }
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
                    "name": { "type": "string", "description": "计划名称（create_plan 需要）" },
                    "description": { "type": "string", "description": "描述（create_plan/create_todo 需要）" },
                    "goal": { "type": "string", "description": "目标（create_plan 需要）" },
                    "steps": { "type": "array", "description": "步骤列表（create_plan 需要）" },
                    "plan_id": { "type": "string", "description": "计划 ID（add_step/update_step_status/get_plan/delete_plan/analyze_dependencies 需要）" },
                    "step_name": { "type": "string", "description": "步骤名称（add_step 需要）" },
                    "step_description": { "type": "string", "description": "步骤描述（add_step 需要）" },
                    "dependencies": { "type": "array", "items": { "type": "string" }, "description": "依赖步骤 ID 列表（add_step 可选）" },
                    "estimated_duration": { "type": "integer", "description": "预计时长分钟数（add_step 可选）" },
                    "resources": { "type": "array", "items": { "type": "string" }, "description": "资源列表（add_step 可选）" },
                    "step_id": { "type": "string", "description": "步骤 ID（update_step_status 需要）" },
                    "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "paused", "cancelled", "failed"], "description": "状态（update_step_status/update_todo_status 需要）" },
                    "title": { "type": "string", "description": "标题（create_todo 需要）" },
                    "priority": { "type": "string", "enum": ["low", "medium", "high", "urgent"], "description": "优先级（create_todo 可选）" },
                    "due_date": { "type": "integer", "description": "截止日期时间戳（create_todo 可选）" },
                    "todo_id": { "type": "string", "description": "待办 ID（update_todo_status/delete_todo 需要）" },
                    "status_filter": { "type": "string", "description": "状态过滤（list_todos 可选）" }
                },
                "required": ["action"]
            }),
            "todolist" => serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["Create", "Update", "Delete", "Get", "List", "Clear"], "description": "待办操作类型" },
                    "id": { "type": "string", "description": "待办事项 ID（Update/Delete/Get 需要）" },
                    "title": { "type": "string", "description": "标题（Create 需要，Update 可选）" },
                    "description": { "type": "string", "description": "描述（Create/Update 可选）" },
                    "priority": { "type": "string", "enum": ["low", "medium", "high"], "description": "优先级（Create/Update 可选）" },
                    "status": { "type": "string", "description": "状态（Update 可选）" },
                    "due_date": { "type": "integer", "description": "截止日期时间戳（Create/Update 可选）" },
                    "status_filter": { "type": "string", "description": "状态过滤（List/Clear 可选）" },
                    "priority_filter": { "type": "string", "description": "优先级过滤（List 可选）" }
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
                    "action": { "type": "string", "enum": ["create", "list", "get", "delete"], "description": "操作类型：create=创建新 Agent，list=列出所有 Agent，get=获取指定 Agent，delete=删除 Agent" },
                    "agent_id": { "type": "string", "description": "Agent ID（get/delete 操作需要）" },
                    "display_name": { "type": "string", "description": "Agent 名称（create 操作需要）" },
                    "description": { "type": "string", "description": "Agent 描述（create 操作可选）" },
                    "skills": { "type": "array", "items": { "type": "string" }, "description": "技能列表（create 操作可选）" },
                    "config": { "type": "object", "description": "额外配置（可选）" }
                },
                "required": ["action"]
            }),
            "agent_document" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["read", "update"], "description": "read=读取文档内容，update=更新文档内容" },
                    "document_type": { "type": "string", "enum": ["soul", "identity", "capabilities", "constraints", "tools", "memory", "agents"], "description": "要读取/更新的文档类型" },
                    "new_content": { "type": "string", "description": "新文档内容（Markdown 格式，update 时必填）" },
                    "reason": { "type": "string", "description": "更新原因（建议填写）" }
                },
                "required": ["action", "document_type"]
            }),
            "tool_creation" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create", "update", "delete", "get", "list", "execute"], "description": "工具创建操作类型" },
                    "tool_id": { "type": "string", "description": "工具 ID（update/delete/get/execute 需要）" },
                    "name": { "type": "string", "description": "工具名称（create 需要）" },
                    "description": { "type": "string", "description": "工具描述（create 需要）" },
                    "code": { "type": "string", "description": "工具代码（create/update 需要）" },
                    "parameters": { "type": "object", "description": "工具参数 schema（create/update 可选）" },
                    "args": { "type": "object", "description": "执行参数（execute 需要）" }
                },
                "required": ["action"]
            }),
            "rollback" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create_snapshot", "create_batch_snapshot", "list_snapshots", "restore_snapshot", "compare_snapshot", "delete_snapshot", "cleanup_snapshots"], "description": "回滚操作类型" },
                    "target_path": { "type": "string", "description": "目标路径（create_snapshot 需要）" },
                    "name": { "type": "string", "description": "快照名称（create_snapshot 需要）" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "标签（create_snapshot/list_snapshots 可选）" },
                    "target_paths": { "type": "array", "items": { "type": "string" }, "description": "多个目标路径（create_batch_snapshot 需要）" },
                    "name_prefix": { "type": "string", "description": "快照名称前缀（create_batch_snapshot 需要）" },
                    "session_id": { "type": "string", "description": "会话 ID 过滤（list_snapshots 可选）" },
                    "snapshot_id": { "type": "string", "description": "快照 ID（restore_snapshot/compare_snapshot/delete_snapshot 需要）" },
                    "restore_path": { "type": "string", "description": "恢复路径（restore_snapshot 可选）" },
                    "force": { "type": "boolean", "description": "是否强制覆盖（restore_snapshot 可选）" },
                    "keep_last": { "type": "integer", "description": "保留最近 N 个快照（cleanup_snapshots 可选）" },
                    "older_than_days": { "type": "integer", "description": "删除 N 天前的快照（cleanup_snapshots 可选）" }
                },
                "required": ["action"]
            }),
            "pubsub" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["publish", "subscribe", "subscriber_count", "list_topics", "get_history", "create_persistent_topic", "create_group", "join_group", "leave_group", "send_group_message", "get_group_info", "list_groups"], "description": "PubSub 操作类型" },
                    "topic": { "type": "string", "description": "主题名称（除 list_topics 外都需要）" },
                    "message": { "type": "string", "description": "消息内容（publish/send_group_message 需要）" },
                    "message_type": { "type": "string", "description": "消息类型（publish 可选）" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "消息标签（publish 可选）" },
                    "limit": { "type": "integer", "description": "消息数量限制（subscribe/get_history 可选，默认 10）" },
                    "description": { "type": "string", "description": "主题/群聊描述（create_persistent_topic/create_group 可选）" },
                    "persistent": { "type": "boolean", "description": "是否持久化（create_persistent_topic 可选，默认 true）" },
                    "group_name": { "type": "string", "description": "群聊名称（create_group 需要）" },
                    "group_id": { "type": "string", "description": "群聊 ID（join_group/leave_group/send_group_message/get_group_info 需要）" }
                },
                "required": ["action"]
            }),
            "message_passing" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["send_message", "subscribe_topic", "list_topics", "get_message_history", "create_topic"], "description": "消息传递操作类型" },
                    "topic": { "type": "string", "description": "主题名称（除 list_topics 外都需要）" },
                    "content": { "type": "string", "description": "消息内容（send_message 需要）" },
                    "message_type": { "type": "string", "description": "消息类型（send_message 可选，默认 text）" },
                    "latest_only": { "type": "boolean", "description": "是否仅获取最新消息（subscribe_topic 可选，默认 false）" },
                    "limit": { "type": "integer", "description": "消息数量限制（subscribe_topic/get_message_history 可选，默认 10）" },
                    "description": { "type": "string", "description": "主题描述（create_topic 可选）" }
                },
                "required": ["action"]
            }),
            "iroh" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["create_doc", "open_doc", "set", "get", "list_entries", "get_node_id", "connect_to_node", "share_doc_ticket", "create_group", "join_group", "leave_group", "send_group_message", "get_group_info", "list_groups"], "description": "Iroh 操作类型" },
                    "name": { "type": "string", "description": "文档名称（create_doc 可选）" },
                    "doc_id": { "type": "string", "description": "文档 ID（open_doc/set/get/list_entries/share_doc_ticket 需要）" },
                    "key": { "type": "string", "description": "键（set/get 需要）" },
                    "value": { "type": "string", "description": "值（set 需要）" },
                    "peer_id": { "type": "string", "description": "对等节点 ID（connect_to_node 需要）" },
                    "addr": { "type": "string", "description": "节点地址（connect_to_node 可选）" },
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
                    "action": { "type": "string", "enum": ["add", "get", "pin", "unpin", "list_pins", "cat"], "description": "IPFS 归档操作类型" },
                    "path": { "type": "string", "description": "文件路径（add 需要）" },
                    "cid": { "type": "string", "description": "内容 ID（get/pin/unpin/cat 需要）" },
                    "output_path": { "type": "string", "description": "输出路径（get 可选）" },
                    "recursive": { "type": "boolean", "description": "是否递归（add 可选）" }
                },
                "required": ["action"]
            }),
            "browser" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["open_page", "close_page", "navigate", "refresh", "click_element", "input_text", "get_element_text", "get_page_title", "get_page_url", "take_screenshot", "wait_for_element", "scroll_to_element", "execute_script", "get_page_source", "set_window_size", "open_new_tab", "switch_tab"], "description": "浏览器操作类型" },
                    "url": { "type": "string", "description": "页面 URL（open_page/navigate/open_new_tab 需要）" },
                    "wait_time": { "type": "integer", "description": "等待时间秒数（open_page/click_element 可选）" },
                    "selector": { "type": "string", "description": "元素选择器（click_element/input_text/get_element_text/wait_for_element/scroll_to_element 需要）" },
                    "text": { "type": "string", "description": "输入文本（input_text 需要）" },
                    "clear_first": { "type": "boolean", "description": "是否先清空（input_text 可选，默认 true）" },
                    "path": { "type": "string", "description": "截图保存路径（take_screenshot 可选）" },
                    "timeout": { "type": "integer", "description": "超时时间秒数（wait_for_element 可选，默认 10）" },
                    "script": { "type": "string", "description": "JavaScript 代码（execute_script 需要）" },
                    "width": { "type": "integer", "description": "窗口宽度（set_window_size 需要）" },
                    "height": { "type": "integer", "description": "窗口高度（set_window_size 需要）" },
                    "index": { "type": "integer", "description": "标签页索引（switch_tab 需要）" }
                },
                "required": ["action"]
            }),
            "ui_control" => serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["click_button", "set_input_text", "get_input_text", "select_dropdown_option", "toggle_checkbox", "select_radio_button", "show_notification", "show_modal", "get_window_state", "set_window_state"], "description": "UI 控制操作类型" },
                    "button_id": { "type": "string", "description": "按钮 ID（click_button 需要）" },
                    "params": { "type": "object", "description": "额外参数（click_button 可选）" },
                    "input_id": { "type": "string", "description": "输入框 ID（set_input_text/get_input_text 需要）" },
                    "text": { "type": "string", "description": "文本内容（set_input_text 需要）" },
                    "dropdown_id": { "type": "string", "description": "下拉菜单 ID（select_dropdown_option 需要）" },
                    "option_value": { "type": "string", "description": "选项值（select_dropdown_option/select_radio_button 需要）" },
                    "checkbox_id": { "type": "string", "description": "复选框 ID（toggle_checkbox 需要）" },
                    "radio_group_id": { "type": "string", "description": "单选按钮组 ID（select_radio_button 需要）" },
                    "title": { "type": "string", "description": "标题（show_notification/show_modal 需要）" },
                    "message": { "type": "string", "description": "消息内容（show_notification 需要）" },
                    "notification_type": { "type": "string", "enum": ["info", "warning", "error", "success"], "description": "通知类型（show_notification 可选）" },
                    "content": { "type": "string", "description": "对话框内容（show_modal 需要）" },
                    "buttons": { "type": "array", "items": { "type": "string" }, "description": "按钮配置（show_modal 可选）" },
                    "state": { "type": "string", "enum": ["minimized", "maximized", "fullscreen", "normal"], "description": "窗口状态（set_window_state 需要）" }
                },
                "required": ["action"]
            }),
            "generate_image" => serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string", "description": "图片描述，详细描述要生成的图片内容" },
                    "width": { "type": "integer", "description": "图片宽度（像素），默认 1024", "default": 1024 },
                    "height": { "type": "integer", "description": "图片高度（像素），默认 1024", "default": 1024 },
                    "provider": { "type": "string", "enum": ["google", "jimeng"], "description": "图片生成 Provider，google=Google Imagen, jimeng=即梦", "default": "google" }
                },
                "required": ["prompt"]
            }),
            "generate_audio" => serde_json::json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "要转换为语音的文字内容" },
                    "voice_id": { "type": "string", "description": "音色 ID（可选），不指定则使用默认音色" },
                    "provider": { "type": "string", "enum": ["minimax"], "description": "语音合成 Provider，minimax=MiniMax TTS", "default": "minimax" }
                },
                "required": ["text"]
            }),
            "generate_video" => serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string", "description": "视频描述，详细描述要生成的视频内容" },
                    "duration": { "type": "integer", "description": "视频时长（秒），默认 5 秒", "default": 5 },
                    "provider": { "type": "string", "enum": ["minimax", "jimeng"], "description": "视频生成 Provider，minimax=MiniMax Video, jimeng=即梦视频", "default": "minimax" }
                },
                "required": ["prompt"]
            }),
            "get_video_status" => serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "string", "description": "视频生成任务 ID" },
                    "provider": { "type": "string", "enum": ["minimax", "jimeng"], "description": "视频生成 Provider" }
                },
                "required": ["task_id", "provider"]
            }),
            _ => serde_json::json!({
                "type": "object",
                "properties": {},
                "description": format!("工具 '{}' 的参数定义", tool_name)
            }),
        }
    }

    /// 获取工具描述
    async fn get_tools_description(
        &self,
        tools: &[String],
        tool_registry: Arc<crate::tools::ToolRegistry>,
    ) -> String {
        let mut desc = "\n## 可用工具\n".to_string();

        // 添加 ToolRegistry 中的工具
        for tool_name in tools {
            desc.push_str(&format!("\n- {}", tool_name));
        }

        // 🔥 添加媒体工具（这些工具在 ToolBus 中，不在 ToolRegistry 中）
        desc.push_str("\n\n### 媒体工具\n");
        desc.push_str("\n- generate_image: 根据文字描述生成图片，支持风景、人物、艺术创作等");
        desc.push_str("\n- generate_audio: 文本转语音，支持多语言、多音色");
        desc.push_str("\n- generate_video: 根据文字描述生成视频，支持风景、动画、特效等。⚠️ 重要：这是异步任务，调用后会立即返回 task_id 和 status: processing，必须使用 get_video_status 轮询查询结果，不要重复调用 generate_video");
        desc.push_str("\n- get_video_status: 查询视频生成任务的状态。当 generate_video 返回 processing 状态时，使用此工具传入 task_id 和 provider 轮询，直到 status 变为 completed 或 failed");

        desc
    }

    /// 获取上下文描述
    fn get_context_description(&self, state: &EnvironmentState) -> String {
        let mut desc = "\n## 当前上下文\n".to_string();

        // 迭代次数
        desc.push_str(&format!("\n- 当前迭代：{}/{}",
            state.iteration_count,
            crate::agent::executor::core::RalphLoopExecutor::MAX_ITERATIONS));

        // 工具调用次数
        desc.push_str(&format!("\n- 工具调用次数：{}", state.tool_call_count));

        // 感知层检索到的上下文
        if !state.retrieved_context.relevant_memories.is_empty() {
            desc.push_str("\n- 相关记忆:");
            for mem in &state.retrieved_context.relevant_memories {
                desc.push_str(&format!("\n  - {}", mem.content.chars().take(100).collect::<String>()));
            }
        }

        if !state.retrieved_context.relevant_docs.is_empty() {
            desc.push_str("\n- 相关文档:");
            for doc in &state.retrieved_context.relevant_docs {
                desc.push_str(&format!("\n  - {} (相关度：{:.0}%)", doc.name, doc.relevance_score * 100.0));
            }
        }

        if !state.retrieved_context.active_goals.is_empty() {
            desc.push_str("\n- 活动目标:");
            for goal in &state.retrieved_context.active_goals {
                desc.push_str(&format!("\n  - [{}] {} (进度：{:.0}%)",
                    goal.id, goal.description, goal.progress * 100.0));
            }
        }

        desc
    }

    /// 获取决策指导
    fn get_decision_guidelines(&self, intent: Intent) -> String {
        let guidelines = match intent {
            Intent::Greeting => {
                r#"
## 决策指导（问候场景）
- 这是简单对话，无需工具
- 直接回复用户，保持友好
- 置信度应高于 0.9
"#
            }
            Intent::CodeTask => {
                r#"
## 决策指导（代码任务）
- 首先检查是否有相关记忆或文档
- 复杂任务先创建目标分解
- 使用 file_read/file_write 工具操作代码
- 完成后调用 verify_code
"#
            }
            Intent::FileOperation => {
                r#"
## 决策指导（文件操作）
- 确认文件路径正确
- 写入前检查现有内容
- 重要操作先备份
"#
            }
            Intent::ContinueProject => {
                r#"
## 决策指导（继续项目）
- 检查之前的目标和进度
- 更新现有目标状态
- 遵循项目规范文档
"#
            }
            Intent::SearchQuery => {
                r#"
## 决策指导（搜索查询）
- 使用 memory_search 检索记忆
- 使用 read_document 查询文档
- 整合信息后回复
"#
            }
            Intent::Planning => {
                r#"
## 决策指导（规划场景）
- 创建清晰的目标层级
- 先创建主要目标，再分解子目标
- 设置合理的优先级
"#
            }
            Intent::Debugging => {
                r#"
## 决策指导（调试场景）
- 首先读取相关代码和日志
- 分析错误信息和堆栈
- 小步修改，验证每个修复
"#
            }
            _ => {
                r#"
## 决策指导（通用）
- 根据意图选择合适的工具
- 不确定时询问用户
- 保持任务聚焦
"#
            }
        };

        guidelines.to_string()
    }

    /// 解析 LLM 响应
    fn parse_response(
        &self,
        response: &str,
        tool_calls: &[crate::agent::ai_client::AiToolCall],
    ) -> Result<Thought, crate::agent::executor::types::ExecutorError> {
        // 如果有工具调用，解析为 ToolCall 动作
        if !tool_calls.is_empty() {
            let tool_call = &tool_calls[0];
            return Ok(Thought {
                analysis: format!("调用工具：{}", tool_call.name),
                reflection: Some(Reflection {
                    information_assessment: InformationAssessment {
                        status: "sufficient".to_string(),
                        missing_info: vec![],
                        suggestion: "proceed".to_string(),
                    },
                    task_progress: TaskProgress::default(),
                    confidence: 0.8,
                    reasoning: "使用工具执行任务".to_string(),
                    suggested_next_step: format!("执行 {}", tool_call.name),
                }),
                action: Action::ToolCall {
                    tool: tool_call.name.clone(),
                    args: tool_call.arguments.clone(),
                    id: tool_call.id.clone(),
                },
            });
        }

        // 尝试从文本响应中解析 JSON（如果模型直接返回 JSON）
        if let Ok(parsed) = serde_json::from_str::<Value>(response) {
            if parsed.get("action").is_some() {
                return self.parse_json_thought(&parsed);
            }
        }

        // 默认返回 Complete 动作
        Ok(Thought {
            analysis: response.to_string(),
            reflection: None,
            action: Action::Complete(response.to_string()),
        })
    }

    /// 解析 JSON 格式的 Thought
    fn parse_json_thought(
        &self,
        parsed: &Value,
    ) -> Result<Thought, crate::agent::executor::types::ExecutorError> {
        let analysis = parsed["analysis"].as_str().unwrap_or("").to_string();

        let reflection = if let Some(reflection_val) = parsed.get("reflection") {
            Some(Reflection {
                information_assessment: serde_json::from_value(
                    reflection_val["information_assessment"].clone()
                ).unwrap_or_default(),
                task_progress: serde_json::from_value(
                    reflection_val["task_progress"].clone()
                ).unwrap_or_default(),
                confidence: reflection_val["confidence"].as_f64().unwrap_or(0.5) as f32,
                reasoning: reflection_val["reasoning"].as_str().unwrap_or("").to_string(),
                suggested_next_step: reflection_val["suggested_next_step"].as_str().unwrap_or("").to_string(),
            })
        } else {
            None
        };

        let action = self.parse_action(&parsed["action"])?;

        Ok(Thought {
            analysis,
            reflection,
            action,
        })
    }

    /// 解析 action 字段
    fn parse_action(&self, action_val: &Value) -> Result<Action, crate::agent::executor::types::ExecutorError> {
        let action_type = action_val["type"].as_str()
            .ok_or_else(|| crate::agent::executor::types::ExecutorError::AiError("action.type 缺失".to_string()))?;

        match action_type {
            "tool_call" => {
                let tool = action_val["tool"].as_str()
                    .ok_or_else(|| crate::agent::executor::types::ExecutorError::AiError("tool_call.tool 缺失".to_string()))?
                    .to_string();
                let args = action_val["args"].clone();
                let id = action_val["id"].as_str()
                    .ok_or_else(|| crate::agent::executor::types::ExecutorError::AiError("tool_call.id 缺失".to_string()))?
                    .to_string();
                Ok(Action::ToolCall { tool, args, id })
            }
            "complete" => {
                let content = action_val["content"].as_str()
                    .ok_or_else(|| crate::agent::executor::types::ExecutorError::AiError("complete.content 缺失".to_string()))?
                    .to_string();
                Ok(Action::Complete(content))
            }
            "continue" => Ok(Action::Continue),
            "fail" => {
                let reason = action_val["reason"].as_str()
                    .ok_or_else(|| crate::agent::executor::types::ExecutorError::AiError("fail.reason 缺失".to_string()))?
                    .to_string();
                Ok(Action::Fail(reason))
            }
            "ask_user" => {
                let question = action_val["question"].as_str()
                    .ok_or_else(|| crate::agent::executor::types::ExecutorError::AiError("ask_user.question 缺失".to_string()))?
                    .to_string();
                Ok(Action::AskUser { question })
            }
            _ => Err(crate::agent::executor::types::ExecutorError::AiError(format!("未知的 action 类型：{}", action_type))),
        }
    }
}
