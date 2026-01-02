use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::storage::kv::KvStore;

const MAX_EVENTS_PER_SESSION: usize = 200;

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
            timestamp: crate::utils::time::now_timestamp_millis_i64(),
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

    pub async fn publish(&self, event: StreamEvent, kv: &KvStore) {
        let mut normalized = event;
        normalized.session_id = self.session_id.clone();
        append_event(normalized, kv).await;
    }
}

pub async fn append_event(event: StreamEvent, kv: &KvStore) {
    let key = format!("stream_events:{}", event.session_id);

    let mut events: Vec<StreamEvent> = match kv.get(&key).await {
        Ok(Some(value)) => value,
        _ => Vec::new(),
    };

    events.push(event);

    if events.len() > MAX_EVENTS_PER_SESSION {
        let overflow = events.len() - MAX_EVENTS_PER_SESSION;
        events.drain(0..overflow);
    }

    let _ = kv.put(&key, &events, Some(3600)).await;
}

pub async fn get_events(session_id: &str, since: Option<i64>, kv: &KvStore) -> Vec<StreamEvent> {
    let key = format!("stream_events:{}", session_id);

    let events: Vec<StreamEvent> = match kv.get(&key).await {
        Ok(Some(value)) => value,
        _ => Vec::new(),
    };

    match since {
        Some(ts) => events
            .iter()
            .filter(|event| event.timestamp > ts)
            .cloned()
            .collect(),
        None => events,
    }
}

pub async fn cleanup_session(session_id: &str, kv: &KvStore) {
    let key = format!("stream_events:{}", session_id);
    let _ = kv.delete(&key).await;
}
