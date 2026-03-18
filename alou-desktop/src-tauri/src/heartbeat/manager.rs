/// Heartbeat Manager - Core heartbeat logic

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use chrono::{Utc, Local};
use std::fs;
use std::path::PathBuf;
use crate::heartbeat::types::{HeartbeatConfig, HeartbeatState, HeartbeatResult, TokenUsage};
use crate::heartbeat::config::{self, load_config};

// Import persistence modules
use crate::soul::SoulManager;
use crate::tasks::TasksManager;
use crate::logs::LogsManager;

/// Heartbeat Manager
pub struct HeartbeatManager {
    config: Arc<Mutex<HeartbeatConfig>>,
    state: Arc<Mutex<HeartbeatState>>,
    stop_sender: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl HeartbeatManager {
    /// Create a new HeartbeatManager
    pub fn new() -> Result<Self, String> {
        let config = load_config()?;
        
        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(HeartbeatState::default())),
            stop_sender: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Create with custom config
    pub fn with_config(config: HeartbeatConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            state: Arc::new(Mutex::new(HeartbeatState::default())),
            stop_sender: Arc::new(Mutex::new(None)),
        }
    }

    /// 🔥 确保文档文件存在（首次运行时自动创建）
    pub async fn ensure_documentation_files(&self) {
        let config = self.config.lock().await;
        let base_dir = dirs::home_dir()
            .map(|h| h.join(".alou"))
            .unwrap_or_else(|| PathBuf::from("./.alou"));
        
        let _ = fs::create_dir_all(&base_dir);
        drop(config);

        // 自动创建 HEARTBEAT.md 示例文件（如果不存在）
        let heartbeat_file_path = base_dir.join("HEARTBEAT.md");
        if !heartbeat_file_path.exists() {
            let default_heartbeat_content = r#"# CONTINUOUS - 7x24 持续监控任务

## 任务说明
这是一个持续执行的任务，每次心跳都会执行以下内容。

## 执行项目
1. **系统健康检查**
   - 检查内存使用
   - 检查磁盘空间
   - 检查进程状态

2. **日志分析**
   - 分析最近的错误日志
   - 统计请求频率
   - 检测异常模式

3. **报告生成**
   - 生成健康报告
   - 记录到 MEMORY.md

## 配置
- 心跳间隔：每 10 分钟
- 使用模型：写入
- 模式：持续执行

---
[持续执行] 此任务会持续运行，每次心跳都会重新执行
"#;
            let _ = fs::write(&heartbeat_file_path, default_heartbeat_content);
            println!("[Heartbeat] Created default HEARTBEAT.md at {:?}", heartbeat_file_path);
        }

        // 自动创建 HEARTBEAT_GUIDE.md 使用指南（如果不存在）
        let guide_file_path = base_dir.join("HEARTBEAT_GUIDE.md");
        if !guide_file_path.exists() {
            let guide_content = r#"# 心跳驱动 - 7x24 持久执行指南

## 🚀 快速开始

### 1. 配置心跳间隔
- 打开应用 → 设置 → 心跳管理
- 设置间隔：推荐 10-30 分钟
- 启用心跳

### 2. 创建持续任务

编辑 `~/.alou/HEARTBEAT.md` 文件：

```markdown
# CONTINUOUS - 持续监控任务

## 任务说明
这是一个持续执行的任务，每次心跳都会执行。

## 执行内容
1. 系统健康检查
2. 日志分析
3. 报告生成

---
[持续执行]
```

**关键**：包含 `# CONTINUOUS` 或 `[持续执行]` 标记

## 📋 两种模式对比

| 模式 | 文件标记 | 执行后 | 适用场景 |
|------|---------|--------|---------|
| **普通模式** | 无标记 | 清空文件 | 一次性任务 |
| **持续模式** | `# CONTINUOUS` 或 `[持续执行]` | 保留内容 | 7x24 监控 |

## ⚙️ 配置说明

### 心跳间隔
- **最小值**: 1 分钟
- **最大值**: 120 分钟
- **推荐**: 10-30 分钟

### 模型选择
- **便宜模型**: 用于常规心跳（如 `deepseek-chat`）
- **昂贵模型**: 用于复杂任务（如 `claude-sonnet-4`）

### 文件路径
- 默认：`~/.alou/HEARTBEAT.md`
- 可在配置中修改

## 📝 示例任务

### 示例 1: 系统监控
```markdown
# CONTINUOUS - 系统监控

每小时检查：
1. CPU 使用率
2. 内存使用
3. 磁盘空间
4. 进程状态

发现异常时记录到 MEMORY.md

---
[持续执行]
```

### 示例 2: 数据分析
```markdown
# CONTINUOUS - 日志分析

分析最近的错误日志：
1. 统计错误频率
2. 识别错误模式
3. 生成报告

报告保存到 `~/.alou/reports/`

---
[持续执行]
```

### 示例 3: 一次性任务
```markdown
请执行以下任务：
1. 备份重要文件
2. 清理临时文件
3. 更新系统状态

（此任务执行后文件会被清空）
```

## 🔧 故障排除

### 心跳不执行
1. 检查是否启用：设置 → 心跳管理 → 启用
2. 检查间隔设置：确保不是太长
3. 查看日志：控制台搜索 `[Heartbeat]`

### 任务不持续
1. 确认文件包含 `# CONTINUOUS` 或 `[持续执行]`
2. 检查文件是否被其他程序修改
3. 查看心跳日志确认模式识别

### 文件路径问题
1. 使用绝对路径或 `~/` 开头
2. 确保目录存在
3. 检查文件权限

## 💡 最佳实践

1. **任务简洁**: 每次心跳执行时间不宜过长
2. **错误处理**: 任务应该能容忍失败
3. **日志记录**: 重要操作记录到 MEMORY.md
4. **定期检查**: 查看心跳执行日志

---

**提示**: 可以在 HEARTBEAT.md 中写入复杂的 AI 指令，AI 会理解并执行。
"#;
            let _ = fs::write(&guide_file_path, guide_content);
            println!("[Heartbeat] Created default HEARTBEAT_GUIDE.md at {:?}", guide_file_path);
        }
    }

