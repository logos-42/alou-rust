//! Bot Gateway Tauri Commands
//!
//! 提供前端与 Bot Gateway 交互的命令接口

use crate::bot_gateway::{BotGatewayManager, TestResult};
use crate::bot_gateway::config::BotGatewayConfig;
use crate::bot_gateway::server::{ServerStatus, LogEntry};
use tauri::{command, AppHandle, Manager};

/// 获取 Bot Gateway 配置
#[command]
pub async fn get_bot_gateway_config(
    app: AppHandle,
) -> Result<BotGatewayConfig, String> {
    let manager = app.state::<BotGatewayManager>();
    Ok(manager.get_config().await)
}

/// 更新 Bot Gateway 配置
#[command]
pub async fn update_bot_gateway_config(
    app: AppHandle,
    config: BotGatewayConfig,
) -> Result<(), String> {
    let manager = app.state::<BotGatewayManager>();
    manager.update_config(config).await?;
    
    // 保存配置到文件
    super::save_config(&app, &manager.get_config().await)
}

/// 测试平台连接
#[command]
pub async fn test_platform_connection(
    app: AppHandle,
    platform: String,
) -> Result<TestResult, String> {
    let manager = app.state::<BotGatewayManager>();
    manager.test_connection(&platform).await
}

/// 获取 Bot Gateway 状态
#[command]
pub async fn get_bot_gateway_status(
    app: AppHandle,
) -> Result<ServerStatus, String> {
    let manager = app.state::<BotGatewayManager>();
    Ok(manager.get_status().await)
}

/// 启动/停止 Bot Gateway
#[command]
pub async fn toggle_bot_gateway(
    app: AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let manager = app.state::<BotGatewayManager>();
    
    // 更新启用状态
    let mut config = manager.get_config().await;
    config.enabled = enabled;
    manager.update_config(config).await?;
    
    if enabled {
        manager.start().await
    } else {
        manager.stop().await
    }
}

/// 获取日志
#[command]
pub async fn get_bot_gateway_logs(
    app: AppHandle,
    lines: Option<usize>,
) -> Result<Vec<LogEntry>, String> {
    let manager = app.state::<BotGatewayManager>();
    Ok(manager.get_logs(lines).await)
}

/// 启动 Bot Gateway（应用启动时调用）
#[command]
pub async fn start_bot_gateway(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<BotGatewayManager>();
    manager.start().await
}

/// 停止 Bot Gateway
#[command]
pub async fn stop_bot_gateway(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<BotGatewayManager>();
    manager.stop().await
}

/// 重置配置
#[command]
pub async fn reset_bot_gateway_config(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<BotGatewayManager>();
    let default_config = BotGatewayConfig::default();
    manager.update_config(default_config).await?;
    super::save_config(&app, &manager.get_config().await)
}
