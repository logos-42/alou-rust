# 保守迁移方案

> **核心原则**: 现有稳定流程不变，新功能逐步接入新架构

---

## 现有流程分析

### 前端调用链

```
AgentChat.jsx
  └── useAgentMessages.ts
       └── sendMessageToAgent()
            └── 1. 检查 loadingByAgent[targetAgentId]
            └── 2. 创建/获取 session
            └── 3. 调用后端 API (agentService)
                 └── 后端：execute_agent_task()
```

### 后端执行链

```
execute_agent_task (agent/commands.rs)
  └── 创建 TaskManager
  └── 创建 AiClient
  └── 创建 RalphLoopExecutor
  └── executor.execute(task_id)
       └── AI 推理循环
       └── 工具调用
       └── 返回结果
```

**关键发现**:
- ✅ 现有流程已经使用 `session_id` 隔离
- ✅ 已有 `loadingByAgent` 防止并发冲突
- ✅ 已有 `sessionsByAgent` 管理多 session

---

## 迁移策略：三层渐进

### Layer 1: 保持现有 API 不变

**目标**: 前端无感知，后端内部切换

```rust
// 现有 Tauri Command 保持不变
#[tauri::command]
pub async fn execute_agent_task(
    message: String,
    agent_id: String,
    config: UserApiConfig,
    bridge_manager: State<BridgeManager>,
) -> Result<TaskFinalResult, String> {
    // 内部调用兼容函数
    execute_agent_task_compatible(
        message,
        agent_id,
        config,
        bridge_manager,
        false, // ← 默认使用旧架构，保证稳定
    ).await
}
```

**优点**:
- 前端无需修改
- 随时可以回滚
- 风险最低

---

### Layer 2: 新功能使用新架构

**适用场景**:
- 新的 Agent 类型
- 新的群聊模式
- 新的工作流功能

**示例**: 新的群聊 Agent

```typescript
// 前端：新群聊模式使用新 API
// alou-desktop/src/components/AgentChat/useNewGroupChat.ts

import { invoke } from '@tauri-apps/api/core'

// 新 API：通过 SessionCommandManager
async function sendGroupChatMessage(
  sessionId: string,
  groupId: string,
  content: string,
  fromAgent: string
) {
  const result = await invoke('execute_agent_task_compatible', {
    agentId: sessionId,
    message: content,
    config: apiConfig,
    useNewArchitecture: true, // ← 新功能使用新架构
  })
}
```

**后端**:

```rust
// 新增 Tauri Command
#[tauri::command]
pub async fn execute_agent_task_with_new_architecture(
    message: String,
    agent_id: String,
    config: UserApiConfig,
    bridge_manager: State<BridgeManager>,
) -> Result<TaskFinalResult, String> {
    execute_agent_task_compatible(
        message,
        agent_id,
        config,
        bridge_manager,
        true, // ← 新架构
    ).await
}
```

---

### Layer 3: 逐步切换旧功能

**前提条件**:
1. 新架构运行稳定 > 2 周
2. 错误率 < 旧架构
3. 性能提升 > 20%

**切换步骤**:

#### 步骤 1: 10% 流量切换

```rust
// 按 session_id hash 分配
let use_new_arch = (session_id.hash() % 10) == 0;

execute_agent_task_compatible(
    ...,
    use_new_arch, // ← 10% 流量
).await
```

#### 步骤 2: 50% 流量切换

```rust
let use_new_arch = (session_id.hash() % 2) == 0;
```

#### 步骤 3: 100% 切换

```rust
// 默认使用新架构
execute_agent_task_compatible(
    ...,
    true, // ← 默认新架构
).await
```

#### 步骤 4: 移除旧代码

```rust
// 删除旧实现
// pub async fn execute_agent_task_old(...) { ... }
```

---

## 具体实施计划

### Week 1: 准备阶段

**任务**:
1. ✅ `actor_commands.rs` 已创建
2. ✅ `execute_agent_task_compatible` 已实现
3. [ ] 添加单元测试
4. [ ] 添加性能基准测试

**验收标准**:
- 新架构单元测试通过率 100%
- 性能基准：并发 > 旧架构 2x

---

### Week 2: 新功能试点

**目标**: 选择一个新功能使用新架构

**推荐试点**: 群聊 Agent 自主响应

