//! Alou Code Kernel Bridge
//!
//! This module provides the integration layer between alou-desktop and the embedded alou-code kernel.
//! It wraps the alou_code runtime, tools, and DIAP identity system for use by the desktop application.

pub mod kernel;
pub mod tool_adapter;
pub mod session_manager;
pub mod identity_handler;

pub use kernel::AlouCodeKernel;
pub use tool_adapter::ToolAdapter;
pub use session_manager::SessionManager;
pub use identity_handler::IdentityHandler;
