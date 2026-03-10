//! Agent Hook 系统 - 支持在执行过程中接收和处理用户的新指令
//!
//! 提供实时指令注入、执行中断、优先级处理等功能

use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono;

/// 指令优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionPriority {
    /// 低优先级 - 排队等待
    Low,
    /// 中优先级 - 正常处理
    Medium,
    /// 高优先级 - 插队处理
    High,
    /// 紧急优先级 - 立即中断当前执行
    Critical,
}

impl Default for InstructionPriority {
    fn default() -> Self {
        InstructionPriority::Medium
    }
}

/// 指令类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionType {
    /// 普通指令 - 添加到执行队列
    Normal,
    /// 修改指令 - 修改当前执行参数
    Modify,
    /// 暂停指令 - 暂停当前执行
    Pause,
    /// 恢复指令 - 恢复执行
    Resume,
    /// 取消指令 - 取消当前执行
    Cancel,
    /// 重置指令 - 重置执行状态
    Reset,
    /// 查询状态 - 查询当前执行状态
    QueryStatus,
}

/// 用户指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInstruction {
    /// 指令 ID
    pub id: String,
    /// 会话 ID
    pub session_id: String,
    /// Agent ID
    pub agent_id: Option<String>,
    /// 指令内容
    pub content: String,
    /// 指令类型
    pub instruction_type: InstructionType,
    /// 优先级
    pub priority: InstructionPriority,
    /// 创建时间
    pub created_at: i64,
    /// 是否已处理
    pub processed: bool,
    /// 处理结果
    pub result: Option<String>,
    /// 附加数据
    pub metadata: Option<serde_json::Value>,
}

impl UserInstruction {
    pub fn new(
        session_id: String,
        content: String,
        instruction_type: InstructionType,
    ) -> Self {
        Self {
            id: format!("inst_{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            session_id,
            agent_id: None,
            content,
            instruction_type,
            priority: InstructionPriority::default(),
            created_at: chrono::Utc::now().timestamp(),
            processed: false,
            result: None,
            metadata: None,
        }
    }

    pub fn with_priority(mut self, priority: InstructionPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }
}

/// Agent 执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentExecutionStatus {
    /// 空闲
    Idle,
    /// 正在执行
    Executing {
        current_task: String,
        progress: f64,
        started_at: i64,
    },
    /// 已暂停
    Paused {
        paused_at: i64,
        reason: String,
    },
    /// 已完成
    Completed {
        result: String,
        completed_at: i64,
    },
    /// 已取消
    Cancelled {
        reason: String,
        cancelled_at: i64,
    },
    /// 执行失败
    Failed {
        error: String,
        failed_at: i64,
    },
}

/// Agent 状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateSnapshot {
    /// Agent ID
    pub agent_id: String,
    /// 会话 ID
    pub session_id: String,
    /// 执行状态
    pub execution_status: AgentExecutionStatus,
    /// 当前任务描述
    pub current_task: Option<String>,
    /// 执行进度 (0-100)
    pub progress: f64,
    /// 已处理的指令数量
    pub processed_instructions: usize,
    /// 待处理的指令数量
    pub pending_instructions: usize,
    /// 快照时间
    pub snapshot_time: i64,
}

/// Hook 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHookConfig {
    /// 是否启用 Hook
    pub enabled: bool,
    /// 是否允许中断执行
    pub allow_interruption: bool,
    /// 是否允许修改指令
    pub allow_modification: bool,
    /// 指令队列最大长度
    pub max_queue_size: usize,
    /// 指令处理超时时间（秒）
    pub instruction_timeout_seconds: u64,
    /// 是否广播状态更新
    pub broadcast_status_updates: bool,
}

impl Default for AgentHookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_interruption: true,
            allow_modification: true,
            max_queue_size: 100,
            instruction_timeout_seconds: 30,
            broadcast_status_updates: true,
        }
    }
}

/// Agent Hook 管理器
pub struct AgentHookManager {
    /// Agent ID
    agent_id: String,
    /// 配置
    config: AgentHookConfig,
    /// 指令队列（按优先级排序）
    instruction_queue: Arc<RwLock<Vec<UserInstruction>>>,
    /// 执行状态
    execution_status: Arc<RwLock<AgentExecutionStatus>>,
    /// 状态快照历史
    state_history: Arc<RwLock<Vec<AgentStateSnapshot>>>,
    /// 指令广播通道
    instruction_tx: broadcast::Sender<UserInstruction>,
    /// 状态广播通道
    status_tx: broadcast::Sender<AgentStateSnapshot>,
    /// 会话 ID
    session_id: String,
}

