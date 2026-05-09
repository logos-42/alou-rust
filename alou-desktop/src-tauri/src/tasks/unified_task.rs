//! 统一任务系统
//!
//! 推理和媒体生成都在一个任务模型里处理

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskType {
    /// LLM 文本推理
    LlmInference {
        provider: String,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    },
    
    /// 图片生成
    ImageGeneration {
        provider: String,
        prompt: String,
        negative_prompt: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        aspect_ratio: Option<String>,
    },
    
    /// 音频生成 (TTS)
    AudioGeneration {
        provider: String,
        text: String,
        voice_id: Option<String>,
        model: Option<String>,
    },
    
    /// 视频生成
    VideoGeneration {
        provider: String,
        prompt: Option<String>,
        image_cid: Option<String>,
        duration_secs: Option<u32>,
        resolution: Option<String>,
    },
    
    /// 上传到 IPFS
    UploadToIpfs {
        file_path: String,
        media_type: String,
    },
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 任务记录
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub session_id: String,
    pub agent_id: Option<String>,
    pub task_type: String,
    pub status: String,
    pub input: String,
    pub output: Option<String>,
    pub ipfs_cid: Option<String>,
    pub error: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub ipfs_cid: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}
