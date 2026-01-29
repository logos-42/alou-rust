# Ralph Loop 工具自动循环修复记录

**日期**: 2025-01-29  
**提交**: e88f225  
**分支**: wasm

## 问题描述

桌面版无法使用工具自动循环工作流（Ralph Loop）。AI 请求工具调用后，前端能获取并执行工具，但提交工具结果后，任务停留在 `processing` 状态，无法继续下一轮 AI 对话。

## 根本原因分析

### 1. Router 拦截问题
- Router 的 `handle_pending_tools()` 直接返回空数组
- 没有将请求转发到 Durable Object
- 导致前端无法获取 AI 请求的工具调用

### 2. 本地环境 Alarm 不触发
- Cloudflare Alarm 在 `wrangler dev` 本地环境中不工作
- `queued` 状态的任务永远不会自动执行
- 依赖 Alarm 的继续执行逻辑无法触发

### 3. 工具结果提交后未继续
- `/tool-result` 端点保存工具结果后返回
- 依赖下次 `/status` 轮询时的 `check_and_execute_pending_task()` 触发继续
- 但检查逻辑要求 `has_tool_results: true` 且 `has_pending_tools: false`
- 时序问题导致无法正确触发继续执行

## 解决方案

### 1. 修复 Router 转发 (alou-edge/src/router/mod.rs)

```rust
async fn handle_pending_tools(&self, env: &Env, task_id: &str) -> Result<Response> {
    console_log!("[Router] 🔍 Forwarding pending-tools request to DO for task: {}", task_id);
    
    // 获取 Durable Object stub
    let namespace = env.durable_object("AI_TASKS")?;
    let id = namespace.id_from_name(task_id)?;
    let stub = id.get_stub()?;
    
    // 转发请求到 DO
    let mut init = worker::RequestInit::new();
    init.with_method(Method::Get);
    let request = worker::Request::new_with_init("http://dummy/pending-tools", &init)?;
    
    stub.fetch_with_request(request).await
}
```

### 2. 直接继续执行 (alou-edge/src/durable_objects/ai_task.rs)

**关键修复**：在 `/tool-result` 端点中，保存工具结果后立即调用 `continue_with_tool_results()`

```rust
(Method::Post, "/tool-result") => {
    let storage = self.state.storage();
    
    // 先处理工具结果
    let result = AITaskHandlers::handle_tool_result(
        &self.task_name(), 
        &storage, 
        req, 
        || self.get_current_timestamp_millis()
    ).await;
    
    // 工具结果已保存，立即触发继续执行
    console_log!("[AITaskDO] 🚀 Tool results saved, immediately continuing with Ralph Loop");
    
    // 检查是否已经在执行中
    if *self.is_executing.borrow() {
        console_log!("[AITaskDO] ⚠️ Already executing, will continue on next check");
        return result;
    }
    
    // 设置执行标志
    *self.is_executing.borrow_mut() = true;
    
    // 立即继续执行
    let continue_result = self.continue_with_tool_results().await;
    
    // 清除执行标志
    *self.is_executing.borrow_mut() = false;
    
    match continue_result {
        Ok(_) => console_log!("[AITaskDO] ✅ Successfully continued after tool result"),
        Err(e) => console_error!("[AITaskDO] ❌ Failed to continue after tool result: {}", e),
    }
    
    result
}
```

### 3. 本地开发支持 (alou-edge/src/durable_objects/ai_task.rs)

在每次 `fetch()` 调用时检查并执行待处理任务：

```rust
async fn check_and_execute_pending_task(&self) -> Result<()> {
    let state = TaskPersistence::load_state(&storage, &task_name).await?;
    
    match state.status {
        TaskStatus::Queued => {
            // 本地开发模式 - 绕过 alarm 直接执行
            console_log!("[CHECK] 🚀 Found queued task, starting execution (LOCAL DEV MODE)");
            if is_workflow {
                self.execute_workflow_task().await
            } else {
                self.execute_task().await
            }
        }
        TaskStatus::Processing => {
            // 检查是否有工具结果需要处理
            if has_tool_results && !has_pending_tools {
                console_log!("[CHECK] ✅ Tool results ready, continuing AI execution");
                self.continue_with_tool_results().await
            }
        }
        _ => Ok(())
    }
}
```

## Ralph Loop 特性

### AI 自主决策
- AI 自己决定何时停止调用工具
- 支持多轮迭代（软性限制：50 轮）
- 连续 3 次无工具调用或明确表示完成时停止

### 停止条件
```rust
let completion_indicators = [
    "任务完成", "已完成", "完成", "结束了", "没有更多",
    "task completed", "completed", "finished", "done", "no more",
    "不需要", "不需要了", "就这样", "可以了"
];

if is_explicit_completion || consecutive_no_tool_calls >= 3 {
    // 任务完成
}
```

### 本地开发友好
- 不依赖 Cloudflare Alarm
- 通过前端轮询驱动执行
- 每次 API 调用都会检查待处理任务

## 测试结果

### 测试脚本：test-ralph-loop.ps1

