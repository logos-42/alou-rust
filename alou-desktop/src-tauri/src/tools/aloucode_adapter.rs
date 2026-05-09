//! AlouCode Tools Adapter
//!
//! This module provides an adapter that exposes alou-code's 40+ built-in tools
//! to the desktop tool registry. This allows the desktop to use tools from
//! both the native tool system and alou-code's tool ecosystem.
//!
//! Alou-code tools include:
//! - bash, read_file, write_file, edit_file
//! - glob_search, grep_search
//! - WebSearch, WebFetch
//! - Agent, Skill, TodoWrite
//! - Polymarket tools (list_markets, get_orderbook, place_order, etc.)
//! - LSP tools
//! - Task/Worker tools
//! - And more...

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tools::{ToolCategory, ToolStatus, ToolMetadata, ToolPriority};

/// AlouCode tool definition
#[derive(Debug, Clone)]
pub struct AlouCodeToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    pub category: ToolCategory,
    pub required_permission: &'static str,
}

impl AlouCodeToolDef {
    pub fn to_metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: format!("aloucode_{}", self.name),
            name: self.name.to_string(),
            description: self.description.to_string(),
            category: self.category,
            priority: ToolPriority::Medium,
            status: ToolStatus::Available,
            version: "1.0.0".to_string(),
            author: "alou-code".to_string(),
            created_at: 0,
            updated_at: 0,
            dependencies: vec![],
            platforms: vec!["linux".to_string(), "macos".to_string(), "windows".to_string()],
            permissions: vec![self.required_permission.to_string()],
            tags: vec!["alou-code".to_string(), "builtin".to_string()],
        }
    }
}