**原因**:
- 独立模块，不影响现有功能
- 并发场景多，能体现新架构优势
- 失败影响小

**实施**:

```typescript
// 新群聊模式
// alou-desktop/src/components/AgentChat/useNewGroupChatAutonomousAgent.ts

export const useNewGroupChatAutonomousAgent = () => {
  const triggerAgentResponse = async (agentId: string, message: GroupChatMessage) => {
    // 使用新架构
    await invoke('execute_agent_task_with_new_architecture', {
      agentId,
      message: message.content,
    })
  }
}
```

---

### Week 3-4: 监控与优化

**监控指标**:

| 指标 | 旧架构 | 新架构目标 |
|------|--------|-----------|
| 并发 session 数 | 100 | 500 |
| 消息延迟 (p99) | 500ms | 200ms |
| 错误率 | 1% | 0.5% |

**监控面板**:

```rust
// 添加监控端点
#[tauri::command]
pub async fn get_architecture_stats() -> ArchitectureStats {
    ArchitectureStats {
        old_architecture: get_old_stats(),
        new_architecture: get_new_stats(),
    }
}
```

---

### Week 5-8: 逐步切换

**切换节奏**:

| 周 | 流量比例 | 监控重点 |
|----|---------|---------|
| 5  | 10%     | 错误率   |
| 6  | 25%     | 性能     |
| 7  | 50%     | 稳定性   |
| 8  | 100%    | 全面监控 |

**回滚条件**:
- 错误率 > 2%
- 性能下降 > 50%
- 用户投诉增加

---

## 回滚方案

### 快速回滚开关

```rust
// 全局配置
pub static USE_NEW_ARCHITECTURE: AtomicBool = AtomicBool::new(false);

// 在 execute_agent_task_compatible 中
let use_new = USE_NEW_ARCHITECTURE.load(Ordering::Relaxed);
if use_new {
    // 新架构
} else {
    // 旧架构
}

// 回滚命令
#[tauri::command]
pub fn rollback_to_old_architecture() {
    USE_NEW_ARCHITECTURE.store(false, Ordering::Relaxed);
    log::warn!("[ROLLBACK] Switched to OLD architecture");
}
```

### 回滚检查清单

- [ ] 所有 session 切换到旧架构
- [ ] 新架构资源释放
- [ ] 用户通知（如有影响）
- [ ] 问题定位和修复

---

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_new_architecture_chat() {
        let result = execute_agent_task_compatible(
            "Hello".to_string(),
            "test_agent".to_string(),
            config,
            bridge_manager,
            true, // 新架构
        ).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_old_architecture_chat() {
        let result = execute_agent_task_compatible(
            "Hello".to_string(),
            "test_agent".to_string(),
            config,
            bridge_manager,
            false, // 旧架构
        ).await;
        assert!(result.is_ok());
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_concurrent_sessions() {
    // 创建 100 个并发 session
    let tasks: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                execute_agent_task_compatible(
                    format!("Message {}", i),
                    format!("agent_{}", i),
                    config.clone(),
                    bridge_manager.clone(),
                    true, // 新架构
                ).await
            })
        })
        .collect();
    
    let results = join_all(tasks).await;
    
    // 新架构应该能处理所有并发
    let success_count = results.iter()
        .filter(|r| matches!(r, Ok(Ok(_))))
        .count();
    
    assert!(success_count > 90); // 90% 成功率
}
```

---

## 成功标准

### Phase 1 (Week 1-2): ✅ 新架构可用
- [ ] 单元测试通过
- [ ] 新功能试点成功

### Phase 2 (Week 3-4): ✅ 监控数据正常
- [ ] 错误率 < 旧架构
- [ ] 性能提升 > 20%

### Phase 3 (Week 5-8): ✅ 全面切换
- [ ] 100% 流量使用新架构
- [ ] 旧代码移除
- [ ] 文档更新

---

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 新架构 bug | 中 | 高 | 快速回滚开关 |
| 性能回归 | 低 | 高 | 性能基准测试 |
| 数据丢失 | 低 | 高 | Session 状态持久化 |
| 学习曲线 | 高 | 低 | 文档 + 代码审查 |

---

**文档版本**: v1.0
**创建时间**: 2026-03-14
**维护者**: Alou Team
