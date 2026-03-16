//! AI Client Pool - 复用 AI Client 实例
//!
//! 解决每次创建 Agent 时都 new AiClient 导致的性能问题

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::agent::ai_client::AiClient;
use crate::agent::config::UserApiConfig;
use crate::agent::error::Result;

/// AI Client Pool
pub struct AiClientPool {
    clients: RwLock<HashMap<String, Arc<AiClient>>>,
}

impl AiClientPool {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, config: &UserApiConfig) -> Result<Arc<AiClient>> {
        let key = self.make_cache_key(config);
        
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(&key) {
                log::debug!("AI Client 缓存命中：{}", key);
                return Ok(client.clone());
            }
        }

        log::info!("AI Client 缓存未命中，创建新实例：{}", key);
        let client = Arc::new(AiClient::new(config)?);

        {
            let mut clients = self.clients.write().await;
            clients.insert(key, client.clone());
        }

        Ok(client)
    }

    fn make_cache_key(&self, config: &UserApiConfig) -> String {
        let model = config.model.as_deref().unwrap_or("default");
        format!("{}::{}", config.provider, model)
    }

    pub async fn clear(&self) {
        let mut clients = self.clients.write().await;
        let count = clients.len();
        clients.clear();
        log::info!("AI Client Pool 已清空，移除 {} 个实例", count);
    }

    pub async fn stats(&self) -> PoolStats {
        let clients = self.clients.read().await;
        PoolStats {
            cached_count: clients.len(),
        }
    }
}

impl Default for AiClientPool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub cached_count: usize,
}
