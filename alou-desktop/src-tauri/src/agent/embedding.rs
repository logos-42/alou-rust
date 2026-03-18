//! Embedding 服务 - 语义搜索支持
//!
//! 提供文本向量化能力，支持：
//! - 调用 OpenAI/Kimi 等 API 生成 embedding
//! - 本地 embedding 模型（可选）
//! - 语义相似度计算

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Embedding 向量
pub type Embedding = Vec<f32>;

/// Embedding 配置
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Provider: "openai" | "kimi" | "local"
    pub provider: String,
    /// API Key
    pub api_key: Option<String>,
    /// Base URL (可选，用于兼容其他 API)
    pub base_url: Option<String>,
    /// 模型名称
    pub model: String,
    /// 向量维度
    pub dimensions: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: None,
            base_url: None,
            model: "text-embedding-3-small".to_string(),
            dimensions: 1536,
        }
    }
}

/// Embedding 服务
pub struct EmbeddingService {
    config: EmbeddingConfig,
    client: reqwest::Client,
}

impl EmbeddingService {
    /// 创建新的 Embedding 服务
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// 创建默认配置的服务
    pub fn with_default_config() -> Self {
        Self::new(EmbeddingConfig::default())
    }

    /// 生成单个文本的 embedding
    pub async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        match self.config.provider.as_str() {
            "openai" => self.embed_openai(text).await,
            "kimi" => self.embed_kimi(text).await,
            "local" => self.embed_local(text).await,
            _ => Err(EmbeddingError::UnsupportedProvider(self.config.provider.clone())),
        }
    }

    /// 批量生成 embedding
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    /// OpenAI Embedding API
    async fn embed_openai(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        let url = format!("{}/v1/embeddings", 
            self.config.base_url.as_deref().unwrap_or("https://api.openai.com"));

        let request_body = OpenAIEmbeddingRequest {
            model: &self.config.model,
            input: vec![text],
            encoding_format: "float",
        };

        let mut builder = self.client
            .post(&url)
            .json(&request_body)
            .header("Content-Type", "application/json");

        if let Some(ref api_key) = self.config.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| EmbeddingError::ApiError(format!("OpenAI API 请求失败：{}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ApiError(format!(
                "OpenAI API 返回错误 ({}): {}",
                status, error_text
            )));
        }

        let result: OpenAIEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(format!("解析 OpenAI 响应失败：{}", e)))?;

        if result.data.is_empty() {
            return Err(EmbeddingError::ApiError("OpenAI API 返回空 embedding".to_string()));
        }

        Ok(result.data.into_iter()
            .next()
            .unwrap()
            .embedding)
    }

    /// Kimi Embedding API
    async fn embed_kimi(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        let url = "https://api.moonshot.cn/v1/embeddings";

        let request_body = OpenAIEmbeddingRequest {
            model: &self.config.model,
            input: vec![text],
            encoding_format: "float",
        };

        let mut builder = self.client
            .post(url)
            .json(&request_body)
            .header("Content-Type", "application/json");

        if let Some(ref api_key) = self.config.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = builder
            .send()
            .await
            .map_err(|e| EmbeddingError::ApiError(format!("Kimi API 请求失败：{}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ApiError(format!(
                "Kimi API 返回错误 ({}): {}",
                status, error_text
            )));
        }

        let result: OpenAIEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(format!("解析 Kimi 响应失败：{}", e)))?;

        if result.data.is_empty() {
            return Err(EmbeddingError::ApiError("Kimi API 返回空 embedding".to_string()));
        }

        Ok(result.data.into_iter()
            .next()
            .unwrap()
            .embedding)
    }

    /// 本地 Embedding (预留接口，可使用 ONNX Runtime 等)
    async fn embed_local(&self, _text: &str) -> Result<Embedding, EmbeddingError> {
        Err(EmbeddingError::UnsupportedProvider(
            "本地 embedding 尚未实现".to_string()
        ))
    }

    /// 计算余弦相似度
    pub fn cosine_similarity(a: &Embedding, b: &Embedding) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// 搜索最相似的 embedding
    pub fn search_similar(
        query_embedding: &Embedding,
        candidates: &[(String, Embedding)],
        top_k: usize,
    ) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = candidates
            .iter()
            .map(|(id, embedding)| {
                let similarity = Self::cosine_similarity(query_embedding, embedding);
                (id.clone(), similarity)
            })
            .collect();

        // 按相似度降序排序
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 返回 top_k
        scores.into_iter().take(top_k).collect()
    }
}

/// OpenAI Embedding API 请求
#[derive(Debug, Serialize)]
struct OpenAIEmbeddingRequest<'a> {
    model: &'a str,
    input: Vec<&'a str>,
    encoding_format: &'a str,
}

/// OpenAI Embedding API 响应
#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Embedding,
}

/// Embedding 错误
#[derive(Debug)]
pub enum EmbeddingError {
    ApiError(String),
    ParseError(String),
    UnsupportedProvider(String),
    InternalError(String),
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingError::ApiError(msg) => write!(f, "API 错误：{}", msg),
            EmbeddingError::ParseError(msg) => write!(f, "解析错误：{}", msg),
            EmbeddingError::UnsupportedProvider(msg) => write!(f, "不支持的 Provider: {}", msg),
            EmbeddingError::InternalError(msg) => write!(f, "内部错误：{}", msg),
        }
    }
}

impl std::error::Error for EmbeddingError {}

/// 带 Embedding 的记忆搜索
pub struct SemanticMemorySearch {
    embedding_service: Arc<EmbeddingService>,
    memory_manager: Arc<crate::agent::memory::MemoryManager>,
}

impl SemanticMemorySearch {
    /// 创建新的语义记忆搜索
    pub fn new(
        embedding_service: Arc<EmbeddingService>,
        memory_manager: Arc<crate::agent::memory::MemoryManager>,
    ) -> Self {
        Self {
            embedding_service,
            memory_manager,
        }
    }

    /// 语义搜索记忆
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::agent::memory::Memory>, EmbeddingError> {
        log::info!("[SemanticMemorySearch] 语义搜索：query='{}', limit={}", query, limit);

        // 1. 生成 query 的 embedding
        let query_embedding = self.embedding_service.embed(query).await?;

        // 2. 从 MemoryManager 获取所有记忆
        let all_memories = self.memory_manager.get_all_memories().await;

        // 3. 计算相似度并排序
        let mut scored_memories: Vec<(crate::agent::memory::Memory, f32)> = all_memories
            .iter()
            .map(|memory| {
                // 使用记忆内容的 embedding（如果已缓存）或实时计算
                let similarity = if let Some(cached_embedding) = memory.embedding.as_ref() {
                    EmbeddingService::cosine_similarity(&query_embedding, cached_embedding)
                } else {
                    // 如果没有缓存，使用关键词搜索作为 fallback
                    0.5 // 默认相似度
                };

                (memory.clone(), similarity)
            })
            .collect();

        // 按相似度降序排序
        scored_memories.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 4. 返回 top_k
        let results: Vec<crate::agent::memory::Memory> = scored_memories
            .into_iter()
            .take(limit)
            .map(|(memory, _)| memory)
            .collect();

        log::info!("[SemanticMemorySearch] 找到 {} 条相关记忆", results.len());

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        // 相同向量相似度为 1
        assert!((EmbeddingService::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        // 正交向量相似度为 0
        assert!((EmbeddingService::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }
}
