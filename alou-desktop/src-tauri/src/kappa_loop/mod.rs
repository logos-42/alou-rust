//! Kappa Loop — 卡帕斯循环（自修复循环测试）
//!
//! 整合 Hyperagent 的 AutoResearch 引擎，实现：
//!   Test → Detect → Repair → Verify → Learn → Repeat
//!
//! 核心编排器，将 Hyperagent 的 Karpathy 循环接入 Alou 的自主循环系统。

mod bridge;       // Alou AiClient → Hyperagent LLMClient 桥接
mod circuit_breaker; // 熔断器
mod health_score;    // 健康评分
pub mod commands;    // Tauri 命令

pub use bridge::AlouLlmBridge;
pub use circuit_breaker::CircuitBreaker;
pub use health_score::HealthScore;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use log::{info, warn, error};

/// Kappa 循环配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KappaLoopConfig {
    /// 项目根目录（用于 AutoResearch）
    pub project_root: PathBuf,
    /// 目标源文件列表（相对 src/ 的路径）
    pub target_files: Vec<String>,
    /// 最大迭代次数
    pub max_iterations: u32,
    /// 安全模式（只看不改）
    pub dry_run: bool,
    /// 严格模式（测试必须100%通过才接受）
    pub strict: bool,
    /// 自动 git push
    pub auto_push: bool,
    /// 循环间隔（秒）
    pub cycle_interval_secs: u64,
    /// 启用 web 搜索
    pub enable_web: bool,
    /// 熔断器：最大连续失败次数
    pub max_consecutive_failures: u32,
    /// 健康分阈值（低于此值触发主动自修复）
    pub health_score_threshold: f32,
}

impl Default for KappaLoopConfig {
    fn default() -> Self {
        Self {
            project_root: PathBuf::from("."),
            target_files: vec![
                "autonomous_loop.rs".to_string(),
                "agent/ai_client.rs".to_string(),
                "agent/executor.rs".to_string(),
                "bridges/mod.rs".to_string(),
                "tools/mod.rs".to_string(),
            ],
            max_iterations: 10,
            dry_run: true,
            strict: false,
            auto_push: false,
            cycle_interval_secs: 300, // 5分钟
            enable_web: true,
            max_consecutive_failures: 5,
            health_score_threshold: 60.0,
        }
    }
}

/// Kappa 循环状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KappaLoopState {
    pub is_running: bool,
    pub is_paused: bool,
    pub current_iteration: u32,
    pub total_cycles_completed: u32,
    pub total_improvements: u32,
    pub total_regressions: u32,
    pub total_failures: u32,
    pub last_cycle_time: i64,
    pub health_score: f32,
    pub circuit_breaker_open: bool,
    pub last_error: Option<String>,
    pub experiments_log: Vec<KappaExperimentSummary>,
}

impl Default for KappaLoopState {
    fn default() -> Self {
        Self {
            is_running: false,
            is_paused: false,
            current_iteration: 0,
            total_cycles_completed: 0,
            total_improvements: 0,
            total_regressions: 0,
            total_failures: 0,
            last_cycle_time: Utc::now().timestamp(),
            health_score: 100.0,
            circuit_breaker_open: false,
            last_error: None,
            experiments_log: Vec::new(),
        }
    }
}

/// 实验摘要（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KappaExperimentSummary {
    pub iteration: u32,
    pub file: String,
    pub outcome: String,
    pub hypothesis: String,
    pub tests_before: (u32, u32),
    pub tests_after: (u32, u32),
    pub reflection: String,
    pub timestamp: String,
    pub health_score_delta: f32,
}

/// Kappa 循环编排器
pub struct KappaLoop {
    state: Arc<Mutex<KappaLoopState>>,
    config: Arc<Mutex<KappaLoopConfig>>,
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    health_tracker: Arc<Mutex<HealthScore>>,
}

impl Clone for KappaLoop {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            config: Arc::clone(&self.config),
            circuit_breaker: Arc::clone(&self.circuit_breaker),
            health_tracker: Arc::clone(&self.health_tracker),
        }
    }
}

