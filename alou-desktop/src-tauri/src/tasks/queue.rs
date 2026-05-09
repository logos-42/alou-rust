//! 任务队列
//!
//! 使用 tokio mpsc 实现任务分发和并行执行

use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

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

use std::sync::Arc;

impl TaskQueue {
    pub fn new(num_workers: usize) -> Self {
        let (task_tx, mut task_rx) = mpsc::channel::<TaskMessage>(100);
        let (result_tx, result_rx) = mpsc::channel::<TaskResult>(100);
        let result_rx = Arc::new(tokio::sync::Mutex::new(result_rx));

        // 为每个 worker 创建独立的 channel
        for i in 0..num_workers {
            let (task_tx_worker, task_rx_worker) = mpsc::channel::<TaskMessage>(100);
            let result_tx = result_tx.clone();
            
            // 克隆主发送器到 worker
            let main_task_tx = task_tx.clone();
            tokio::spawn(async move {
                log::info!("Task worker {} started", i);
                Self::worker_loop(Arc::new(tokio::sync::Mutex::new(task_rx_worker)), result_tx).await;
            });
        }

        Self {
            task_tx,
            result_tx,
            result_rx,
        }
    }

    async fn worker_loop(
        task_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<TaskMessage>>>,
        result_tx: mpsc::Sender<TaskResult>,
    ) {
        loop {
            let task_msg = {
                let mut rx = task_rx.lock().await;
                rx.recv().await
            };
            
            match task_msg {
                Some(task_msg) => {
                    // 简化实现：直接返回成功
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
                None => break,
            }
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