```powershell
$body = @{
    prompt = "Please do the following tasks step by step: 
              1) Use bash to echo 'Step 1 Complete', 
              2) Use bash to echo 'Step 2 Complete', 
              3) Use bash to echo 'Final Step Done'. 
              You MUST use the bash tool for each step."
    tools = @(@{
        name = "bash"
        description = "Execute bash commands"
        parameters = @{...}
    })
}
```

### 测试输出

```
=== Ralph Loop Test ===
Task ID: task_1769685850140_646491

--- Iteration 1 ---
Status: processing, Progress: 70%
🔧 Found 1 tool call(s)
  Executing: bash with args: {"command":"echo 'Step 1 Complete'"}
✅ Tool results submitted

--- Iteration 2 ---
Status: processing, Progress: 80%
🔧 Found 1 tool call(s)
  Executing: bash with args: {"command":"echo 'Step 2 Complete'"}
✅ Tool results submitted

--- Iteration 3 ---
Status: processing, Progress: 80%
🔧 Found 1 tool call(s)
  Executing: bash with args: {"command":"echo 'Final Step Done'"}
✅ Tool results submitted

--- Iteration 4 ---
Status: completed, Progress: 100%
🎉 Task completed!

Final result: All tasks have been completed step by step as requested:
1. ✓ Used bash to echo 'Step 1 Complete'
2. ✓ Used bash to echo 'Step 2 Complete'
3. ✓ Used bash to echo 'Final Step Done'
```

## 工作流程

```
1. 前端创建任务
   POST /api/ai-task/init-and-start
   ↓
2. AI 返回第一个工具调用
   状态: processing
   ↓
3. 前端轮询获取工具
   GET /api/ai-task/{id}/pending-tools
   ↓
4. 前端执行工具（模拟）
   bash: echo 'Step 1 Complete'
   ↓
5. 前端提交工具结果
   POST /api/ai-task/{id}/tool-result
   ↓
6. 后端立即继续 AI 调用
   continue_with_tool_results()
   ↓
7. AI 决定是否需要更多工具
   - 需要 → 返回步骤 3
   - 不需要 → 任务完成
```

## 修改文件清单

1. **alou-edge/src/durable_objects/ai_task.rs**
   - 修改 `/tool-result` 端点，直接调用 `continue_with_tool_results()`
   - 添加 `is_executing` 标志防止并发执行
   - 增强 `check_and_execute_pending_task()` 支持本地开发

2. **alou-edge/src/durable_objects/ai_task_execution.rs**
   - 实现 Ralph Loop 核心逻辑
   - AI 自主决策停止条件
   - 支持多轮工具调用迭代

3. **alou-edge/src/durable_objects/ai_task_handlers.rs**
   - 优化工具结果处理逻辑
   - 添加详细日志记录

4. **alou-edge/src/durable_objects/task_executor.rs**
   - 添加 Content-Type 头
   - 增强错误日志

5. **alou-edge/src/router/mod.rs**
   - 修复 `handle_pending_tools()` 转发到 DO
   - 修复 `handle_tool_result()` 转发到 DO

6. **test-ralph-loop.ps1**
   - 完整的 Ralph Loop 测试脚本
   - 模拟前端工具执行和结果提交

7. **test-english-tool.ps1**
   - 单个工具调用测试脚本
   - 用于验证基础工具调用功能

## 验证清单

- ✅ AI 能正确请求工具调用
- ✅ 前端能通过 `/pending-tools` 获取工具调用
- ✅ 前端提交工具结果后任务继续执行
- ✅ Ralph Loop 能进行多轮迭代
- ✅ AI 能自主决定何时停止
- ✅ 本地开发环境正常工作（不依赖 Alarm）
- ✅ 任务最终能正确完成

## 后续优化建议

1. **性能优化**
   - 考虑批量处理多个工具调用
   - 优化对话历史存储

2. **错误处理**
   - 添加工具执行超时机制
   - 增强错误恢复能力

3. **监控和日志**
   - 添加 Ralph Loop 迭代次数统计
   - 记录每轮工具调用的耗时

4. **测试覆盖**
   - 添加单元测试
   - 添加集成测试
   - 测试边界情况（如达到最大迭代次数）

## 相关文档

- [WORKFLOW_LOOP_FIX.md](./WORKFLOW_LOOP_FIX.md) - 之前的工作流修复记录
- [ASYNC_TASK_STATUS.md](./ASYNC_TASK_STATUS.md) - 异步任务状态管理
- [DO_DEBUG_GUIDE.md](./DO_DEBUG_GUIDE.md) - Durable Object 调试指南

## 总结

通过这次修复，Ralph Loop 工具自动循环功能已经完全可用：

1. **问题根源**：Router 拦截、Alarm 不触发、继续逻辑时序问题
2. **核心修复**：在 `/tool-result` 端点直接调用 `continue_with_tool_results()`
3. **关键特性**：AI 自主决策、本地开发友好、防止并发执行
4. **测试验证**：成功完成 3 轮工具调用迭代

桌面版现在可以完整支持 AI 驱动的多轮工具调用工作流！🎉
