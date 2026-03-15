//! MiniMax 视频生成实现
//! 
//! 参考文档：https://platform.minimaxi.com/document
//! 
//! API 端点：
//! - 视频生成：POST https://api.minimax.chat/v1/video/generation
//! - 状态查询：GET https://api.minimax.chat/v1/video/status
//! 
//! 认证方式：Bearer Token (API Key)
//! 
//! 视频生成是异步任务：
//! 1. 调用生成接口，返回 task_id
//! 2. 轮询状态接口，直到状态为 completed
//! 3. 下载生成的视频

use super::MiniMaxConfig;
use crate::agent::providers::media_provider::{MediaTask, MediaOutput, MediaType, TaskStatus};
use crate::agent::error::{AgentError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 视频生成请求
#[derive(Debug, Serialize)]
pub struct VideoGenerationRequest {
    /// 模型名称
    #[serde(rename = "model")]
    pub model: String,
    
    /// 视频描述（prompt）
    #[serde(rename = "prompt")]
    pub prompt: String,
    
    /// 负面描述（可选）
    #[serde(rename = "negative_prompt", skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    
    /// 视频时长（秒），支持 5-10 秒
    #[serde(rename = "duration")]
    pub duration: u32,
    
    /// 分辨率：720p, 1080p
    #[serde(rename = "resolution", skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    
    /// 帧率：24, 25, 30
    #[serde(rename = "frame_rate", skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<u32>,
    
    /// 风格：realistic(写实), anime(动漫), cinematic(电影感)
    #[serde(rename = "style", skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

/// 视频生成响应
#[derive(Debug, Deserialize)]
pub struct VideoGenerationResponse {
    /// 错误码，0 表示成功
    #[serde(rename = "code")]
    pub code: u32,
    
    /// 错误信息
    #[serde(rename = "msg")]
    pub msg: String,
    
    /// 数据
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<VideoTaskData>,
    
    /// 请求 ID
    #[serde(rename = "request_id", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VideoTaskData {
    /// 任务 ID
    #[serde(rename = "task_id")]
    pub task_id: String,
    
    /// 任务状态：processing, completed, failed
    #[serde(rename = "status")]
    pub status: String,
    
    /// 视频 URL（完成后才有）
    #[serde(rename = "video_url", skip_serializing_if = "Option::is_none")]
    pub video_url: Option<String>,
    
    /// 进度百分比（0-100）
    #[serde(rename = "progress", skip_serializing_if = "Option::is_none")]
    pub progress: Option<u32>,
}

/// 状态查询响应
#[derive(Debug, Deserialize)]
pub struct VideoStatusResponse {
    #[serde(rename = "code")]
    pub code: u32,
    
    #[serde(rename = "msg")]
    pub msg: String,
    
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<VideoStatusData>,
}

#[derive(Debug, Deserialize)]
pub struct VideoStatusData {
    #[serde(rename = "task_id")]
    pub task_id: String,
    
    #[serde(rename = "status")]
    pub status: String,
    
    #[serde(rename = "video_url", skip_serializing_if = "Option::is_none")]
    pub video_url: Option<String>,
    
    #[serde(rename = "progress", skip_serializing_if = "Option::is_none")]
    pub progress: Option<u32>,
    
    #[serde(rename = "error_message", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    
    /// 视频时长（秒）
    #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
    
    /// 分辨率
    #[serde(rename = "resolution", skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

pub struct MiniMaxVideo {
    config: MiniMaxConfig,
    client: Client,
}

impl MiniMaxVideo {
    pub fn new(config: MiniMaxConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 视频生成可能需要较长时间
            .build()
            .unwrap_or_default();
        
        Self { config, client }
    }

    /// 生成视频（异步任务）
    /// 
    /// # 参数
    /// * `prompt` - 视频描述
    /// * `duration` - 视频时长（秒）
    /// 
    /// # 返回
    /// * `Ok(String)` - 任务 ID
    /// * `Err(AgentError)` - 错误信息
    pub async fn generate(
        &self,
        prompt: String,
        duration: u32,
    ) -> Result<String> {
        // 验证参数
        if duration < 5 || duration > 10 {
            return Err(AgentError::InvalidInput(
                "视频时长必须在 5-10 秒之间".to_string()
            ));
        }

        let request = VideoGenerationRequest {
            model: "video-01".to_string(),
            prompt,
            negative_prompt: None,
            duration,
            resolution: Some("720p".to_string()),
            frame_rate: Some(24),
            style: None,
        };

        let url = format!("{}/v1/video/generation", self.config.base_url);

        log::info!("[MiniMax Video] 生成视频，URL: {}", url);
        log::debug!("[MiniMax Video] 请求参数：{:?}", request);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("[MiniMax Video] 网络请求失败：{}", e);
                AgentError::ExternalApiError(format!("网络请求失败：{}", e))
            })?;

        let status = response.status();
        log::info!("[MiniMax Video] 响应状态码：{}", status);

        if !status.is_success() {
            let error = response.text().await.unwrap_or_default();
            log::error!("[MiniMax Video] API 错误：{}", error);
            return Err(AgentError::ExternalApiError(
                format!("MiniMax Video API 错误 ({}): {}", status, error)
            ));
        }

        let response_text = response.text().await
            .map_err(|e| {
                log::error!("[MiniMax Video] 读取响应失败：{}", e);
                AgentError::ExternalApiError(format!("读取响应失败：{}", e))
            })?;

        log::debug!("[MiniMax Video] 响应内容：{}", response_text);

        let generation_response: VideoGenerationResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("[MiniMax Video] 解析响应失败：{}, 响应内容：{}", e, response_text);
                AgentError::ExternalApiError(format!("解析响应失败：{}", e))
            })?;

        if generation_response.code != 0 {
            log::error!("[MiniMax Video] 业务错误：code={}, msg={}", 
                generation_response.code, generation_response.msg);
            return Err(AgentError::ExternalApiError(
                format!("MiniMax Video 错误 ({}): {}", generation_response.code, generation_response.msg)
            ));
        }

        let task_id = generation_response
            .data
            .ok_or_else(|| {
                log::error!("[MiniMax Video] 响应中没有任务数据");
                AgentError::ExternalApiError("响应中没有任务数据".to_string())
            })?
            .task_id;

        log::info!("[MiniMax Video] 任务创建成功，task_id: {}", task_id);

        Ok(task_id)
    }

    /// 查询视频生成状态
    /// 
    /// # 参数
    /// * `task_id` - 任务 ID
    /// 
    /// # 返回
    /// * `Ok(MediaTask)` - 任务状态和结果
    pub async fn get_status(&self, task_id: String) -> Result<MediaTask> {
        let url = format!("{}/v1/video/status?task_id={}", self.config.base_url, task_id);

        log::info!("[MiniMax Video] 查询状态，task_id: {}", task_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| {
                log::error!("[MiniMax Video] 网络请求失败：{}", e);
                AgentError::ExternalApiError(format!("网络请求失败：{}", e))
            })?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AgentError::ExternalApiError(
                format!("查询状态失败 ({}): {}", response.status(), error)
            ));
        }

        let status_response: VideoStatusResponse = response
            .json()
            .await
            .map_err(|e| {
                log::error!("[MiniMax Video] 解析响应失败：{}", e);
                AgentError::ExternalApiError(format!("解析响应失败：{}", e))
            })?;

        if status_response.code != 0 {
            return Err(AgentError::ExternalApiError(
                format!("查询状态错误 ({}): {}", status_response.code, status_response.msg)
            ));
        }

        let status_data = status_response
            .data
            .ok_or_else(|| {
                log::error!("[MiniMax Video] 响应中没有状态数据");
                AgentError::ExternalApiError("响应中没有状态数据".to_string())
            })?;

        // 转换状态
        let status = match status_data.status.as_str() {
            "completed" => TaskStatus::Completed,
            "failed" => TaskStatus::Failed,
            "processing" => TaskStatus::Processing,
            _ => TaskStatus::Pending,
        };

        log::info!("[MiniMax Video] 任务状态：{:?}, 进度：{:?}%", 
            status, status_data.progress);

        // 构建结果
        let result = if status == TaskStatus::Completed && status_data.video_url.is_some() {
            Some(MediaOutput {
                media_type: MediaType::Video,
                provider: "minimax".to_string(),
                url: status_data.video_url.clone(),
                file_path: None,
                ipfs_cid: None,
                metadata: crate::agent::providers::media_provider::MediaMetadata {
                    duration_secs: status_data.duration,
                    format: Some("mp4".to_string()),
                    prompt: None,
                    ..Default::default()
                },
            })
        } else {
            None
        };

        let error = if status == TaskStatus::Failed {
            status_data.error_message.clone().or(Some("视频生成失败".to_string()))
        } else {
            None
        };

        Ok(MediaTask {
            task_id,
            provider: "minimax".to_string(),
            media_type: MediaType::Video,
            status,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            result,
            error,
            progress: status_data.progress.unwrap_or(0) as f32,
        })
    }

    /// 轮询等待视频生成完成
    /// 
    /// # 参数
    /// * `task_id` - 任务 ID
    /// * `interval_secs` - 轮询间隔（秒）
    /// * `timeout_secs` - 超时时间（秒）
    /// 
    /// # 返回
    /// * `Ok(MediaTask)` - 完成的任务状态
    pub async fn wait_for_completion(
        &self,
        task_id: String,
        interval_secs: u64,
        timeout_secs: u64,
    ) -> Result<MediaTask> {
        let start_time = std::time::Instant::now();

        loop {
            // 检查超时
            if start_time.elapsed().as_secs() >= timeout_secs {
                log::error!("[MiniMax Video] 轮询超时，task_id: {}", task_id);
                return Err(AgentError::ExternalApiError(
                    format!("视频生成超时（{}秒）", timeout_secs)
                ));
            }

            // 查询状态
            let task = self.get_status(task_id.clone()).await?;

            // 检查是否完成
            match task.status {
                TaskStatus::Completed => {
                    log::info!("[MiniMax Video] 视频生成完成，task_id: {}", task_id);
                    return Ok(task);
                }
                TaskStatus::Failed => {
                    log::error!("[MiniMax Video] 视频生成失败，task_id: {}", task_id);
                    return Err(AgentError::ExternalApiError(
                        task.error.unwrap_or_else(|| "视频生成失败".to_string())
                    ));
                }
                _ => {
                    log::info!("[MiniMax Video] 视频生成中，进度：{}%", task.progress as u32);
                }
            }

            // 等待
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要真实 API Key，手动运行
    async fn test_generate_video() {
        let config = MiniMaxConfig {
            api_key: std::env::var("MINIMAX_API_KEY").unwrap_or_default(),
            group_id: std::env::var("MINIMAX_GROUP_ID").unwrap_or_default(),
            base_url: "https://api.minimax.chat".to_string(),
        };

        let video = MiniMaxVideo::new(config);
        
        // 创建任务
        let task_id = video.generate(
            "一只可爱的小猫在草地上玩耍".to_string(),
            5
        ).await.unwrap();
        
        println!("任务 ID: {}", task_id);

        // 轮询等待完成
        let task = video.wait_for_completion(task_id.clone(), 5, 300).await.unwrap();
        
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.result.is_some());
        
        println!("视频 URL: {:?}", task.result.unwrap().url);
    }
}
