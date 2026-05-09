//! Agent Context 分层架构
//!
//! # 架构设计
//!
//! AgentContext
//! │
//! ├─ SystemLayer    - 系统提示词（角色定义，固定）
//! ├─ MemoryLayer    - 任务摘要和关键信息
//! ├─ TaskLayer      - 当前任务状态（工作台）
//! ├─ ToolLayer      - 工具规格说明（精简）
//! └─ HistoryLayer   - 最近对话历史（3-5 条）

use serde::{Deserialize, Serialize};

/// Agent 上下文（分层结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub system: SystemLayer,
    pub memory: MemoryLayer,
    pub task: TaskLayer,
    pub tools: ToolLayer,
    pub history: HistoryLayer,
}

impl AgentContext {
    pub fn new(system_prompt: String) -> Self {
        Self {
            system: SystemLayer::new(system_prompt),
            memory: MemoryLayer::new(),
            task: TaskLayer::new(),
            tools: ToolLayer::new(),
            history: HistoryLayer::new(),
        }
    }

    /// 构建 Prompt（按需组合各层）
    pub fn build_prompt(&self, max_tokens: usize) -> Vec<ContextMessage> {
        let system_budget = max_tokens * 12 / 100;
        let memory_budget = max_tokens * 8 / 100;
        let task_budget = max_tokens * 10 / 100;
        let tools_budget = max_tokens * 20 / 100;
        let history_budget = max_tokens - system_budget - memory_budget - task_budget - tools_budget;
        
        let mut messages = Vec::new();
        let mut tokens_used = 0;
        
        if let Some(msg) = self.system.to_message(system_budget) {
            tokens_used += msg.estimate_tokens();
            messages.push(msg);
        }
        
        for msg in self.memory.to_messages(memory_budget) {
            let msg_tokens = msg.estimate_tokens();
            if tokens_used + msg_tokens > max_tokens { break; }
            tokens_used += msg_tokens;
            messages.push(msg);
        }
        
        if let Some(msg) = self.task.to_message(task_budget) {
            let msg_tokens = msg.estimate_tokens();
            if tokens_used + msg_tokens <= max_tokens {
                tokens_used += msg_tokens;
                messages.push(msg);
            }
        }
        
        for msg in self.tools.to_messages(tools_budget) {
            let msg_tokens = msg.estimate_tokens();
            if tokens_used + msg_tokens > max_tokens { break; }
            tokens_used += msg_tokens;
            messages.push(msg);
        }
        
        for msg in self.history.to_messages(history_budget) {
            let msg_tokens = msg.estimate_tokens();
            if tokens_used + msg_tokens > max_tokens { break; }
            tokens_used += msg_tokens;
            messages.push(msg);
        }
        
        messages
    }

    pub fn add_user_message(&mut self, content: String) {
        self.history.add_user(content);
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.history.add_assistant(content);
    }

    pub fn add_tool_result(&mut self, tool_name: &str, result: &str, max_tokens: usize) {
        let compressed = compress_tool_result(tool_name, result, max_tokens);
        self.memory.add_tool_summary(tool_name.to_string(), compressed.clone());
        self.task.add_tool_execution(tool_name.to_string(), compressed);
    }

    pub fn update_task(&mut self, status: String, details: Option<String>) {
        self.task.update(status, details);
    }

    pub fn add_task_step(&mut self, step: String, status: TaskStepStatus) {
        self.task.add_step(step, status);
    }

    pub fn complete_step(&mut self, step: &str) {
        self.task.mark_step_completed(step);
    }

