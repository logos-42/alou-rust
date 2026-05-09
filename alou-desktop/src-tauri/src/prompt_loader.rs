use crate::prompt_system::{
    HierarchicalPromptSystem, TaskPrompt, ToolPrompt, AIDecisionLogic,
    DecisionStrategy, Condition, ConditionType, Action, ActionType,
    RiskAssessment, RiskFactor, RiskLevel, PriorityAlgorithm, PriorityMethod,
    PriorityFactor, Priority
};
use crate::prompts::PromptManager;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

impl HierarchicalPromptSystem {
    /// 从配置文件加载分层提示系统
    pub async fn from_config(config_path: &str, prompt_manager: Arc<PromptManager>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(config_path)?;
        let config: Value = serde_json::from_str(&config_str)?;

        let system = HierarchicalPromptSystem::new(prompt_manager);

        // 加载任务提示
        if let Some(tasks) = config["tasks"].as_array() {
            for task_val in tasks {
                let task = TaskPrompt {
                    id: task_val["id"].as_str().unwrap_or("").to_string(),
                    description: task_val["description"].as_str().unwrap_or("").to_string(),
                    objectives: task_val["objectives"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                    constraints: task_val["constraints"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                    priority: match task_val["priority"].as_str().unwrap_or("Medium") {
                        "Low" => Priority::Low,
                        "High" => Priority::High,
                        "Critical" => Priority::Critical,
                        _ => Priority::Medium,
                    },
                    relevant_tools: task_val["relevant_tools"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                };
                system.add_task_prompt(task).await;
            }
        }

        // 加载工具提示
        if let Some(tools) = config["tools"].as_object() {
            for (tool_name, tool_val) in tools {
                let tool = ToolPrompt {
                    name: tool_val["name"].as_str().unwrap_or(tool_name).to_string(),
                    description: tool_val["description"].as_str().unwrap_or("").to_string(),
                    use_cases: tool_val["use_cases"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                    guidance: tool_val["guidance"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                    considerations: tool_val["considerations"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                    success_criteria: tool_val["success_criteria"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect(),
                };
                system.add_tool_prompt(tool_name.clone(), tool).await;
            }
        }

        Ok(system)
    }
}

impl AIDecisionLogic {
    /// 从配置文件加载AI决策逻辑
    pub fn from_config(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(config_path)?;
        let config: Value = serde_json::from_str(&config_str)?;

        let mut strategies = Vec::new();

        // 加载决策策略
        if let Some(strategy_vals) = config["decision_strategies"].as_array() {
            for strategy_val in strategy_vals {
                let conditions = if let Some(condition_vals) = strategy_val["conditions"].as_array() {
                    condition_vals.iter().map(|cond_val| {
                        let condition_type = match cond_val["condition_type"].as_str().unwrap_or("") {
                            "ToolSuccess" => ConditionType::ToolSuccess,
                            "ToolFailure" => ConditionType::ToolFailure,
                            "ContextChange" => ConditionType::ContextChange,
                            "TimeThreshold" => ConditionType::TimeThreshold,
                            "ResourceAvailability" => ConditionType::ResourceAvailability,
                            "UserInput" => ConditionType::UserInput,
                            _ => ConditionType::ContextChange,
                        };

                        let parameters = if let Some(params_obj) = cond_val["parameters"].as_object() {
                            params_obj.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()
                        } else {
                            HashMap::new()
                        };

                        Condition {
                            condition_type,
                            parameters,
                        }
                    }).collect()
                } else {
                    Vec::new()
                };

                let actions = if let Some(action_vals) = strategy_val["actions"].as_array() {
                    action_vals.iter().map(|act_val| {
                        let action_type = match act_val["action_type"].as_str().unwrap_or("") {
                            "UseTool" => ActionType::UseTool,
                            "Wait" => ActionType::Wait,
                            "Retry" => ActionType::Retry,
                            "Abort" => ActionType::Abort,
                            "SwitchContext" => ActionType::SwitchContext,
                            "LogEvent" => ActionType::LogEvent,
                            _ => ActionType::LogEvent,
                        };

                        let parameters = if let Some(params_obj) = act_val["parameters"].as_object() {
                            params_obj.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()
                        } else {
                            HashMap::new()
                        };

                        Action {
                            action_type,
                            parameters,
                        }
                    }).collect()
                } else {
                    Vec::new()
                };

                let strategy = DecisionStrategy {
                    name: strategy_val["name"].as_str().unwrap_or("").to_string(),
                    description: strategy_val["description"].as_str().unwrap_or("").to_string(),
                    conditions,
                    actions,
                };

                strategies.push(strategy);
            }
        }

        // 加载风险评估
        let risk_assessment = if let Some(risk_val) = config["risk_assessment"].as_object() {
            let factors = if let Some(factor_vals) = risk_val["factors"].as_array() {
                factor_vals.iter().map(|factor_val| {
                    RiskFactor {
                        name: factor_val["name"].as_str().unwrap_or("").to_string(),
                        impact: factor_val["impact"].as_f64().unwrap_or(0.0),
                        probability: factor_val["probability"].as_f64().unwrap_or(0.0),
                    }
                }).collect()
            } else {
                Vec::new()
            };

            let mitigation_strategies = if let Some(mitigation_vals) = risk_val["mitigation_strategies"].as_array() {
                mitigation_vals.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            };

            RiskAssessment {
                level: match risk_val["level"].as_str().unwrap_or("Medium") {
                    "Low" => RiskLevel::Low,
                    "High" => RiskLevel::High,
                    "Critical" => RiskLevel::Critical,
                    _ => RiskLevel::Medium,
                },
                factors,
                mitigation_strategies,
            }
        } else {
            RiskAssessment {
                level: RiskLevel::Medium,
                factors: Vec::new(),
                mitigation_strategies: Vec::new(),
            }
        };

        // 加载优先级算法
        let priority_algorithm = if let Some(algo_val) = config["priority_algorithm"].as_object() {
            let weights = if let Some(weights_obj) = algo_val["weights"].as_object() {
                let mut weight_map = HashMap::new();
                for (factor_str, weight_val) in weights_obj {
                    let factor = match factor_str.as_str() {
                        "Urgency" => PriorityFactor::Urgency,
                        "Importance" => PriorityFactor::Importance,
                        "ResourceCost" => PriorityFactor::ResourceCost,
                        "Risk" => PriorityFactor::Risk,
                        "Dependency" => PriorityFactor::Dependency,
                        "UserPreference" => PriorityFactor::UserPreference,
                        _ => PriorityFactor::Urgency,
                    };
                    weight_map.insert(factor, weight_val.as_f64().unwrap_or(0.0));
                }
                weight_map
            } else {
                HashMap::new()
            };

            PriorityAlgorithm {
                method: match algo_val["method"].as_str().unwrap_or("WeightedSum") {
                    "MultiCriteria" => PriorityMethod::MultiCriteria,
                    "TimeBased" => PriorityMethod::TimeBased,
                    _ => PriorityMethod::WeightedSum,
                },
                weights,
            }
        } else {
            PriorityAlgorithm {
                method: PriorityMethod::WeightedSum,
                weights: HashMap::new(),
            }
        };

        Ok(AIDecisionLogic {
            strategies,
            risk_assessment,
            priority_algorithm,
        })
    }
}