//! 兼容性工具框架 - 用于在Durable Objects中执行工具调用

use crate::compatibility::models::{Tool, ToolCall, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use worker::Result;

/// 工具特征
#[async_trait(?Send)]
#[allow(dead_code)]
pub trait CompatibleTool {
    /// 工具名称
    fn name(&self) -> &str;
    
    /// 工具描述
    fn description(&self) -> &str;
    
    /// 工具参数模式
    fn parameters(&self) -> Value;
    
    /// 执行工具调用
    async fn execute(&self, arguments: &Value) -> Result<ToolResult>;
}

/// 工具执行器
#[allow(dead_code)]
pub struct ToolExecutor {
    tools: HashMap<String, Box<dyn CompatibleTool>>,
}

impl ToolExecutor {
    /// 创建新的工具执行器
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        
        // 注册基础工具
        tools.insert("get_current_time".to_string(), Box::new(GetCurrentTimeTool) as Box<dyn CompatibleTool>);
        tools.insert("calculate".to_string(), Box::new(CalculateTool) as Box<dyn CompatibleTool>);
        tools.insert("search_web".to_string(), Box::new(SearchWebTool) as Box<dyn CompatibleTool>);
        
        Self { tools }
    }
    
    /// 执行工具调用
    pub async fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        if let Some(tool) = self.tools.get(&tool_call.tool) {
            tool.execute(&tool_call.arguments).await
        } else {
            // 标记为需要本地执行
            Ok(ToolResult {
                tool: tool_call.tool.clone(),
                result: json!({
                    "status": "requires_local_execution",
                    "message": "This tool requires local execution on the client side"
                }),
                error: None,
            })
        }
    }
    
    /// 批量执行工具调用
    pub async fn execute_batch(&self, tool_calls: &[ToolCall]) -> Result<Vec<ToolResult>> {
        let mut results = Vec::new();
        
        for tool_call in tool_calls {
            match self.execute(tool_call).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(ToolResult {
                        tool: tool_call.tool.clone(),
                        result: Value::Null,
                        error: Some(format!("Failed to execute tool: {}", e)),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// 获取所有可用工具
    pub fn get_available_tools(&self) -> Vec<Tool> {
        self.tools.values().map(|tool| {
            Tool {
                name: tool.name().to_string(),
                description: Some(tool.description().to_string()),
                parameters: Some(tool.parameters()),
            }
        }).collect()
    }
}

/// 获取当前时间工具
#[allow(dead_code)]
struct GetCurrentTimeTool;

#[async_trait(?Send)]
impl CompatibleTool for GetCurrentTimeTool {
    fn name(&self) -> &str {
        "get_current_time"
    }
    
    fn description(&self) -> &str {
        "获取当前时间（UTC）"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "description": "时间格式（可选），支持：timestamp, iso8601, human",
                    "enum": ["timestamp", "iso8601", "human"]
                }
            }
        })
    }
    
    async fn execute(&self, arguments: &Value) -> Result<ToolResult> {
        use chrono::Utc;
        
        let format = arguments.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("iso8601");
        
        let now = Utc::now();
        let result = match format {
            "timestamp" => json!({
                "timestamp": now.timestamp(),
                "timestamp_ms": now.timestamp_millis()
            }),
            "human" => json!({
                "date": now.format("%Y-%m-%d").to_string(),
                "time": now.format("%H:%M:%S").to_string(),
                "timezone": "UTC"
            }),
            _ => json!({
                "iso8601": now.to_rfc3339(),
                "timestamp": now.timestamp()
            }),
        };
        
        Ok(ToolResult {
            tool: self.name().to_string(),
            result,
            error: None,
        })
    }
}

/// 计算工具
#[allow(dead_code)]
struct CalculateTool;

#[async_trait(?Send)]
impl CompatibleTool for CalculateTool {
    fn name(&self) -> &str {
        "calculate"
    }
    
