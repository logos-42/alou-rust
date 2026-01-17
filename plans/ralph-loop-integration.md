# Ralph Loop集成文档

## 概述

Ralph Loop是一种让AI编程助手持续工作直到任务真正完成的机制。本项目已成功将Ralph Loop集成到工作流执行器中，支持自动化的迭代执行和完成条件检查。

## 核心特性

### 1. Ralph Loop配置
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RalphLoopConfig {
    /// 是否启用Ralph Loop
    pub enabled: bool,
    /// 最大迭代次数
    pub max_iterations: u32,
    /// 每次迭代间的延迟（毫秒）
    pub iteration_delay_ms: u64,
    /// 完成条件检查函数（返回true表示完成）
    pub completion_checker: Option<String>, // JSONPath或简单的字符串匹配
    /// 最大总执行时间（毫秒）
    pub max_total_time_ms: Option<u64>,
    /// 每次迭代的超时时间（毫秒）
    pub iteration_timeout_ms: u64,
    /// 成本限制（可选）
    pub max_cost: Option<f64>,
}
```

### 2. 使用方式

#### 启用Ralph Loop的工作流执行
```rust
// 创建Ralph Loop配置
let ralph_config = RalphLoopConfig {
    enabled: true,
    max_iterations: 5,
    iteration_delay_ms: 2000,
    completion_checker: Some("任务完成".to_string()),
    max_total_time_ms: Some(300000), // 5分钟
    iteration_timeout_ms: 60000,    // 1分钟
    max_cost: Some(1.0),
};

// 使用Ralph Loop启动执行
let execution_id = executor.start_execution_with_ralph_loop(
    workflow_id,
    api_key,
    agent_info,
    ralph_config
).await?;
```

#### 普通执行（不使用Ralph Loop）
```rust
let execution_id = executor.start_execution(
    workflow_id,
    api_key,
    agent_info
).await?;
```

### 3. 完成条件检查

Ralph Loop支持多种完成条件检查方式：

#### 字符串匹配
```rust
let config = RalphLoopConfig {
    completion_checker: Some("任务已完成".to_string()),
    ..Default::default()
};
```

#### 文件编辑操作完成条件
Ralph Loop现在支持基于文件编辑操作的完成条件检查：

```rust
// 检查文件是否已被删除
let config = RalphLoopConfig {
    completion_checker: Some("file:deleted:/path/to/file.txt".to_string()),
    ..Default::default()
};

// 检查是否删除了指定行数
let config = RalphLoopConfig {
    completion_checker: Some("file:lines_deleted:/path/to/file.txt:5".to_string()),
    ..Default::default()
};

// 检查是否删除了指定的文本块
let config = RalphLoopConfig {
    completion_checker: Some("file:block_deleted:/path/to/file.txt:function oldFunction".to_string()),
    ..Default::default()
};
```

#### 默认检查（所有步骤完成）
```rust
let config = RalphLoopConfig {
    completion_checker: None, // 默认检查所有步骤状态为Completed
    ..Default::default()
};
```

### 4. 安全机制

Ralph Loop内置了多层安全保护：

- **最大迭代次数限制**：防止无限循环
- **总执行时间限制**：防止长时间运行
- **每次迭代超时**：防止单次执行卡住
- **成本控制**：可选的API成本限制

### 5. 执行流程

```
开始执行
    ↓
检查Ralph Loop是否启用
    ↓
是 → 执行Ralph Loop循环
    ↓
否 → 执行普通工作流
    ↓
每次迭代：
    执行工作流步骤
    检查完成条件
    如果未完成且未达限制 → 等待后继续
    如果完成 → 结束循环
    如果达到限制 → 失败退出
```

## API接口

### Tauri命令

#### 获取执行状态
```typescript
invoke('get_execution_status', { executionId: 'exec_123' })
```

#### 暂停执行
```typescript
invoke('pause_execution', { executionId: 'exec_123' })
```

#### 恢复执行
```typescript
invoke('resume_execution', { executionId: 'exec_123' })
```

#### 取消执行
```typescript
invoke('cancel_execution', { executionId: 'exec_123' })
```

#### 获取执行日志
```typescript
invoke('get_execution_logs', { executionId: 'exec_123' })
```

#### 获取性能指标
```typescript
invoke('get_performance_metrics')
```

## 示例场景

### 1. 代码优化任务
```rust
let config = RalphLoopConfig {
    enabled: true,
    max_iterations: 3,
    completion_checker: Some("代码优化完成".to_string()),
    iteration_delay_ms: 5000,
    ..Default::default()
};
```

### 2. 测试覆盖率提升
```rust
let config = RalphLoopConfig {
    enabled: true,
    max_iterations: 5,
    completion_checker: Some("测试覆盖率>90%".to_string()),
    max_total_time_ms: Some(600000), // 10分钟
    ..Default::default()
};
```

### 3. 文档生成
```rust
let config = RalphLoopConfig {
    enabled: true,
    max_iterations: 2,
    completion_checker: Some("文档生成完毕".to_string()),
    ..Default::default()
};
```

## 监控和调试

### 日志输出
Ralph Loop会在控制台输出详细的执行日志：

```
🚀 [RALPH-LOOP] Starting Ralph Loop execution for workflow: wf_123
🔄 [RALPH-LOOP] Starting iteration 1 for execution exec_456
⏱️ [RALPH-LOOP] Iteration 1 completed in 1500ms
🔄 [RALPH-LOOP] Completion condition not met, continuing loop...
⏳ [RALPH-LOOP] Waiting 2000ms before next iteration
✅ [RALPH-LOOP] Completion condition met! Terminating loop.
```

### 事件监听
通过事件系统可以实时监控执行进度：

```typescript
// 监听执行事件
window.addEventListener('workflow-event', (event) => {
    const { execution_id, progress, current_step } = event.detail;
    console.log(`Execution ${execution_id}: ${progress}% - ${current_step}`);
});
```

## 最佳实践

### 1. 设置合理的限制
- 根据任务复杂度设置合适的`max_iterations`
- 考虑API成本和时间预算
- 设置合理的迭代延迟

### 2. 设计明确的完成条件
- 使用具体的、可验证的完成信号
- 避免模糊的表述如"优化完成"
- 考虑使用JSONPath进行精确检查

### 3. 监控执行状态
- 定期检查执行状态和进度
- 设置超时处理机制
- 准备手动干预方案

### 4. 错误处理
- 处理迭代失败的情况
- 实现降级策略
- 记录详细的错误信息

## 扩展计划

### 未来功能
- 支持更复杂的完成条件检查（JSONPath、自定义函数）
- 添加智能重试策略
- 实现执行历史和回滚功能
- 支持并发Ralph Loop执行
- 添加执行模板和预设配置

## 总结

Ralph Loop的集成成功实现了AI任务的自动化持续执行，为复杂任务的可靠完成提供了有力保障。通过合理的配置和监控，可以有效避免AI"下班"问题，确保任务真正达标。