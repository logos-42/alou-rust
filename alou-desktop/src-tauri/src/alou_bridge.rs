use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{Value, json};
use anyhow::Result;

pub mod session;
pub use session::{UnifiedSession, SessionManager};

#[derive(Debug, Clone)]
pub struct ToolExecutor {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AlouBridge {
    session_manager: Arc<SessionManager>,
    tool_executors: Arc<RwLock<HashMap<String, ToolExecutor>>>,
}

impl AlouBridge {
    pub fn new() -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new()),
            tool_executors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, diap_identity: Value) -> Result<String> {
        self.session_manager.create_session(diap_identity).await
    }

    pub async fn get_session(&self, session_id: &str) -> Option<UnifiedSession> {
        self.session_manager.get_session(session_id).await
    }

    pub async fn send_message(&self, session_id: &str, message: String) -> Result<Value> {
        let session = self.session_manager.get_session(session_id).await
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session.send_message(message).await
    }

    pub async fn execute_tool(&self, session_id: &str, tool_name: String, input: Value) -> Result<Value> {
        let session = self.session_manager.get_session(session_id).await
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session.execute_tool(tool_name, input).await
    }

    pub async fn register_tool_executor(&self, name: String, executor: ToolExecutor) {
        let mut executors = self.tool_executors.write().await;
        executors.insert(name, executor);
    }

    pub async fn list_tool_executors(&self) -> Vec<String> {
        let executors = self.tool_executors.read().await;
        executors.keys().cloned().collect()
    }
}

impl Default for AlouBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_creation() {
        let bridge = AlouBridge::new();
        assert!(bridge.session_manager.get_session("test").await.is_none());
    }
}
