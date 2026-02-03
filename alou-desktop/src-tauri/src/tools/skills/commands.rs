//! Skills 命令处理

use super::{SkillsTool, types::*};
use super::super::{ToolResult, ToolError, ExecutionContext};

/// 执行命令
pub async fn execute_command(
    tool: &SkillsTool,
    args: serde_json::Value,
    context: &ExecutionContext,
) -> Result<ToolResult, ToolError> {
    let action = args.get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArguments("Missing 'action' field".to_string()))?;

    match action {
        // === AI 核心接口 ===
        "search_skills" => handle_search(tool, args).await,
        "execute_skill" => handle_execute(tool, args, context).await,
        "create_agent_skill" => handle_create_agent(tool, args).await,
        
        // === 管理接口 ===
        "list_skills" => handle_list(tool, args).await,
        "get_skill_detail" => handle_detail(tool, args).await,
        "get_stats" => handle_stats(tool).await,
        "delete_skill" => handle_delete(tool, args).await,
        
        _ => Err(ToolError::InvalidArguments(format!("Unknown action: {}", action))),
    }
}

// 命令处理函数

async fn handle_search(tool: &SkillsTool, args: serde_json::Value) -> Result<ToolResult, ToolError> {
    let req = SearchSkillsRequest {
        query: args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        category: args.get("category").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };

    let skills = super::executor::search_skills(tool, req).await;
    
    let skill_list: Vec<serde_json::Value> = skills.iter().map(|s| serde_json::json!({
        "id": s.id,
        "name": s.display_name,
        "description": s.description,
        "category": s.category.as_str(),
        "tags": s.tags,
    })).collect();

    Ok(ToolResult {
        success: true,
        data: serde_json::json!({
            "skills": skill_list,
            "count": skill_list.len()
        }),
        error: None,
        execution_time_ms: 0,
        output: Some(format!("Found {} skills", skill_list.len())),
        warnings: vec![],
        context: None,
    })
}

async fn handle_execute(
    tool: &SkillsTool, 
    args: serde_json::Value, 
    context: &ExecutionContext
) -> Result<ToolResult, ToolError> {
    let skill_id = args.get("skill_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArguments("Missing 'skill_id' field".to_string()))?;

    let inputs: std::collections::HashMap<String, serde_json::Value> = args.get("inputs")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();

    let req = ExecuteSkillRequest {
        skill_id: skill_id.to_string(),
        inputs,
    };

    let result = super::executor::execute_skill(tool, req, &context.session_id).await
        .map_err(|e| ToolError::ExecutionFailed(e))?;

    Ok(ToolResult {
        success: result.success,
        data: result.to_json(),
        error: result.error,
        execution_time_ms: result.execution_time_ms,
        output: Some(format!("Skill '{}' executed: {}", skill_id, 
            if result.success { "success" } else { "failed" })),
        warnings: vec![],
        context: None,
    })
}

async fn handle_create_agent(tool: &SkillsTool, args: serde_json::Value) -> Result<ToolResult, ToolError> {
    let req = CreateSkillRequest {
        display_name: args.get("display_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'display_name' field".to_string()))?
            .to_string(),
        description: args.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        category: "agent".to_string(),
        persona: args.get("persona").and_then(|v| v.as_str()).map(|s| s.to_string()),
        capabilities: args.get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        constraints: args.get("constraints")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        tags: args.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
    };

    let skill = super::executor::create_agent_skill(tool, req).await
        .map_err(|e| ToolError::ExecutionFailed(e))?;

    Ok(ToolResult {
        success: true,
        data: serde_json::json!({
            "skill_id": skill.id,
            "skill": {
                "id": skill.id,
                "display_name": skill.display_name,
                "description": skill.description,
                "category": skill.category.as_str(),
            }
        }),
        error: None,
        execution_time_ms: 0,
        output: Some(format!("Created agent skill '{}'", skill.display_name)),
        warnings: vec![],
        context: None,
    })
}