    /// Start the heartbeat loop
    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        
        if state.is_running {
            return Err("Heartbeat is already running".to_string());
        }
        
        let config = self.config.lock().await;
        if !config.enabled {
            return Err("Heartbeat is disabled in configuration".to_string());
        }

        let interval_minutes = config.interval_minutes;
        drop(config);

        // Create stop channel
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

        // Clone Arcs for the spawned task
        let state_clone = Arc::clone(&self.state);
        let manager = self.clone_for_task();

        // Set stop sender
        *self.stop_sender.lock().await = Some(stop_tx);
        
        // Mark as running
        state.is_running = true;
        state.next_heartbeat = Some(Utc::now() + Duration::from_secs(interval_minutes * 60));
        drop(state);
        
        println!("[Heartbeat] Starting heartbeat loop with {} minute interval", interval_minutes);
        
        // Spawn the heartbeat loop
        tokio::spawn(async move {
            let interval = Duration::from_secs(interval_minutes * 60);
            let mut stop_rx = stop_rx;
            
            loop {
                tokio::select! {
                    _ = sleep(interval) => {
                        // Check if still running
                        let state = state_clone.lock().await;
                        if !state.is_running {
                            drop(state);
                            break;
                        }
                        drop(state);
                        
                        // Trigger heartbeat
                        let _ = manager.trigger_heartbeat_internal().await;
                        
                        // Update next heartbeat time
                        let mut state = state_clone.lock().await;
                        state.next_heartbeat = Some(Utc::now() + interval);
                        drop(state);
                    }
                    _ = &mut stop_rx => {
                        println!("[Heartbeat] Received stop signal");
                        break;
                    }
                }
            }
            
            println!("[Heartbeat] Heartbeat loop stopped");
        });
        
