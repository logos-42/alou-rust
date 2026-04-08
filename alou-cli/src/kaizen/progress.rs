//! Kaizen 进度追踪和监控

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::time::Instant;

use super::get_kaizen_log_dir;

/// Kaizen 循环状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaizenProgress {
    /// 是否正在运行
    pub is_running: bool,

    /// 当前模式
    pub mode: String,

    /// 当前迭代次数
    pub current_iteration: usize,

    /// 最大迭代次数
    pub max_iterations: usize,

    /// 成功次数
    pub success_count: usize,

    /// 失败次数
    pub failure_count: usize,

    /// 中性结果次数
    pub neutral_count: usize,

    /// 开始时间
    pub start_time: Option<String>,

    /// 最后更新时间
    pub last_update: String,

    /// 当前阶段
    pub current_phase: String,

    /// 当前最佳分数
    pub best_score: f32,

    /// 实验日志路径
    pub log_path: Option<String>,
}

impl Default for KaizenProgress {
    fn default() -> Self {
        Self {
            is_running: false,
            mode: "evolution".to_string(),
            current_iteration: 0,
            max_iterations: 5,
            success_count: 0,
            failure_count: 0,
            neutral_count: 0,
            start_time: None,
            last_update: chrono::Utc::now().to_rfc3339(),
            current_phase: "idle".to_string(),
            best_score: 0.0,
            log_path: None,
        }
    }
}

impl KaizenProgress {
    /// 创建新的进度实例
    pub fn new(mode: &str, max_iterations: usize) -> Self {
        Self {
            is_running: false,
            mode: mode.to_string(),
            current_iteration: 0,
            max_iterations,
            success_count: 0,
            failure_count: 0,
            neutral_count: 0,
            start_time: None,
            last_update: chrono::Utc::now().to_rfc3339(),
            current_phase: "idle".to_string(),
            best_score: 0.0,
            log_path: None,
        }
    }

    /// 开始循环
    pub fn start(&mut self) {
        self.is_running = true;
        self.start_time = Some(chrono::Utc::now().to_rfc3339());
        self.last_update = chrono::Utc::now().to_rfc3339();
        self.current_phase = "initializing".to_string();
    }

    /// 更新迭代进度
    pub fn update_iteration(&mut self, iteration: usize, outcome: &str, score: f32) {
        self.current_iteration = iteration;
        self.last_update = chrono::Utc::now().to_rfc3339();

        match outcome {
            "improved" | "success" => {
                self.success_count += 1;
                if score > self.best_score {
                    self.best_score = score;
                }
            }
            "failed" | "error" => {
                self.failure_count += 1;
            }
            "neutral" => {
                self.neutral_count += 1;
            }
            _ => {}
        }

        self.current_phase = format!("iteration_{}/{}", iteration, self.max_iterations);
    }

    /// 停止循环
    pub fn stop(&mut self) {
        self.is_running = false;
        self.last_update = chrono::Utc::now().to_rfc3339();
        self.current_phase = "completed".to_string();
    }

    /// 暂停循环
    pub fn pause(&mut self) {
        self.current_phase = "paused".to_string();
    }

    /// 恢复循环
    pub fn resume(&mut self) {
        self.current_phase = "running".to_string();
    }

    /// 生成进度摘要
    pub fn summary(&self) -> String {
        if !self.is_running && self.current_iteration == 0 {
            return "Kaizen 循环未启动".to_string();
        }

        let status_icon = if self.is_running { "🔄" } else { "✅" };
        let progress_pct = if self.max_iterations > 0 {
            (self.current_iteration as f32 / self.max_iterations as f32 * 100.0) as u32
        } else {
            0
        };

        format!(
            "{} Kaizen {} 循环\n\
             进度: {}/{} ({}%)\n\
             ✅ 成功: {} | ❌ 失败: {} | ➖ 中性: {}\n\
             🏆 最佳分数: {:.2}\n\
             阶段: {}",
            status_icon,
            self.mode,
            self.current_iteration,
            self.max_iterations,
            progress_pct,
            self.success_count,
            self.failure_count,
            self.neutral_count,
            self.best_score,
            self.current_phase
        )
    }

    /// 保存进度到文件
    pub fn save(&self) -> Result<()> {
        let data_dir = super::get_kaizen_data_dir()?;
        let progress_path = data_dir.join("progress.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(progress_path, content)?;
        Ok(())
    }

    /// 从文件加载进度
    pub fn load() -> Result<Self> {
        let data_dir = super::get_kaizen_data_dir()?;
        let progress_path = data_dir.join("progress.json");

        if progress_path.exists() {
            let content = std::fs::read_to_string(progress_path)?;
            let progress: KaizenProgress = serde_json::from_str(&content)?;
            Ok(progress)
        } else {
            Ok(KaizenProgress::default())
        }
    }

    /// 追加实验日志
    pub fn append_log(&self, log_entry: &str) -> Result<()> {
        let log_dir = get_kaizen_log_dir()?;
        let log_path = log_dir.join(format!(
            "kaizen_{}.log",
            chrono::Utc::now().format("%Y%m%d")
        ));

        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?
            .write_all(format!("{}\n", log_entry).as_bytes())?;

        Ok(())
    }
}

/// 指标追踪器
pub struct MetricsTracker {
    start_time: Instant,
    iterations_completed: usize,
    total_tests_run: usize,
    avg_iteration_duration_ms: f64,
}

impl MetricsTracker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            iterations_completed: 0,
            total_tests_run: 0,
            avg_iteration_duration_ms: 0.0,
        }
    }

    pub fn record_iteration(&mut self, tests_run: usize, duration_ms: f64) {
        self.iterations_completed += 1;
        self.total_tests_run += tests_run;

        // 移动平均
        let n = self.iterations_completed as f64;
        self.avg_iteration_duration_ms =
            (self.avg_iteration_duration_ms * (n - 1.0) + duration_ms) / n;
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    pub fn summary(&self) -> String {
        format!(
            "⏱️  运行时间: {:.1}秒\n\
             📊 已完成迭代: {}\n\
             🧪 总测试数: {}\n\
             ⚡ 平均迭代时间: {:.1}ms",
            self.elapsed_seconds(),
            self.iterations_completed,
            self.total_tests_run,
            self.avg_iteration_duration_ms
        )
    }
}
