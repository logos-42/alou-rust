# Step 3: Executor 拆分计划

**状态**: 待执行  
**优先级**: 高

---

## 当前状态

### 已完成

- [x] Phase 1: BridgeManager 无锁化
- [x] Phase 2: SessionActor 集成 RalphLoopExecutor
- [x] 并发验证: 5/10/20 sessions 测试通过
- [x] 日志时间戳: 已添加到 executor.rs

### 未完成

- [ ] **Executor 拆分**: `execute()` → `execute_step()`
- [ ] **真实 sessions 测试**: 需要启动 alou-desktop 运行
- [ ] **Agent Scheduler**: 未来规划

---

## Executor 拆分方案

### 当前结构

```rust
pub async fn execute(&self, task_id: &str) -> Result<TaskFinalResult> {
    loop {
        // AI → Tool → AI → Tool → ...
        call_ai()
        execute_tools()
    }
}
```

### 目标结构

```rust
// Step 定义
enum AgentStep {
    AI,
    Tool(Vec<ToolCall>),
    Finish,
}

// Step 执行
pub async fn execute_step(&self, task_id: &str) -> Result<StepResult> {
    let step = self.next_step(task_id).await?;
    match step {
        AgentStep::AI => self.call_ai_step(task_id).await,
        AgentStep::Tool(calls) => self.execute_tools_step(task_id, calls).await,
        AgentStep::Finish => self.finish_step(task_id).await,
    }
}

// Step 结果
struct StepResult {
    step: AgentStep,
    data: serde_json::Value,
    next_step: Option<AgentStep>,
}
```

---

## 拆分步骤

### Step 1: 定义 Step 枚举

```rust
// agent/step.rs
pub enum AgentStep {
    AI { prompt: String },
    Tool { calls: Vec<ToolCall> },
    Finish { result: String },
}
```

### Step 2: 实现 next_step()

```rust
pub async fn next_step(&self, task_id: &str) -> Result<AgentStep> {
    let task = self.task_manager.get_task(task_id).await?;
    
    if task.is_complete() {
        return Ok(AgentStep::Finish { result: task.final_response });
    }
    
    if task.pending_tools.is_empty() {
        Ok(AgentStep::AI { prompt: task.current_prompt })
    } else {
        Ok(AgentStep::Tool { calls: task.pending_tools })
    }
}
```

### Step 3: 实现 execute_step()

```rust
pub async fn execute_step(&self, task_id: &str) -> Result<StepResult> {
    let step = self.next_step(task_id).await?;
    let step_name = format!("{:?}", step);
    let start = Instant::now();
    
    log::info!("[task={}][step={}] start", task_id, step_name);
    
    let result = match step {
        AgentStep::AI { prompt } => {
            let response = self.call_ai_with_prompt(task_id, &prompt).await?;
            StepResult::AI { response }
        }
        AgentStep::Tool { calls } => {
            let results = self.execute_tools_parallel(task_id, &calls).await?;
            StepResult::Tool { results }
        }
        AgentStep::Finish { result } => {
            StepResult::Finish { result }
        }
    };
    
    log::info!(
        "[task={}][step={}] end (latency={:?}ms)",
        task_id, step_name, start.elapsed().as_millis()
    );
    
    Ok(result)
}
```

### Step 4: 保留 execute() 向后兼容

```rust
pub async fn execute(&self, task_id: &str) -> Result<TaskFinalResult> {
    loop {
        let step_result = self.execute_step(task_id).await?;
        
        if matches!(step_result, StepResult::Finish { .. }) {
            break;
        }
    }
}
```

---

## 收益

### 1. 支持 Parallel Tool Calls

```rust
AgentStep::Tool { calls } => {
    // 并发执行所有工具
    let results = join_all(
        calls.iter().map(|c| self.execute_tool(task_id, c))
    ).await;
}
```

### 2. 为 Scheduler 打基础

```rust
// 未来 Scheduler 可以决定执行顺序
pub struct AgentScheduler {
    executor: Arc<StepExecutor>,
}

impl AgentScheduler {
    pub async fn schedule(&self, plan: ExecutionPlan) -> Result<()> {
        for step in plan.steps {
            self.executor.execute_step(step).await?;
        }
    }
}
```

### 3. 更好的错误处理

```rust
match step_result {
    StepResult::AI { response } => { /* handle */ }
    StepResult::Tool { results } => { /* handle */ }
    StepResult::Finish { result } => { /* handle */ }
}
```

### 4. 可观测性

```
[task=123][step=AI] start
[task=123][step=AI] end (latency=523ms)
[task=123][step=Tool] start
[task=123][step=Tool] end (latency=89ms)
```

---

## 风险

### 1. 状态管理复杂度

需要跟踪：
- 当前 step
- 上一步结果
- 下一步依赖

**缓解**: 使用状态机模式

### 2. 向后兼容

现有代码调用 `execute()`

**缓解**: 保留 `execute()` 内部调用 `execute_step()`

### 3. 测试工作量

需要测试：
- 每个 step 类型
- step 转换
- 错误处理

**缓解**: 增量测试，先测试 AI step

---

## 实施计划

### Day 1: 基础结构

- [ ] 创建 `agent/step.rs`
- [ ] 定义 `AgentStep` 枚举
- [ ] 定义 `StepResult` 结构
- [ ] 实现 `next_step()`

### Day 2: 执行器

- [ ] 实现 `execute_step()`
- [ ] 实现 `call_ai_step()`
- [ ] 实现 `execute_tools_step()`
- [ ] 实现 `finish_step()`

### Day 3: 集成

- [ ] 修改 `execute()` 调用 `execute_step()`
- [ ] 添加日志时间戳
- [ ] 运行测试

### Day 4: 测试

- [ ] 单元测试
- [ ] 集成测试
- [ ] 并发测试

---

## 成功标准

- [ ] `execute_step()` 可以独立调用
- [ ] `execute()` 依然工作（向后兼容）
- [ ] 日志显示 step 时间戳
- [ ] 支持 parallel tool calls
- [ ] 测试通过率 100%

---

**下一步**: 开始 Day 1 实施
