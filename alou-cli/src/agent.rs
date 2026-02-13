//! Alou CLI Agent Module
//! 处理Agent对话和自主行动功能

use crate::api::{self, Config, ChatMessage};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Agent状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub name: String,
    pub mode: String,
    pub is_autonomous: bool,
    pub conversation_history: VecDeque<ChatMessage>,
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: String,
    pub assigned_to: Option<String>,
    pub progress: u8,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Agent实例
pub struct Agent {
    pub state: AgentState,
    pub config: Config,
}

impl Agent {
    /// 创建新Agent
    pub fn new(name: String, config: Config) -> Self {
        Self {
            state: AgentState {
                name,
                mode: "interactive".to_string(),
                is_autonomous: false,
                conversation_history: VecDeque::new(),
            },
            config,
        }
    }

    /// 发送聊天消息并获取响应
    pub async fn chat(&mut self, message: &str) -> Result<String, String> {
        // 添加用户消息到历史
        self.state.conversation_history.push_back(ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
        });

        // 构建消息列表
        let messages: Vec<ChatMessage> = self.state.conversation_history.iter().cloned().collect();

        // 调用API
        let response = api::send_chat_request(&self.config, messages).await?;

        // 添加助手回复到历史
        self.state.conversation_history.push_back(ChatMessage {
            role: "assistant".to_string(),
            content: response.clone(),
        });

        // 限制历史长度
        if self.state.conversation_history.len() > 20 {
            self.state.conversation_history.pop_front();
        }

        Ok(response)
    }

    /// 启用自主模式
    pub fn enable_autonomous(&mut self) {
        self.state.is_autonomous = true;
        self.state.mode = "autonomous".to_string();
    }

    /// 禁用自主模式
    pub fn disable_autonomous(&mut self) {
        self.state.is_autonomous = false;
        self.state.mode = "interactive".to_string();
    }

    /// 获取Agent状态描述
    pub fn get_status(&self) -> String {
        let status = if self.state.is_autonomous {
            "🟢 自主模式"
        } else {
            "🔵 交互模式"
        };
        
        let history_len = self.state.conversation_history.len();
        
        format!(
            "{} | 对话历史: {}条 | 名称: {}",
            status,
            history_len,
            self.state.name
        )
    }

    /// 清除对话历史
    pub fn clear_history(&mut self) {
        self.state.conversation_history.clear();
    }
}

/// 加载保存的Agent状态
pub fn load_agent_state(name: &str) -> Option<AgentState> {
    let path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".alou")
        .join("agents")
        .join(format!("{}.json", name));

    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&content) {
                return Some(state);
            }
        }
    }
    None
}

/// 保存Agent状态
pub fn save_agent_state(name: &str, state: &AgentState) -> Result<(), String> {
    let path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".alou")
        .join("agents");
    
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    
    let file_path = path.join(format!("{}.json", name));
    let content = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(file_path, content).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 列出所有保存的Agent
pub fn list_saved_agents() -> Vec<String> {
    let path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".alou")
        .join("agents");
    
    if !path.exists() {
        return vec![];
    }
    
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension().map(|s| s == "json").unwrap_or(false) {
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}