    pub fn cleanup(&mut self) {
        self.history.cleanup();
        self.memory.cleanup();
        self.task.cleanup();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLayer {
    pub prompt: String,
}

impl SystemLayer {
    pub fn new(prompt: String) -> Self { Self { prompt } }

    pub fn to_message(&self, max_tokens: usize) -> Option<ContextMessage> {
        if self.prompt.is_empty() { return None; }
        let content = if self.prompt.len() > max_tokens * 4 {
            format!("{}...[截断]", self.prompt.chars().take(max_tokens * 4).collect::<String>())
        } else { self.prompt.clone() };
        Some(ContextMessage { role: "system".to_string(), content })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryLayer {
    pub short_term: Vec<MemoryEntry>,
    pub tool_summaries: Vec<ToolSummary>,
    pub conversation_summary: Option<String>,
}

impl MemoryLayer {
    pub fn new() -> Self { Self::default() }

    pub fn add_short_term(&mut self, key: String, value: String) {
        self.short_term.push(MemoryEntry { key, value });
        if self.short_term.len() > 10 { self.short_term.remove(0); }
    }

    pub fn add_tool_summary(&mut self, tool_name: String, summary: String) {
        self.tool_summaries.push(ToolSummary { tool_name, summary, timestamp: chrono::Utc::now().timestamp() });
        if self.tool_summaries.len() > 5 { self.tool_summaries.remove(0); }
    }

    pub fn to_messages(&self, max_tokens: usize) -> Vec<ContextMessage> {
        let mut messages = Vec::new();
        let mut tokens_used = 0;

        if let Some(summary) = &self.conversation_summary {
            let msg = ContextMessage { role: "system".to_string(), content: format!("[对话摘要]\n{}", summary) };
            tokens_used += msg.estimate_tokens();
            messages.push(msg);
        }

        if !self.tool_summaries.is_empty() {
            let content = self.tool_summaries.iter().map(|s| format!("- {}: {}", s.tool_name, s.summary)).collect::<Vec<_>>().join("\n");
            let msg = ContextMessage { role: "system".to_string(), content: format!("[工具执行摘要]\n{}", content) };
            if tokens_used + msg.estimate_tokens() <= max_tokens { messages.push(msg); }
        }

        if !self.short_term.is_empty() {
            let content = self.short_term.iter().map(|e| format!("- {}: {}", e.key, e.value)).collect::<Vec<_>>().join("\n");
            let msg = ContextMessage { role: "system".to_string(), content: format!("[短期记忆]\n{}", content) };
            if tokens_used + msg.estimate_tokens() <= max_tokens { messages.push(msg); }
        }

        messages
    }

    pub fn cleanup(&mut self) {
        if self.short_term.len() > 20 { self.short_term.drain(0..self.short_term.len() - 20); }
        if self.tool_summaries.len() > 10 { self.tool_summaries.drain(0..self.tool_summaries.len() - 10); }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry { pub key: String, pub value: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSummary { pub tool_name: String, pub summary: String, pub timestamp: i64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskLayer {
    pub current_task: String,
    pub completed_steps: Vec<String>,
    pub pending_steps: Vec<String>,
    pub tool_executions: Vec<ToolExecutionRecord>,
    pub status: String,
    pub details: Option<String>,
}

impl TaskLayer {
    pub fn new() -> Self { Self::default() }
    pub fn update(&mut self, status: String, details: Option<String>) { self.status = status; self.details = details; }
    pub fn set_current_task(&mut self, task: String) { self.current_task = task; }

    pub fn add_step(&mut self, step: String, status: TaskStepStatus) {
        match status {
            TaskStepStatus::Pending => { if !self.pending_steps.contains(&step) { self.pending_steps.push(step); } }
            TaskStepStatus::Completed => { self.pending_steps.retain(|s| s != &step); if !self.completed_steps.contains(&step) { self.completed_steps.push(step); } }
        }
    }

    pub fn mark_step_completed(&mut self, step: &str) {
        self.pending_steps.retain(|s| s != step);
        if !self.completed_steps.iter().any(|s| s == step) { self.completed_steps.push(step.to_string()); }
    }

    pub fn add_tool_execution(&mut self, tool_name: String, result_summary: String) {
        self.tool_executions.push(ToolExecutionRecord { tool_name, result_summary, timestamp: chrono::Utc::now().timestamp() });
        if self.tool_executions.len() > 10 { self.tool_executions.remove(0); }
    }

    pub fn to_message(&self, max_tokens: usize) -> Option<ContextMessage> {
        if self.current_task.is_empty() && self.completed_steps.is_empty() && self.pending_steps.is_empty() { return None; }

        let mut parts = Vec::new();
        if !self.current_task.is_empty() { parts.push(format!("任务：{}", self.current_task)); }
        if !self.status.is_empty() { parts.push(format!("状态：{}", self.status)); }
        
        if !self.completed_steps.is_empty() {
            parts.push(format!("已完成:\n{}", self.completed_steps.iter().map(|s| format!("  ✓ {}", s)).collect::<Vec<_>>().join("\n")));
        }
        if !self.pending_steps.is_empty() {
            parts.push(format!("待处理:\n{}", self.pending_steps.iter().map(|s| format!("  ○ {}", s)).collect::<Vec<_>>().join("\n")));
        }
        if !self.tool_executions.is_empty() {
            parts.push(format!("工具执行:\n{}", self.tool_executions.iter().rev().take(5).map(|e| format!("  - {}: {}", e.tool_name, e.result_summary)).collect::<Vec<_>>().join("\n")));
        }

        let mut content = parts.join("\n\n");
        if content.len() > max_tokens * 4 { content = format!("{}...[截断]", content.chars().take(max_tokens * 4).collect::<String>()); }
        Some(ContextMessage { role: "system".to_string(), content })
    }

    pub fn cleanup(&mut self) {
        if self.completed_steps.len() > 20 { self.completed_steps.drain(0..self.completed_steps.len() - 20); }
        if self.pending_steps.len() > 10 { self.pending_steps.drain(0..self.pending_steps.len() - 10); }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStepStatus { Pending, Completed }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRecord { pub tool_name: String, pub result_summary: String, pub timestamp: i64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolLayer { pub tool_specs: Vec<ToolSpec> }

impl ToolLayer {
    pub fn new() -> Self { Self::default() }
    pub fn add_tool(&mut self, name: String, description: String, parameters: String) {
        self.tool_specs.push(ToolSpec { name, description, parameters });
    }

    pub fn to_messages(&self, max_tokens: usize) -> Vec<ContextMessage> {
        if self.tool_specs.is_empty() { return Vec::new(); }
        let mut content = String::from("[可用工具]\n");
        let mut tokens_used = 0;
        for spec in &self.tool_specs {
            let spec_content = format!("- {}({}): {}\n  参数：{}\n", spec.name, spec.parameters, spec.description, spec.parameters);
            if tokens_used + spec_content.len() / 4 > max_tokens { break; }
            content.push_str(&spec_content);
            tokens_used += spec_content.len() / 4;
        }
        vec![ContextMessage { role: "system".to_string(), content }]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec { pub name: String, pub description: String, pub parameters: String }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryLayer { pub messages: Vec<ContextMessage> }

impl HistoryLayer {
    pub fn new() -> Self { Self::default() }
    pub fn add_user(&mut self, content: String) { self.messages.push(ContextMessage { role: "user".to_string(), content }); }
    pub fn add_assistant(&mut self, content: String) { self.messages.push(ContextMessage { role: "assistant".to_string(), content }); }

    pub fn to_messages(&self, max_tokens: usize) -> Vec<ContextMessage> {
        let mut result = Vec::new();
        let mut tokens_used = 0;
        for msg in self.messages.iter().rev() {
            if tokens_used + msg.estimate_tokens() > max_tokens { break; }
            result.push(msg.clone());
            tokens_used += msg.estimate_tokens();
        }
        result.reverse();
        result
    }

    pub fn cleanup(&mut self) {
        if self.messages.len() > 10 { self.messages.drain(0..self.messages.len() - 10); }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage { pub role: String, pub content: String }

impl ContextMessage {
    pub fn estimate_tokens(&self) -> usize { self.content.len() / 4 + 10 }
}

pub fn compress_tool_result(tool_name: &str, result: &str, max_tokens: usize) -> String {
    let max_chars = max_tokens * 4;
    match tool_name {
        "filesystem" | "read_file" => result.lines().take(15).collect::<Vec<_>>().join("\n"),
        "search" | "grep" | "glob" => {
            let lines: Vec<_> = result.lines().collect();
            if lines.len() <= 5 { result.to_string() }
            else { format!("找到 {} 个匹配项，前 5 个:\n{}", lines.len(), lines.iter().take(5).map(|l| format!("  - {}", l)).collect::<Vec<_>>().join("\n")) }
        }
        "bash" | "shell" | "execute" => {
            let mut output = String::new();
            for line in result.lines() {
                if line.starts_with("stdout:") || line.starts_with("stderr:") { output.push_str(line); output.push('\n'); }
            }
            if output.is_empty() { result.lines().take(20).collect::<Vec<_>>().join("\n") } else { output }
        }
        "browser" | "web" | "fetch" => if result.len() > 300 { format!("{}...[内容截断]", result.chars().take(300).collect::<String>()) } else { result.to_string() },
        "code" | "execute_code" => result.lines().take(20).collect::<Vec<_>>().join("\n"),
        _ => if result.len() > max_chars { format!("{}...[输出截断，共 {} 字符]", result.chars().take(max_chars).collect::<String>(), result.len()) } else { result.to_string() }
    }
}
