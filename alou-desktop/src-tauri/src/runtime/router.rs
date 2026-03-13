//! SessionRouter - Session Actor 路由管理
//!
//! 使用 DashMap 实现无锁的 Session 查找和创建

use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::runtime::message::SessionMessage;
use crate::runtime::actor::SessionActor;
use crate::runtime::handle::ActorHandle;
use crate::bridges::BridgeManager;

/// Session 路由管理器
/// 管理所有 Session Actor 的生命周期
pub struct SessionRouter {
    /// Session Actor 映射表
    /// 使用 DashMap 实现无锁并发访问
    actors: DashMap<String, ActorHandle>,
    /// 资源池引用
    bridge_manager: Arc<BridgeManager>,
    /// 配置
    config: RouterConfig,
}

/// 路由配置
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// 最大活跃 Session 数量
    pub max_sessions: usize,
    /// Session 空闲超时时间
    pub idle_timeout: Duration,
    /// 是否启用自动清理
    pub enable_cleanup: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_sessions: 1000,           // 最多 1000 个活跃 Session
            idle_timeout: Duration::from_secs(30 * 60),  // 30 分钟空闲超时
            enable_cleanup: true,
        }
    }
}

impl SessionRouter {
    /// 创建新的 SessionRouter
    pub fn new(bridge_manager: Arc<BridgeManager>) -> Self {
        Self {
            actors: DashMap::new(),
            bridge_manager,
            config: RouterConfig::default(),
        }
    }
    
    /// 创建带配置的 SessionRouter
    pub fn with_config(bridge_manager: Arc<BridgeManager>, config: RouterConfig) -> Self {
        Self {
            actors: DashMap::new(),
            bridge_manager,
            config,
        }
    }
    
    /// 获取或创建 Session Actor
    /// 如果 Session 已存在，返回现有 Actor
    /// 如果不存在，创建新的 Actor
    pub fn get_or_create(&self, session_id: &str) -> ActorHandle {
        // DashMap::entry 是无锁操作
        self.actors
            .entry(session_id.to_string())
            .or_insert_with(|| {
                log::info!("Creating new SessionActor for: {}", session_id);
                SessionActor::spawn(
                    session_id.to_string(),
                    self.bridge_manager.clone(),
                )
            })
            .clone()
    }
    
    /// 发送消息到指定 Session
    pub fn send(&self, session_id: &str, msg: SessionMessage) {
        let handle = self.get_or_create(session_id);
        handle.send(msg);
    }

    /// 移除 Session
    pub fn remove(&self, session_id: &str) -> Option<ActorHandle> {
        self.actors.remove(session_id).map(|(_, handle)| {
            // 发送销毁消息
            handle.send(SessionMessage::SessionDestroy);
            handle
        })
    }
    
    /// 获取活跃 Session 数量
    pub fn active_count(&self) -> usize {
        self.actors.len()
    }
    
    /// 检查 Session 是否存在
    pub fn exists(&self, session_id: &str) -> bool {
        self.actors.contains_key(session_id)
    }
    
    /// 获取所有 Session ID
    pub fn session_ids(&self) -> Vec<String> {
        self.actors.iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// 清理空闲 Session（可选）
    pub async fn cleanup_idle(&self) -> usize {
        if !self.config.enable_cleanup {
            return 0;
        }
        
        let mut removed = 0;
        
        // 遍历所有 Actor（注意：DashMap 遍历是安全的）
        let to_remove: Vec<String> = self.actors
            .iter()
            .filter(|entry| {
                // TODO: 需要记录最后活跃时间
                // 这里简化处理，实际需要从 ActorHandle 获取
                false
            })
            .map(|entry| entry.key().clone())
            .collect();
        
        for session_id in to_remove {
            self.remove(&session_id);
            removed += 1;
        }
        
        removed
    }
    
    /// 优雅关闭所有 Session
    pub async fn shutdown(&self) {
        log::info!("Shutting down SessionRouter with {} sessions", self.actors.len());
        
        // 发送销毁消息到所有 Session
        for entry in self.actors.iter() {
            entry.value().send(SessionMessage::SessionDestroy);
        }
        
        // 等待一段时间让所有 Actor 处理销毁消息
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // 强制移除所有 Actor
        self.actors.clear();
    }
}

/// 全局 SessionRouter 管理器
pub struct SessionRouterManager {
    /// 全局 Router 实例
    router: Arc<tokio::sync::OnceCell<SessionRouter>>,
    /// 资源池
    bridge_manager: Arc<BridgeManager>,
}

impl SessionRouterManager {
    /// 创建新的 Manager
    pub fn new(bridge_manager: Arc<BridgeManager>) -> Self {
        Self {
            router: Arc::new(tokio::sync::OnceCell::new()),
            bridge_manager,
        }
    }
    
    /// 获取 Router 实例（延迟初始化）
    pub async fn get_router(&self) -> &SessionRouter {
        self.router
            .get_or_init(|| async {
                SessionRouter::new(self.bridge_manager.clone())
            })
            .await
    }
    
    /// 获取或创建 Session
    pub async fn get_or_create_session(&self, session_id: &str) -> ActorHandle {
        let router = self.get_router().await;
        router.get_or_create(session_id)
    }
    
    /// 发送消息到 Session
    pub async fn send_message(&self, session_id: &str, msg: SessionMessage) {
        let router = self.get_router().await;
        router.send(session_id, msg);
    }
}

// ========== 便捷函数 ==========

/// 全局 SessionRouter 管理器（单例）
/// 实际使用时应通过 AppState 注入
static ROUTER_MANAGER: std::sync::OnceLock<SessionRouterManager> = std::sync::OnceLock::new();

/// 初始化全局 SessionRouter
pub fn init_session_router(bridge_manager: Arc<BridgeManager>) {
    let _ = ROUTER_MANAGER.set(SessionRouterManager::new(bridge_manager));
}

/// 获取全局 SessionRouter
pub fn get_session_router() -> Option<&'static SessionRouterManager> {
    ROUTER_MANAGER.get()
}
