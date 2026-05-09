//! 资源配额管理

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// 资源配额
#[derive(Debug, Clone)]
pub struct ResourceQuota {
    /// 最大并发 LLM 调用
    pub max_llm_concurrent: usize,
    
    /// 最大并发工具调用
    pub max_tool_concurrent: usize,
    
    /// 最大并发工作流
    pub max_workflow_concurrent: usize,
    
    /// 每 session 最大任务数
    pub max_tasks_per_session: usize,
    
    /// 每 session 每分钟最大请求数
    pub max_requests_per_minute: usize,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_llm_concurrent: 10,
            max_tool_concurrent: 20,
            max_workflow_concurrent: 5,
            max_tasks_per_session: 10,
            max_requests_per_minute: 60,
        }
    }
}

/// 配额管理器
///
/// 管理所有资源的配额和限流
pub struct QuotaManager {
    /// 全局配额
    global_quota: ResourceQuota,
    
    /// LLM 信号量
    llm_semaphore: Arc<Semaphore>,
    
    /// 工具信号量
    tool_semaphore: Arc<Semaphore>,
    
    /// 工作流信号量
    workflow_semaphore: Arc<Semaphore>,
    
    /// Session 任务计数
    session_task_count: std::sync::Mutex<HashMap<String, usize>>,
    
    /// Session 请求速率限制（session_id -> (count, window_start)）
    session_rate_limit: std::sync::Mutex<HashMap<String, (usize, i64)>>,
}

impl QuotaManager {
    pub fn new(quota: ResourceQuota) -> Self {
        Self {
            global_quota: quota.clone(),
            llm_semaphore: Arc::new(Semaphore::new(quota.max_llm_concurrent)),
            tool_semaphore: Arc::new(Semaphore::new(quota.max_tool_concurrent)),
            workflow_semaphore: Arc::new(Semaphore::new(quota.max_workflow_concurrent)),
            session_task_count: std::sync::Mutex::new(HashMap::new()),
            session_rate_limit: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    /// 获取 LLM 许可
    pub async fn acquire_llm(&self) -> Result<ResourcePermit, String> {
        let permit = self.llm_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Failed to acquire LLM permit: {}", e))?;
        
        Ok(ResourcePermit::Llm(permit))
    }
    
    /// 获取工具许可
    pub async fn acquire_tool(&self) -> Result<ResourcePermit, String> {
        let permit = self.tool_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Failed to acquire tool permit: {}", e))?;
        
        Ok(ResourcePermit::Tool(permit))
    }
    
    /// 获取工作流许可
    pub async fn acquire_workflow(&self) -> Result<ResourcePermit, String> {
        let permit = self.workflow_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Failed to acquire workflow permit: {}", e))?;
        
        Ok(ResourcePermit::Workflow(permit))
    }
    
    /// 检查 session 任务配额
    pub fn check_session_quota(&self, session_id: &str) -> bool {
        let count = self.session_task_count
            .lock()
            .unwrap()
            .get(session_id)
            .copied()
            .unwrap_or(0);
        
        count < self.global_quota.max_tasks_per_session
    }
    
    /// 增加 session 任务计数
    pub fn increment_session_task(&self, session_id: &str) {
        *self.session_task_count
            .lock()
            .unwrap()
            .entry(session_id.to_string())
            .or_insert(0) += 1;
    }
    
    /// 减少 session 任务计数
    pub fn decrement_session_task(&self, session_id: &str) {
        let mut count = self.session_task_count.lock().unwrap();
        if let Some(c) = count.get_mut(session_id) {
            if *c > 0 {
                *c -= 1;
            }
        }
    }
    
    /// 检查速率限制
    pub fn check_rate_limit(&self, session_id: &str) -> bool {
        let now = chrono::Utc::now().timestamp();
        let mut rate_limit = self.session_rate_limit.lock().unwrap();
        
        let (count, window_start) = rate_limit
            .entry(session_id.to_string())
            .or_insert((0, now));
        
        // 重置窗口（每分钟）
        if now - *window_start >= 60 {
            *count = 0;
            *window_start = now;
        }
        
        *count < self.global_quota.max_requests_per_minute
    }
    
    /// 记录请求
    pub fn record_request(&self, session_id: &str) {
        let now = chrono::Utc::now().timestamp();
        let mut rate_limit = self.session_rate_limit.lock().unwrap();
        
        let (count, window_start) = rate_limit
            .entry(session_id.to_string())
            .or_insert((0, now));
        
        // 重置窗口（每分钟）
        if now - *window_start >= 60 {
            *count = 0;
            *window_start = now;
        }
        
        *count += 1;
    }
    
    /// 获取配额统计
    pub fn get_stats(&self) -> QuotaStats {
        QuotaStats {
            llm_available: self.llm_semaphore.available_permits(),
            tool_available: self.tool_semaphore.available_permits(),
            workflow_available: self.workflow_semaphore.available_permits(),
            active_sessions: self.session_task_count.lock().unwrap().len(),
        }
    }
}

/// 资源许可（RAII）
pub enum ResourcePermit {
    Llm(tokio::sync::OwnedSemaphorePermit),
    Tool(tokio::sync::OwnedSemaphorePermit),
    Workflow(tokio::sync::OwnedSemaphorePermit),
}

/// 配额统计
#[derive(Debug, Clone)]
pub struct QuotaStats {
    pub llm_available: usize,
    pub tool_available: usize,
    pub workflow_available: usize,
    pub active_sessions: usize,
}
