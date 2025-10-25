use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Performance metrics collector for monitoring system health
/// 
/// This module provides lightweight metrics collection for the edge worker.
/// Metrics are stored in memory and reset on worker restart.
#[derive(Clone)]
pub struct MetricsCollector {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    // Request counters
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    
    // Endpoint-specific counters
    health_checks: AtomicU64,
    session_operations: AtomicU64,
    wallet_auth_operations: AtomicU64,
    agent_chat_operations: AtomicU64,
    
    // Performance metrics (in microseconds)
    total_response_time_us: AtomicU64,
    min_response_time_us: AtomicU64,
    max_response_time_us: AtomicU64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    
    pub health_checks: u64,
    pub session_operations: u64,
    pub wallet_auth_operations: u64,
    pub agent_chat_operations: u64,
    
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                total_requests: AtomicU64::new(0),
                successful_requests: AtomicU64::new(0),
                failed_requests: AtomicU64::new(0),
                health_checks: AtomicU64::new(0),
                session_operations: AtomicU64::new(0),
                wallet_auth_operations: AtomicU64::new(0),
                agent_chat_operations: AtomicU64::new(0),
                total_response_time_us: AtomicU64::new(0),
                min_response_time_us: AtomicU64::new(u64::MAX),
                max_response_time_us: AtomicU64::new(0),
            }),
        }
    }
    
    /// Record a request
    pub fn record_request(&self, path: &str, success: bool, duration_us: u64) {
        // Increment total requests
        self.inner.total_requests.fetch_add(1, Ordering::Relaxed);
        
        // Increment success/failure counter
        if success {
            self.inner.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.inner.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        // Increment endpoint-specific counter
        if path.starts_with("/api/health") || path.starts_with("/api/status") {
            self.inner.health_checks.fetch_add(1, Ordering::Relaxed);
        } else if path.starts_with("/api/session") {
            self.inner.session_operations.fetch_add(1, Ordering::Relaxed);
        } else if path.starts_with("/api/wallet") {
            self.inner.wallet_auth_operations.fetch_add(1, Ordering::Relaxed);
        } else if path.starts_with("/api/agent") {
            self.inner.agent_chat_operations.fetch_add(1, Ordering::Relaxed);
        }
        
        // Update response time metrics
        self.inner.total_response_time_us.fetch_add(duration_us, Ordering::Relaxed);
        
        // Update min response time
        let mut current_min = self.inner.min_response_time_us.load(Ordering::Relaxed);
        while duration_us < current_min {
            match self.inner.min_response_time_us.compare_exchange_weak(
                current_min,
                duration_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }
        
        // Update max response time
        let mut current_max = self.inner.max_response_time_us.load(Ordering::Relaxed);
        while duration_us > current_max {
            match self.inner.max_response_time_us.compare_exchange_weak(
                current_max,
                duration_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }
    
    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let total = self.inner.total_requests.load(Ordering::Relaxed);
        let successful = self.inner.successful_requests.load(Ordering::Relaxed);
        let failed = self.inner.failed_requests.load(Ordering::Relaxed);
        
        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        let total_time_us = self.inner.total_response_time_us.load(Ordering::Relaxed);
        let avg_time_ms = if total > 0 {
            (total_time_us as f64 / total as f64) / 1000.0
        } else {
            0.0
        };
        
        let min_time_us = self.inner.min_response_time_us.load(Ordering::Relaxed);
        let min_time_ms = if min_time_us == u64::MAX {
            0.0
        } else {
            min_time_us as f64 / 1000.0
        };
        
        let max_time_us = self.inner.max_response_time_us.load(Ordering::Relaxed);
        let max_time_ms = max_time_us as f64 / 1000.0;
        
        MetricsSnapshot {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            success_rate,
            health_checks: self.inner.health_checks.load(Ordering::Relaxed),
            session_operations: self.inner.session_operations.load(Ordering::Relaxed),
            wallet_auth_operations: self.inner.wallet_auth_operations.load(Ordering::Relaxed),
            agent_chat_operations: self.inner.agent_chat_operations.load(Ordering::Relaxed),
            avg_response_time_ms: avg_time_ms,
            min_response_time_ms: min_time_ms,
            max_response_time_ms: max_time_ms,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
