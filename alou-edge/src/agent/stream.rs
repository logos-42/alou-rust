use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::RwLock;

const MAX_EVENTS_PER_SESSION: usize = 200;

static EVENT_STORE: Lazy<RwLock<HashMap<String, Vec<StreamEvent>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub session_id: String,
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    #[serde(default)]
    pub payload: Value,
    pub timestamp: i64,
    #[serde(default)]
    pub is_final: bool,
}

impl StreamEvent {
    pub fn new<E: Into<String>>(session_id: &str, event: E) -> Self {
        Self {
            session_id: session_id.to_string(),
            event: event.into(),
            label: None,
            step_id: None,
            payload: Value::Null,
            timestamp: crate::utils::time::now_timestamp_millis(),
            is_final: false,
        }
    }

    pub fn with_label<L: Into<String>>(mut self, label: L) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    pub fn mark_final(mut self) -> Self {
        self.is_final = true;
        self
    }
}

#[derive(Clone)]
pub struct StreamPublisher {
    session_id: String,
}

impl StreamPublisher {
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
        }
    }

    pub async fn publish(&self, event: StreamEvent) {
        let mut normalized = event;
        normalized.session_id = self.session_id.clone();
        append_event(normalized).await;
    }
}

pub async fn append_event(event: StreamEvent) {
    let mut store = EVENT_STORE.write().await;
    let entry = store
        .entry(event.session_id.clone())
        .or_insert_with(Vec::new);
    entry.push(event);
    if entry.len() > MAX_EVENTS_PER_SESSION {
        let overflow = entry.len() - MAX_EVENTS_PER_SESSION;
        entry.drain(0..overflow);
    }
}

pub async fn get_events(session_id: &str, since: Option<i64>) -> Vec<StreamEvent> {
    let store = EVENT_STORE.read().await;
    match store.get(session_id) {
        Some(events) => match since {
            Some(ts) => events
                .iter()
                .filter(|event| event.timestamp > ts)
                .cloned()
                .collect(),
            None => events.clone(),
        },
        None => Vec::new(),
    }
}

pub async fn cleanup_session(session_id: &str) {
    let mut store = EVENT_STORE.write().await;
    store.remove(session_id);
}