    fn description(&self) -> &str {
        "执行数学计算"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "数学表达式，例如：2 + 3 * 4"
                }
            },
            "required": ["expression"]
        })
    }
    
    async fn execute(&self, arguments: &Value) -> Result<ToolResult> {
        let expression = arguments.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| worker::Error::RustError("Missing expression".to_string()))?;
        
        // 简单的表达式求值（在实际实现中应该使用更安全的库）
        let result = match evaluate_simple_expression(expression) {
            Ok(value) => json!({
                "expression": expression,
                "result": value,
                "success": true
            }),
            Err(e) => json!({
                "expression": expression,
                "error": e,
                "success": false
            }),
        };
        
        Ok(ToolResult {
            tool: self.name().to_string(),
            result,
            error: None,
        })
    }
}

/// 网页搜索工具
#[allow(dead_code)]
struct SearchWebTool;

#[async_trait(?Send)]
impl CompatibleTool for SearchWebTool {
    fn name(&self) -> &str {
        "search_web"
    }
    
    fn description(&self) -> &str {
        "搜索网页信息"
    }
    
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索查询"
                },
                "limit": {
                    "type": "number",
                    "description": "结果数量限制（默认：5）",
                    "minimum": 1,
                    "maximum": 10
                }
            },
            "required": ["query"]
        })
    }
    
    async fn execute(&self, arguments: &Value) -> Result<ToolResult> {
        let query = arguments.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| worker::Error::RustError("Missing query".to_string()))?;
        
        let limit = arguments.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);
        
        // 在实际实现中，这里会调用实际的搜索API
        // 现在返回模拟结果
        let result = json!({
            "query": query,
            "results": (1..=limit).map(|i| {
                json!({
                    "title": format!("搜索结果 {} - {}", i, query),
                    "url": format!("https://example.com/result/{}", i),
                    "snippet": format!("这是关于'{}'的搜索结果摘要 {}", query, i)
                })
            }).collect::<Vec<_>>(),
            "total_results": 100,
            "search_time": 0.5
        });
        
        Ok(ToolResult {
            tool: self.name().to_string(),
            result,
            error: None,
        })
    }
}

/// 简单的表达式求值函数
#[allow(dead_code)]
fn evaluate_simple_expression(expr: &str) -> std::result::Result<f64, String> {
    // 移除空白字符
    let expr = expr.replace(' ', "");
    
    // 简单的安全检查
    if expr.contains(|c: char| !c.is_ascii_digit() && c != '.' && c != '+' && c != '-' && c != '*' && c != '/' && c != '(' && c != ')') {
        return Err("表达式包含不安全字符".to_string());
    }
    
    // 在实际实现中应该使用更安全的表达式求值库
    // 这里使用简单的解析作为示例
    if let Ok(value) = expr.parse::<f64>() {
        Ok(value)
    } else {
        // 尝试简单的四则运算
        if let Some(pos) = expr.find('+') {
            let left = evaluate_simple_expression(&expr[..pos])?;
            let right = evaluate_simple_expression(&expr[pos + 1..])?;
            Ok(left + right)
        } else if let Some(pos) = expr.find('-') {
            let left = evaluate_simple_expression(&expr[..pos])?;
            let right = evaluate_simple_expression(&expr[pos + 1..])?;
            Ok(left - right)
        } else if let Some(pos) = expr.find('*') {
            let left = evaluate_simple_expression(&expr[..pos])?;
            let right = evaluate_simple_expression(&expr[pos + 1..])?;
            Ok(left * right)
        } else if let Some(pos) = expr.find('/') {
            let left = evaluate_simple_expression(&expr[..pos])?;
            let right = evaluate_simple_expression(&expr[pos + 1..])?;
            if right == 0.0 {
                return Err("除以零错误".to_string());
            }
            Ok(left / right)
        } else {
            Err("无法解析表达式".to_string())
        }
    }
}