async fn handle_list(tool: &SkillsTool, args: serde_json::Value) -> Result<ToolResult, ToolError> {
    let category = args.get("category").and_then(|v| v.as_str());
    let skills = super::executor::list_skills(tool, category).await;

    let skill_list: Vec<serde_json::Value> = skills.iter().map(|s| serde_json::json!({
        "id": s.id,
        "display_name": s.display_name,
        "description": s.description,
        "category": s.category.as_str(),
        "version": s.version,
        "tags": s.tags,
        "enabled": s.enabled,
    })).collect();

    Ok(ToolResult {
        success: true,
        data: serde_json::json!({
            "skills": skill_list,
            "total": skill_list.len()
        }),
        error: None,
        execution_time_ms: 0,
        output: Some(format!("Found {} skills", skill_list.len())),
        warnings: vec![],
        context: None,
    })
}

async fn handle_detail(tool: &SkillsTool, args: serde_json::Value) -> Result<ToolResult, ToolError> {
    let skill_id = args.get("skill_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArguments("Missing 'skill_id' field".to_string()))?;

    let skill = super::executor::get_skill_detail(tool, skill_id).await
        .ok_or_else(|| ToolError::InvalidArguments(format!("Skill '{}' not found", skill_id)))?;

    Ok(ToolResult {
        success: true,
        data: serde_json::json!({
            "skill": {
                "id": skill.id,
                "display_name": skill.display_name,
                "description": skill.description,
                "category": skill.category.as_str(),
                "version": skill.version,
                "input_schema": skill.input_schema,
                "output_schema": skill.output_schema,
                "tags": skill.tags,
                "created_at": skill.created_at,
                "updated_at": skill.updated_at,
            }
        }),
        error: None,
        execution_time_ms: 0,
        output: Some(format!("Retrieved skill '{}'", skill.display_name)),
        warnings: vec![],
        context: None,
    })
}

async fn handle_stats(tool: &SkillsTool) -> Result<ToolResult, ToolError> {
    let stats = super::executor::get_stats(tool).await;

    Ok(ToolResult {
        success: true,
        data: stats,
        error: None,
        execution_time_ms: 0,
        output: Some("Retrieved skill statistics".to_string()),
        warnings: vec![],
        context: None,
    })
}

async fn handle_delete(tool: &SkillsTool, args: serde_json::Value) -> Result<ToolResult, ToolError> {
    let skill_id = args.get("skill_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArguments("Missing 'skill_id' field".to_string()))?;

    let deleted = super::executor::delete_skill(tool, skill_id).await
        .map_err(|e| ToolError::ExecutionFailed(e))?;

    if deleted {
        Ok(ToolResult {
            success: true,
            data: serde_json::json!({"deleted": true}),
            error: None,
            execution_time_ms: 0,
            output: Some(format!("Deleted skill '{}'", skill_id)),
            warnings: vec![],
            context: None,
        })
    } else {
        Err(ToolError::InvalidArguments(format!("Skill '{}' not found", skill_id)))
    }
}

/// 帮助文本
pub fn help_text() -> String {
    r#"Skills Management Tool

Manage and execute reusable skills. Supports AI automatic skill discovery and execution.

Actions:
  === AI Core Actions ===
  - search_skills: Search skills by query
    params: { "query": "text processing", "category": "text_processing" }
  
  - execute_skill: Execute a skill with inputs
    params: { "skill_id": "skill_xxx", "inputs": { "text": "..." } }
  
  - create_agent_skill: Create an AI Agent skill
    params: { 
      "display_name": "Code Reviewer",
      "description": "Reviews code for issues",
      "persona": "You are a code review expert...",
      "capabilities": ["find bugs", "suggest improvements"],
      "constraints": ["be concise", "focus on security"]
    }

  === Management Actions ===
  - list_skills: List all skills
    params: { "category": "agent" } (optional)
  
  - get_skill_detail: Get detailed skill information
    params: { "skill_id": "skill_xxx" }
  
  - get_stats: Get skill storage statistics
  
  - delete_skill: Delete a skill
    params: { "skill_id": "skill_xxx" }

Examples:

Search skills:
{
  "action": "search_skills",
  "query": "text summarization"
}

Execute skill:
{
  "action": "execute_skill",
  "skill_id": "skill_text_summarizer",
  "inputs": {
    "text": "Long text to summarize...",
    "max_length": 200
  }
}

Create Agent skill:
{
  "action": "create_agent_skill",
  "display_name": "Bug Finder",
  "description": "Finds bugs in code",
  "persona": "You are an expert code reviewer...",
  "capabilities": ["identify bugs", "suggest fixes"],
  "constraints": ["be specific", "provide examples"]
}"#.to_string()
}
