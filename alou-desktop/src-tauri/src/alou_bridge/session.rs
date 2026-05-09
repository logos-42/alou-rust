use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{Value, json};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct UnifiedSession {
    pub id: String,
    pub diap_identity: Value,
    pub alou_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub state: SessionState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Initializing,
    Active,
    Paused,
    Terminated,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, UnifiedSession>>>,
}

impl UnifiedSession {
    pub fn new(diap_identity: Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            diap_identity,
            alou_session_id: None,
            created_at: Utc::now(),
            last_active: Utc::now(),
            state: SessionState::Initializing,
        }
    }

    pub async fn send_message(&self, message: String) -> Result<Value> {
        Ok(json!({
            "type": "message",
            "session_id": self.id,
            "content": message,
            "timestamp": Utc::now().to_rfc3339(),
        }))
    }

    pub async fn execute_tool(&self, tool_name: String, input: Value) -> Result<Value> {
        Ok(json!({
            "type": "tool_result",
            "session_id": self.id,
            "tool": tool_name,
            "input": input,
            "timestamp": Utc::now().to_rfc3339(),
        }))
    }

    pub fn mark_active(&mut self) {
        self.last_active = Utc::now();
    }

    pub fn set_alou_session_id(&mut self, alou_session_id: String) {
        self.alou_session_id = Some(alou_session_id);
        self.state = SessionState::Active;
    }

    pub fn pause(&mut self) {
        self.state = SessionState::Paused;
        self.last_active = Utc::now();
    }

    pub fn resume(&mut self) {
        self.state = SessionState::Active;
        self.last_active = Utc::now();
    }

    pub fn terminate(&mut self) {
        self.state = SessionState::Terminated;
        self.last_active = Utc::now();
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, diap_identity: Value) -> Result<String> {
        let session = UnifiedSession::new(diap_identity);
        let session_id = session.id.clone();
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<UnifiedSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn get_session_mut(&self, session_id: &str) -> Option<std::sync::RwLockWriteGuard<'_, HashMap<String, UnifiedSession>>> {
        None
    }

    pub async fn update_session(&self, session_id: &str, updater: impl FnOnce(&mut UnifiedSession)) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            updater(session);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    pub async fn delete_session(&self, session_id: &str) -> Option<UnifiedSession> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
    }

    pub async fn list_sessions(&self) -> Vec<UnifiedSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn cleanup_inactive(&self, max_age_secs: i64) -> usize {
        let now = Utc::now();
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();
        
        sessions.retain(|_, session| {
            let age = now.signed_duration_since(session.last_active);
            age.num_seconds() < max_age_secs || session.state != SessionState::Terminated
        });
        
        before - sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let manager = SessionManager::new();
        let identity = json!({"did": "example:123"});
        
        let session_id = manager.create_session(identity).await.unwrap();
        assert!(!session_id.is_empty());
        
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.id, session_id);
        assert_eq!(session.state, SessionState::Initializing);
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let manager = SessionManager::new();
        let identity = json!({"did": "example:456"});
        
        let session_id = manager.create_session(identity).await.unwrap();
        
        manager.update_session(&session_id, |s| {
            s.set_alou_session_id("alou-123".to_string());
        }).await.unwrap();
        
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.state, SessionState::Active);
        assert_eq!(session.alou_session_id, Some("alou-123".to_string()));
        
        manager.update_session(&session_id, |s| s.pause()).await.unwrap();
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.state, SessionState::Paused);
        
        manager.update_session(&session_id, |s| s.terminate()).await.unwrap();
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.state, SessionState::Terminated);
    }
}
