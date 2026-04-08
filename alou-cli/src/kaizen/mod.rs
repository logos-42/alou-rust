//! Kaizen 自进化循环模块
//!
//! 整合 hyperagent 的进化能力，提供两种运行模式：
//! 1. **进化引擎** - 进化"解决任务的代码"
//! 2. **自动研究** - Karpathy 风格，进化"系统自身的代码"
//!
//! ## 使用示例
//!
//! ```bash
//! # 启动进化引擎
//! alou kaizen evolution --iterations 10
//!
//! # 启动自动研究
//! alou kaizen research --auto-push
//!
//! # 查看状态
//! alou kaizen status
//! ```

pub mod config;
pub mod evolution_mode;
pub mod research_mode;
pub mod progress;

pub use config::KaizenConfig;
pub use progress::KaizenProgress;

use anyhow::Result;
use std::path::PathBuf;

/// 获取 Kaizen 数据目录
pub fn get_kaizen_data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
    let data_dir = home.join(".alou/kaizen");
    
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    
    Ok(data_dir)
}

/// 获取 hyperagent 数据目录
pub fn get_hyperagent_data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
    let data_dir = home.join(".alou/.hyperagent/data");
    
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    
    Ok(data_dir)
}

/// 获取 Kaizen 日志目录
pub fn get_kaizen_log_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
    let log_dir = home.join(".alou/kaizen/logs");
    
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
    }
    
    Ok(log_dir)
}
