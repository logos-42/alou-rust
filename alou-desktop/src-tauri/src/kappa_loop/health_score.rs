//! 健康评分 — 量化系统状态
//!
//! 0-100 分量化系统健康度，低于阈值触发主动自修复。
//! 评分基于：成功/失败比例、最近趋势、改进频率

use serde::{Deserialize, Serialize};

/// 健康评分追踪器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    /// 当前健康分（0-100）
    score: f32,
    /// 最近N次结果
    recent_results: Vec<HealthEvent>,
    /// 最大历史记录数
    max_history: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HealthEvent {
    Improvement,
    Neutral,
    Failure,
}

impl HealthScore {
    pub fn new() -> Self {
        Self {
            score: 100.0,
            recent_results: Vec::new(),
            max_history: 20,
        }
    }

    /// 获取当前健康分
    pub fn score(&self) -> f32 {
        self.score
    }

    /// 记录一次改进
    pub fn record_improvement(&mut self) {
        self.recent_results.push(HealthEvent::Improvement);
        self.recalc();
    }

    /// 记录一次中性结果
    pub fn record_neutral(&mut self) {
        self.recent_results.push(HealthEvent::Neutral);
        self.recalc();
    }

    /// 记录一次失败
    pub fn record_failure(&mut self) {
        self.recent_results.push(HealthEvent::Failure);
        self.recalc();
    }

    /// 重新计算健康分
    fn recalc(&mut self) {
        // 裁剪历史
        if self.recent_results.len() > self.max_history {
            self.recent_results.drain(0..self.recent_results.len() - self.max_history);
        }

        if self.recent_results.is_empty() {
            self.score = 100.0;
            return;
        }

        // 加权计算：最近的事件权重更高
        let mut total_weight = 0.0;
        let mut weighted_score = 0.0;

        for (i, event) in self.recent_results.iter().enumerate() {
            let recency = (i + 1) as f32 / self.recent_results.len() as f32;
            let weight = 0.5 + 0.5 * recency; // 0.5 ~ 1.0

            let event_score = match event {
                HealthEvent::Improvement => 100.0,
                HealthEvent::Neutral => 70.0,
                HealthEvent::Failure => 20.0,
            };

            weighted_score += event_score * weight;
            total_weight += weight;
        }

        self.score = (weighted_score / total_weight).clamp(0.0, 100.0);
    }

    /// 是否低于阈值
    pub fn is_below_threshold(&self, threshold: f32) -> bool {
        self.score < threshold
    }

    /// 获取最近趋势
    pub fn trend(&self) -> HealthTrend {
        if self.recent_results.len() < 3 {
            return HealthTrend::Stable;
        }

        let recent: Vec<_> = self.recent_results.iter().rev().take(5).collect();
        let improvements = recent.iter().filter(|&&e| matches!(e, HealthEvent::Improvement)).count();
        let failures = recent.iter().filter(|&&e| matches!(e, HealthEvent::Failure)).count();

        if improvements > failures {
            HealthTrend::Improving
        } else if failures > improvements {
            HealthTrend::Declining
        } else {
            HealthTrend::Stable
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthTrend {
    Improving,
    Stable,
    Declining,
}

impl Default for HealthScore {
    fn default() -> Self {
        Self::new()
    }
}
