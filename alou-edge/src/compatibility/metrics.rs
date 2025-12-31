//! 兼容性监控指标 - 用于跟踪Durable Objects的性能和状态

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 兼容性监控指标
#[derive(Clone)]
pub struct CompatibilityMetrics {
    inner: Arc<CompatibilityMetricsInner>,
}

struct CompatibilityMetricsInner {
    // 兼容性API调用
    legacy_api_calls: AtomicU64,
    legacy_api_errors: AtomicU64,
    
    // 任务成功率
    tasks_created: AtomicU64,
    tasks_completed: AtomicU64,
    tasks_failed: AtomicU64,
    
    // 响应时间（微秒）
    sync_response_times: AtomicU64,
    sync_response_count: AtomicU64,
    async_response_times: AtomicU64,
    async_response_count: AtomicU64,
    
    // 工具调用统计
    tool_calls: AtomicU64,
    tool_errors: AtomicU64,
    tool_local_executions: AtomicU64,
}

/// 监控指标快照
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompatibilityMetricsSnapshot {
    // 兼容性API统计
    pub legacy_api_calls: u64,
    pub legacy_api_errors: u64,
    pub legacy_api_error_rate: f64,
    
    // 任务统计
    pub tasks_created: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub task_success_rate: f64,
    
    // 响应时间统计（毫秒）
    pub avg_sync_response_time_ms: f64,
    pub avg_async_response_time_ms: f64,
    
    // 工具调用统计
    pub tool_calls: u64,
    pub tool_errors: u64,
    pub tool_local_executions: u64,
    pub tool_error_rate: f64,
    pub tool_local_execution_rate: f64,
}