impl AgentHookManager {
    /// 创建新的 Hook 管理器
    pub fn new(agent_id: String, session_id: String, config: AgentHookConfig) -> Self {
        let (instruction_tx, _) = broadcast::channel(100);
        let (status_tx, _) = broadcast::channel(100);

        Self {
            agent_id,
            session_id,
            config,
            instruction_queue: Arc::new(RwLock::new(Vec::new())),
            execution_status: Arc::new(RwLock::new(AgentExecutionStatus::Idle)),
            state_history: Arc::new(RwLock::new(Vec::new())),
            instruction_tx,
            status_tx,
        }
    }

    /// 获取 Agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// 获取会话 ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// 提交用户指令
    pub async fn submit_instruction(&self, mut instruction: UserInstruction) -> Result<String, String> {
        if !self.config.enabled {
            return Err("Hook system is disabled".to_string());
        }

        // 确保指令的会话 ID 正确
        instruction.session_id = self.session_id.clone();

        // 检查队列大小
        let queue = self.instruction_queue.read().await;
        if queue.len() >= self.config.max_queue_size {
            return Err("Instruction queue is full".to_string());
        }
        drop(queue);

        // 根据优先级插入队列
        let mut queue = self.instruction_queue.write().await;
        
        // Critical 优先级的指令需要特殊处理
        if instruction.priority == InstructionPriority::Critical {
            // 如果是 Critical 优先级，插入到队列最前面
            queue.insert(0, instruction.clone());
        } else {
            // 其他优先级按优先级顺序插入
            let insert_pos = queue.iter().position(|i| {
                match (instruction.priority, i.priority) {
                    (InstructionPriority::High, InstructionPriority::Low) => true,
                    (InstructionPriority::High, InstructionPriority::Medium) => true,
                    (InstructionPriority::Medium, InstructionPriority::Low) => true,
                    _ => false,
                }
            }).unwrap_or(queue.len());
            queue.insert(insert_pos, instruction.clone());
        }

        // 广播指令
        let _ = self.instruction_tx.send(instruction.clone());

        // 保存状态快照
        self.save_state_snapshot().await;

        // 如果是 Critical 优先级，尝试中断当前执行
        if instruction.priority == InstructionPriority::Critical && self.config.allow_interruption {
            self.handle_critical_instruction(&instruction).await?;
        }

        Ok(instruction.id)
    }

    /// 获取下一条待处理的指令
    pub async fn fetch_next_instruction(&self) -> Option<UserInstruction> {
        let mut queue = self.instruction_queue.write().await;
        
        // 查找第一条未处理的指令
        if let Some(pos) = queue.iter().position(|i| !i.processed) {
            let instruction = queue.remove(pos);
            return Some(instruction);
        }

        None
    }

    /// 标记指令为已处理
    pub async fn mark_instruction_processed(&self, instruction_id: &str, result: Option<String>) {
        let mut queue = self.instruction_queue.write().await;
        
        if let Some(instruction) = queue.iter_mut().find(|i| i.id == instruction_id) {
            instruction.processed = true;
            instruction.result = result;
        }
    }

    /// 获取所有待处理的指令
    pub async fn get_pending_instructions(&self) -> Vec<UserInstruction> {
        let queue = self.instruction_queue.read().await;
        queue.iter()
            .filter(|i| !i.processed)
            .cloned()
            .collect()
    }

    /// 获取指令历史
    pub async fn get_instruction_history(&self, limit: usize) -> Vec<UserInstruction> {
        let queue = self.instruction_queue.read().await;
        let mut history: Vec<_> = queue.iter()
            .filter(|i| i.processed)
            .cloned()
            .collect();
        
        // 按时间倒序排列
        history.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        history.truncate(limit);
        history
    }

    /// 清除指令队列
    pub async fn clear_queue(&self) {
        let mut queue = self.instruction_queue.write().await;
        queue.clear();
    }

