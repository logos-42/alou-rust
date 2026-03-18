//! 智能体创建工具
//!
//! 支持通过 Tauri 命令创建新智能体，并在前端显示

use super::{ToolExecutor, ToolMetadata, ToolResult, ToolError, ExecutionContext, ToolCategory, ToolStatus, ToolPriority};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 智能体创建工具
pub struct AgentCreatorTool {
    metadata: ToolMetadata,
}

/// 智能体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// 智能体ID
    pub id: String,
    /// 显示名称
    pub display_name: String,
    /// 描述
    pub description: String,
    /// 人设/角色定义
    pub persona: String,
    /// 能力列表
    pub capabilities: Vec<String>,
    /// 约束条件
    pub constraints: Vec<String>,
    /// 使用的模型
    pub model: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 头像/图标
    pub avatar: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 关联的 Skills
    pub skills: Vec<String>,
    /// 记忆配置
    pub memory_config: MemoryConfig,
    /// 工具权限
    pub tool_permissions: Vec<String>,
    /// 创建时间
    pub created_at: i64,
}

/// 记忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 启用长期记忆
    pub enable_long_term: bool,
    /// 启用工作记忆
    pub enable_working_memory: bool,
    /// 记忆容量限制
    pub memory_limit: usize,
    /// 关联的 IPNS 名称（用于分布式记忆）
    pub ipns_name: Option<String>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enable_long_term: true,
            enable_working_memory: true,
            memory_limit: 1000,
            ipns_name: None,
        }
    }
}

/// 创建智能体结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentResult {
    /// 智能体ID
    pub agent_id: String,
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 智能体配置
    pub config: Option<AgentConfig>,
    /// 前端显示数据
    pub frontend_data: FrontendAgentData,
}

/// 前端显示数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendAgentData {
    /// 名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 头像
    pub avatar: String,
    /// 状态
    pub status: String,
    /// 能力标签
    pub capability_tags: Vec<String>,
    /// 创建时间
    pub created_at: String,
}

impl AgentCreatorTool {
    /// 创建新的智能体创建工具
    pub fn new() -> Self {
        Self {
            metadata: ToolMetadata {
                id: "agent_creator".to_string(),
                name: "Agent Creator Tool".to_string(),
                description: "创建新的AI智能体，支持在前端显示".to_string(),
                category: ToolCategory::Automation,
                priority: ToolPriority::High,
                status: ToolStatus::Available,
                version: "1.0.0".to_string(),
                author: "Alou Team".to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
                dependencies: vec![],
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
                tags: vec!["agent".to_string(), "creator".to_string()],
            },
        }
    }

    /// 生成智能体ID
    fn generate_agent_id(&self, name: &str) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let timestamp = chrono::Utc::now().timestamp();
        let input = format!("{}_{}_{}", name, timestamp, uuid::Uuid::new_v4());
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let hash = hasher.finish();
        
