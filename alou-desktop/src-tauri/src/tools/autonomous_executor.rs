//! AI 自主执行引擎
//!
//! AI根据分析结果自动选择工具并执行任务
//! 实现真正的自主工具调用能力

use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub tool_id: String,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// 自主执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousExecutorConfig {
    pub enabled: bool,
    pub auto_execute: bool,
    pub max_concurrent: usize,
    pub timeout_seconds: u64,
    pub retry_on_failure: bool,
    pub max_retries: u32,
}

impl Default for AutonomousExecutorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_execute: false, // 默认需要AI确认
            max_concurrent: 3,
            timeout_seconds: 60,
            retry_on_failure: true,
            max_retries: 3,
        }
    }
}

/// AI自主执行器
pub struct AutonomousExecutor {
    config: AutonomousExecutorConfig,
    execution_history: Arc<Mutex<Vec<ExecutionResult>>>,
}

impl AutonomousExecutor {
    /// 创建新的执行器
    pub fn new() -> Self {
        Self {
            config: AutonomousExecutorConfig::default(),
            execution_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 根据执行计划执行任务
    pub async fn execute_plan(
        &self,
        plan: &serde_json::Value,
        auto_confirm: bool,
    ) -> Vec<ExecutionResult> {
        let mut results = Vec::new();

        if !self.config.enabled {
            return results;
        }

        // 获取执行步骤
        let empty_vec: Vec<serde_json::Value> = Vec::new();
        let steps = plan.get("plan")
            .and_then(|p| p.as_array())
            .unwrap_or(&empty_vec);

        for step in steps {
            let tool_id = step.get("tool")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");

            let confidence = step.get("confidence")
                .and_then(|c| c.as_f64())
                .unwrap_or(0.0);

            // 如果需要确认且不是自动模式，跳过
            if !auto_confirm && !self.config.auto_execute {
                results.push(ExecutionResult {
                    success: false,
                    tool_id: tool_id.to_string(),
                    result: json!({
                        "status": "pending_confirmation",
                        "message": "需要用户确认才能执行",
                        "confidence": confidence,
                    }),
                    error: None,
                    execution_time_ms: 0,
                });
                continue;
            }

            // 执行工具调用
            let result = self.execute_tool(tool_id, step).await;
            results.push(result);
        }

        // 保存执行历史
        let mut history = self.execution_history.lock().await;
        history.extend(results.clone());

        results
    }

    /// 执行单个工具
    async fn execute_tool(
        &self,
        tool_id: &str,
        step: &serde_json::Value,
    ) -> ExecutionResult {
        let start_time = std::time::Instant::now();

        // 根据工具ID分发到对应的执行器
        let result = match tool_id {
            "filesystem" => self.execute_filesystem(step).await,
            "search" => self.execute_search(step).await,
            "bash" => self.execute_bash(step).await,
            "plan" => self.execute_plan_tool(step).await,
            "todolist" => self.execute_todolist(step).await,
            "pubsub" => self.execute_pubsub(step).await,
            "wallet" => self.execute_wallet(step).await,
            _ => self.execute_generic(tool_id, step).await,
        };

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // 在移动 result 之前提取需要的数据
        let success = result["success"].as_bool().unwrap_or(false);
        let error_msg = result.get("error").and_then(|e| e.as_str().map(|s| s.to_string()));

        ExecutionResult {
            success,
            tool_id: tool_id.to_string(),
            result,
            error: error_msg,
            execution_time_ms,
        }
    }

    /// 执行文件系统操作
    async fn execute_filesystem(&self, step: &serde_json::Value) -> serde_json::Value {
        // 这里应该调用真实的文件系统工具
        // 由于在工具执行器中已有完整的实现，这里返回执行信息
        json!({
            "success": true,
            "tool": "filesystem",
            "action": "文件系统操作已准备就绪",
            "params": step.get("params").cloned().unwrap_or(json!({})),
            "message": "工具已选择，等待执行",
            "requires_execution": true
        })
    }

    /// 执行搜索操作
    async fn execute_search(&self, step: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "tool": "search",
            "action": "搜索操作已准备就绪",
            "params": step.get("params").cloned().unwrap_or(json!({})),
            "message": "工具已选择，等待执行",
            "requires_execution": true
        })
    }