    /// 设置执行状态
    pub async fn set_execution_status(&self, status: AgentExecutionStatus) {
        let mut current_status = self.execution_status.write().await;
        *current_status = status;

        // 广播状态更新
        if self.config.broadcast_status_updates {
            let snapshot = self.create_snapshot().await;
            let _ = self.status_tx.send(snapshot);
        }
    }

    /// 获取执行状态
    pub async fn get_execution_status(&self) -> AgentExecutionStatus {
        self.execution_status.read().await.clone()
    }

    /// 创建订阅者接收指令
    pub fn subscribe_instructions(&self) -> broadcast::Receiver<UserInstruction> {
        self.instruction_tx.subscribe()
    }

    /// 创建订阅者接收状态更新
    pub fn subscribe_status(&self) -> broadcast::Receiver<AgentStateSnapshot> {
        self.status_tx.subscribe()
    }

    /// 保存状态快照
    async fn save_state_snapshot(&self) {
        let snapshot = self.create_snapshot().await;
        let mut history = self.state_history.write().await;
        history.push(snapshot);
        
        // 限制历史记录大小
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// 创建状态快照
    async fn create_snapshot(&self) -> AgentStateSnapshot {
        let status = self.execution_status.read().await;
        let queue = self.instruction_queue.read().await;

        let (current_task, progress) = match &*status {
            AgentExecutionStatus::Executing { current_task, progress, .. } => {
                (Some(current_task.clone()), *progress)
            }
            _ => (None, 0.0),
        };

        let processed_count = queue.iter().filter(|i| i.processed).count();
        let pending_count = queue.iter().filter(|i| !i.processed).count();

        AgentStateSnapshot {
            agent_id: self.agent_id.clone(),
            session_id: self.session_id.clone(),
            execution_status: status.clone(),
            current_task,
            progress,
            processed_instructions: processed_count,
            pending_instructions: pending_count,
            snapshot_time: chrono::Utc::now().timestamp(),
        }
    }

    /// 获取状态快照
    pub async fn get_snapshot(&self) -> AgentStateSnapshot {
        self.create_snapshot().await
    }

    /// 获取状态历史
    pub async fn get_state_history(&self, limit: usize) -> Vec<AgentStateSnapshot> {
        let history = self.state_history.read().await;
        let mut result = history.clone();
        result.sort_by(|a, b| b.snapshot_time.cmp(&a.snapshot_time));
        result.truncate(limit);
        result
    }

    /// 处理 Critical 指令
    async fn handle_critical_instruction(&self, instruction: &UserInstruction) -> Result<(), String> {
        match instruction.instruction_type {
            InstructionType::Cancel => {
                let status = self.execution_status.read().await;
                if let AgentExecutionStatus::Executing { .. } = &*status {
                    drop(status);
                    self.set_execution_status(AgentExecutionStatus::Cancelled {
                        reason: instruction.content.clone(),
                        cancelled_at: chrono::Utc::now().timestamp(),
                    }).await;
                }
            }
            InstructionType::Pause => {
                let status = self.execution_status.read().await;
                if let AgentExecutionStatus::Executing { .. } = &*status {
                    drop(status);
                    self.set_execution_status(AgentExecutionStatus::Paused {
                        paused_at: chrono::Utc::now().timestamp(),
                        reason: instruction.content.clone(),
                    }).await;
                }
            }
            InstructionType::Reset => {
                self.set_execution_status(AgentExecutionStatus::Idle).await;
                self.clear_queue().await;
            }
            _ => {}
        }
        Ok(())
    }

    /// 检查是否有 Critical 指令
    pub async fn has_critical_instruction(&self) -> bool {
        let queue = self.instruction_queue.read().await;
        queue.iter()
            .any(|i| i.priority == InstructionPriority::Critical && !i.processed)
    }

    /// 等待指令（带超时）
    pub async fn wait_for_instruction(&self, timeout_secs: u64) -> Option<UserInstruction> {
        let mut receiver = self.subscribe_instructions();
        tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            receiver.recv()
        ).await
        .ok()
        .and_then(|r| r.ok())
    }

    /// 注入简单指令（快捷方法）
    pub async fn inject(&self, content: String) -> Result<String, String> {
        let instruction = UserInstruction::new(
            self.session_id.clone(),
            content,
            InstructionType::Normal,
        );
        self.submit_instruction(instruction).await
    }

