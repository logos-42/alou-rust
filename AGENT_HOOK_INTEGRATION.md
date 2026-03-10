# Agent Hook 集成指南

## 问题描述

当前问题：在 AI Agent 执行过程中，用户发送新消息时：
1. 消息没有发送出去
2. 回车按钮没有反应
3. 无法在 Agent 执行过程中注入新指令

## 解决方案

使用 **Agent Hook 系统** 实现执行中实时指令注入。

## 架构设计

```
用户输入消息
    ↓
ChatInput 组件 (onKeyDown: Enter)
    ↓
sendMessage() 函数
    ↓
判断 Agent 是否正在执行
    ├─→ 未执行：正常创建/选择 Agent
    └─→ 执行中：通过 Agent Hook 注入指令
            ↓
    AgentHookManager.inject()
            ↓
    广播通道 (broadcast)
            ↓
    Agent 执行循环监听
            ↓
    实时响应新指令
```

## 实现步骤

### 1. 在 AgentChat 中集成 Hook

```typescript
// src/components/AgentChat/useAgentHookIntegration.ts
import { invoke } from '@tauri-apps/api/core'

export interface UseAgentHookIntegrationProps {
  activeChannelId: string | null
  sessionId: string
  isLoading: boolean
}

export function useAgentHookIntegration({
  activeChannelId,
  sessionId,
  isLoading,
}: UseAgentHookIntegrationProps) {
  
  // 注入指令到正在执行的 Agent
  const injectInstruction = async (
    content: string,
    priority: 'low' | 'medium' | 'high' | 'critical' = 'medium'
  ) => {
    if (!activeChannelId) {
      throw new Error('没有活动的智能体频道')
    }

    try {
      const result = await invoke('agent_hook_inject', {
        agent_id: activeChannelId,
        session_id: sessionId,
        content,
        priority,
      })

      if (result.success) {
        console.log('[Hook] 指令注入成功:', result.instruction_id)
        return result
      } else {
        throw new Error(result.message)
      }
    } catch (error) {
      console.error('[Hook] 指令注入失败:', error)
      throw error
    }
  }

  // 暂停 Agent 执行
  const pauseAgent = async (reason: string) => {
    return await invoke('agent_hook_pause', {
      agent_id: activeChannelId,
      session_id,
      reason,
    })
  }

  // 恢复 Agent 执行
  const resumeAgent = async () => {
    return await invoke('agent_hook_resume', {
      agent_id: activeChannelId,
      session_id,
    })
  }

  // 取消 Agent 执行
  const cancelAgent = async (reason: string) => {
    return await invoke('agent_hook_cancel', {
      agent_id: activeChannelId,
      session_id,
      reason,
    })
  }

  // 获取 Agent 状态
  const getAgentStatus = async () => {
    return await invoke('agent_hook_get_status', {
      agent_id: activeChannelId,
      session_id,
    })
  }

  return {
    injectInstruction,
    pauseAgent,
    resumeAgent,
    cancelAgent,
    getAgentStatus,
  }
}
```

### 2. 修改 sendMessage 函数

```typescript
// src/components/AgentChat/useAgentMessages.ts

// 在 sendMessage 函数中添加 Hook 逻辑
const sendMessage = useCallback(async () => {
  const text = currentMessage.trim()
  if (!text || isLoading) {
    return
  }

  // 检查 Agent 是否正在执行
  const isAgentExecuting = isLoading && activeChannelId

  if (isAgentExecuting) {
    // Agent 正在执行，使用 Hook 注入指令
    try {
      console.log('[sendMessage] Agent 执行中，注入新指令:', text)
      
      // 调用 Agent Hook 注入指令
      const result = await invoke('agent_hook_inject', {
        agent_id: activeChannelId,
        session_id: sessionId,
        content: text,
        priority: 'medium', // 可以根据内容判断优先级
      })

      if (result.success) {
        // 显示用户消息（作为指令）
        appendMessage({
          id: `user_instruction_${Date.now()}`,
          type: 'user',
          content: text,
          timestamp: Date.now(),
          source: 'user',
          metadata: {
            isInstruction: true,
            instructionId: result.instruction_id,
          },
        })

        // 显示系统确认消息
        appendMessage({
          id: `system_ack_${Date.now()}`,
          type: 'system',
          content: `✅ 指令已发送给正在执行的 Agent: "${text}"`,
          timestamp: Date.now(),
          source: 'system',
        })
      } else {
        throw new Error(result.message)
      }
    } catch (error) {
      console.error('[sendMessage] 注入指令失败:', error)
      appendMessage({
        id: `system_err_${Date.now()}`,
        type: 'error',
        content: `❌ 指令发送失败：${(error as Error).message}`,
        timestamp: Date.now(),
        source: 'system',
      })
    }
  } else {
    // Agent 未执行，正常流程
    // ... 现有的创建/选择 Agent 逻辑
  }

  setCurrentMessage('')
}, [currentMessage, isLoading, activeChannelId, sessionId, appendMessage, invoke])
```

### 3. 在 Agent 执行循环中监听 Hook 指令

