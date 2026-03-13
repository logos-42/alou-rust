//! 任务队列管理

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use crate::scheduler::types::{AgentTask, AgentPriority};

/// 优先级队列项
struct QueueItem {
    task: AgentTask,
    score: i64,  // 优先级分数（越高越优先）
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for QueueItem {}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // 分数高的优先（BinaryHeap 是最大堆）
        self.score.cmp(&other.score)
    }
}

/// 任务队列
///
/// 支持：
/// - 优先级调度
/// - 截止时间感知
/// - 公平性保证
pub struct TaskQueue {
    /// 优先级队列
    heap: BinaryHeap<QueueItem>,
    
    /// Session 任务计数（用于公平性）
    session_task_count: std::collections::HashMap<String, usize>,
    
    /// 最大队列大小
    max_size: usize,
}

impl TaskQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            heap: BinaryHeap::new(),
            session_task_count: std::collections::HashMap::new(),
            max_size,
        }
    }
    
    /// 计算任务优先级分数
    fn calculate_score(task: &AgentTask) -> i64 {
        let mut score = match task.priority {
            AgentPriority::Background => 0,
            AgentPriority::Normal => 100,
            AgentPriority::Interactive => 200,
            AgentPriority::Critical => 300,
        };
        
        // 截止时间越近，分数越高
        if let Some(deadline) = task.deadline {
            let now = chrono::Utc::now().timestamp();
            let time_left = deadline - now;
            if time_left > 0 {
                score += (1000 - time_left.min(1000)) as i64;
            } else {
                // 已过期，最高优先级
                score += 10000;
            }
        }
        
        // 等待时间越长，分数越高（防止饥饿）
        let wait_time = chrono::Utc::now().timestamp() - task.created_at;
        score += wait_time as i64;
        
        // 重试次数越多，分数越高（避免任务被永久丢弃）
        score += (task.retry_count as i64) * 50;
        
        score
    }
    
    /// 入队任务
    pub fn enqueue(&mut self, task: AgentTask) -> bool {
        // 检查队列大小
        if self.heap.len() >= self.max_size {
            log::warn!(
                "[TaskQueue] Queue is full (max={}), rejecting task {}",
                self.max_size,
                task.task_id
            );
            return false;
        }
        
        // 公平性检查：防止单个 session 独占
        let session_count = self.session_task_count.get(&task.session_id).copied().unwrap_or(0);
        if session_count >= 10 {
            log::warn!(
                "[TaskQueue] Session {} has too many pending tasks ({}), rejecting",
                task.session_id,
                session_count
            );
            return false;
        }
        
        let score = Self::calculate_score(&task);
        
        // 更新 session 计数
        *self.session_task_count.entry(task.session_id.clone()).or_insert(0) += 1;
        
        self.heap.push(QueueItem { task, score });
        
        log::info!(
            "[TaskQueue] Enqueued task {} with score {}",
            self.heap.peek().map(|i| i.task.task_id.as_str()).unwrap_or("unknown"),
            score
        );
        
        true
    }
    
    /// 出队任务（获取最高优先级的任务）
    pub fn dequeue(&mut self) -> Option<AgentTask> {
        if let Some(item) = self.heap.pop() {
            // 更新 session 计数
            if let Some(count) = self.session_task_count.get_mut(&item.task.session_id) {
                if *count > 0 {
                    *count -= 1;
                }
            }
            
            log::info!(
                "[TaskQueue] Dequeued task {} (priority={:?})",
                item.task.task_id,
                item.task.priority
            );
            
            Some(item.task)
        } else {
            None
        }
    }
    
    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.heap.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
    
    /// 获取下一个任务的预览（不弹出）
    pub fn peek(&self) -> Option<&AgentTask> {
        self.heap.peek().map(|i| &i.task)
    }
    
    /// 移除指定 session 的所有任务
    pub fn remove_session_tasks(&mut self, session_id: &str) -> Vec<AgentTask> {
        let mut removed = Vec::new();
        
        // 重建堆，过滤掉指定 session 的任务
        let old_heap = std::mem::take(&mut self.heap);
        for item in old_heap {
            if item.task.session_id == session_id {
                removed.push(item.task);
            } else {
                self.heap.push(item);
            }
        }
        
        // 更新计数
        self.session_task_count.remove(session_id);
        
        log::info!(
            "[TaskQueue] Removed {} tasks for session {}",
            removed.len(),
            session_id
        );
        
        removed
    }
    
    /// 获取队列统计
    pub fn get_stats(&self) -> QueueStats {
        let mut stats = QueueStats {
            total_tasks: self.heap.len(),
            by_priority: [0; 4],
            unique_sessions: self.session_task_count.len(),
            max_session_tasks: self.session_task_count.values().copied().max().unwrap_or(0),
        };
        
        for item in &self.heap {
            let idx = match item.task.priority {
                AgentPriority::Background => 0,
                AgentPriority::Normal => 1,
                AgentPriority::Interactive => 2,
                AgentPriority::Critical => 3,
            };
            stats.by_priority[idx] += 1;
        }
        
        stats
    }
    
    /// 清理过期任务
    pub fn cleanup_expired(&mut self) -> Vec<AgentTask> {
        let mut expired = Vec::new();
        
        let old_heap = std::mem::take(&mut self.heap);
        for item in old_heap {
            if item.task.is_expired() {
                expired.push(item.task);
            } else {
                self.heap.push(item);
            }
        }
        
        log::info!(
            "[TaskQueue] Cleaned up {} expired tasks",
            expired.len()
        );
        
        expired
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// 队列统计
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total_tasks: usize,
    pub by_priority: [usize; 4],
    pub unique_sessions: usize,
    pub max_session_tasks: usize,
}