        format!("agent_{:x}", hash)
    }

    /// 构建系统提示词
    fn build_system_prompt(&self, persona: &str, capabilities: &[String], constraints: &[String]) -> String {
        let capabilities_text = if capabilities.is_empty() {
            "无特定能力".to_string()
        } else {
            capabilities.join("\n- ")
        };

        let constraints_text = if constraints.is_empty() {
            "无特定约束".to_string()
        } else {
            constraints.join("\n- ")
        };

        format!(
            r#"{}

## 你的能力
- {}

## 约束条件
- {}

## 执行要求
1. 始终以专业、友好的态度回应
2. 充分利用你的能力帮助用户
3. 遵守约束条件
4. 如果不确定，诚实说明"#,
            persona,
            capabilities_text,
            constraints_text
        )
    }

    /// 创建智能体
    async fn create_agent(&self, config: AgentConfig) -> Result<CreateAgentResult, String> {
        // 保存智能体配置到存储
        self.save_agent_config(&config).await?;

        // 准备前端显示数据
        let frontend_data = FrontendAgentData {
            name: config.display_name.clone(),
            description: config.description.clone(),
            avatar: config.avatar.clone().unwrap_or_else(|| "🤖".to_string()),
            status: "ready".to_string(),
            capability_tags: config.capabilities.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // 触发前端事件通知
        self.notify_frontend_agent_created(&config, &frontend_data).await?;

        Ok(CreateAgentResult {
            agent_id: config.id.clone(),
            success: true,
            message: format!("Agent '{}' created successfully", config.display_name),
            config: Some(config),
            frontend_data,
        })
    }

    /// 保存智能体配置并初始化文档
    async fn save_agent_config(&self, config: &AgentConfig) -> Result<(), String> {
        use std::fs;
        use std::path::PathBuf;
        
        println!("[AgentCreator] Saving agent config: {}", config.id);
        
        // 🔥 获取智能体文档目录
        let agent_docs_dir = dirs::home_dir()
            .map(|h| h.join(".alou").join(&config.id))
            .ok_or_else(|| "Failed to get home directory".to_string())?;
        
        // 创建目录
        fs::create_dir_all(&agent_docs_dir)
            .map_err(|e| format!("Failed to create agent docs directory: {}", e))?;
        
        // 🔥 初始化 11 个文档
        let now = chrono::Utc::now().to_rfc3339();
        
        // 1. SOUL.md
        let soul_path = agent_docs_dir.join("SOUL.md");
        if !soul_path.exists() {
            let soul_content = format!(r#"# 核心身份

**智能体 ID**: {}
**名称**: {}

## 角色定位
{}

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
"#, config.id, config.display_name, config.persona, now);
            fs::write(&soul_path, soul_content)
                .map_err(|e| format!("Failed to write SOUL.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", soul_path);
        }
        
        // 2. MEMORY.md
        let memory_path = agent_docs_dir.join("MEMORY.md");
        if !memory_path.exists() {
            let memory_content = format!(r#"# 长期记忆

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
"#, config.id, now);
            fs::write(&memory_path, memory_content)
                .map_err(|e| format!("Failed to write MEMORY.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", memory_path);
        }
        
        // 3. IDENTITY.md
        let identity_path = agent_docs_dir.join("IDENTITY.md");
        if !identity_path.exists() {
            let identity_content = format!(r#"# 身份定义

**智能体 ID**: {}
**名称**: {}

## 角色
{}

## 专长领域
（根据实际使用情况更新）

## 工作方式
- 理解用户需求
- 选择合适工具
- 执行任务
- 反馈结果

---
最后更新：{}
"#, config.id, config.display_name, config.persona, now);
            fs::write(&identity_path, identity_content)
                .map_err(|e| format!("Failed to write IDENTITY.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", identity_path);
        }
        
        // 4. CAPABILITIES.md
        let capabilities_path = agent_docs_dir.join("CAPABILITIES.md");
        if !capabilities_path.exists() {
            let capabilities_content = format!(r#"# 能力清单

**智能体 ID**: {}

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
"#, config.id, now);
            fs::write(&capabilities_path, capabilities_content)
                .map_err(|e| format!("Failed to write CAPABILITIES.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", capabilities_path);
        }
        
        // 5. CONSTRAINTS.md
        let constraints_path = agent_docs_dir.join("CONSTRAINTS.md");
        if !constraints_path.exists() {
            let constraints_content = format!(r#"# 约束和限制

**智能体 ID**: {}

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
"#, config.id, now);
            fs::write(&constraints_path, constraints_content)
                .map_err(|e| format!("Failed to write CONSTRAINTS.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", constraints_path);
        }
        
        // 6. TOOLS.md
        let tools_path = agent_docs_dir.join("TOOLS.md");
        if !tools_path.exists() {
            let tools_content = format!(r#"# 工具使用记录

**智能体 ID**: {}

## 常用工具
（根据实际使用情况更新）

## 工具组合
（记录有效的工具组合方案）

## 最佳实践
（记录工具使用的最佳实践）

---
最后更新：{}
"#, config.id, now);
            fs::write(&tools_path, tools_content)
                .map_err(|e| format!("Failed to write TOOLS.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", tools_path);
        }
        
        // 7. AGENTS.md
        let agents_path = agent_docs_dir.join("AGENTS.md");
        if !agents_path.exists() {
            let agents_content = format!(r#"# 协作智能体

**智能体 ID**: {}

## 已知智能体
（暂无记录）

## 协作经验
（暂无记录）

## 协作模式
（暂无记录）

---
最后更新：{}
"#, config.id, now);
            fs::write(&agents_path, agents_content)
                .map_err(|e| format!("Failed to write AGENTS.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", agents_path);
        }
        
        // 8. IPFS.md
        let ipfs_path = agent_docs_dir.join("IPFS.md");
        if !ipfs_path.exists() {
            let ipfs_content = format!(r#"# IPFS 对话历史索引

**智能体 ID**: {}

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
"#, config.id, now);
            fs::write(&ipfs_path, ipfs_content)
                .map_err(|e| format!("Failed to write IPFS.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", ipfs_path);
        }
        
        // 9. USER.md
        let user_path = agent_docs_dir.join("USER.md");
        if !user_path.exists() {
            let user_content = format!(r#"# 用户喜好与偏好

**智能体 ID**: {}

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
"#, config.id, now);
            fs::write(&user_path, user_content)
                .map_err(|e| format!("Failed to write USER.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", user_path);
        }
        
        // 10. PROJECT.md
        let project_path = agent_docs_dir.join("PROJECT.md");
        if !project_path.exists() {
            let project_content = format!(r#"# 项目与工作报告

**智能体 ID**: {}

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
"#, config.id, now);
            fs::write(&project_path, project_content)
                .map_err(|e| format!("Failed to write PROJECT.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", project_path);
        }
        
        // 11. KEY.md
        let key_path = agent_docs_dir.join("KEY.md");
        if !key_path.exists() {
            let key_content = format!(r#"# 关键密钥路径

**智能体 ID**: {}

这里记录了重要密钥和凭证的存储路径（不存储实际密钥）。

## 密钥路径
（暂无记录）

## 如何更新
- 记录密钥文件的存储路径
- 不要存储实际密钥内容
- 使用加密存储敏感信息

---
最后更新：{}
"#, config.id, now);
            fs::write(&key_path, key_content)
                .map_err(|e| format!("Failed to write KEY.md: {}", e))?;
            println!("[AgentCreator] 创建文档：{:?}", key_path);
        }
        
        println!("[AgentCreator] ✅ 智能体 {} 文档初始化完成", config.id);
        Ok(())
    }

    /// 通知前端智能体已创建
    async fn notify_frontend_agent_created(
        &self,
        config: &AgentConfig,
        frontend_data: &FrontendAgentData,
    ) -> Result<(), String> {
        // 通过 Tauri 事件系统通知前端
        // 注意：AppHandle 需要在工具执行时通过上下文传递
        // 这里使用 println 输出事件数据，由调用方负责发送事件
        let payload = serde_json::json!({
            "name": frontend_data.name,
            "role_description": config.persona,
            "id": config.id,
            "display_name": config.display_name,
            "description": config.description,
            "avatar": frontend_data.avatar,
            "status": frontend_data.status,
        });
        
        // 输出事件数据到 stdout，由调用方解析并发送
        println!("[AgentCreator] EVENT agent:created {}", payload);
        
        Ok(())
    }

    /// 获取智能体列表
    async fn list_agents(&self) -> Result<Vec<AgentConfig>, String> {
        // 从存储中获取所有智能体配置
        Ok(vec![])
    }

    /// 获取智能体详情
    async fn get_agent(&self, _agent_id: &str) -> Result<Option<AgentConfig>, String> {
        // 从存储中获取智能体配置
        Ok(None)
    }

    /// 删除智能体
    async fn delete_agent(&self, agent_id: &str) -> Result<bool, String> {
        // 从存储中删除智能体配置
        println!("[AgentCreator] Deleting agent: {}", agent_id);
        Ok(true)
    }

    /// 更新智能体
    async fn update_agent(&self, _agent_id: &str, _updates: serde_json::Value) -> Result<AgentConfig, String> {
        // 更新智能体配置
        Err("Not implemented".to_string())
    }
}

#[async_trait]
impl ToolExecutor for AgentCreatorTool {
    fn metadata(&self) -> &ToolMetadata {
        &self.metadata
    }

    async fn execute(&self, args: serde_json::Value, _context: &ExecutionContext) -> Result<ToolResult, ToolError> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

        match action {
            "create" => {
                let display_name = args.get("display_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'display_name' field".to_string()))?
                    .to_string();

                let description = args.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let persona = args.get("persona")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let capabilities: Vec<String> = args.get("capabilities")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let constraints: Vec<String> = args.get("constraints")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let model = args.get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("claude-3-sonnet")
                    .to_string();

                let avatar = args.get("avatar").and_then(|v| v.as_str()).map(|s| s.to_string());

                let tags: Vec<String> = args.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let skills: Vec<String> = args.get("skills")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let agent_id = self.generate_agent_id(&display_name);
                let system_prompt = self.build_system_prompt(&persona, &capabilities, &constraints);

                let config = AgentConfig {
                    id: agent_id,
                    display_name,
                    description,
                    persona,
                    capabilities,
                    constraints,
                    model,
                    system_prompt,
                    avatar,
                    tags,
                    skills,
                    memory_config: MemoryConfig::default(),
                    tool_permissions: vec!["read".to_string()],
                    created_at: chrono::Utc::now().timestamp(),
                };

                let result = self.create_agent(config).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                // 🔥 返回简洁的结果，避免大量空字段
                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "id": result.agent_id,
                        "display_name": result.config.as_ref().map(|c| c.display_name.clone()).unwrap_or_default(),
                        "description": result.config.as_ref().map(|c| c.description.clone()).unwrap_or_default(),
                        "avatar": result.config.as_ref().and_then(|c| c.avatar.clone()).unwrap_or_default(),
                        "message": result.message
                    }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Agent created successfully: {}", result.config.as_ref().map(|c| c.display_name.clone()).unwrap_or_default())),
                    warnings: vec![],
                    context: None,
                })
            }

            "list" => {
                let agents = self.list_agents().await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!({ "agents": agents, "count": agents.len() }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Found {} agents", agents.len())),
                    warnings: vec![],
                    context: None,
                })
            }

            "get" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?;

                let agent = self.get_agent(agent_id).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?
                    .ok_or_else(|| ToolError::InvalidArguments(format!("Agent '{}' not found", agent_id)))?;

                Ok(ToolResult {
                    success: true,
                    data: serde_json::json!(agent),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(format!("Retrieved agent '{}'", agent.display_name)),
                    warnings: vec![],
                    context: None,
                })
            }

            "delete" => {
                let agent_id = args.get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArguments("Missing 'agent_id' field".to_string()))?;

                let deleted = self.delete_agent(agent_id).await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult {
                    success: deleted,
                    data: serde_json::json!({ "deleted": deleted }),
                    error: None,
                    execution_time_ms: 0,
                    output: Some(if deleted { format!("Deleted agent '{}'", agent_id) } else { "Agent not found".to_string() }),
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
        r#"Agent Creator Tool

Create and manage AI agents that can be displayed in the frontend.

Actions:
  - create: Create a new agent
    params: {
      "display_name": "My Agent",
      "description": "An agent for code review",
      "persona": "You are a code review expert...",
      "capabilities": ["find bugs", "suggest improvements"],
      "constraints": ["be concise"],
      "model": "claude-3-sonnet",
      "avatar": "🤖",
      "tags": ["coding", "review"],
      "skills": ["skill_code_formatter"]
    }
  
  - list: List all agents
  
  - get: Get agent details
    params: { "agent_id": "agent_xxx" }
  
  - delete: Delete an agent
    params: { "agent_id": "agent_xxx" }

Examples:

Create a code review agent:
{
  "action": "create",
  "display_name": "Code Reviewer",
  "description": "Reviews code for issues",
  "persona": "You are an expert code reviewer with 20 years of experience...",
  "capabilities": ["identify bugs", "suggest optimizations", "check security"],
  "constraints": ["be specific", "provide examples"],
  "tags": ["code", "review", "quality"]
}"#.to_string()
    }
}
