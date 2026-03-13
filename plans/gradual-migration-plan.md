# 渐进式迁移计划

> **核心原则**: 新功能使用新架构，旧功能保持不变，逐步迁移

---

## 架构对比

| 特性 | 旧架构 | 新架构 |
|------|--------|--------|
| **并发模型** | 全局锁 (Mutex) | Actor 模型 (无锁) |
| **状态管理** | 全局共享 | Session 隔离 |
| **资源管理** | 共享资源池 | RAII 借用 |
| **扩展性** | 单节点 | 多节点就绪 |

---

## 迁移阶段

### Phase 1: 双架构并存 (当前)

**目标**: 新架构可用，旧功能正常

**实现**:
```rust
// runtime/actor_commands.rs
pub async fn execute_agent_task_compatible(
    message: String,
    agent_id: String,
    config: UserApiConfig,
    bridge_manager: BridgeManager,
    use_new_architecture: bool,  // ← 迁移开关
) -> Result<TaskFinalResult, String> {
    if use_new_architecture {
        // 新架构：通过 SessionActor 执行
        let command_manager = SessionCommandManager::new(bridge_manager);
        command_manager.send_command_with_response(...).await
    } else {
        // 旧架构：使用原有函数
        execute_agent_task_old(message, agent_id, config, bridge_manager).await
    }
}
```

**使用策略**:
- 新功能：`use_new_architecture = true`
- 旧功能：`use_new_architecture = false`

---

### Phase 2: 新功能迁移 (1-2 周)

**目标**: 所有新功能使用新架构

**迁移列表**:

| 功能 | 新实现 | 状态 |
|------|--------|------|
| Agent 聊天 | `SessionCommandManager::send_command()` | ✅ 就绪 |
| 群聊消息 | `SessionMessage::GroupChatMessage` | ✅ 就绪 |
| 工具调用 | `SessionMessage::ToolCall` | ✅ 就绪 |
| 工作流执行 | `WorkflowEngine::start_execution()` | ✅ 就绪 |
| 资源借用 | `LlmPool::borrow()` (RAII) | ✅ 就绪 |

**示例代码**:
```rust
// 新功能：使用 SessionCommandManager
let command_manager = SessionCommandManager::new(bridge_manager);

// 发送聊天命令
let result = command_manager.send_command_with_response(
    &session_id,
    AgentCommand::Chat { content: "Hello".to_string(), metadata: None },
    60, // 60 秒超时
).await;

// 群聊消息
command_manager.send_command(
    &session_id,
    AgentCommand::GroupChat {
        group_id: "group_123".to_string(),
        content: "Hi everyone!".to_string(),
        from_agent: Some("agent_456".to_string()),
    },
).await;
```

---

### Phase 3: 旧功能逐步迁移 (2-4 周)

**目标**: 每周迁移 20% 旧功能

**迁移优先级**:

1. **高优先级** (第 1 周)
   - Agent 聊天 (`execute_agent_task`)
   - 群聊消息处理
   - 工具调用

2. **中优先级** (第 2 周)
   - 工作流执行
   - 资源管理
   - Session 管理

3. **低优先级** (第 3-4 周)
   - 配置管理
   - 日志系统
   - 监控指标

**迁移步骤**:

```rust
// 步骤 1: 添加迁移标志
pub async fn execute_agent_task(
    message: String,
    agent_id: String,
    config: UserApiConfig,
    bridge_manager: BridgeManager,
) -> Result<TaskFinalResult, String> {
    // 默认使用新架构
    execute_agent_task_compatible(
        message, agent_id, config, bridge_manager,
        true, // ← 改为 true
    ).await
}

// 步骤 2: 测试验证
// - 单元测试
// - 集成测试
// - 性能基准

// 步骤 3: 移除旧代码
// 删除 execute_agent_task_old 函数
```

---

### Phase 4: 清理旧架构 (第 5 周)

**目标**: 移除所有旧架构代码

**待删除文件**:
- `agent/executor.rs` (旧执行器)
- `agent/task.rs` (旧任务管理)
- 旧的全局状态管理代码

**待更新调用点**:
```rust
// 旧代码 ❌
execute_agent_task(message, agent_id, config, bridge_manager).await;

// 新代码 ✅
let command_manager = SessionCommandManager::new(bridge_manager);
command_manager.send_command_with_response(
    &agent_id,
    AgentCommand::Chat { content: message, metadata: None },
    60,
).await;
```

---

## 回滚策略

如果新架构出现问题，可以快速回滚：

```rust
// 在 execute_agent_task_compatible 中切换
execute_agent_task_compatible(
    message, agent_id, config, bridge_manager,
    false, // ← 改为 false 回滚到旧架构
).await
```

**回滚检查点**:
1. 性能下降 > 50%
2. 错误率 > 5%
3. 用户投诉增加

---

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_new_architecture_chat() {
        let command_manager = SessionCommandManager::new(bridge_manager);
        let result = command_manager.send_command_with_response(
            "test_session",
            AgentCommand::Chat { content: "Test".to_string(), metadata: None },
            60,
        ).await;
        assert!(result.success);
    }
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_session_actor_integration() {
    // 1. 创建 SessionActor
    // 2. 发送命令
    // 3. 验证响应
    // 4. 验证状态变化
}
```

### 性能基准

```rust
#[bench]
fn bench_old_architecture(b: &mut Bencher) {
    // 测试旧架构性能
}

#[bench]
fn bench_new_architecture(b: &mut Bencher) {
    // 测试新架构性能
    // 目标：并发性能提升 > 2x
}
```

---

## 监控指标

### 关键指标

| 指标 | 旧架构 | 新架构目标 |
|------|--------|-----------|
| 并发 session 数 | < 100 | > 1000 |
| 消息延迟 (p99) | ~500ms | < 100ms |
| 错误率 | < 1% | < 0.5% |
| 内存/session | ~10MB | ~5MB |

### 监控面板

```
Session Actor Dashboard
├── Active Sessions: 1234
├── Pending Tasks: 56
├── LLM Pool Available: 8/10
├── Tool Pool Available: 15/20
├── Message Latency (p99): 85ms
└── Error Rate: 0.3%
```

---

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 性能回归 | 中 | 高 | 性能基准测试 + 快速回滚 |
| 数据丢失 | 低 | 高 | Session 状态持久化 |
| 兼容性问题 | 中 | 中 | 双架构并存 + 充分测试 |
| 学习曲线 | 高 | 低 | 文档 + 代码示例 |

---

## 时间表

```
Week 1: Phase 2 (新功能迁移)
Week 2: Phase 3 (旧功能迁移 50%)
Week 3: Phase 3 (旧功能迁移 100%)
Week 4: Phase 3 (缓冲周)
Week 5: Phase 4 (清理旧架构)
```

---

## 成功标准

✅ **迁移完成标志**:
1. 所有功能使用新架构
2. 性能提升 > 2x (并发场景)
3. 错误率 < 0.5%
4. 旧架构代码 100% 移除
5. 文档完整

---

**文档版本**: v1.0
**创建时间**: 2026-03-14
**维护者**: Alou Team
