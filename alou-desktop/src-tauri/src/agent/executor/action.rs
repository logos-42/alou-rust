//! 行动层模块
//!
//! 负责：
//! - 执行 Thought 中的 Action
//! - 处理工具调用（包括元行动工具和 agent_document 特殊处理）
//! - 返回 ActionResult

use std::sync::Arc;
use std::collections::HashMap;

use serde_json::Value;
use tauri::{Emitter, Manager};

use crate::agent::executor::types::{Thought, Action, ActionResult};
use crate::agent::executor::core::ExecutorCore;
use crate::bridges::{ToolBridge, ToolCallRequest, ToolCallResponse};
use crate::tools::ToolRegistry;
use crate::tools::meta_tool::MetaActionTools;
use crate::agent::memory::MemoryManager;
use crate::agent::task::TaskManager;

/// 行动层
pub struct ActionLayer {
    tool_bridge: Arc<ToolBridge>,
    tool_registry: Arc<ToolRegistry>,
    meta_tools: Option<Arc<MetaActionTools>>,
}

impl ActionLayer {
    /// 创建行动层实例
    pub fn new(
        tool_bridge: Arc<ToolBridge>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        Self {
            tool_bridge,
            tool_registry,
            meta_tools: None,
        }
    }

    /// 创建带元行动工具的实例
    pub fn with_meta_tools(
        tool_bridge: Arc<ToolBridge>,
        tool_registry: Arc<ToolRegistry>,
        memory_manager: Arc<MemoryManager>,
        task_manager: Arc<TaskManager>,
    ) -> Self {
        let meta_tools = Arc::new(MetaActionTools::new(memory_manager, task_manager));
        Self {
            tool_bridge,
            tool_registry,
            meta_tools: Some(meta_tools),
        }
    }

    /// 执行 Thought 中的所有行动
    pub async fn execute(
        &self,
        thought: &Thought,
        core: &ExecutorCore,
    ) -> Result<Vec<ActionResult>, crate::agent::executor::types::ExecutorError> {
        match &thought.action {
            Action::ToolCall { tool, args, id } => {
                // 🔥 特殊处理 agent_document 工具
                if tool == "agent_document" {
                    let result = self.execute_agent_document(args, id, core).await?;
                    Ok(vec![result])
                } else {
                    let result = self.execute_tool_call(tool, args, id, core).await?;
                    Ok(vec![result])
                }
            }
            Action::Complete(content) => {
                Ok(vec![ActionResult {
                    action_id: "complete".to_string(),
                    tool_name: "complete".to_string(),
                    success: true,
                    output: Some(Value::String(content.clone())),
                    error: None,
                }])
            }
            Action::Continue => {
                Ok(vec![ActionResult {
                    action_id: "continue".to_string(),
                    tool_name: "continue".to_string(),
                    success: true,
                    output: None,
                    error: None,
                }])
            }
            Action::Fail(reason) => {
                Err(crate::agent::executor::types::ExecutorError::InternalError(
                    format!("Agent 决定放弃：{}", reason)
                ))
            }
            Action::AskUser { question } => {
                Ok(vec![ActionResult {
                    action_id: "ask_user".to_string(),
                    tool_name: "ask_user".to_string(),
                    success: true,
                    output: Some(Value::String(question.clone())),
                    error: None,
                }])
            }
            Action::CreateGoal { description, priority } => {
                self.execute_create_goal(description, priority, core).await
            }
            Action::UpdateGoal { goal_id, progress } => {
                self.execute_update_goal(goal_id, *progress, core).await
            }
        }
    }

