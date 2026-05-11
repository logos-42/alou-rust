//! Session Manager
//!
//! Manages sessions using alou_code's session store.

use alou_code_runtime::{Session, SessionStore, SessionControlError};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct SessionManager {
    store: SessionStore,
    active_sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionManager {
    pub fn new() -> Result<Self, String> {
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let store = SessionStore::from_cwd(&cwd)
            .map_err(|e| format!("Failed to create session store: {}", e))?;

        Ok(Self {
            store,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn create_session(&self, prompt: String) -> Result<Session, String> {
        let session = Session::new(prompt)
            .map_err(|e| format!("Failed to create session: {}", e))?;

        let session_id = session.id().to_string();
        self.active_sessions
            .write()
            .await
            .insert(session_id, session.clone());

        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>, String> {
        if let Some(session) = self.active_sessions.read().await.get(session_id) {
            return Ok(Some(session.clone()));
        }

        let loaded = self.store.load_session(session_id)
            .map_err(|e| format!("Failed to load session: {}", e))?;
        Ok(Some(loaded.session))
    }

    pub async fn list_sessions(&self) -> Result<Vec<String>, String> {
        let sessions = self.store.list_sessions()
            .map_err(|e| format!("Failed to list sessions: {}", e))?;
        Ok(sessions.into_iter().map(|s| s.id).collect())
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), String> {
        self.active_sessions
            .write()
            .await
            .remove(session_id);
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SessionManager")
    }
}
