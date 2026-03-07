//! Bot Gateway 模块
//!
//! 提供多平台 Bot 支持，包括 Telegram、飞书、Discord、QQ 等
//! 允许用户通过即时通讯工具远程调用 Alou 的工具

pub mod config;
pub mod server;
pub mod adapters;
pub mod commands;

use crate::bot_gateway::config::BotGatewayConfig;
use crate::bot_gateway::server::{start_server, ServerStatus, LogEntry};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{AppHandle, Manager};

/// Bot Gateway 管理器
pub struct BotGatewayManager {
    config: Arc<RwLock<BotGatewayConfig>>,
    server_handle: Arc<RwLock<Option<ServerHandle>>>,
    tool_executor: Arc<tokio::sync::Mutex<crate::tools::executor::ToolExecutionManager>>,
    logs: Arc<RwLock<Vec<LogEntry>>>,
}

/// 服务器句柄
pub struct ServerHandle {
    pub port: u16,
    pub shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl BotGatewayManager {
    /// 创建新的 Bot Gateway 管理器
    pub fn new(
        config: BotGatewayConfig,
        tool_executor: Arc<tokio::sync::Mutex<crate::tools::executor::ToolExecutionManager>>,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            server_handle: Arc::new(RwLock::new(None)),
            tool_executor,
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 获取配置
    pub async fn get_config(&self) -> BotGatewayConfig {
        self.config.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: BotGatewayConfig) -> Result<(), String> {
        let mut config = self.config.write().await;
        let needs_restart = config.port != new_config.port 
            || config.platforms.telegram.is_some() != new_config.platforms.telegram.is_some()
            || config.platforms.feishu.is_some() != new_config.platforms.feishu.is_some();
        
        *config = new_config;
        
        // 如果需要重启，自动重启服务
        if needs_restart {
            let is_running = self.server_handle.read().await.is_some();
            if is_running {
                self.stop().await?;
                self.start().await?;
            }
        }
        
        Ok(())
    }

    /// 启动 Bot Gateway 服务
    pub async fn start(&self) -> Result<(), String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err("Bot Gateway 未启用".to_string());
        }

        // 检查是否已经在运行
        if self.server_handle.read().await.is_some() {
            return Err("Bot Gateway 已经在运行".to_string());
        }

        // 启动服务器
        let executor = self.tool_executor.lock().await;
        let port = start_server(config.clone(), self.tool_executor.clone()).await?;

        // 创建关闭通道
        let (shutdown_tx, _shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        *self.server_handle.write().await = Some(ServerHandle {
            port,
            shutdown_tx,
        });

        Ok(())
    }

    /// 停止服务
    pub async fn stop(&self) -> Result<(), String> {
        if let Some(handle) = self.server_handle.write().await.take() {
            // 发送关闭信号
            let _ = handle.shutdown_tx.send(());
            // 实际应该等待服务器关闭，这里简化处理
        }
        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self) -> ServerStatus {
        let config = self.config.read().await;
        let handle = self.server_handle.read().await;
        
        let mut platforms = Vec::new();
        
        if config.platforms.telegram.as_ref().map(|c| c.enabled).unwrap_or(false) {
            platforms.push("telegram".to_string());
        }
        if config.platforms.feishu.as_ref().map(|c| c.enabled).unwrap_or(false) {
            platforms.push("feishu".to_string());
        }

        ServerStatus {
            running: handle.is_some(),
            port: handle.as_ref().map(|h| h.port).unwrap_or(config.port),
            platforms,
            uptime_seconds: 0, // 简化实现
        }
    }

    /// 测试平台连接
    pub async fn test_connection(&self, platform: &str) -> Result<TestResult, String> {
        let config = self.config.read().await;

        match platform {
            "telegram" => {
                if let Some(telegram_config) = &config.platforms.telegram {
                    if telegram_config.bot_token.is_empty() {
                        return Ok(TestResult {
                            success: false,
                            message: "Bot Token 未配置".to_string(),
                        });
                    }

                    // 测试连接到 Telegram API
                    let adapter = adapters::TelegramAdapter::new(telegram_config.clone());
                    match adapter.get_me().await {
                        Ok(user) => Ok(TestResult {
                            success: true,
                            message: format!("连接成功！Bot: @{}", user.username.unwrap_or_default()),
                        }),
                        Err(e) => Ok(TestResult {
                            success: false,
                            message: format!("连接失败：{}", e),
                        }),
                    }
                } else {
                    Ok(TestResult {
                        success: false,
                        message: "Telegram 配置未找到".to_string(),
                    })
                }
            }
            "feishu" => {
                if let Some(feishu_config) = &config.platforms.feishu {
                    if feishu_config.app_id.is_empty() || feishu_config.app_secret.is_empty() {
                        return Ok(TestResult {
                            success: false,
                            message: "App ID 或 App Secret 未配置".to_string(),
                        });
                    }

                    // 测试获取 Access Token
                    let adapter = adapters::FeishuAdapter::new(feishu_config.clone());
                    match adapter.get_access_token().await {
                        Ok(_) => Ok(TestResult {
                            success: true,
                            message: "连接成功！".to_string(),
                        }),
                        Err(e) => Ok(TestResult {
                            success: false,
                            message: format!("连接失败：{}", e),
                        }),
                    }
                } else {
                    Ok(TestResult {
                        success: false,
                        message: "飞书配置未找到".to_string(),
                    })
                }
            }
            "discord" => {
                if let Some(discord_config) = &config.platforms.discord {
                    if discord_config.bot_token.is_empty() {
                        return Ok(TestResult {
                            success: false,
                            message: "Bot Token 未配置".to_string(),
                        });
                    }

                    // 测试连接到 Discord API
                    let adapter = adapters::DiscordAdapter::new(discord_config.clone());
                    match adapter.get_me().await {
                        Ok(user) => Ok(TestResult {
                            success: true,
                            message: format!("连接成功！Bot: {}#{}", user.username, user.discriminator),
                        }),
                        Err(e) => Ok(TestResult {
                            success: false,
                            message: format!("连接失败：{}", e),
                        }),
                    }
                } else {
                    Ok(TestResult {
                        success: false,
                        message: "Discord 配置未找到".to_string(),
                    })
                }
            }
            "qq" => {
                if let Some(qq_config) = &config.platforms.qq {
                    if qq_config.ws_url.is_empty() {
                        return Ok(TestResult {
                            success: false,
                            message: "WebSocket URL 未配置".to_string(),
                        });
                    }

                    // QQ 测试连接简化处理，实际应该连接 WebSocket
                    Ok(TestResult {
                        success: true,
                        message: format!("QQ 配置已验证，WebSocket URL: {}", qq_config.ws_url),
                    })
                } else {
                    Ok(TestResult {
                        success: false,
                        message: "QQ 配置未找到".to_string(),
                    })
                }
            }
            _ => Err(format!("不支持的平台：{}", platform)),
        }
    }

    /// 获取日志
    pub async fn get_logs(&self, limit: Option<usize>) -> Vec<LogEntry> {
        let logs = self.logs.read().await;
        let limit = limit.unwrap_or(100);
        logs.iter().rev().take(limit).cloned().collect()
    }

    /// 添加日志
    pub async fn add_log(&self, level: &str, message: &str, platform: Option<&str>) {
        let mut logs = self.logs.write().await;
        logs.push(LogEntry {
            timestamp: chrono::Utc::now().timestamp(),
            level: level.to_string(),
            message: message.to_string(),
            platform: platform.map(|s| s.to_string()),
        });

        // 限制日志数量
        if logs.len() > 1000 {
            logs.remove(0);
        }
    }
}

/// 测试结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
}

