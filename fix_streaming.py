#!/usr/bin/env python3
"""修复 streaming.rs 中的 ToolExecuting 事件处理"""

with open('/Users/apple/Downloads/alou/alou-desktop/src-tauri/src/agent/streaming.rs', 'r') as f:
    content = f.read()

# 替换 ToolExecuting 事件处理
old_event = '''TaskEvent::ToolExecuting { task_id, tool_name } => StreamEvent {
            r#type: "tool_executing".to_string(),
            content: None,
            name: Some(tool_name.clone()),
            success: None,
            progress: None,
            message: Some(format!("正在执行工具：{}", tool_name)),
            result: None,
            error: None,
        },'''

new_event = '''TaskEvent::ToolExecuting { task_id, tool_name, arguments } => StreamEvent {
            r#type: "tool_executing".to_string(),
            content: arguments.and_then(|args| serde_json::to_string(&args).ok()),
            name: Some(tool_name.clone()),
            success: None,
            progress: None,
            message: Some(format!("正在执行工具：{}", tool_name)),
            result: None,
            error: None,
        },'''

if old_event in content:
    content = content.replace(old_event, new_event)
    with open('/Users/apple/Downloads/alou/alou-desktop/src-tauri/src/agent/streaming.rs', 'w') as f:
        f.write(content)
    print("streaming.rs 已修复")
else:
    print("未找到匹配的事件处理")