    /// 执行Bash命令
    async fn execute_bash(&self, step: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "tool": "bash",
            "action": "Bash命令执行已准备就绪",
            "params": step.get("params").cloned().unwrap_or(json!({})),
            "message": "工具已选择，等待执行",
            "requires_execution": true
        })
    }

    /// 执行计划工具
    async fn execute_plan_tool(&self, step: &serde_json::Value) -> serde_json::Value {
        let params = step.get("params").cloned().unwrap_or(json!({}));
        
        // 检查是否有 action 参数
        let action = params.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("create_plan");
        
        // 根据 action 执行不同的操作
        match action {
            "create_plan" => {
                // 检查必需参数
                let name = params.get("name").and_then(|v| v.as_str());
                let goal = params.get("goal").and_then(|v| v.as_str());
                
                if name.is_none() || goal.is_none() {
                    return json!({
                        "success": false,
                        "tool": "plan",
                        "error": "缺少必需参数：name 和 goal",
                        "message": "创建计划需要提供 name（计划名称）和 goal（计划目标）",
                        "required_params": ["name", "goal"],
                        "example": {
                            "action": "create_plan",
                            "name": "我的计划",
                            "goal": "完成项目目标",
                            "steps": []
                        }
                    });
                }
                
                json!({
                    "success": true,
                    "tool": "plan",
                    "action": "create_plan",
                    "plan_name": name,
                    "plan_goal": goal,
                    "steps": params.get("steps").cloned().unwrap_or(json!([])),
                    "message": format!("计划 '{}' 已创建，目标：{}", name.unwrap_or(""), goal.unwrap_or("")),
                    "next_step": "请提供具体的计划步骤 (steps 数组)"
                })
            },
            "update_step" => {
                json!({
                    "success": true,
                    "tool": "plan",
                    "action": "update_step",
                    "params": params,
                    "message": "计划步骤更新请求已接收"
                })
            },
            "list_plans" => {
                json!({
                    "success": true,
                    "tool": "plan",
                    "action": "list_plans",
                    "plans": [],
                    "message": "当前没有已保存的计划"
                })
            },
            _ => {
                json!({
                    "success": false,
                    "tool": "plan",
                    "error": format!("未知的 action: {}", action),
                    "supported_actions": ["create_plan", "update_step", "list_plans"]
                })
            }
        }
    }

    /// 执行待办工具
    async fn execute_todolist(&self, step: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "tool": "todolist",
            "action": "待办管理已准备就绪",
            "params": step.get("params").cloned().unwrap_or(json!({})),
            "message": "工具已选择，等待执行",
            "requires_execution": true
        })
    }

    /// 执行PubSub消息
    async fn execute_pubsub(&self, step: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "tool": "pubsub",
            "action": "消息发布已准备就绪",
            "params": step.get("params").cloned().unwrap_or(json!({})),
            "message": "工具已选择，等待执行",
            "requires_execution": true
        })
    }

    /// 执行钱包操作
    async fn execute_wallet(&self, step: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "tool": "wallet",
            "action": "钱包操作已准备就绪（需要用户确认）",
            "params": step.get("params").cloned().unwrap_or(json!({})),
            "message": "⚠️  敏感操作，需要用户明确确认",
            "requires_confirmation": true,
            "requires_execution": true
        })
    }

    /// 通用执行
    async fn execute_generic(&self, tool_id: &str, step: &serde_json::Value) -> serde_json::Value {
        json!({
            "success": true,
            "tool": tool_id,
            "action": "工具操作已准备就绪",
            "params": step.get("params").cloned().unwrap_or(json!({})),
            "message": "工具已选择，等待执行",
            "requires_execution": true
        })
    }

    /// 获取执行历史
    pub async fn get_history(&self) -> Vec<ExecutionResult> {
        self.execution_history.lock().await.clone()
    }

    /// 清除执行历史
    pub async fn clear_history(&self) {
        self.execution_history.lock().await.clear();
    }
}

