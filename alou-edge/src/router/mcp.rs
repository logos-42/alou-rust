use crate::agent::context::AgentContext;
use crate::mcp::executor::McpExecutor;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::*;

#[derive(Deserialize)]
struct ExecuteToolRequest {
    tool_name: String,
    args: Value,
    wallet_address: Option<String>,
    chain: Option<String>,
}

#[derive(Serialize)]
struct ExecuteToolResponse {
    result: Value,
    error: Option<String>,
}

/// Handle tool execution request
pub(crate) async fn handle_execute_tool(
    executor: &McpExecutor,
    req: &mut Request,
) -> worker::Result<Response> {
    let body: ExecuteToolRequest = req.json().await?;

    // Build context (use empty session_id for tool execution)
    let context = AgentContext {
        session_id: String::new(), // Tool execution doesn't require session_id
        wallet_address: body.wallet_address.clone(),
        chain: body.chain.clone(),
        recent_events: Vec::new(),
        event_summary: None,
    };

    // Execute tool
    match executor.execute(&body.tool_name, body.args, &context).await {
        Ok(result) => {
            let response = ExecuteToolResponse {
                result,
                error: None,
            };
            json_response(&response)
        }
        Err(e) => {
            let response = ExecuteToolResponse {
                result: Value::Null,
                error: Some(e.to_string()),
            };
            json_response_with_status(&response, 500)
        }
    }
}

/// Handle list tools request
#[derive(Serialize)]
struct ListToolsResponse {
    tools: Vec<crate::mcp::registry::ToolInfo>,
}

pub(crate) async fn handle_list_tools(executor: &McpExecutor) -> worker::Result<Response> {
    let tools = executor.list_tools();
    let response = ListToolsResponse { tools };
    json_response(&response)
}

fn json_response<T: Serialize>(data: &T) -> worker::Result<Response> {
    Response::from_json(data)
}

fn json_response_with_status<T: Serialize>(data: &T, status: u16) -> worker::Result<Response> {
    Ok(Response::from_json(data)?.with_status(status))
}