/// Get all alou-code tool definitions
pub fn get_aloucode_tool_definitions() -> Vec<AlouCodeToolDef> {
    vec![
        // File operations
        AlouCodeToolDef {
            name: "read_file",
            description: "Read a text file from the workspace.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "offset": { "type": "integer", "minimum": 0 },
                    "limit": { "type": "integer", "minimum": 1 }
                },
                "required": ["path"]
            }),
            category: ToolCategory::FileSystem,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "write_file",
            description: "Write a text file in the workspace.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }),
            category: ToolCategory::FileSystem,
            required_permission: "write",
        },
        AlouCodeToolDef {
            name: "edit_file",
            description: "Replace text in a workspace file.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "old_string": { "type": "string" },
                    "new_string": { "type": "string" },
                    "replace_all": { "type": "boolean" }
                },
                "required": ["path", "old_string", "new_string"]
            }),
            category: ToolCategory::FileSystem,
            required_permission: "write",
        },
        AlouCodeToolDef {
            name: "glob_search",
            description: "Find files by glob pattern.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" },
                    "path": { "type": "string" }
                },
                "required": ["pattern"]
            }),
            category: ToolCategory::FileSystem,
            required_permission: "read",
        },

        // Terminal
        AlouCodeToolDef {
            name: "bash",
            description: "Execute a shell command in the current workspace.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "timeout": { "type": "integer", "minimum": 1 },
                    "description": { "type": "string" }
                },
                "required": ["command"]
            }),
            category: ToolCategory::Terminal,
            required_permission: "danger-full-access",
        },

        // Search
        AlouCodeToolDef {
            name: "grep_search",
            description: "Search file contents with a regex pattern.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" },
                    "path": { "type": "string" },
                    "glob": { "type": "string" },
                    "-n": { "type": "boolean" },
                    "-i": { "type": "boolean" },
                    "head_limit": { "type": "integer", "minimum": 1 }
                },
                "required": ["pattern"]
            }),
            category: ToolCategory::Search,
            required_permission: "read",
        },

        // Web
        AlouCodeToolDef {
            name: "WebFetch",
            description: "Fetch a URL and answer a prompt about it.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "format": "uri" },
                    "prompt": { "type": "string" }
                },
                "required": ["url", "prompt"]
            }),
            category: ToolCategory::Network,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "WebSearch",
            description: "Search the web for current information.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "minLength": 2 }
                },
                "required": ["query"]
            }),
            category: ToolCategory::Network,
            required_permission: "read",
        },

        // Planning
        AlouCodeToolDef {
            name: "TodoWrite",
            description: "Update the structured task list for the current session.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "todos": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": { "type": "string" },
                                "activeForm": { "type": "string" },
                                "status": { "type": "string", "enum": ["pending", "in_progress", "completed"] }
                            },
                            "required": ["content", "activeForm", "status"]
                        }
                    }
                },
                "required": ["todos"]
            }),
            category: ToolCategory::Planning,
            required_permission: "write",
        },

        // Agent/Skills
        AlouCodeToolDef {
            name: "Skill",
            description: "Load a local skill definition and its instructions.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill": { "type": "string" },
                    "args": { "type": "string" }
                },
                "required": ["skill"]
            }),
            category: ToolCategory::Skills,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "Agent",
            description: "Launch a specialized agent task.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "description": { "type": "string" },
                    "prompt": { "type": "string" },
                    "subagent_type": { "type": "string" },
                    "name": { "type": "string" },
                    "model": { "type": "string" }
                },
                "required": ["description", "prompt"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },

        // Polymarket
        AlouCodeToolDef {
            name: "PolymarketListMarkets",
            description: "List active prediction markets on Polymarket.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "active_only": { "type": "boolean" },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
                }
            }),
            category: ToolCategory::Web3,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "PolymarketGetOrderbook",
            description: "Get the order book for a specific token on Polymarket.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_id": { "type": "string" }
                },
                "required": ["token_id"]
            }),
            category: ToolCategory::Web3,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "PolymarketWalletBalance",
            description: "Check the connected Polymarket wallet balance (MATIC and USDC).",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            category: ToolCategory::Web3,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "PolymarketPlaceOrder",
            description: "Place a limit order on Polymarket.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token_id": { "type": "string" },
                    "side": { "type": "string", "enum": ["BUY", "SELL"] },
                    "price": { "type": "number", "minimum": 0.01, "maximum": 0.99 },
                    "size": { "type": "number", "minimum": 1.0 }
                },
                "required": ["token_id", "side", "price", "size"]
            }),
            category: ToolCategory::Web3,
            required_permission: "danger-full-access",
        },
        AlouCodeToolDef {
            name: "PolymarketGetOrders",
            description: "Get your open and recent orders from Polymarket CLOB.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "market": { "type": "string" },
                    "asset_id": { "type": "string" },
                    "limit": { "type": "number" }
                }
            }),
            category: ToolCategory::Web3,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "PolymarketGetTrades",
            description: "Get your trade history from Polymarket CLOB.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "market": { "type": "string" },
                    "asset_id": { "type": "string" },
                    "taker_address": { "type": "string" },
                    "maker_address": { "type": "string" },
                    "before": { "type": "number" },
                    "after": { "type": "number" },
                    "limit": { "type": "number" }
                }
            }),
            category: ToolCategory::Web3,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "PolymarketApproveTokens",
            description: "Approve USDC and CTF tokens for Polymarket trading contracts.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            category: ToolCategory::Web3,
            required_permission: "danger-full-access",
        },

        // LSP
        AlouCodeToolDef {
            name: "LSP",
            description: "Query Language Server Protocol for code intelligence.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["symbols", "references", "diagnostics", "definition", "hover"] },
                    "path": { "type": "string" },
                    "line": { "type": "integer", "minimum": 0 },
                    "character": { "type": "integer", "minimum": 0 },
                    "query": { "type": "string" }
                },
                "required": ["action"]
            }),
            category: ToolCategory::Development,
            required_permission: "read",
        },

        // Task/Worker
        AlouCodeToolDef {
            name: "TaskCreate",
            description: "Create a background task that runs in a separate subprocess.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string" },
                    "description": { "type": "string" }
                },
                "required": ["prompt"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },
        AlouCodeToolDef {
            name: "TaskGet",
            description: "Get the status and details of a background task by ID.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "string" }
                },
                "required": ["task_id"]
            }),
            category: ToolCategory::Automation,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "TaskList",
            description: "List all background tasks and their current status.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            category: ToolCategory::Automation,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "TaskStop",
            description: "Stop a running background task by ID.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "string" }
                },
                "required": ["task_id"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },

        // Team/Cron
        AlouCodeToolDef {
            name: "TeamCreate",
            description: "Create a team of sub-agents for parallel task execution.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "tasks": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "prompt": { "type": "string" },
                                "description": { "type": "string" }
                            },
                            "required": ["prompt"]
                        }
                    }
                },
                "required": ["name", "tasks"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },
        AlouCodeToolDef {
            name: "TeamDelete",
            description: "Delete a team and stop all its running tasks.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "team_id": { "type": "string" }
                },
                "required": ["team_id"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },
        AlouCodeToolDef {
            name: "CronCreate",
            description: "Create a scheduled recurring task.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "schedule": { "type": "string" },
                    "prompt": { "type": "string" },
                    "description": { "type": "string" }
                },
                "required": ["schedule", "prompt"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },
        AlouCodeToolDef {
            name: "CronDelete",
            description: "Delete a scheduled recurring task by ID.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "cron_id": { "type": "string" }
                },
                "required": ["cron_id"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },
        AlouCodeToolDef {
            name: "CronList",
            description: "List all scheduled recurring tasks.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            category: ToolCategory::Automation,
            required_permission: "read",
        },

        // MCP
        AlouCodeToolDef {
            name: "ListMcpResources",
            description: "List available resources from connected MCP servers.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" }
                }
            }),
            category: ToolCategory::Development,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "ReadMcpResource",
            description: "Read a specific resource from an MCP server by URI.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" },
                    "uri": { "type": "string" }
                },
                "required": ["uri"]
            }),
            category: ToolCategory::Development,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "MCP",
            description: "Execute a tool provided by a connected MCP server.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" },
                    "tool": { "type": "string" },
                    "arguments": { "type": "object" }
                },
                "required": ["server", "tool"]
            }),
            category: ToolCategory::Development,
            required_permission: "danger-full-access",
        },

        // Utility
        AlouCodeToolDef {
            name: "Sleep",
            description: "Wait for a specified duration without holding a shell process.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "duration_ms": { "type": "integer", "minimum": 0 }
                },
                "required": ["duration_ms"]
            }),
            category: ToolCategory::System,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "SendUserMessage",
            description: "Send a message to the user.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" },
                    "status": { "type": "string", "enum": ["normal", "proactive"] }
                },
                "required": ["message", "status"]
            }),
            category: ToolCategory::Communication,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "AskUserQuestion",
            description: "Ask the user a question and wait for their response.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "question": { "type": "string" },
                    "options": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                },
                "required": ["question"]
            }),
            category: ToolCategory::Communication,
            required_permission: "read",
        },
        AlouCodeToolDef {
            name: "StructuredOutput",
            description: "Return structured output in the requested format.",
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": true
            }),
            category: ToolCategory::System,
            required_permission: "read",
        },

        // Worker
        AlouCodeToolDef {
            name: "WorkerCreate",
            description: "Create a coding worker boot session.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "cwd": { "type": "string" },
                    "trusted_roots": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "auto_recover_prompt_misdelivery": { "type": "boolean" }
                },
                "required": ["cwd"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },
        AlouCodeToolDef {
            name: "WorkerSendPrompt",
            description: "Send a task prompt to a worker.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "worker_id": { "type": "string" },
                    "prompt": { "type": "string" }
                },
                "required": ["worker_id"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },
        AlouCodeToolDef {
            name: "WorkerTerminate",
            description: "Terminate a worker.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "worker_id": { "type": "string" }
                },
                "required": ["worker_id"]
            }),
            category: ToolCategory::Automation,
            required_permission: "danger-full-access",
        },

        // Plan mode
        AlouCodeToolDef {
            name: "EnterPlanMode",
            description: "Enable planning mode override.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            category: ToolCategory::Planning,
            required_permission: "write",
        },
        AlouCodeToolDef {
            name: "ExitPlanMode",
            description: "Restore or clear planning mode override.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            category: ToolCategory::Planning,
            required_permission: "write",
        },
    ]
}

/// Get tool metadata for all alou-code tools
pub fn get_aloucode_tool_metadata() -> Vec<ToolMetadata> {
    get_aloucode_tool_definitions()
        .into_iter()
        .map(|t| t.to_metadata())
        .collect()
}

/// Create a map of alou-code tool names to their definitions
pub fn get_aloucode_tool_map() -> HashMap<String, AlouCodeToolDef> {
    get_aloucode_tool_definitions()
        .into_iter()
        .map(|t| (t.name.to_string(), t))
        .collect()
}
