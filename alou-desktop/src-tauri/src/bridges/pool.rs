//! 资源池管理
//!
//! 实现无状态资源池 + RAII 借用模式

use std::sync::Arc;
use tokio::sync::{Semaphore, OwnedSemaphorePermit};

/// LLM 连接池
pub struct LlmPool {
    semaphore: Arc<Semaphore>,
    config: LlmPoolConfig,
}

/// LLM 池配置
#[derive(Debug, Clone)]
pub struct LlmPoolConfig {
    pub max_concurrent: usize,
    pub timeout_secs: u64,
}

impl Default for LlmPoolConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            timeout_secs: 60,
        }
    }
}

impl LlmPool {
    pub fn new(config: LlmPoolConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
        }
    }
    
    /// 借用 LLM 连接（RAII 模式）
    pub async fn borrow(&self) -> Result<LlmHandle, String> {
        let permit = self.semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Failed to acquire permit: {}", e))?;
        
        Ok(LlmHandle {
            _permit: permit,
            config: self.config.clone(),
        })
    }
    
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// LLM 借用句柄（RAII）
pub struct LlmHandle {
    _permit: OwnedSemaphorePermit,
    config: LlmPoolConfig,
}

impl LlmHandle {
    pub fn timeout_secs(&self) -> u64 {
        self.config.timeout_secs
    }
}

/// 工具执行池
pub struct ToolPool {
    semaphore: Arc<Semaphore>,
    config: ToolPoolConfig,
}

/// 工具池配置
#[derive(Debug, Clone)]
pub struct ToolPoolConfig {
    pub max_concurrent: usize,
}

impl Default for ToolPoolConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 20,
        }
    }
}

impl ToolPool {
    pub fn new(config: ToolPoolConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
        }
    }
    
    pub async fn borrow(&self) -> Result<ToolHandle, String> {
        let permit = self.semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Failed to acquire permit: {}", e))?;
        
        Ok(ToolHandle {
            _permit: permit,
        })
    }
}

pub struct ToolHandle {
    _permit: OwnedSemaphorePermit,
}

/// 浏览器实例池
pub struct BrowserPool {
    semaphore: Arc<Semaphore>,
    config: BrowserPoolConfig,
}

/// 浏览器池配置
#[derive(Debug, Clone)]
pub struct BrowserPoolConfig {
    pub max_concurrent: usize,
    pub headless: bool,
}

impl Default for BrowserPoolConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            headless: true,
        }
    }
}

impl BrowserPool {
    pub fn new(config: BrowserPoolConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
        }
    }
    
    pub async fn borrow(&self) -> Result<BrowserHandle, String> {
        let permit = self.semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Failed to acquire permit: {}", e))?;
        
        Ok(BrowserHandle {
            _permit: permit,
            config: self.config.clone(),
        })
    }
}

pub struct BrowserHandle {
    _permit: OwnedSemaphorePermit,
    config: BrowserPoolConfig,
}

impl BrowserHandle {
    pub fn is_headless(&self) -> bool {
        self.config.headless
    }
}