```rust
// src/agent/executor.rs 或相关的 Agent 执行模块

use crate::agent::agent_hook::{AgentHookManager, UserInstruction, InstructionType};

async fn execute_agent_task(
    hook: Arc<AgentHookManager>,
    task: AgentTask,
) -> Result<ExecutionResult, String> {
    // 订阅指令广播
    let mut instruction_rx = hook.subscribe_instructions();
    
    // 设置执行状态
    hook.set_execution_status(AgentExecutionStatus::Executing {
        current_task: task.description.clone(),
        progress: 0.0,
        started_at: chrono::Utc::now().timestamp(),
    }).await;

    // 执行任务循环
    for step in task.steps {
        // 检查是否有新指令（非阻塞）
        if let Ok(instruction) = instruction_rx.try_recv() {
            println!("[Agent] 收到新指令：{:?}", instruction);
            
            // 处理指令
            match instruction.instruction_type {
                InstructionType::Cancel => {
                    hook.set_execution_status(AgentExecutionStatus::Cancelled {
                        reason: instruction.content,
                        cancelled_at: chrono::Utc::now().timestamp(),
                    }).await;
                    return Err("任务已取消".to_string());
                }
                InstructionType::Pause => {
                    hook.set_execution_status(AgentExecutionStatus::Paused {
                        paused_at: chrono::Utc::now().timestamp(),
                        reason: instruction.content,
                    }).await;
                    
                    // 等待恢复
                    while let Ok(instr) = instruction_rx.recv().await {
                        if instr.instruction_type == InstructionType::Resume {
                            break;
                        }
                    }
                    
                    hook.set_execution_status(AgentExecutionStatus::Executing {
                        current_task: task.description.clone(),
                        progress: step.progress,
                        started_at: chrono::Utc::now().timestamp(),
                    }).await;
                }
                InstructionType::Modify => {
                    // 根据新指令修改任务参数
                    println!("修改任务参数：{}", instruction.content);
                    // 解析并应用新参数...
                }
                _ => {
                    // 普通指令，添加到待处理队列
                    // Agent 可以在适当时机处理
                }
            }
        }

        // 执行当前步骤
        execute_step(step).await?;
        
        // 更新进度
        hook.set_execution_status(AgentExecutionStatus::Executing {
            current_task: task.description.clone(),
            progress: step.progress,
            started_at: chrono::Utc::now().timestamp(),
        }).await;
    }

    // 完成
    hook.set_execution_status(AgentExecutionStatus::Completed {
        result: "任务完成".to_string(),
        completed_at: chrono::Utc::now().timestamp(),
    }).await;

    Ok(ExecutionResult::Success)
}
```

### 4. 显示 Agent 执行状态

```tsx
// src/components/AgentChat/AgentStatusIndicator.tsx

interface AgentStatusIndicatorProps {
  activeChannelId: string | null
  sessionId: string
}

export function AgentStatusIndicator({ activeChannelId, sessionId }: AgentStatusIndicatorProps) {
  const [status, setStatus] = useState<any>(null)
  const [pendingInstructions, setPendingInstructions] = useState(0)

  useEffect(() => {
    if (!activeChannelId) return

    // 定期轮询状态
    const pollStatus = async () => {
      try {
        const result = await invoke('agent_hook_get_status', {
          agent_id: activeChannelId,
          session_id: sessionId,
        })
        setStatus(result)
        setPendingInstructions(result.pending_instructions)
      } catch (error) {
        console.error('获取状态失败:', error)
      }
    }

    pollStatus()
    const interval = setInterval(pollStatus, 2000) // 每 2 秒轮询一次

    return () => clearInterval(interval)
  }, [activeChannelId, sessionId])

  if (!status) return null

  return (
    <div className="agent-status-indicator">
      <div className={`status-dot ${status.status}`}></div>
      <span className="status-text">
        {status.status === 'executing' ? `执行中 ${status.progress}%` : status.status}
      </span>
      {pendingInstructions > 0 && (
        <span className="pending-badge">{pendingInstructions} 条待处理指令</span>
      )}
    </div>
  )
}
```

## CSS 样式

```css
/* src/components/AgentChat/AgentStatusIndicator.css */

.agent-status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-radius: 20px;
  font-size: 12px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

.status-dot.executing {
  background: #4CAF50;
}

.status-dot.paused {
  background: #FFC107;
}

.status-dot.cancelled {
  background: #F44336;
}

.status-dot.completed {
  background: #2196F3;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.pending-badge {
  background: #FF5722;
  color: white;
  padding: 2px 6px;
  border-radius: 10px;
  font-size: 10px;
}
```

## 使用示例

### 用户在 Agent 执行时发送消息

1. **用户输入**: "请更详细地分析数据"
2. **系统检测**: Agent 正在执行
3. **调用 Hook**: `agent_hook_inject` with priority="medium"
4. **Agent 接收**: 通过广播通道收到指令
5. **Agent 响应**: 调整执行策略
6. **显示反馈**: "✅ 指令已发送"

### 紧急中断

1. **用户输入**: "停止！"
2. **系统检测**: Critical 优先级
3. **调用 Hook**: `agent_hook_inject` with priority="critical"
4. **Agent 立即停止**: 中断当前执行
5. **显示状态**: 已取消

## 测试步骤

1. 启动应用
2. 创建一个 Agent 并让它执行长时间任务
3. 在执行过程中输入新指令
4. 观察：
   - 消息是否显示
   - Agent 状态是否更新
   - 指令是否被处理

## 相关文件

- `src/components/AgentChat/useAgentHookIntegration.ts` - Hook 集成
- `src/components/AgentChat/useAgentMessages.ts` - 消息发送逻辑
- `src/agent/agent_hook.rs` - Hook 系统核心
- `src/agent/hook_commands.rs` - Tauri 命令