    /// 注入高优先级指令
    pub async fn inject_high_priority(&self, content: String) -> Result<String, String> {
        let instruction = UserInstruction::new(
            self.session_id.clone(),
            content,
            InstructionType::Normal,
        ).with_priority(InstructionPriority::High);
        self.submit_instruction(instruction).await
    }

    /// 暂停执行
    pub async fn pause(&self, reason: String) -> Result<String, String> {
        let instruction = UserInstruction::new(
            self.session_id.clone(),
            reason,
            InstructionType::Pause,
        ).with_priority(InstructionPriority::Critical);
        self.submit_instruction(instruction).await
    }

    /// 恢复执行
    pub async fn resume(&self) -> Result<String, String> {
        let instruction = UserInstruction::new(
            self.session_id.clone(),
            "Resume execution".to_string(),
            InstructionType::Resume,
        ).with_priority(InstructionPriority::High);
        self.submit_instruction(instruction).await
    }

    /// 取消执行
    pub async fn cancel(&self, reason: String) -> Result<String, String> {
        let instruction = UserInstruction::new(
            self.session_id.clone(),
            reason,
            InstructionType::Cancel,
        ).with_priority(InstructionPriority::Critical);
        self.submit_instruction(instruction).await
    }
}

/// 创建 Hook 管理器的工厂
pub struct AgentHookFactory;

impl AgentHookFactory {
    /// 创建新的 Hook 管理器
    pub fn create(agent_id: String, session_id: String) -> Arc<AgentHookManager> {
        Arc::new(AgentHookManager::new(
            agent_id,
            session_id,
            AgentHookConfig::default(),
        ))
    }

    /// 使用自定义配置创建
    pub fn create_with_config(
        agent_id: String,
        session_id: String,
        config: AgentHookConfig,
    ) -> Arc<AgentHookManager> {
        Arc::new(AgentHookManager::new(agent_id, session_id, config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_and_fetch_instruction() {
        let hook = AgentHookManager::new(
            "test_agent".to_string(),
            "test_session".to_string(),
            AgentHookConfig::default(),
        );

        // 提交指令
        let instruction = UserInstruction::new(
            "test_session".to_string(),
            "Please analyze the data more carefully".to_string(),
            InstructionType::Normal,
        );

        let result = hook.submit_instruction(instruction).await;
        assert!(result.is_ok());

        // 获取指令
        let fetched = hook.fetch_next_instruction().await;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().content, "Please analyze the data more carefully");
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let hook = AgentHookManager::new(
            "test_agent".to_string(),
            "test_session".to_string(),
            AgentHookConfig::default(),
        );

        // 提交低优先级指令
        let low_priority = UserInstruction::new(
            "test_session".to_string(),
            "Low priority task".to_string(),
            InstructionType::Normal,
        ).with_priority(InstructionPriority::Low);
        hook.submit_instruction(low_priority).await.unwrap();

        // 提交高优先级指令
        let high_priority = UserInstruction::new(
            "test_session".to_string(),
            "High priority task".to_string(),
            InstructionType::Normal,
        ).with_priority(InstructionPriority::High);
        hook.submit_instruction(high_priority).await.unwrap();

        // 获取指令，应该先获取到高优先级的
        let fetched = hook.fetch_next_instruction().await.unwrap();
        assert_eq!(fetched.priority, InstructionPriority::High);
    }

    #[tokio::test]
    async fn test_critical_instruction() {
        let hook = AgentHookManager::new(
            "test_agent".to_string(),
            "test_session".to_string(),
            AgentHookConfig::default(),
        );

        // 设置为执行状态
        hook.set_execution_status(AgentExecutionStatus::Executing {
            current_task: "Processing data".to_string(),
            progress: 50.0,
            started_at: chrono::Utc::now().timestamp(),
        }).await;

        // 提交取消指令
        let cancel_instruction = UserInstruction::new(
            "test_session".to_string(),
            "User requested cancellation".to_string(),
            InstructionType::Cancel,
        ).with_priority(InstructionPriority::Critical);

        hook.submit_instruction(cancel_instruction).await.unwrap();

        // 检查状态是否变为已取消
        let status = hook.get_execution_status().await;
        assert!(matches!(status, AgentExecutionStatus::Cancelled { .. }));
    }
}