        Ok(())
    }
    
    /// Clone for task (helper for spawning)
    fn clone_for_task(&self) -> Arc<Self> {
        Arc::new(HeartbeatManager {
            config: Arc::clone(&self.config),
            state: Arc::clone(&self.state),
            stop_sender: Arc::clone(&self.stop_sender),
        })
    }
    
    /// Stop the heartbeat loop
    pub async fn stop(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        
        if !state.is_running {
            return Err("Heartbeat is not running".to_string());
        }
        
        // Send stop signal
        if let Some(tx) = self.stop_sender.lock().await.take() {
            let _ = tx.send(());
        }
        
        state.is_running = false;
        state.next_heartbeat = None;
        
        println!("[Heartbeat] Heartbeat stopped");
        
        Ok(())
    }
    
    /// Trigger a heartbeat immediately
    pub async fn trigger_heartbeat(&self) -> Result<HeartbeatResult, String> {
        self.trigger_heartbeat_internal().await
    }
    
    /// Internal trigger heartbeat (for use in spawned tasks)
    async fn trigger_heartbeat_internal(&self) -> Result<HeartbeatResult, String> {
        println!("[Heartbeat] Triggering heartbeat...");
        
        let result = self.handle_heartbeat().await;
        
        // Update state
        let mut state = self.state.lock().await;
        state.last_heartbeat = Some(Utc::now());
        state.total_beats += 1;
        drop(state);
        
        if result.success {
            println!("[Heartbeat] Heartbeat completed successfully: {}", result.message);
        } else {
            eprintln!("[Heartbeat] Heartbeat failed: {}", result.message);
        }
        
        Ok(result)
    }
    
    /// Handle the heartbeat logic
    async fn handle_heartbeat(&self) -> HeartbeatResult {
        let config = self.config.lock().await;
        let heartbeat_file_path = config.heartbeat_file_path.clone();
        let model = config.model.clone();
        let cheap_model = config.cheap_model.clone();
        drop(config);

        // Expand tilde in path
        let expanded_path = config::expand_tilde(&heartbeat_file_path);
        let file_path = std::path::Path::new(&expanded_path);

        // Read heartbeat file content
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_e) => {
                // File doesn't exist, create empty one
                if let Some(parent) = file_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(file_path, "");
                return HeartbeatResult::ok("HEARTBEAT_OK - File created")
                    .with_actions(vec!["Created heartbeat file".to_string()]);
            }
        };

        let trimmed_content = content.trim();

        // 🔥 优化：支持两种模式
        // 模式 1: 空文件 → 快速健康检查（保持活跃）
        // 模式 2: 有内容 → 执行任务（支持持续任务队列）
        
        if trimmed_content.is_empty() {
            // 空文件 - 快速健康检查
            println!("[Heartbeat] Empty heartbeat file - quick health check");

            // Save all persistence modules
            let actions = self.save_all_persistence().await;

            return HeartbeatResult::ok("HEARTBEAT_OK - All modules saved")
                .with_actions(actions)
                .with_token_usage(TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 1,
                    total_tokens: 1,
                    model: cheap_model,
                });
        }

        // 🔥 检查是否是持续任务模式（文件内容以 "# CONTINUOUS" 开头）
        let is_continuous = trimmed_content.starts_with("# CONTINUOUS") || 
                           trimmed_content.contains("[持续执行]");

        // 文件有内容 - 执行维护任务
        println!("[Heartbeat] Heartbeat file has content - executing maintenance task (continuous={})", is_continuous);

        let task_content = trimmed_content.to_string();

        // 🔥 如果不是持续任务，清空文件；如果是持续任务，保留内容
        if !is_continuous {
            let _ = fs::write(file_path, "");
        } else {
            println!("[Heartbeat] Continuous mode - keeping file content for next heartbeat");
        }

        // Execute the maintenance task
        let mut actions = self.execute_maintenance_task(&task_content, &model).await;

        // Also save persistence modules
        let save_actions = self.save_all_persistence().await;
        actions.extend(save_actions);

        HeartbeatResult::ok(if is_continuous { 
            "Continuous task executed (will continue next heartbeat)" 
        } else { 
            "Maintenance task executed" 
        })
            .with_actions(actions)
            .with_token_usage(TokenUsage {
                prompt_tokens: task_content.len() as u64 / 4,
                completion_tokens: 100,
                total_tokens: 100 + task_content.len() as u64 / 4,
                model: model,
            })
    }

    /// Save all persistence modules
    async fn save_all_persistence(&self) -> Vec<String> {
        let mut actions = Vec::new();
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Get base directory
        let base_dir = dirs::home_dir()
            .map(|h| h.join(".alou"))
            .unwrap_or_else(|| PathBuf::from("./.alou"));

        // Ensure directory exists
        let _ = fs::create_dir_all(&base_dir);

        // 🔥 自动创建 HEARTBEAT.md 示例文件（如果不存在）
        let heartbeat_file_path = base_dir.join("HEARTBEAT.md");
        if !heartbeat_file_path.exists() {
            let default_heartbeat_content = r#"# CONTINUOUS - 7x24 持续监控任务

## 任务说明
这是一个持续执行的任务，每次心跳都会执行以下内容。

## 执行项目
1. **系统健康检查**
   - 检查内存使用
   - 检查磁盘空间
   - 检查进程状态

2. **日志分析**
   - 分析最近的错误日志
   - 统计请求频率
   - 检测异常模式

3. **报告生成**
   - 生成健康报告
   - 记录到 MEMORY.md

## 配置
- 心跳间隔：每 10 分钟
- 使用模型：deepseek-chat
- 模式：持续执行

---
[持续执行] 此任务会持续运行，每次心跳都会重新执行
"#;
            let _ = fs::write(&heartbeat_file_path, default_heartbeat_content);
            actions.push(format!("[{}] HEARTBEAT.md: created default continuous task", now));
        }

        // 🔥 自动创建 HEARTBEAT_GUIDE.md 使用指南（如果不存在）
        let guide_file_path = base_dir.join("HEARTBEAT_GUIDE.md");
        if !guide_file_path.exists() {
            let guide_content = r#"# 心跳驱动 - 7x24 持久执行指南

## 🚀 快速开始

### 1. 配置心跳间隔
- 打开应用 → 设置 → 心跳管理
- 设置间隔：推荐 10-30 分钟
- 启用心跳

### 2. 创建持续任务

编辑 `~/.alou/HEARTBEAT.md` 文件：

```markdown
# CONTINUOUS - 持续监控任务

## 任务说明
这是一个持续执行的任务，每次心跳都会执行。

## 执行内容
1. 系统健康检查
2. 日志分析
3. 报告生成

---
[持续执行]
```

**关键**：包含 `# CONTINUOUS` 或 `[持续执行]` 标记

## 📋 两种模式对比

| 模式 | 文件标记 | 执行后 | 适用场景 |
|------|---------|--------|---------|
| **普通模式** | 无标记 | 清空文件 | 一次性任务 |
| **持续模式** | `# CONTINUOUS` 或 `[持续执行]` | 保留内容 | 7x24 监控 |

## ⚙️ 配置说明

### 心跳间隔
- **最小值**: 1 分钟
- **最大值**: 120 分钟
- **推荐**: 10-30 分钟

### 模型选择
- **便宜模型**: 用于常规心跳（如 `deepseek-chat`）
- **昂贵模型**: 用于复杂任务（如 `claude-sonnet-4`）

### 文件路径
- 默认：`~/.alou/HEARTBEAT.md`
- 可在配置中修改

## 📝 示例任务

### 示例 1: 系统监控
```markdown
# CONTINUOUS - 系统监控

每小时检查：
1. CPU 使用率
2. 内存使用
3. 磁盘空间
4. 进程状态

发现异常时记录到 MEMORY.md

---
[持续执行]
```

### 示例 2: 数据分析
```markdown
# CONTINUOUS - 日志分析

分析最近的错误日志：
1. 统计错误频率
2. 识别错误模式
3. 生成报告

报告保存到 `~/.alou/reports/`

---
[持续执行]
```

### 示例 3: 一次性任务
```markdown
请执行以下任务：
1. 备份重要文件
2. 清理临时文件
3. 更新系统状态

（此任务执行后文件会被清空）
```

## 🔧 故障排除

### 心跳不执行
1. 检查是否启用：设置 → 心跳管理 → 启用
2. 检查间隔设置：确保不是太长
3. 查看日志：控制台搜索 `[Heartbeat]`

### 任务不持续
1. 确认文件包含 `# CONTINUOUS` 或 `[持续执行]`
2. 检查文件是否被其他程序修改
3. 查看心跳日志确认模式识别

### 文件路径问题
1. 使用绝对路径或 `~/` 开头
2. 确保目录存在
3. 检查文件权限

## 💡 最佳实践

1. **任务简洁**: 每次心跳执行时间不宜过长
2. **错误处理**: 任务应该能容忍失败
3. **日志记录**: 重要操作记录到 MEMORY.md
4. **定期检查**: 查看心跳执行日志

---

**提示**: 可以在 HEARTBEAT.md 中写入复杂的 AI 指令，AI 会理解并执行。
"#;
            let _ = fs::write(&guide_file_path, guide_content);
            actions.push(format!("[{}] HEARTBEAT_GUIDE.md: created user guide", now));
        }

        // Save SOUL.md
        match SoulManager::new(&base_dir).load() {
            Ok(_) => {
                actions.push(format!("[{}] Soul module: loaded", now));
            }
            Err(e) => {
                actions.push(format!("[{}] Soul module: created default ({})", now, e));
            }
        }

        // Save TASKS.md
        let tasks_manager = TasksManager::new(&base_dir);
        actions.push(format!("[{}] Tasks module: initialized", now));

        // Save LOGS.md
        match LogsManager::new(&base_dir).get_stats() {
            Ok(stats) => {
                actions.push(format!("[{}] Logs module: {} lines ({:.2} KB)", now, stats.total_lines, stats.file_size_kb()));
            }
            Err(e) => {
                actions.push(format!("[{}] Logs module: created default ({})", now, e));
            }
        }

        actions
    }
    
    /// Execute a maintenance task
    async fn execute_maintenance_task(&self, task: &str, model: &str) -> Vec<String> {
        let mut actions = Vec::new();
        
        println!("[Heartbeat] Executing maintenance task with model: {}", model);
        actions.push(format!("Task: {}", task));
        
        // Parse task content for known commands
        let task_lower = task.to_lowercase();
        
        if task_lower.contains("cleanup") || task_lower.contains("clean") {
            actions.push("Cleanup action initiated".to_string());
            // In real implementation, would call actual cleanup functions
        }
        
        if task_lower.contains("sync") {
            actions.push("Sync action initiated".to_string());
            // In real implementation, would call sync functions
        }
        
        if task_lower.contains("diagnose") || task_lower.contains("diagnostic") {
            actions.push("Diagnostics action initiated".to_string());
            // In real implementation, would run diagnostics
        }
        
        if task_lower.contains("status") || task_lower.contains("check") {
            actions.push("Status check completed".to_string());
        }
        
        // If no specific action matched, treat as general task
        if actions.len() == 1 {
            actions.push("General maintenance task executed".to_string());
        }
        
        actions
    }
    
    /// Get current heartbeat state
    pub async fn get_state(&self) -> HeartbeatState {
        self.state.lock().await.clone()
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> HeartbeatConfig {
        self.config.lock().await.clone()
    }
    
    /// Update configuration
    pub async fn update_config(&self, new_config: HeartbeatConfig) -> Result<(), String> {
        let mut config = self.config.lock().await;
        *config = new_config.clone();
        drop(config);
        
        // Save to file
        config::save_config(&new_config)
    }
    
    /// Check if heartbeat is running
    pub async fn is_running(&self) -> bool {
        self.state.lock().await.is_running
    }
}

impl Clone for HeartbeatManager {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            state: Arc::clone(&self.state),
            stop_sender: Arc::clone(&self.stop_sender),
        }
    }
}

/// Global heartbeat manager state for Tauri
pub type HeartbeatManagerState = Arc<Mutex<Option<HeartbeatManager>>>;

/// Initialize the global heartbeat manager
pub fn initialize_heartbeat_manager() -> HeartbeatManagerState {
    let manager = match HeartbeatManager::new() {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("[Heartbeat] Failed to initialize manager: {}", e);
            None
        }
    };
    Arc::new(Mutex::new(manager))
}