    /// 执行单个工具调用
    async fn execute_tool_call(
        &self,
        tool: &str,
        args: &Value,
        id: &str,
        executor_core: &ExecutorCore,
    ) -> Result<ActionResult, crate::agent::executor::types::ExecutorError> {
        // 🔥 优先处理元行动工具
        if let Some(ref meta_tools) = self.meta_tools {
            if matches!(tool, "memory_search" | "read_document" | "remember" | "manage_goal") {
                log::info!("[ActionLayer] 执行元行动工具：{} ({})", tool, id);
                return self.execute_meta_tool(meta_tools, tool, args, id).await;
            }
        }

        // 🔥 识别长任务
        let is_long_running = matches!(tool, "generate_video" | "get_video_status" | "browser" | "ipfs_archive");
        if is_long_running {
            log::info!("[ActionLayer] 检测到长任务工具：{}", tool);
        }

        log::info!("[ActionLayer] 执行工具：{} ({})", tool, id);
        log::info!("[ActionLayer] 工具参数：{}", serde_json::to_string(args).unwrap_or_default());

        // 🔥 构建 ToolCallRequest
        let request = ToolCallRequest {
            session_id: format!("agent_{}", id),
            user_id: None,
            tool_id: tool.to_string(),
            args: args.clone(),
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            environment: std::env::vars().collect(),
            timeout_seconds: Some(60),
            permissions: vec![],
        };

        // 🔥 发送工具调用开始事件到前端
        if let Some(app_handle) = executor_core.app_handle() {
            let start_payload = serde_json::json!({
                "task_id": id,
                "tool_name": tool,
                "type": "tool_calling",
            });
            if let Err(e) = app_handle.emit("agent:progress", &start_payload) {
                log::warn!("[ActionLayer] 发送 tool_calling 事件失败：{}", e);
            } else {
                log::info!("[ActionLayer] 已发送 tool_calling 事件：{}", tool);
            }
            
            // 🔥 发送工具日志事件到前端（通用日志）
            let tool_log_start = serde_json::json!({
                "tool": tool,
                "action": "start",
                "args": args,
                "message": format!("开始执行工具: {}", tool),
            });
            let _ = app_handle.emit("tool:log", &tool_log_start);
        }

        // 🔥 添加超时控制
        log::info!("[ActionLayer] 准备调用 tool_bridge，工具：{}", tool);
        
        let tool_call_result = tokio::time::timeout(
            std::time::Duration::from_secs(60),  // 60 秒超时
            self.tool_bridge.handle_request(request)
        ).await;

        log::info!("[ActionLayer] tool_bridge 调用返回");

        match tool_call_result {
            Ok(Ok(ToolCallResponse { success, result, error })) => {
                log::info!("[ActionLayer] 工具执行成功：{}, success={}", tool, success);
                log::info!("[ActionLayer] 工具返回结果：result={:?}, error={:?}", result, error);
                log::info!("[ActionLayer] 工具参数：{}", serde_json::to_string(args).unwrap_or_default());
                
                // 🔥 如果是 agent_creator create 成功，通知前端更新侧边栏
                if success && tool == "agent_creator" {
                    let action_val = args
                        .get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    log::info!("[ActionLayer] agent_creator action: '{}'", action_val);
                    
                    if action_val == "create" {
                        log::info!("[ActionLayer] 检测到 agent_creator create 操作，准备发送事件");
                        
                        let app_handle_opt = executor_core.app_handle();
                        if app_handle_opt.is_none() {
                            log::warn!("[ActionLayer] ⚠️ app_handle 不存在，无法发送 agent:created 事件！请检查 executor 初始化");
                        }
                        
                        if let Some(app_handle) = app_handle_opt {
                            // 从 agent_config 对象中获取 display_name 和 description
                            let agent_config = args.get("agent_config").and_then(|v| v.as_object());
                            let display_name = agent_config
                                .and_then(|c| c.get("display_name"))
                                .and_then(|v| v.as_str())
                                .or_else(|| args.get("display_name").and_then(|v| v.as_str()))
                                .unwrap_or("New Agent")
                                .to_string();
                            let description = agent_config
                                .and_then(|c| c.get("description"))
                                .and_then(|v| v.as_str())
                                .or_else(|| args.get("description").and_then(|v| v.as_str()))
                                .unwrap_or("")
                                .to_string();
                            // 获取 agent_id（如果有）
                            let agent_id = agent_config
                                .and_then(|c| c.get("id"))
                                .and_then(|v| v.as_str())
                                .or_else(|| args.get("agent_id").and_then(|v| v.as_str()))
                                .map(|s| s.to_string());

                            let mut payload = serde_json::json!({
                                "name": display_name,
                                "role_description": description,
                                "autoActivate": false, // 🔥 不要自动激活新智能体，避免循环创建
                            });
                            // 如果有 id，添加到 payload
                            if let Some(id) = agent_id {
                                if let serde_json::Value::Object(ref mut map) = payload {
                                    map.insert("id".to_string(), serde_json::Value::String(id));
                                }
                            }

                            log::info!("[ActionLayer] 准备发送 agent:created 事件，payload: {}", payload);
                            
                            if let Err(e) = app_handle.emit("agent:created", &payload) {
                                log::warn!("[ActionLayer] 发送 agent:created 事件失败：{}", e);
                            } else {
                                log::info!("[ActionLayer] 已发送 agent:created 事件，名称：{}", display_name);
                            }
                        } else {
                            log::warn!("[ActionLayer] app_handle 不存在，无法发送 agent:created 事件");
                        }
                    } else {
                        log::warn!("[ActionLayer] agent_creator action 不是 'create': '{}'", action_val);
                    }
                }

                // 🔥 如果是 tool_creation create_tool 成功，通知前端更新工具列表
                if success && tool == "tool_creation" {
                    let action_val = args
                        .get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if action_val == "create_tool" || action_val == "create_and_log" {
                        if let Some(app_handle) = executor_core.app_handle() {
                            // 从 tool_definition 对象中获取工具信息
                            let tool_def = args.get("tool_definition").and_then(|v| v.as_object());
                            let tool_name = tool_def
                                .and_then(|d| d.get("name"))
                                .and_then(|v| v.as_str())
                                .or_else(|| args.get("tool_name").and_then(|v| v.as_str()))
                                .unwrap_or("New Tool")
                                .to_string();
                            let tool_description = tool_def
                                .and_then(|d| d.get("description"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let payload = serde_json::json!({
                                "tool_name": tool_name,
                                "description": tool_description,
                                "status": "created",
                            });

                            if let Err(e) = app_handle.emit("tool:created", &payload) {
                                log::warn!("[ActionLayer] 发送 tool:created 事件失败：{}", e);
                            } else {
                                log::info!("[ActionLayer] 已发送 tool:created 事件，工具：{}", tool_name);
                            }
                        }
                    }
                }

                // 🔥 发送工具调用完成事件到前端
                if let Some(app_handle) = executor_core.app_handle() {
                    // 构建 preview：优先使用 output，否则使用 data 的序列化
                    let preview_str = result.as_ref().map(|r| {
                        if let Some(ref output) = r.output {
                            output.clone()
                        } else {
                            serde_json::to_string(&r.data).unwrap_or_default()
                        }
                    }).unwrap_or_default();

                    let done_payload = serde_json::json!({
                        "task_id": id,
                        "tool_name": tool,
                        "type": "tool_done",
                        "success": success,
                        "preview": preview_str,
                        "error": error,
                    });
                    if let Err(e) = app_handle.emit("agent:progress", &done_payload) {
                        log::warn!("[ActionLayer] 发送 tool_done 事件失败：{}", e);
                    } else {
                        log::info!("[ActionLayer] 已发送 tool_done 事件：{} (success={})", tool, success);
                    }
                    
                    // 🔥 发送工具日志事件到前端（通用日志）
                    let tool_log_end = serde_json::json!({
                        "tool": tool,
                        "action": "done",
                        "success": success,
                        "message": format!("工具执行完成: {} (success={})", tool, success),
                    });
                    let _ = app_handle.emit("tool:log", &tool_log_end);
                }

                // 🔥 修复：优先使用 output，如果没有则使用 data 的序列化结果
                let output_value = result.as_ref().and_then(|r| {
                    if let Some(ref output_str) = r.output {
                        // 如果 output 字段有值，使用它
                        Some(Value::String(output_str.clone()))
                    } else if !r.data.is_null() {
                        // 否则使用 data 字段的 JSON 序列化
                        Some(r.data.clone())
                    } else {
                        None
                    }
                });

                Ok(ActionResult {
                    action_id: id.to_string(),
                    tool_name: tool.to_string(),
                    success,
                    output: output_value,
                    error,
                })
            }
            Ok(Err(e)) => {
                log::error!("[ActionLayer] 工具执行失败：{} - {:?}", tool, e);
                Ok(ActionResult {
                    action_id: id.to_string(),
                    tool_name: tool.to_string(),
                    success: false,
                    output: None,
                    error: Some(format!("工具执行失败：{}", e)),
                })
            }
            Err(timeout_err) => {
                log::error!("[ActionLayer] 工具执行超时（60 秒）: {} - {:?}", tool, timeout_err);
                Ok(ActionResult {
                    action_id: id.to_string(),
                    tool_name: tool.to_string(),
                    success: false,
                    output: None,
                    error: Some(format!("工具执行超时：{}", timeout_err)),
                })
            }
        }
    }

    /// 🔥 执行元行动工具
    async fn execute_meta_tool(
        &self,
        meta_tools: &MetaActionTools,
        tool: &str,
        args: &Value,
        id: &str,
    ) -> Result<ActionResult, crate::agent::executor::types::ExecutorError> {
        log::info!("[ActionLayer:MetaTool] 执行：{} ({})", tool, id);
        log::info!("[ActionLayer:MetaTool] 参数：{}", serde_json::to_string(args).unwrap_or_default());

        match meta_tools.execute(tool, args.clone()).await {
            Ok(output) => Ok(ActionResult {
                action_id: id.to_string(),
                tool_name: tool.to_string(),
                success: true,
                output: Some(output),
                error: None,
            }),
            Err(e) => Ok(ActionResult {
                action_id: id.to_string(),
                tool_name: tool.to_string(),
                success: false,
                output: None,
                error: Some(e.to_string()),
            }),
        }
    }

    /// 🔥 特殊处理 agent_document 工具（内联处理，不经过 ToolBridge）
    async fn execute_agent_document(
        &self,
        args: &Value,
        id: &str,
        core: &ExecutorCore,
    ) -> Result<ActionResult, crate::agent::executor::types::ExecutorError> {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let doc_type = args.get("document_type").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "read" => {
                // 获取 agent_id
                let agent_id = core.agent_id().unwrap_or_default();

                if agent_id.is_empty() {
                    return Ok(ActionResult {
                        action_id: id.to_string(),
                        tool_name: "agent_document".to_string(),
                        success: false,
                        output: None,
                        error: Some("无法获取 agent_id".to_string()),
                    });
                }

                // 从文件读取文档
                let content = if let Some(app_handle) = core.app_handle() {
                    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
                        let doc_path = std::path::PathBuf::from(&app_data_dir)
                            .join("agent-documents")
                            .join(&agent_id)
                            .join(format!("{}.md", doc_type.to_uppercase()));

                        log::info!("[ActionLayer] 尝试读取文档：{:?}", doc_path);

                        // 自动创建目录和初始文档（如果不存在）
                        if !doc_path.exists() {
                            if let Some(parent) = doc_path.parent() {
                                if let Err(e) = std::fs::create_dir_all(parent) {
                                    log::warn!("[ActionLayer] 创建文档目录失败：{}", e);
                                    None
                                } else {
                                    // 写入初始内容
                                    let initial_content = Self::generate_initial_document_content(doc_type, &agent_id);
                                    if let Err(e) = std::fs::write(&doc_path, &initial_content) {
                                        log::warn!("[ActionLayer] 创建初始文档失败：{}", e);
                                        None
                                    } else {
                                        log::info!("[ActionLayer] 自动创建文档：{:?}", doc_path);
                                        Some(initial_content)
                                    }
                                }
                            } else {
                                None
                            }
                        } else {
                            // 文件存在，读取内容
                            match tokio::fs::read_to_string(&doc_path).await {
                                Ok(text) => Some(text),
                                Err(e) => {
                                    log::warn!("[ActionLayer] 读取文档失败：{}", e);
                                    None
                                }
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                match content {
                    Some(ref text) => {
                        let len: usize = text.len();
                        log::info!("[ActionLayer] agent_document read: 已读取 {} 文档，长度：{}", doc_type, len);
                        Ok(ActionResult {
                            action_id: id.to_string(),
                            tool_name: "agent_document".to_string(),
                            success: true,
                            output: Some(serde_json::json!({ "document_type": doc_type, "content": text.clone() })),
                            error: None,
                        })
                    }
                    None => {
                        log::warn!("[ActionLayer] agent_document read: 未找到文档 '{}' 或读取失败", doc_type);
                        Ok(ActionResult {
                            action_id: id.to_string(),
                            tool_name: "agent_document".to_string(),
                            success: false,
                            output: None,
                            error: Some(format!("文档 '{}' 未找到或读取失败", doc_type)),
                        })
                    }
                }
            }
            "update" => {
                let new_content = args.get("new_content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let reason = args.get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if new_content.is_empty() {
                    return Ok(ActionResult {
                        action_id: id.to_string(),
                        tool_name: "agent_document".to_string(),
                        success: false,
                        output: None,
                        error: Some("update 操作需要提供 new_content 字段".to_string()),
                    });
                }

                // 获取 agent_id
                let agent_id = core.agent_id().unwrap_or_default();

                if agent_id.is_empty() {
                    return Ok(ActionResult {
                        action_id: id.to_string(),
                        tool_name: "agent_document".to_string(),
                        success: false,
                        output: None,
                        error: Some("无法获取 agent_id".to_string()),
                    });
                }

                // 写入文件系统
                let write_result = if let Some(app_handle) = core.app_handle() {
                    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
                        let doc_path = std::path::PathBuf::from(&app_data_dir)
                            .join("agent-documents")
                            .join(&agent_id)
                            .join(format!("{}.md", doc_type.to_uppercase()));

                        log::info!("[ActionLayer] 尝试写入文档：{:?}", doc_path);

                        // 确保目录存在
                        if let Some(parent) = doc_path.parent() {
                            if let Err(e) = std::fs::create_dir_all(parent) {
                                log::warn!("[ActionLayer] 创建文档目录失败：{}", e);
                            }
                        }

                        // 写入文件
                        match std::fs::write(&doc_path, new_content) {
                            Ok(_) => {
                                log::info!("[ActionLayer] agent_document update: 已写入 {} 文档，长度：{}", doc_type, new_content.len());

                                // 发送事件通知前端
                                let payload = serde_json::json!({
                                    "document_type": doc_type,
                                    "new_content": new_content,
                                    "reason": reason,
                                    "agent_id": agent_id.clone(),
                                    "persisted_to_file": true,
                                });
                                let _ = app_handle.emit("document:updated", &payload);

                                Ok(true)
                            },
                            Err(e) => {
                                log::warn!("[ActionLayer] 写入文档失败：{}", e);
                                Err(e)
                            }
                        }
                    } else {
                        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "无法获取 app_data_dir"))
                    }
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "app_handle 不存在"))
                };

                match write_result {
                    Ok(_) => Ok(ActionResult {
                        action_id: id.to_string(),
                        tool_name: "agent_document".to_string(),
                        success: true,
                        output: Some(serde_json::json!({
                            "document_type": doc_type,
                            "status": "updated",
                            "message": format!("文档 '{}' 已成功写入文件系统", doc_type),
                            "reason": reason,
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(ActionResult {
                        action_id: id.to_string(),
                        tool_name: "agent_document".to_string(),
                        success: false,
                        output: None,
                        error: Some(format!("写入文档失败：{}", e)),
                    }),
                }
            }
            _ => Ok(ActionResult {
                action_id: id.to_string(),
                tool_name: "agent_document".to_string(),
                success: false,
                output: None,
                error: Some(format!("agent_document: 未知的 action '{}'，支持 read / update", action)),
            }),
        }
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
最后更新：{}
"#, agent_id, now),
            "soul" => format!(r#"# 核心身份

**智能体 ID**: {}

## 角色定位
专业的 AI 助手

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
最后更新：{}
"#, agent_id, now),
            "identity" => format!(r#"# 身份定义

**智能体 ID**: {}

## 名称
智能体

## 角色
专业的 AI 助手

## 专长领域
（根据实际使用情况更新）

## 工作方式
- 理解用户需求
- 选择合适工具
- 执行任务
- 反馈结果

---
最后更新：{}
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
最后更新：{}
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
最后更新：{}
"#, now),
            "tools" => format!(r#"# 工具使用记录

## 常用工具
（根据实际使用情况更新）

## 工具组合
（记录有效的工具组合方案）

## 最佳实践
（记录工具使用的最佳实践）

---
最后更新：{}
"#, now),
            "agents" => format!(r#"# 协作智能体

## 已知智能体
（暂无记录）

## 协作经验
（暂无记录）

## 协作模式
（暂无记录）

---
最后更新：{}
"#, now),
            "ipfs" => format!(r#"# IPFS 对话历史索引

这里记录了存储在 IPFS 上的重要对话会话。

## 最近会话
（暂无记录）

## 统计信息
- 总会话数：0
- 总消息数：0
- 最早会话：暂无
- 最近会话：暂无

## 如何更新
- 重要对话结束后，自动记录到本文件
- 每条记录包含：CID、时间、主题、消息数

---
最后更新：{}
"#, now),
            "user" => format!(r#"# 用户喜好与偏好

这里记录了用户的个人偏好、习惯和工作方式。

## 用户偏好
（暂无记录）

## 沟通风格
（暂无记录）

## 技术栈偏好
（暂无记录）

## 如何更新
- 用户分享偏好时自动记录
- 使用 agent_document 工具的 update 操作更新

---
最后更新：{}
"#, now),
            "project" => format!(r#"# 项目与工作报告

这里记录了参与的项目、工作报告和重要成果。

## 当前项目
（暂无记录）

## 已完成项目
（暂无记录）

## 工作报告
（暂无记录）

## 如何更新
- 项目启动时创建记录
- 定期更新工作报告
- 项目完成后归档

---
最后更新：{}
"#, now),
            "key" => format!(r#"# 关键密钥路径

这里记录了重要密钥和凭证的存储路径（不存储实际密钥）。

## 密钥路径
（暂无记录）

## 如何更新
- 记录密钥文件的存储路径
- 不要存储实际密钥内容
- 使用加密存储敏感信息

---
最后更新：{}
"#, now),
            _ => format!(r#"# {}

---
最后更新：{}
"#, document_type, now),
        }
    }

    /// 执行创建目标
    async fn execute_create_goal(
        &self,
        description: &str,
        priority: &str,
        core: &ExecutorCore,
    ) -> Result<Vec<ActionResult>, crate::agent::executor::types::ExecutorError> {
        if let Some(ref goal_tracker) = core.goal_tracker() {
            use crate::agent::goal::Priority;
            let priority_enum = Priority::from_str(priority);
            let goal = goal_tracker
                .create_goal(description, priority_enum, None, None)
                .await;

            Ok(vec![ActionResult {
                action_id: format!("goal_{}", goal.id),
                tool_name: "create_goal".to_string(),
                success: true,
                output: Some(serde_json::json!({
                    "goal_id": goal.id,
                    "description": goal.description,
                    "priority": format!("{:?}", goal.priority),
                    "status": format!("{:?}", goal.status)
                })),
                error: None,
            }])
        } else {
            Err(crate::agent::executor::types::ExecutorError::InternalError(
                "GoalTracker 未初始化".to_string()
            ))
        }
    }

    /// 执行更新目标
    async fn execute_update_goal(
        &self,
        goal_id: &str,
        progress: f32,
        core: &ExecutorCore,
    ) -> Result<Vec<ActionResult>, crate::agent::executor::types::ExecutorError> {
        if let Some(ref goal_tracker) = core.goal_tracker() {
            let goal_id_obj = goal_id.to_string();

            goal_tracker
                .update_progress(&goal_id_obj, progress, Some("通过 execute_update_goal 更新"))
                .await
                .map_err(|e| crate::agent::executor::types::ExecutorError::InternalError(
                    format!("更新目标失败：{}", e)
                ))?;

            if progress >= 1.0 {
                goal_tracker
                    .complete_goal(&goal_id_obj, Some("目标已完成"))
                    .await
                    .map_err(|e| crate::agent::executor::types::ExecutorError::InternalError(
                        format!("完成目标失败：{}", e)
                    ))?;
            }

            Ok(vec![ActionResult {
                action_id: format!("update_goal_{}", goal_id),
                tool_name: "update_goal".to_string(),
                success: true,
                output: Some(serde_json::json!({
                    "goal_id": goal_id,
                    "progress": progress,
                    "completed": progress >= 1.0
                })),
                error: None,
            }])
        } else {
            Err(crate::agent::executor::types::ExecutorError::InternalError(
                "GoalTracker 未初始化".to_string()
            ))
        }
    }
}
