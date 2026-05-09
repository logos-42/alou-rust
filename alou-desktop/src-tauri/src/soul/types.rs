/**
 * types.rs - Soul Module Types
 *
 * Type definitions for personality/identity management.
 */

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Personality trait structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTrait {
    pub name: String,
    pub description: String,
    pub strength: u8, // 1-10
}

/// Behavioral principle structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principle {
    pub name: String,
    pub description: String,
}

/// Preference structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub category: String,
    pub key: String,
    pub value: String,
}

/// Growth log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthLog {
    pub timestamp: DateTime<Local>,
    pub event: String,
    pub impact: String,
}

/// Soul profile structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulProfile {
    pub name: String,
    pub role: String,
    pub created_at: DateTime<Local>,
    pub traits: Vec<PersonalityTrait>,
    pub principles: Vec<Principle>,
    pub preferences: Vec<Preference>,
    pub growth_logs: Vec<GrowthLog>,
}
