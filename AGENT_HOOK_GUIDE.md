# Agent Hook 系统使用指南

## 概述

Agent Hook 系统允许你在 Agent 执行过程中**实时注入新的指令（prompt）**，实现执行中断、优先级调整、动态修改任务等功能。

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                         User                                 │
│                    发送新指令/Prompt                          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              Agent Hook Manager (钩子管理器)                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Instruction Queue (指令队列 - 按优先级排序)            │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │ [Critical] 立即停止！改变策略...                │  │   │
│  │  │ [High]     请更详细地分析数据...                │  │   │
│  │  │ [Medium]   完成后发送邮件通知...                │  │   │
│  │  │ [Low]      记录执行日志...                      │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Broadcast Channel (广播通道)                         │   │
│  │  - 指令更新广播                                        │   │
│  │  - 状态更新广播                                        │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                      AI Agent                                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  执行循环                                              │   │
│  │  1. 检查是否有 Critical 指令 → 立即处理                │   │
│  │  2. 获取下一条待处理指令 → 更新任务                    │   │
│  │  3. 继续执行当前任务                                   │   │
│  │  4. 定期报告状态                                       │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 核心特性

### 1. 指令优先级

| 优先级 | 说明 | 处理时机 |
|--------|------|----------|
| **Critical** | 紧急 | 立即中断当前执行 |
| **High** | 高 | 插队到队列前面 |
| **Medium** | 中 | 正常排队（默认） |
| **Low** | 低 | 等待空闲时处理 |

### 2. 指令类型

| 类型 | 说明 | 使用场景 |
|------|------|----------|
| `Normal` | 普通指令 | 添加新任务到队列 |
| `Modify` | 修改指令 | 修改当前执行参数 |
| `Pause` | 暂停指令 | 暂停当前执行 |
| `Resume` | 恢复指令 | 恢复执行 |
| `Cancel` | 取消指令 | 取消当前执行 |
| `Reset` | 重置指令 | 重置执行状态 |
| `QueryStatus` | 查询状态 | 查询当前执行状态 |

### 3. Agent 执行状态

```rust
pub enum AgentExecutionStatus {
    Idle,                          // 空闲
    Executing { ... },             // 正在执行
    Paused { ... },                // 已暂停
    Completed { ... },             // 已完成
    Cancelled { ... },             // 已取消
    Failed { ... },                // 执行失败
}
```

## 使用示例

### 1. 基本使用

```rust
use crate::agent::agent_hook::{
    AgentHookFactory, 
    UserInstruction, 
    InstructionType,
    InstructionPriority,
};

// 创建 Hook 管理器
let hook = AgentHookFactory::create(
    "my_agent".to_string(),
    "session_123".to_string(),
);

// 注入简单指令
let instruction_id = hook.inject("请更详细地分析数据".to_string()).await?;

// 注入高优先级指令
let high_priority_id = hook.inject_high_priority(
    "改变策略，使用不同的方法".to_string()
).await?;
```

### 2. 暂停/恢复/取消执行

```rust
// 暂停执行
hook.pause("用户要求暂停".to_string()).await?;

// ... 稍后恢复执行
hook.resume().await?;

// 或者取消执行
hook.cancel("用户取消了任务".to_string()).await?;
```

### 3. 监听指令（Agent 端）

