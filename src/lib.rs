//! # Alou - 智能自动化工作流系统
//!
//! 一个基于Rust和Model Context Protocol (MCP)的智能自动化工作流系统，集成了DeepSeek API，
//! 提供强大的上下文感知和工具调用能力。
//!
//! 这个库提供了完整的MCP协议实现，支持AI模型与其运行环境之间的通信。
//! 支持客户端和服务器实现，通过stdio传输层进行通信。
//!
//! 项目地址: https://github.com/your-username/alou-rust
//!
//! ## Features
//!
//! - Full implementation of MCP protocol specification
//! - Stdio transport layer
//! - Async/await support using Tokio
//! - Type-safe message handling
//! - Comprehensive error handling
//!
//! ## Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use mcp_client_rs::client::Client;
//! use mcp_client_rs::transport::stdio::StdioTransport;
//! use tokio::io::{stdin, stdout};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a Stdio transport using standard input/output
//!     let transport = StdioTransport::with_streams(stdin(), stdout())?;
//!     
//!     // Create the client with Arc-wrapped transport
//!     let client = Client::new(Arc::new(transport));
//!     
//!     // Use the client...
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Usage
//! The client can be used to send requests and notifications to an MCP-compliant server.
//! See the [client](crate::client) module for details on initialization and tool usage.

/// Agent module provides the intelligent agent implementation
pub mod agent;
/// Client module provides the MCP client implementation
pub mod client;
/// Connection pool for managing multiple MCP connections
pub mod connection_pool;
/// Error types and handling for the SDK
pub mod error;
/// Protocol-specific types and implementations
pub mod protocol;
/// Prompt registry for managing MCP prompts
pub mod prompt_registry;
/// System prompts and templates
pub mod prompts;
/// Server module provides the MCP server implementation
pub mod server;
/// Transport layer implementations (stdio)
pub mod transport;
/// Common types used throughout the SDK
pub mod types;
/// Workspace context for managing project directories
pub mod workspace_context;

// Re-export commonly used types for convenience
pub use error::Error;
pub use protocol::{Notification, Request, Response};
pub use types::*;

/// The latest supported protocol version of MCP
///
/// This version represents the most recent protocol specification that this SDK supports.
/// It is used during client-server handshake to ensure compatibility.
pub const LATEST_PROTOCOL_VERSION: &str = "2024-11-05";

/// List of all protocol versions supported by this SDK
///
/// This list is used during version negotiation to determine compatibility between
/// client and server. The versions are listed in order of preference, with the
/// most recent version first.
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &[LATEST_PROTOCOL_VERSION, "2024-10-07"];

/// JSON-RPC version used by the MCP protocol
///
/// MCP uses JSON-RPC 2.0 for its message format. This constant is used to ensure
/// all messages conform to the correct specification.
pub const JSONRPC_VERSION: &str = "2.0";

// Simple example function to demonstrate library usage in tests
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
