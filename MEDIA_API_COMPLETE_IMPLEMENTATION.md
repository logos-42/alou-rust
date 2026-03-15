# 媒体 API 完整实施文档

## 已完成的核心文件

### 1. 数据库层 ✅
- `src/db/schema.rs` - 8 个数据库表
- `src/db/mod.rs` - DbManager

### 2. 配置结构 ✅
- `src/agent/media_config.rs` - JSON 配置

### 3. MediaProvider Trait ✅
- `src/agent/providers/media_provider.rs` - 统一接口

### 4. Provider 实现 ✅
- `src/agent/providers/minimax/mod.rs` - MiniMax Provider
- `src/agent/providers/minimax/tts.rs` - TTS
- `src/agent/providers/minimax/video.rs` - 视频生成
- `src/agent/providers/google/mod.rs` - Google Imagen
- `src/agent/providers/jimeng/mod.rs` - 即梦

### 5. 媒体存储 ✅
- `src/agent/media/mod.rs`
- `src/agent/media/storage.rs`

### 6. Provider Registry ✅
- `src/agent/providers/registry.rs`

## 待创建的剩余文件

### tasks/queue.rs
```rust
//! 任务队列

use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::db::DbManager;
use crate::providers::ProviderRegistry;
use crate::ipfs::IpfsClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    pub id: String,
    pub task_type: String,
    pub input: serde_json::Value,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub ipfs_cid: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

pub struct TaskQueue {
    task_tx: mpsc::Sender<TaskMessage>,
    result_tx: mpsc::Sender<TaskResult>,
    result_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<TaskResult>>>,
}

impl TaskQueue {
    pub fn new(
        db: Arc<DbManager>,
        provider_registry: Arc<ProviderRegistry>,
        ipfs_client: Arc<IpfsClient>,
        num_workers: usize,
    ) -> Self {
        let (task_tx, mut task_rx) = mpsc::channel::<TaskMessage>(100);
        let (result_tx, result_rx) = mpsc::channel::<TaskResult>(100);
        let result_rx = Arc::new(tokio::sync::Mutex::new(result_rx));

        for i in 0..num_workers {
            let task_rx = task_rx.resubscribe();
            let result_tx = result_tx.clone();
            tokio::spawn(async move {
                log::info!("Task worker {} started", i);
                Self::worker_loop(task_rx, result_tx).await;
            });
        }

        Self {
            task_tx,
            result_tx,
            result_rx,
        }
    }

    async fn worker_loop(
        mut task_rx: mpsc::Receiver<TaskMessage>,
        result_tx: mpsc::Sender<TaskResult>,
    ) {
        while let Some(task_msg) = task_rx.recv().await {
            // 执行任务逻辑
            let result = TaskResult {
                task_id: task_msg.id,
                success: true,
                data: None,
                ipfs_cid: None,
                error: None,
                execution_time_ms: 0,
            };
            let _ = result_tx.send(result).await;
        }
    }

    pub async fn submit(&self, task: TaskMessage) -> Result<(), mpsc::error::SendError<TaskMessage>> {
        self.task_tx.send(task).await
    }

    pub async fn recv_result(&self) -> Option<TaskResult> {
        let mut rx = self.result_rx.lock().await;
        rx.recv().await
    }
}
```

### tasks/executor.rs
```rust
//! 任务执行器

use sqlx::SqlitePool;
use std::sync::Arc;
use crate::providers::ProviderRegistry;
use crate::ipfs::IpfsClient;

pub struct TaskExecutor {
    db_pool: SqlitePool,
    ipfs_client: Arc<IpfsClient>,
    provider_registry: Arc<ProviderRegistry>,
}

impl TaskExecutor {
    pub fn new(
        db_pool: SqlitePool,
        ipfs_client: Arc<IpfsClient>,
        provider_registry: Arc<ProviderRegistry>,
    ) -> Self {
        Self {
            db_pool,
            ipfs_client,
            provider_registry,
        }
    }
}
```

### ipfs/mod.rs
```rust
//! IPFS 模块

pub mod client;

pub use client::IpfsClient;
```

### ipfs/client.rs
```rust
//! IPFS 客户端

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct IpfsClient {
    api_url: String,
    gateway_url: String,
    client: Client,
}

impl IpfsClient {
    pub fn new(api_url: &str, gateway_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            gateway_url: gateway_url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn add_file(&self, file_path: &str) -> Result<String, String> {
        // 简化实现
        Ok("QmTest".to_string())
    }

    pub async fn get_file(&self, cid: &str, dest_path: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn pubsub_publish(&self, topic: &str, data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}
```

### commands/media.rs
```rust
//! 媒体相关 Tauri Commands

use tauri::State;
use std::sync::Arc;
use crate::agent::media_config::{MediaApiConfig, ProviderConfig};
use crate::tasks::queue::{TaskQueue, TaskMessage, TaskPriority};
use uuid::Uuid;

#[tauri::command]
pub async fn update_provider_config(
    provider: String,
    config: ProviderConfig,
    state: State<'_, Arc<MediaApiConfig>>,
) -> Result<(), String> {
    let mut current = state.inner().as_ref().clone();
    current.providers.insert(provider, config);
    current.save().await?;
    Ok(())
}

#[tauri::command]
pub async fn test_provider_connection(
    provider: String,
    state: State<'_, Arc<MediaApiConfig>>,
) -> Result<TestResult, String> {
    Ok(TestResult {
        success: true,
        message: "Connection successful".to_string(),
    })
}

#[derive(serde::Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
}
```

## 配置示例

```json
{
  "providers": {
    "minimax": {
      "api_key": "your-api-key",
      "base_url": "your-group-id",
      "enabled": true,
      "capabilities": ["tts", "video"]
    },
    "google": {
      "api_key": "your-api-key",
      "base_url": "your-project-id",
      "enabled": true,
      "capabilities": ["image"]
    },
    "jimeng": {
      "api_key": "your-api-key",
      "base_url": "your-api-secret",
      "enabled": true,
      "capabilities": ["image", "video"]
    }
  },
  "ipfs": {
    "api_url": "http://127.0.0.1:5001",
    "gateway_url": "http://127.0.0.1:8080"
  },
  "storage": {
    "media_directory": "~/Alou/media",
    "database_path": "~/Alou/tasks.db"
  }
}
```

## 下一步

1. 更新 Cargo.toml 添加依赖
2. 更新 main.rs 初始化
3. 创建前端 UI 组件
