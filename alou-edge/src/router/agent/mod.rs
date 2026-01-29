//! Agent Router Module
//! 
//! This module serves as main entry point for agent-related operations.
//! It re-exports handlers from specialized sub-modules:
//! - agent_discovery: Agent discovery and resolution
//! - agent_creation: Agent creation operations
//! - agent_batch: Batch operations

// Sub-modules
mod agent_discovery;
mod agent_creation;
mod agent_batch;
pub mod agent_diap;

// Re-export public types and handlers
pub(crate) use agent_creation::{
    CreateAgentRequest,
    CreateAgentResult,
    McpPortConfig,
    ProvidedDiapIdentity,
};

pub(crate) use agent_discovery::{
    ResolveAgentRequest,
    SearchAgentRequest,
};

// Re-export discovery handlers
pub(crate) use agent_discovery::{
    handle_resolve_agent,
    handle_search_agents,
};

// Re-export creation handlers
pub(crate) use agent_creation::{
    handle_create_agent,
    handle_create_agent_internal,
    handle_parse_creation_command,
    handle_create_agent_from_command,
};

// Re-export batch handlers
pub(crate) use agent_batch::{
    handle_batch_create_agent,
    handle_get_batch_create_task,
    handle_get_batch_agent_sessions,
};