impl KappaLoop {
    pub fn new(config: KappaLoopConfig) -> Self {
        let max_failures = config.max_consecutive_failures;
        Self {
            state: Arc::new(Mutex::new(KappaLoopState::default())),
            config: Arc::new(Mutex::new(config)),
            circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::new(max_failures))),
            health_tracker: Arc::new(Mutex::new(HealthScore::new())),
        }
    }

    /// 启动 Kappa 循环
    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if state.is_running {
            return Err("Kappa 循环已经在运行".to_string());
        }
        state.is_running = true;
        state.is_paused = false;
        state.last_cycle_time = Utc::now().timestamp();
        info!("[KappaLoop] 🔄 卡帕斯循环已启动");

        drop(state);

        // 在后台任务中运行循环
        let loop_ref = self.clone();
        tokio::spawn(async move {
            loop_ref.run_loop().await;
        });

        Ok(())
    }

    /// 停止 Kappa 循环
    pub async fn stop(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if !state.is_running {
            return Err("Kappa 循环未运行".to_string());
        }
        state.is_running = false;
        state.is_paused = false;
        info!("[KappaLoop] 🛑 卡帕斯循环已停止");
        Ok(())
    }

    /// 暂停 Kappa 循环
    pub async fn pause(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if !state.is_running {
            return Err("Kappa 循环未运行".to_string());
        }
        state.is_paused = true;
        info!("[KappaLoop] ⏸️ 卡帕斯循环已暂停");
        Ok(())
    }

    /// 恢复 Kappa 循环
    pub async fn resume(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if !state.is_running {
            return Err("Kappa 循环未运行".to_string());
        }
        state.is_paused = false;
        info!("[KappaLoop] ▶️ 卡帕斯循环已恢复");
        Ok(())
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> KappaLoopState {
        let state = self.state.lock().await;
        let health = self.health_tracker.lock().await;
        let cb = self.circuit_breaker.lock().await;

        let mut result = state.clone();
        result.health_score = health.score();
        result.circuit_breaker_open = cb.is_open();
        result
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: KappaLoopConfig) {
        let max_failures = new_config.max_consecutive_failures;
        let mut config = self.config.lock().await;
        *config = new_config;
        drop(config);

        // 更新熔断器阈值
        let mut cb = self.circuit_breaker.lock().await;
        *cb = CircuitBreaker::new(max_failures);
    }

    /// 主循环
    async fn run_loop(&self) {
        info!("[KappaLoop] 🔄 主循环开始运行");

        loop {
            // 检查是否停止
            {
                let state = self.state.lock().await;
                if !state.is_running {
                    break;
                }
                if state.is_paused {
                    drop(state);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
            }

            // 检查熔断器
            {
                let cb = self.circuit_breaker.lock().await;
                if cb.is_open() {
                    warn!("[KappaLoop] 🔥 熔断器已打开，等待恢复...");
                    drop(cb);
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    // 半开状态尝试
                    let mut cb = self.circuit_breaker.lock().await;
                    cb.half_open();
                    continue;
                }
            }

            // 执行一个卡帕斯循环
            match self.run_cycle().await {
                Ok(improved) => {
                    let mut cb = self.circuit_breaker.lock().await;
                    cb.record_success();

                    let mut health = self.health_tracker.lock().await;
                    if improved {
                        health.record_improvement();
                    } else {
                        health.record_neutral();
                    }
                }
                Err(e) => {
                    error!("[KappaLoop] ❌ 循环执行失败: {}", e);

                    let mut cb = self.circuit_breaker.lock().await;
                    cb.record_failure();

                    let mut health = self.health_tracker.lock().await;
                    health.record_failure();

                    let mut state = self.state.lock().await;
                    state.last_error = Some(e);
                }
            }

            // 更新状态
            {
                let mut state = self.state.lock().await;
                state.total_cycles_completed += 1;
                state.last_cycle_time = Utc::now().timestamp();
            }

            // 等待下一个周期
            let interval = {
                let config = self.config.lock().await;
                config.cycle_interval_secs
            };
            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
        }

        info!("[KappaLoop] 🔚 主循环已结束");
    }

    /// 执行一个卡帕斯循环
    async fn run_cycle(&self) -> Result<bool, String> {
        let config = self.config.lock().await.clone();

        info!("[KappaLoop] 🧪 开始卡帕斯循环迭代 #{}", {
            let state = self.state.lock().await;
            state.total_cycles_completed + 1
        });

        // 1. 读取 API 配置并创建桥接 LLM 客户端
        let api_config = crate::agent::config::ApiConfig::load().await
            .map_err(|e| format!("无法加载 API 配置: {}", e))?;

        let user_api = api_config.get_active_api()
            .ok_or("未配置 API Key。请在 APP 中配置 API Key 后再启动卡帕斯循环。")?;

        let llm_client = AlouLlmBridge::new(&user_api)
            .map_err(|e| format!("创建 LLM 桥接失败: {}", e))?;

        // 2. 构建 Hyperagent ResearchConfig
        let research_config = hyperagent::auto_research::ResearchConfig {
            project_root: config.project_root.clone(),
            target_files: config.target_files.clone(),
            max_iterations: config.max_iterations,
            auto_push: config.auto_push,
            dry_run: config.dry_run,
            strict: config.strict,
            enable_web: config.enable_web,
            ..Default::default()
        };

        // 3. 创建 AutoResearch 引擎并运行
        let mut engine = hyperagent::AutoResearch::new(llm_client, research_config);

        let experiments: Vec<hyperagent::auto_research::Experiment> = engine.run().await
            .map_err(|e| format!("AutoResearch 执行失败: {}", e))?;

        // 4. 处理结果
        let mut improved = false;
        let mut improvements = 0u32;
        let mut regressions = 0u32;
        let mut failures = 0u32;

        for exp in &experiments {
            let outcome_str = match exp.outcome {
                hyperagent::auto_research::ExperimentOutcome::Improved => {
                    improved = true;
                    improvements += 1;
                    "Improved"
                }
                hyperagent::auto_research::ExperimentOutcome::Neutral => "Neutral",
                hyperagent::auto_research::ExperimentOutcome::Regressed => {
                    regressions += 1;
                    "Regressed"
                }
                hyperagent::auto_research::ExperimentOutcome::Failed => {
                    failures += 1;
                    "Failed"
                }
            };

            let summary = KappaExperimentSummary {
                iteration: exp.iteration,
                file: exp.file.clone(),
                outcome: outcome_str.to_string(),
                hypothesis: exp.hypothesis.chars().take(100).collect(),
                tests_before: exp.tests_before,
                tests_after: exp.tests_after,
                reflection: exp.reflection.chars().take(200).collect(),
                timestamp: exp.timestamp.clone(),
                health_score_delta: 0.0, // Will be calculated
            };

            let mut state = self.state.lock().await;
            state.experiments_log.push(summary);
            // 保持日志在最近100条
            let len = state.experiments_log.len();
            if len > 100 {
                state.experiments_log.drain(0..len - 100);
            }
        }

        // 5. 更新状态
        {
            let mut state = self.state.lock().await;
            state.current_iteration += experiments.len() as u32;
            state.total_improvements += improvements;
            state.total_regressions += regressions;
            state.total_failures += failures;
        }

        info!(
            "[KappaLoop] ✅ 循环完成: {} 改进, {} 回退, {} 失败",
            improvements, regressions, failures
        );

        Ok(improved)
    }

    /// 手动触发一次自修复
    pub async fn trigger_self_repair(&self) -> Result<serde_json::Value, String> {
        info!("[KappaLoop] 🔧 手动触发自修复");

        // 临时执行一个循环
        match self.run_cycle().await {
            Ok(improved) => {
                let state = self.state.lock().await;
                Ok(serde_json::json!({
                    "success": true,
                    "improved": improved,
                    "total_improvements": state.total_improvements,
                    "total_failures": state.total_failures,
                    "health_score": self.health_tracker.lock().await.score(),
                }))
            }
            Err(e) => {
                Ok(serde_json::json!({
                    "success": false,
                    "error": e,
                }))
            }
        }
    }
}
