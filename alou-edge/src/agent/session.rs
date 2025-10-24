use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::storage::kv::KvStore;
use crate::utils::error::{AloudError, Result};

const SESSION_TTL_SECONDS: u64 = 24 * 60 * 60; // 24 hours
const MAX_MESSAGES_PER_SESSION: usize = 50;

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,      // "user", "assistant", or "tool"
    pub content: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn new(role: String, content: String) -> Self {
        Self {
            role,
            content,
            timestamp: Utc::now().timestamp(),
            tool_call_id: None,
        }
    }
    
    pub fn with_tool_call_id(mut self, tool_call_id: String) -> Self {
        self.tool_call_id = Some(tool_call_id);
        self
    }
}

/// Session containing conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    pub messages: Vec<Message>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Session {
    pub fn new(session_id: String, wallet_address: Option<String>) -> Self {
        let now = Utc::now().timestamp();
        Self {
            session_id,
            wallet_address,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        // Limit messages to MAX_MESSAGES_PER_SESSION
        if self.messages.len() >= MAX_MESSAGES_PER_SESSION {
            // Remove oldest message
            self.messages.remove(0);
        }
        
        self.messages.push(message);
        self.updated_at = Utc::now().timestamp();
    }
}

/// Session manager for handling conversation sessions
pub struct SessionManager {
    kv: KvStore,
}

impl SessionManager {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }
    
    /// Create a new session
    pub async fn create_session(&self, wallet_address: Option<String>) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let session = Session::new(session_id.clone(), wallet_address);
        
        let key = Self::session_key(&session_id);
        self.kv.put(&key, &session, Some(SESSION_TTL_SECONDS)).await?;
        
        Ok(session_id)
    }
    
    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        let key = Self::session_key(session_id);
        
        match self.kv.get::<Session>(&key).await? {
            Some(session) => Ok(session),
            None => Err(AloudError::InvalidInput(format!("Session not found: {}", session_id))),
        }
    }
    
    /// Add a message to a session
    pub async fn add_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<()> {
        let mut session = self.get_session(session_id).await?;
        
        let message = Message::new(role.to_string(), content.to_string());
        session.add_message(message);
        
        let key = Self::session_key(session_id);
        self.kv.put(&key, &session, Some(SESSION_TTL_SECONDS)).await?;
        
        Ok(())
    }
    
    /// Add a message with tool call ID to a session
    pub async fn add_message_with_tool_call(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        tool_call_id: &str,
    ) -> Result<()> {
        let mut session = self.get_session(session_id).await?;
        
        let message = Message::new(role.to_string(), content.to_string())
            .with_tool_call_id(tool_call_id.to_string());
        session.add_message(message);
        
        let key = Self::session_key(session_id);
        self.kv.put(&key, &session, Some(SESSION_TTL_SECONDS)).await?;
        
        Ok(())
    }
    
    /// Get conversation history for a session
    pub async fn get_history(&self, session_id: &str) -> Result<Vec<Message>> {
        let session = self.get_session(session_id).await?;
        Ok(session.messages)
    }
    
    /// Clear a session
    pub async fn clear_session(&self, session_id: &str) -> Result<()> {
        let key = Self::session_key(session_id);
        self.kv.delete(&key).await?;
        Ok(())
    }
    
    /// Generate KV key for a session
    fn session_key(session_id: &str) -> String {
        format!("session:{}", session_id)
    }
}