```rust
use crate::agent::agent_hook::AgentHookManager;
use tokio::sync::broadcast;

async fn agent_execution_loop(
    hook: Arc<AgentHookManager>,
) {
    // 订阅指令广播
    let mut instruction_receiver = hook.subscribe_instructions();
    
    // 设置执行状态
    hook.set_execution_status(AgentExecutionStatus::Executing {
        current_task: "Processing data analysis".to_string(),
        progress: 0.0,
        started_at: chrono::Utc::now().timestamp(),
    }).await;
    
    loop {
        // 非阻塞方式检查新指令
        if let Ok(instruction) = instruction_receiver.try_recv() {
            println!("收到新指令：{:?}", instruction);
            
            // 处理 Critical 指令
            if instruction.priority == InstructionPriority::Critical {
                match instruction.instruction_type {
                    InstructionType::Cancel => {
                        hook.set_execution_status(AgentExecutionStatus::Cancelled {
                            reason: instruction.content,
                            cancelled_at: chrono::Utc::now().timestamp(),
                        }).await;
                        break;
                    }
                    InstructionType::Pause => {
                        hook.set_execution_status(AgentExecutionStatus::Paused {
                            paused_at: chrono::Utc::now().timestamp(),
                            reason: instruction.content,
                        }).await;
                        continue;
                    }
                    _ => {}
                }
            }
            
            // 标记指令为已处理
            hook.mark_instruction_processed(&instruction.id, Some("处理中".to_string())).await;
        }
        
        // 执行实际任务...
        // 定期更新进度
        hook.set_execution_status(AgentExecutionStatus::Executing {
            current_task: "Processing data analysis".to_string(),
            progress: 50.0,
            started_at: chrono::Utc::now().timestamp(),
        }).await;
    }
}
```

### 4. 获取待处理指令

```rust
// 获取所有待处理的指令
let pending = hook.get_pending_instructions().await;

for instruction in pending {
    println!("待处理指令：{} - {}", instruction.id, instruction.content);
}

// 获取下一条指令
if let Some(next_instruction) = hook.fetch_next_instruction().await {
    // 处理指令
    process_instruction(next_instruction).await;
}
```

### 5. 获取 Agent 状态

```rust
// 获取当前状态快照
let snapshot = hook.get_snapshot().await;
println!("Agent 状态：{:?}", snapshot.execution_status);
println!("执行进度：{}%", snapshot.progress);
println!("待处理指令：{}", snapshot.pending_instructions);

// 获取状态历史
let history = hook.get_state_history(10).await;
for state in history {
    println!("状态历史：{:?} @ {}", state.execution_status, state.snapshot_time);
}
```

### 6. 自定义配置

```rust
use crate::agent::agent_hook::AgentHookConfig;

let config = AgentHookConfig {
    enabled: true,
    allow_interruption: true,      // 允许中断
    allow_modification: true,       // 允许修改指令
    max_queue_size: 50,             // 队列最大长度
    instruction_timeout_seconds: 60, // 指令处理超时
    broadcast_status_updates: true, // 广播状态更新
};

let hook = AgentHookFactory::create_with_config(
    "my_agent".to_string(),
    "session_123".to_string(),
    config,
);
```

### 7. 在 TypeScript 中使用（通过 Tauri）

```typescript
import { invoke } from '@tauri-apps/api/core'

// 注入指令到正在执行的 Agent
async function injectInstruction(
  agentId: string, 
  sessionId: string, 
  content: string,
  priority: 'low' | 'medium' | 'high' | 'critical' = 'medium'
) {
  const result = await invoke('agent_hook_inject', {
    agentId,
    sessionId,
    content,
    priority
  })
  return result
}

// 暂停 Agent 执行
async function pauseAgent(agentId: string, sessionId: string, reason: string) {
  return await invoke('agent_hook_pause', {
    agentId,
    sessionId,
    reason
  })
}

// 获取 Agent 状态
async function getAgentStatus(agentId: string, sessionId: string) {
  return await invoke('agent_hook_get_status', {
    agentId,
    sessionId
  })
}

// 使用示例
await injectInstruction(
  'data_analyzer',
  'session_123',
  '请优先处理前 100 条记录',
  'high'
)
```

## 完整示例：可中断的数据分析任务