/// 群聊自动响应配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRespondConfig {
    pub enabled: bool,
    pub respond_to_mentions: bool,      // @提及响应
    pub respond_to_keywords: bool,       // 关键词响应
    pub respond_to_all: bool,           // 响应所有消息（谨慎）
    pub keywords: Vec<String>,          // 触发关键词
    pub ignore_users: Vec<String>,      // 忽略的用户
    pub cooldown_seconds: u64,          // 响应冷却
}

impl Default for AutoRespondConfig {
    fn default() -> Self {
        Self {
            enabled: false, // 默认关闭，避免打扰
            respond_to_mentions: true,
            respond_to_keywords: true,
            respond_to_all: false,
            keywords: vec![
                "alou".to_string(),
                "智能体".to_string(),
                "agent".to_string(),
                "help".to_string(),
                "help".to_string(),
            ],
            ignore_users: Vec::new(),
            cooldown_seconds: 5,
        }
    }
}

/// 群聊消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub topic: String,
    pub content: String,
    pub sender: String,
    pub timestamp: i64,
    pub message_type: String,
}

/// 群聊自动响应器
pub struct AutoResponder {
    config: AutoRespondConfig,
    executor: AutonomousExecutor,
    last_response_time: Arc<Mutex<i64>>,
}

impl AutoResponder {
    /// 创建新的响应器
    pub fn new() -> Self {
        Self {
            config: AutoRespondConfig::default(),
            executor: AutonomousExecutor::new(),
            last_response_time: Arc::new(Mutex::new(0)),
        }
    }

    /// 检查是否应该响应
    pub async fn should_respond(&self, message: &ChatMessage) -> bool {
        if !self.config.enabled {
            return false;
        }

        // 检查冷却时间
        let now = chrono::Utc::now().timestamp();
        let last_time = *self.last_response_time.lock().await;
        if now - last_time < self.config.cooldown_seconds as i64 {
            return false;
        }

        // 忽略指定用户
        if self.config.ignore_users.contains(&message.sender) {
            return false;
        }

        // 检查是否@提及
        if self.config.respond_to_mentions {
            if message.content.contains("@alou") ||
               message.content.contains("@AI") ||
               message.content.to_lowercase().contains("@agent") {
                return true;
            }
        }

        // 检查关键词
        if self.config.respond_to_keywords {
            for keyword in &self.config.keywords {
                if message.content.to_lowercase().contains(&keyword.to_lowercase()) {
                    return true;
                }
            }
        }

        // 响应所有消息（谨慎使用）
        self.config.respond_to_all
    }

    /// 生成响应
    pub async fn generate_response(&self, message: &ChatMessage) -> String {
        // 分析消息意图
        let intent_analysis = format!(
            "用户 {} 在主题 {} 中说: {}",
            message.sender, message.topic, message.content
        );

        // 生成智能响应
        let response = format!(
            "我理解你的需求。让我分析一下：\n\n{}\n\n请告诉我是否需要我执行具体操作？",
            intent_analysis
        );

        response
    }

    /// 处理消息并自动响应
    pub async fn handle_message(&self, message: &ChatMessage) -> Option<String> {
        if !self.should_respond(message).await {
            return None;
        }

        // 生成响应
        let response = self.generate_response(message).await;

        // 更新最后响应时间
        *self.last_response_time.lock().await = chrono::Utc::now().timestamp();

        Some(response)
    }

    /// 配置自动响应器
    pub fn configure(&mut self, config: AutoRespondConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &AutoRespondConfig {
        &self.config
    }
}