impl CompatibilityMetrics {
    /// 创建新的监控指标收集器
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CompatibilityMetricsInner {
                legacy_api_calls: AtomicU64::new(0),
                legacy_api_errors: AtomicU64::new(0),
                tasks_created: AtomicU64::new(0),
                tasks_completed: AtomicU64::new(0),
                tasks_failed: AtomicU64::new(0),
                sync_response_times: AtomicU64::new(0),
                sync_response_count: AtomicU64::new(0),
                async_response_times: AtomicU64::new(0),
                async_response_count: AtomicU64::new(0),
                tool_calls: AtomicU64::new(0),
                tool_errors: AtomicU64::new(0),
                tool_local_executions: AtomicU64::new(0),
            }),
        }
    }
    
    /// 记录兼容性API调用
    pub fn record_legacy_api_call(&self, success: bool) {
        self.inner.legacy_api_calls.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.inner.legacy_api_errors.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 记录任务创建
    pub fn record_task_created(&self) {
        self.inner.tasks_created.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录任务完成
    pub fn record_task_completed(&self, success: bool) {
        if success {
            self.inner.tasks_completed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.inner.tasks_failed.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 记录同步响应时间
    pub fn record_sync_response_time(&self, duration_us: u64) {
        self.inner.sync_response_times.fetch_add(duration_us, Ordering::Relaxed);
        self.inner.sync_response_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录异步响应时间
    pub fn record_async_response_time(&self, duration_us: u64) {
        self.inner.async_response_times.fetch_add(duration_us, Ordering::Relaxed);
        self.inner.async_response_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录工具调用
    pub fn record_tool_call(&self, success: bool, requires_local_execution: bool) {
        self.inner.tool_calls.fetch_add(1, Ordering::Relaxed);
        
        if !success {
            self.inner.tool_errors.fetch_add(1, Ordering::Relaxed);
        }
        
        if requires_local_execution {
            self.inner.tool_local_executions.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 获取监控指标快照
    pub fn snapshot(&self) -> CompatibilityMetricsSnapshot {
        let legacy_api_calls = self.inner.legacy_api_calls.load(Ordering::Relaxed);
        let legacy_api_errors = self.inner.legacy_api_errors.load(Ordering::Relaxed);
        let legacy_api_error_rate = if legacy_api_calls > 0 {
            (legacy_api_errors as f64 / legacy_api_calls as f64) * 100.0
        } else {
            0.0
        };
        
        let tasks_created = self.inner.tasks_created.load(Ordering::Relaxed);
        let tasks_completed = self.inner.tasks_completed.load(Ordering::Relaxed);
        let tasks_failed = self.inner.tasks_failed.load(Ordering::Relaxed);
        let task_success_rate = if tasks_created > 0 {
            (tasks_completed as f64 / tasks_created as f64) * 100.0
        } else {
            0.0
        };
        
        let sync_response_times = self.inner.sync_response_times.load(Ordering::Relaxed);
        let sync_response_count = self.inner.sync_response_count.load(Ordering::Relaxed);
        let avg_sync_response_time_ms = if sync_response_count > 0 {
            (sync_response_times as f64 / sync_response_count as f64) / 1000.0
        } else {
            0.0
        };
        
        let async_response_times = self.inner.async_response_times.load(Ordering::Relaxed);
        let async_response_count = self.inner.async_response_count.load(Ordering::Relaxed);
        let avg_async_response_time_ms = if async_response_count > 0 {
            (async_response_times as f64 / async_response_count as f64) / 1000.0
        } else {
            0.0
        };
        
        let tool_calls = self.inner.tool_calls.load(Ordering::Relaxed);
        let tool_errors = self.inner.tool_errors.load(Ordering::Relaxed);
        let tool_local_executions = self.inner.tool_local_executions.load(Ordering::Relaxed);
        let tool_error_rate = if tool_calls > 0 {
            (tool_errors as f64 / tool_calls as f64) * 100.0
        } else {
            0.0
        };
        let tool_local_execution_rate = if tool_calls > 0 {
            (tool_local_executions as f64 / tool_calls as f64) * 100.0
        } else {
            0.0
        };
        
        CompatibilityMetricsSnapshot {
            legacy_api_calls,
            legacy_api_errors,
            legacy_api_error_rate,
            tasks_created,
            tasks_completed,
            tasks_failed,
            task_success_rate,
            avg_sync_response_time_ms,
            avg_async_response_time_ms,
            tool_calls,
            tool_errors,
            tool_local_executions,
            tool_error_rate,
            tool_local_execution_rate,
        }
    }
    
    /// 将监控指标输出到日志
    pub async fn report_to_logs(&self) {
        let snapshot = self.snapshot();
        
        worker::console_log!(
            "COMPATIBILITY_METRICS: legacy_api_calls={}, legacy_api_errors={}, legacy_api_error_rate={:.2}%",
            snapshot.legacy_api_calls,
            snapshot.legacy_api_errors,
            snapshot.legacy_api_error_rate
        );
        
        worker::console_log!(
            "COMPATIBILITY_METRICS: tasks_created={}, tasks_completed={}, tasks_failed={}, task_success_rate={:.2}%",
            snapshot.tasks_created,
            snapshot.tasks_completed,
            snapshot.tasks_failed,
            snapshot.task_success_rate
        );
        
        worker::console_log!(
            "COMPATIBILITY_METRICS: avg_sync_response_time={:.2}ms, avg_async_response_time={:.2}ms",
            snapshot.avg_sync_response_time_ms,
            snapshot.avg_async_response_time_ms
        );
        
        worker::console_log!(
            "COMPATIBILITY_METRICS: tool_calls={}, tool_errors={}, tool_local_executions={}, tool_error_rate={:.2}%, tool_local_execution_rate={:.2}%",
            snapshot.tool_calls,
            snapshot.tool_errors,
            snapshot.tool_local_executions,
            snapshot.tool_error_rate,
            snapshot.tool_local_execution_rate
        );
    }
}

impl Default for CompatibilityMetrics {
    fn default() -> Self {
        Self::new()
    }
}