//! 熔断器 — 防止连续失败耗尽资源
//!
//! 当连续失败次数超过阈值时，自动打开熔断器，
//! 阻止后续请求直到冷却期结束。

use serde::{Deserialize, Serialize};

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// 关闭（正常工作）
    Closed,
    /// 打开（阻止请求）
    Open,
    /// 半开（尝试恢复）
    HalfOpen,
}

/// 熔断器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// 最大连续失败次数
    max_failures: u32,
    /// 当前连续失败次数
    consecutive_failures: u32,
    /// 当前状态
    state: CircuitState,
    /// 总成功次数
    total_successes: u64,
    /// 总失败次数
    total_failures: u64,
}

impl CircuitBreaker {
    pub fn new(max_failures: u32) -> Self {
        Self {
            max_failures: max_failures.max(1),
            consecutive_failures: 0,
            state: CircuitState::Closed,
            total_successes: 0,
            total_failures: 0,
        }
    }

    /// 熔断器是否打开
    pub fn is_open(&self) -> bool {
        self.state == CircuitState::Open
    }

    /// 记录成功
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.total_successes += 1;
        self.state = CircuitState::Closed;
    }

    /// 记录失败
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.total_failures += 1;

        if self.consecutive_failures >= self.max_failures {
            self.state = CircuitState::Open;
        }
    }

    /// 尝试半开（允许一次试探请求）
    pub fn half_open(&mut self) {
        if self.state == CircuitState::Open {
            self.state = CircuitState::HalfOpen;
        }
    }

    /// 重置熔断器
    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.state = CircuitState::Closed;
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f32 {
        let total = self.total_successes + self.total_failures;
        if total == 0 {
            return 1.0;
        }
        self.total_successes as f32 / total as f32
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }
}