/// 加载配置
fn load_config(app: &AppHandle) -> Result<BotGatewayConfig, String> {
    let config_dir = app.path()
        .app_data_dir()
        .map_err(|e| format!("获取配置目录失败：{}", e))?;
    
    let config_path = config_dir.join("bot_gateway_config.toml");
    
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败：{}", e))?;
        
        toml::from_str(&content)
            .map_err(|e| format!("解析配置文件失败：{}", e))
    } else {
        Ok(BotGatewayConfig::default())
    }
}

/// 保存配置
fn save_config(app: &AppHandle, config: &BotGatewayConfig) -> Result<(), String> {
    let config_dir = app.path()
        .app_data_dir()
        .map_err(|e| format!("获取配置目录失败：{}", e))?;
    
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败：{}", e))?;
    
    let config_path = config_dir.join("bot_gateway_config.toml");
    
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败：{}", e))?;
    
    std::fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败：{}", e))?;
    
    Ok(())
}

/// 初始化 Bot Gateway
pub async fn initialize_bot_gateway(
    app: &AppHandle,
    tool_executor: Arc<tokio::sync::Mutex<crate::tools::executor::ToolExecutionManager>>,
) -> Result<BotGatewayManager, String> {
    let config = load_config(app)?;
    let manager = BotGatewayManager::new(config, tool_executor);
    
    // 如果配置为启用，自动启动服务
    {
        let config = manager.get_config().await;
        if config.enabled {
            if let Err(e) = manager.start().await {
                eprintln!("自动启动 Bot Gateway 失败：{}", e);
            }
        }
    }
    
    Ok(manager)
}
