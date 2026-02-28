use crate::prompts::{PromptManager, PromptContext};
use crate::tools::{ToolRegistry, ToolResult, ExecutionContext};
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use std::sync::Arc;

/// 分层提示系统 - 与现有 PromptManager 集成
pub struct HierarchicalPromptSystem {
    /// 引用现有的 PromptManager
    pub prompt_manager: Arc<PromptManager>,
    /// 任务级提示存储
    pub task_prompts: HashMap<String, TaskPrompt>,
    /// 工具级提示存储
    pub tool_prompts: HashMap<String, ToolPrompt>,
    /// 上下文管理器
    pub context_manager: ContextManager,
}

#[derive(Debug, Clone)]
pub struct TaskPrompt {
    /// 任务ID
    pub id: String,
    /// 任务描述
    pub description: String,
    /// 任务目标
    pub objectives: Vec<String>,
    /// 任务约束
    pub constraints: Vec<String>,
    /// 任务优先级
    pub priority: Priority,
    /// 相关工具
    pub relevant_tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ToolPrompt {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 使用场景
    pub use_cases: Vec<String>,
    /// 使用指导
    pub guidance: Vec<String>,
    /// 注意事项
    pub considerations: Vec<String>,
    /// 成功指标
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContextManager {
    /// 当前上下文摘要
    pub summary: String,
    /// 重要信息提取
    pub key_facts: Vec<String>,
    /// 决策历史
    pub decision_log: Vec<DecisionEntry>,
    /// 上下文窗口大小限制
    pub max_context_size: usize,
    /// 摘要阈值
    pub summary_threshold: usize,
}

#[derive(Debug, Clone)]
pub struct DecisionEntry {
    /// 决策时间戳
    pub timestamp: i64,
    /// 决策内容
    pub decision: String,
    /// 使用的工具
    pub tools_used: Vec<String>,
    /// 决策结果
    pub result: String,
    /// 影响评估
    pub impact: ImpactLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl HierarchicalPromptSystem {
    pub fn new(prompt_manager: Arc<PromptManager>) -> Self {
        Self {
            prompt_manager,
            task_prompts: HashMap::new(),
            tool_prompts: HashMap::new(),
            context_manager: ContextManager {
                summary: String::new(),
                key_facts: Vec::new(),
                decision_log: Vec::new(),
                max_context_size: 5000, // 字符数限制
                summary_threshold: 1000, // 摘要阈值
            },
        }
    }

    /// 添加任务提示
    pub fn add_task_prompt(&mut self, task: TaskPrompt) {
        self.task_prompts.insert(task.id.clone(), task);
    }

    /// 添加工具提示
    pub fn add_tool_prompt(&mut self, tool_name: String, prompt: ToolPrompt) {
        self.tool_prompts.insert(tool_name, prompt);
    }

    /// 获取任务级提示
    pub fn get_task_prompt(&self, task_id: &str) -> Option<String> {
        self.task_prompts.get(task_id).map(|task| {
            format!(
                "任务: {}\n目标: {}\n约束: {}\n优先级: {:?}\n相关工具: {:?}",
                task.description,
                task.objectives.join(", "),
                task.constraints.join(", "),
                task.priority,
                task.relevant_tools
            )
        })
    }

    /// 获取工具级提示
    pub fn get_tool_prompt(&self, tool_name: &str) -> Option<String> {
        self.tool_prompts.get(tool_name).map(|tool| {
            format!(
                "工具: {}\n描述: {}\n使用场景: {}\n使用指导: {}\n注意事项: {}\n成功标准: {}",
                tool.name,
                tool.description,
                tool.use_cases.join(", "),
                tool.guidance.join("\n- "),
                tool.considerations.join(", "),
                tool.success_criteria.join(", ")
            )
        })
    }

    /// 更新上下文
    pub fn update_context(&mut self, new_info: &str) {
        self.context_manager.key_facts.push(new_info.to_string());
        
        // 如果上下文超过阈值，生成摘要
        let total_chars: usize = self.context_manager.key_facts.iter()
            .map(|f| f.len())
            .sum();
            
        if total_chars > self.context_manager.summary_threshold {
            self.generate_summary();
        }
    }

    /// 生成上下文摘要
    fn generate_summary(&mut self) {
        let summary_parts = vec![
            "关键事实:".to_string(),
            self.context_manager.key_facts
                .iter()
                .take(10) // 只取最近的10个关键事实
                .cloned()
                .collect::<Vec<_>>()
                .join("\n- "),
            "\n决策历史:".to_string(),
            self.context_manager.decision_log
                .iter()
                .rev()
                .take(5) // 只取最近的5个决策
                .map(|d| format!("{}: {} -> {}", d.timestamp, d.decision, d.result))
                .collect::<Vec<_>>()
                .join("\n- "),
        ];
        
        self.context_manager.summary = summary_parts.join("\n\n");
    }

    /// 添加决策记录
    pub fn log_decision(&mut self, decision: String, tools_used: Vec<String>, result: String, impact: ImpactLevel) {
        self.context_manager.decision_log.push(DecisionEntry {
            timestamp: chrono::Utc::now().timestamp(),
            decision,
            tools_used,
            result,
            impact,
        });
    }

    /// 获取当前上下文
    pub fn get_current_context(&self) -> String {
        if self.context_manager.summary.is_empty() {
            self.context_manager.key_facts.join("\n")
        } else {
            self.context_manager.summary.clone()
        }
    }
}

/// AI决策逻辑
#[derive(Debug, Clone)]
pub struct AIDecisionLogic {
    /// 决策策略
    pub strategies: Vec<DecisionStrategy>,
    /// 风险评估
    pub risk_assessment: RiskAssessment,
    /// 优先级算法
    pub priority_algorithm: PriorityAlgorithm,
}

#[derive(Debug, Clone)]
pub struct DecisionStrategy {
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 触发条件
    pub conditions: Vec<Condition>,
    /// 执行动作
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    /// 条件类型
    pub condition_type: ConditionType,
    /// 条件参数
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    ToolSuccess,
    ToolFailure,
    ContextChange,
    TimeThreshold,
    ResourceAvailability,
    UserInput,
}

#[derive(Debug, Clone)]
pub struct Action {
    /// 动作类型
    pub action_type: ActionType,
    /// 动作参数
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    UseTool,
    Wait,
    Retry,
    Abort,
    SwitchContext,
    LogEvent,
}

#[derive(Debug, Clone)]
pub struct RiskAssessment {
    /// 风险等级
    pub level: RiskLevel,
    /// 风险因素
    pub factors: Vec<RiskFactor>,
    /// 缓解措施
    pub mitigation_strategies: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RiskFactor {
    /// 因素名称
    pub name: String,
    /// 影响程度
    pub impact: f64, // 0.0-1.0
    /// 发生概率
    pub probability: f64, // 0.0-1.0
}

#[derive(Debug, Clone)]
pub struct PriorityAlgorithm {
    /// 优先级计算方法
    pub method: PriorityMethod,
    /// 权重配置
    pub weights: HashMap<PriorityFactor, f64>,
}

#[derive(Debug, Clone)]
pub enum PriorityMethod {
    WeightedSum,
    MultiCriteria,
    TimeBased,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PriorityFactor {
    Urgency,
    Importance,
    ResourceCost,
    Risk,
    Dependency,
    UserPreference,
}

impl AIDecisionLogic {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                DecisionStrategy {
                    name: "Standard Tool Usage".to_string(),
                    description: "标准工具使用策略".to_string(),
                    conditions: vec![
                        Condition {
                            condition_type: ConditionType::ResourceAvailability,
                            parameters: [("resource_type".to_string(), "tool".to_string())].iter().cloned().collect(),
                        }
                    ],
                    actions: vec![
                        Action {
                            action_type: ActionType::UseTool,
                            parameters: [("max_retries".to_string(), "3".to_string())].iter().cloned().collect(),
                        }
                    ],
                },
                DecisionStrategy {
                    name: "Error Recovery".to_string(),
                    description: "错误恢复策略".to_string(),
                    conditions: vec![
                        Condition {
                            condition_type: ConditionType::ToolFailure,
                            parameters: [("failure_type".to_string(), "transient".to_string())].iter().cloned().collect(),
                        }
                    ],
                    actions: vec![
                        Action {
                            action_type: ActionType::Retry,
                            parameters: [("delay_seconds".to_string(), "5".to_string())].iter().cloned().collect(),
                        }
                    ],
                },
                DecisionStrategy {
                    name: "Critical Failure".to_string(),
                    description: "严重故障处理策略".to_string(),
                    conditions: vec![
                        Condition {
                            condition_type: ConditionType::ToolFailure,
                            parameters: [("failure_type".to_string(), "critical".to_string())].iter().cloned().collect(),
                        }
                    ],
                    actions: vec![
                        Action {
                            action_type: ActionType::Abort,
                            parameters: [("notify_user".to_string(), "true".to_string())].iter().cloned().collect(),
                        }
                    ],
                }
            ],
            risk_assessment: RiskAssessment {
                level: RiskLevel::Medium,
                factors: vec![
                    RiskFactor {
                        name: "Data Loss".to_string(),
                        impact: 0.8,
                        probability: 0.1,
                    },
                    RiskFactor {
                        name: "Privacy Breach".to_string(),
                        impact: 1.0,
                        probability: 0.05,
                    },
                    RiskFactor {
                        name: "Resource Exhaustion".to_string(),
                        impact: 0.6,
                        probability: 0.2,
                    },
                ],
                mitigation_strategies: vec![
                    "Implement proper error handling".to_string(),
                    "Validate all inputs".to_string(),
                    "Monitor resource usage".to_string(),
                    "Log all actions for audit trail".to_string(),
                ],
            },
            priority_algorithm: PriorityAlgorithm {
                method: PriorityMethod::WeightedSum,
                weights: [
                    (PriorityFactor::Urgency, 0.3),
                    (PriorityFactor::Importance, 0.3),
                    (PriorityFactor::ResourceCost, 0.2),
                    (PriorityFactor::Risk, 0.1),
                    (PriorityFactor::Dependency, 0.1),
                ].iter().cloned().collect(),
            },
        }
    }

    /// 评估决策
    pub fn evaluate_decision(&self, strategy: &DecisionStrategy, context: &str) -> DecisionOutcome {
        // 检查所有条件是否满足
        let all_conditions_met = strategy.conditions.iter().all(|condition| {
            self.check_condition(condition, context)
        });

        if all_conditions_met {
            DecisionOutcome::Proceed(strategy.actions.clone())
        } else {
            DecisionOutcome::Defer
        }
    }

    /// 检查条件
    fn check_condition(&self, condition: &Condition, context: &str) -> bool {
        match condition.condition_type {
            ConditionType::ToolSuccess => {
                // 检查上下文中是否包含成功信息
                context.contains("success") || context.contains("completed")
            },
            ConditionType::ToolFailure => {
                // 检查上下文中是否包含失败信息
                context.contains("failed") || context.contains("error") || context.contains("timeout")
            },
            ConditionType::ContextChange => {
                // 检查上下文是否发生变化
                !context.is_empty()
            },
            ConditionType::TimeThreshold => {
                // 检查时间阈值（这里简化处理）
                true
            },
            ConditionType::ResourceAvailability => {
                // 检查资源可用性（这里简化处理）
                true
            },
            ConditionType::UserInput => {
                // 检查是否有用户输入
                context.contains("user:") || context.contains("input:")
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum DecisionOutcome {
    Proceed(Vec<Action>),
    Defer,
    Abort(String),
}

impl Default for AIDecisionLogic {
    fn default() -> Self {
        Self::new()
    }
}