```rust
use crate::agent::agent_hook::{
    AgentHookFactory, AgentHookManager,
    UserInstruction, InstructionType, InstructionPriority,
    AgentExecutionStatus,
};
use std::sync::Arc;

async fn run_interruptible_analysis(
    hook: Arc<AgentHookManager>,
    data: Vec<DataItem>,
) -> Result<AnalysisResult, String> {
    // 开始执行
    hook.set_execution_status(AgentExecutionStatus::Executing {
        current_task: "Analyzing data".to_string(),
        progress: 0.0,
        started_at: chrono::Utc::now().timestamp(),
    }).await;

    let total = data.len();
    let mut results = Vec::new();

    // 订阅指令广播
    let mut instruction_rx = hook.subscribe_instructions();

    for (i, item) in data.into_iter().enumerate() {
        // 检查是否有 Critical 指令
        if let Ok(instruction) = instruction_rx.try_recv() {
            match instruction.instruction_type {
                InstructionType::Cancel => {
                    hook.set_execution_status(AgentExecutionStatus::Cancelled {
                        reason: format!("用户取消：{}", instruction.content),
                        cancelled_at: chrono::Utc::now().timestamp(),
                    }).await;
                    return Err("任务已取消".to_string());
                }
                InstructionType::Pause => {
                    hook.set_execution_status(AgentExecutionStatus::Paused {
                        paused_at: chrono::Utc::now().timestamp(),
                        reason: instruction.content.clone(),
                    }).await;
                    
                    // 等待恢复指令
                    loop {
                        if let Ok(instr) = instruction_rx.recv().await {
                            if instr.instruction_type == InstructionType::Resume {
                                break;
                            }
                        }
                    }
                    
                    hook.set_execution_status(AgentExecutionStatus::Executing {
                        current_task: "Analyzing data (resumed)".to_string(),
                        progress: (i as f64 / total as f64) * 100.0,
                        started_at: chrono::Utc::now().timestamp(),
                    }).await;
                }
                InstructionType::Modify => {
                    // 根据新指令修改分析参数
                    println!("修改分析参数：{}", instruction.content);
                    // 解析并应用新参数...
                }
                _ => {}
            }
        }

        // 执行分析
        let result = analyze_item(item).await;
        results.push(result);

        // 更新进度
        let progress = ((i + 1) as f64 / total as f64) * 100.0;
        hook.set_execution_status(AgentExecutionStatus::Executing {
            current_task: "Analyzing data".to_string(),
            progress,
            started_at: chrono::Utc::now().timestamp(),
        }).await;
    }

    // 完成
    hook.set_execution_status(AgentExecutionStatus::Completed {
        result: format!("Analyzed {} items", results.len()),
        completed_at: chrono::Utc::now().timestamp(),
    }).await;

    Ok(AnalysisResult { items: results })
}
```

## API 参考

### AgentHookManager

| 方法 | 说明 | 参数 | 返回值 |
|------|------|------|--------|
| `inject(content)` | 注入简单指令 | content: String | Result<String, String> (instruction_id) |
| `inject_high_priority(content)` | 注入高优先级指令 | content: String | Result<String, String> |
| `pause(reason)` | 暂停执行 | reason: String | Result<String, String> |
| `resume()` | 恢复执行 | - | Result<String, String> |
| `cancel(reason)` | 取消执行 | reason: String | Result<String, String> |
| `submit_instruction(instr)` | 提交指令 | instruction: UserInstruction | Result<String, String> |
| `fetch_next_instruction()` | 获取下一条指令 | - | Option<UserInstruction> |
| `get_pending_instructions()` | 获取待处理指令 | - | Vec<UserInstruction> |
| `get_snapshot()` | 获取状态快照 | - | AgentStateSnapshot |
| `subscribe_instructions()` | 订阅指令 | - | broadcast::Receiver<UserInstruction> |
| `subscribe_status()` | 订阅状态 | - | broadcast::Receiver<AgentStateSnapshot> |

## 相关文件

- 实现：`alou-desktop/src-tauri/src/agent/agent_hook.rs`
- 模块：`alou-desktop/src-tauri/src/agent/mod.rs`

## 最佳实践

1. **定期检查指令**: Agent 执行循环应定期检查新指令
2. **合理设置进度**: 及时更新执行进度，让用户了解状态
3. **处理 Critical 指令**: 优先处理 Critical 优先级指令
4. **状态持久化**: 重要状态应持久化以便恢复
5. **广播订阅**: 使用广播通道实现松耦合的通信

