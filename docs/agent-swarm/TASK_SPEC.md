# Task 规范 v2.0

基于《人月神话》分布式原则的并行任务系统规范

## 核心概念

### 人月神话的启示

1. **不能简单堆叠人力** → Task 必须支持真正的并行执行
2. **沟通成本随人数平方增长** → Task 之间通过明确接口通信，减少耦合
3. **需要外科手术式团队** → Chief Agent + 多个独立 Task Agent

### Task 定义

Task 是**一次性的、自包含的执行单元**，具有以下特征：

- **原子性**: 要么完全成功，要么完全失败
- **隔离性**: 独立的执行上下文
- **幂等性**: 重复执行结果一致
- **可观测性**: 完整的状态和日志

## Task 数据结构

### 完整 Task 定义

```typescript
interface Task {
  // ========== 身份标识 ==========
  /** 全局唯一标识符 (UUID) */
  id: string;
  
  /** 人类可读的任务名称 */
  name: string;
  
  /** 任务类型 */
  type: 'analysis' | 'generation' | 'validation' | 'transformation' | 'custom';
  
  // ========== 执行定义 ==========
  /** 使用的 Skill (可选) */
  skill?: {
    name: string;
    version?: string;
    entrypoint?: string;
  };
  
  /** 直接执行的代码/命令 */
  exec?: {
    command: string;
    args?: string[];
    env?: Record<string, string>;
    cwd?: string;
  };
  
  /** 自定义处理器 */
  handler?: string;
  
  // ========== 输入输出 ==========
  /** 输入参数 */
  input: TaskInput;
  
  /** 输出定义 */
  output?: {
    schema?: JSONSchema;
    format?: 'json' | 'text' | 'binary' | 'stream';
    destination?: string;
  };
  
  // ========== 依赖关系 ==========
  /** 依赖的其他任务 */
  dependsOn: string[];
  
  /** 被哪些任务依赖 */
  dependedBy: string[];
  
  /** 执行条件 */
  condition?: {
    expression: string;  // 如: "${task1.output.success} === true"
    fallback?: 'skip' | 'fail' | 'continue';
  };
  
  // ========== 执行配置 ==========
  config: TaskConfig;
  
  // ========== 状态信息 ==========
  status: TaskStatus;
  
  /** 执行结果 */
  result?: TaskResult;
  
  /** 错误信息 */
  error?: TaskError;
  
  // ========== 元数据 ==========
  metadata: TaskMetadata;
  
  /** 创建时间 */
  createdAt: Date;
  
  /** 开始时间 */
  startedAt?: Date;
  
  /** 完成时间 */
  completedAt?: Date;
}

// ========== 子类型定义 ==========

interface TaskInput {
  /** 静态参数 */
  params?: Record<string, any>;
  
  /** 引用其他任务的输出 */
  refs?: {
    [key: string]: {
      taskId: string;
      path: string;      // 如: "output.data.items"
      transform?: string; // 可选转换
    }
  };
  
  /** 模板字符串 */
  templates?: {
    [key: string]: string;  // 如: "基于 ${task1.output.summary} 生成报告"
  };
}

interface TaskConfig {
  // --- 超时控制 ---
  /** 超时时间 (秒) */
  timeout: number;
  
  /** 超时后的动作 */
  timeoutAction: 'fail' | 'retry' | 'continue';
  
  // --- 重试策略 ---
  /** 最大重试次数 */
  retries: number;
  
  /** 重试间隔 (秒) */
  retryDelay: number;
  
  /** 退避策略 */
  backoff: 'fixed' | 'linear' | 'exponential';
  
  // --- 并行控制 ---
  /** 是否可并行执行 */
  parallelizable: boolean;
  
  /** 资源组 (同组任务串行) */
  resourceGroup?: string;
  
  // --- 资源限制 ---
  resources: {
    memory?: string;    // 如: "512MB"
    cpu?: number;       // CPU 核心数
    disk?: string;      // 磁盘限制
    network?: boolean;  // 是否需要网络
  };
  
  // --- 优先级 ---
  priority: number;  // 1-10, 数字越大优先级越高
  
  // --- 检查点 ---
  checkpoint: boolean;  // 是否保存检查点
}

type TaskStatus = 
  | 'pending'      // 等待执行
  | 'scheduled'    // 已调度
  | 'running'      // 执行中
  | 'paused'       // 已暂停
  | 'completed'    // 已完成
  | 'failed'       // 失败
  | 'cancelled'    // 已取消
  | 'skipped'      // 已跳过(条件不满足)
  | 'timeout';     // 超时

interface TaskResult {
  /** 输出数据 */
  output: any;
  
  /** 执行指标 */
  metrics: {
    /** 开始时间 */
    startTime: Date;
    
    /** 结束时间 */
    endTime: Date;
    
    /** 执行时长 (毫秒) */
    duration: number;
    
    /** CPU 时间 (毫秒) */
    cpuTime?: number;
    
    /** 内存峰值 (MB) */
    memoryPeak?: number;
    
    /** Token 使用量 */
    tokensUsed?: number;
    
    /** API 调用次数 */
    apiCalls?: number;
  };
  
  /** 生成的文件 */
  artifacts: Artifact[];
  
  /** 日志 */
  logs: LogEntry[];
  
  /** 缓存信息 */
  cache?: {
    hit: boolean;
    key?: string;
  };
}

interface TaskError {
  /** 错误代码 */
  code: string;
  
  /** 错误消息 */
  message: string;
  
  /** 错误详情 */
  details?: any;
  
  /** 堆栈跟踪 */
  stack?: string;
  
  /** 可恢复性 */
  recoverable: boolean;
  
  /** 建议的重试策略 */
  retryStrategy?: {
    retryable: boolean;
    suggestedDelay: number;
  };
}

interface TaskMetadata {
  /** 创建者 */
  createdBy: string;
  
  /** 来源 */
  source?: string;
  
  /** 标签 */
  tags?: string[];
  
  /** 关联的会话 */
  sessionId?: string;
  
  /** 关联的计划 */
  planId?: string;
}

interface Artifact {
  /** 文件路径 */
  path: string;
  
  /** 文件类型 */
  type: string;
  
  /** 文件大小 */
  size: number;
  
  /** 文件哈希 */
  hash?: string;
  
  /** 描述 */
  description?: string;
}

interface LogEntry {
  /** 时间戳 */
  timestamp: Date;
  
  /** 级别 */
  level: 'debug' | 'info' | 'warn' | 'error';
  
  /** 消息 */
  message: string;
  
  /** 上下文 */
  context?: Record<string, any>;
}
```

## Task 定义格式 (YAML)

### 基本格式

```yaml
# task.yaml
version: "2.0"
task:
  id: "task-001"
  name: "代码分析任务"
  type: "analysis"
  
  # 使用 Skill
  skill:
    name: "code-analyzer"
    version: ">=1.0.0"
  
  # 输入
  input:
    params:
      target: "./src"
      options:
        recursive: true
        exclude: ["node_modules", "dist"]
  
  # 配置
  config:
    timeout: 300
    retries: 2
    parallelizable: true
    priority: 5
    resources:
      memory: "512MB"
      cpu: 2
```

### 高级示例 (带依赖)

```yaml
version: "2.0"
tasks:
  # ========== 并行分析阶段 ==========
  - id: "analyze-code"
    name: "代码质量分析"
    type: "analysis"
    skill:
      name: "code-analyzer"
    input:
      params:
        target: "./src"
        rules: ["complexity", "duplication", "style"]
    config:
      timeout: 180
      parallelizable: true
      priority: 8

  - id: "analyze-security"
    name: "安全漏洞扫描"
    type: "analysis"
    skill:
      name: "security-scanner"
    input:
      params:
        scanType: "dependency-check"
    config:
      timeout: 300
      parallelizable: true
      priority: 9  # 安全优先

  - id: "analyze-performance"
    name: "性能分析"
    type: "analysis"
    skill:
      name: "perf-analyzer"
    input:
      params:
        target: "./src"
        benchmarks: true
    config:
      timeout: 600
      parallelizable: true
      resources:
        memory: "1GB"

  # ========== 报告生成阶段 ==========
  - id: "generate-report"
    name: "生成综合报告"
    type: "generation"
    skill:
      name: "report-generator"
    input:
      # 引用其他任务的输出
      refs:
        codeIssues:
          taskId: "analyze-code"
          path: "output.issues"
        securityIssues:
          taskId: "analyze-security"
          path: "output.vulnerabilities"
        perfMetrics:
          taskId: "analyze-performance"
          path: "output.metrics"
      templates:
        title: "代码质量报告 - ${{date}}"
        summary: "发现 ${{analyze-code.output.issueCount}} 个问题"
    dependsOn:
      - "analyze-code"
      - "analyze-security"
      - "analyze-performance"
    config:
      timeout: 60
      parallelizable: false
      priority: 5

  # ========== 可选: 自动修复 ==========
  - id: "auto-fix"
    name: "自动修复问题"
    type: "transformation"
    skill:
      name: "code-fixer"
    input:
      refs:
        issues:
          taskId: "analyze-code"
          path: "output.fixableIssues"
    dependsOn:
      - "analyze-code"
    # 条件执行
    condition:
      expression: "${analyze-code.output.hasFixableIssues} === true"
      fallback: "skip"
    config:
      timeout: 120
      parallelizable: false
      priority: 3

# ========== 执行策略 ==========
strategy:
  type: "parallel-pipeline"
  maxConcurrency: 4
  failFast: true
  
# ========== 全局配置 ==========
globals:
  workingDirectory: "/tmp/agent-tasks"
  environment:
    NODE_ENV: "production"
    LOG_LEVEL: "info"
  
  # 检查点配置
  checkpoints:
    enabled: true
    interval: 30
    retention: 7  # 保留 7 天
```

## 执行模式

### 1. 顺序执行 (Sequential)

```yaml
strategy:
  type: "sequential"
```

任务按定义顺序依次执行，前一个失败则停止。

### 2. 并行执行 (Parallel)

```yaml
strategy:
  type: "parallel"
  maxConcurrency: 5
```

所有无依赖的任务同时执行。

### 3. 流水线 (Pipeline)

```yaml
strategy:
  type: "pipeline"
  stages:
    - name: "analyze"
      tasks: ["analyze-code", "analyze-security"]
    - name: "report"
      tasks: ["generate-report"]
      dependsOn: ["analyze"]
```

分阶段执行，前一阶段完成后才进入下一阶段。

### 4. 分支合并 (Fork-Join)

```yaml
tasks:
  - id: "fork"
    type: "fork"
    branches:
      - tasks: ["task-a1", "task-a2"]
      - tasks: ["task-b1", "task-b2"]
  
  - id: "join"
    type: "join"
    dependsOn: ["fork"]
```

### 5. Map-Reduce

```yaml
tasks:
  - id: "map"
    type: "map"
    input:
      items: "${files}"
    taskTemplate:
      skill:
        name: "file-processor"
      input:
        file: "${item}"
  
  - id: "reduce"
    type: "reduce"
    dependsOn: ["map"]
    skill:
      name: "result-aggregator"
    input:
      results: "${map.outputs}"
```

## 输入引用语法

### 基础引用

```yaml
input:
  refs:
    # 引用单个值
    data:
      taskId: "task-1"
      path: "output.data"
    
    # 引用嵌套值
    nested:
      taskId: "task-1"
      path: "output.items.0.name"
```

### 模板字符串

```yaml
input:
  templates:
    # 简单插值
    greeting: "Hello, ${user.name}!"
    
    # 表达式
    summary: "发现 ${task1.output.count} 个问题，耗时 ${task1.metrics.duration}ms"
    
    # 条件
    status: "${task1.output.success ? '成功' : '失败'}"
```

### 数据转换

```yaml
input:
  refs:
    filtered:
      taskId: "task-1"
      path: "output.items"
      transform: "filter(x => x.severity === 'high')"
    
    mapped:
      taskId: "task-1"
      path: "output.items"
      transform: "map(x => x.name)"
    
    counted:
      taskId: "task-1"
      path: "output.items"
      transform: "length"
```

## Task Plan (任务计划)

Task Plan 是一组 Task 的集合，由 Chief Agent 生成：

```typescript
interface TaskPlan {
  /** 计划 ID */
  id: string;
  
  /** 目标描述 */
  objective: string;
  
  /** 任务列表 */
  tasks: Task[];
  
  /** 执行策略 */
  strategy: ExecutionStrategy;
  
  /** 全局配置 */
  globals: GlobalConfig;
  
  /** 检查点 */
  checkpoints: CheckPointConfig;
  
  /** 依赖图 (自动生成) */
  dependencyGraph: DependencyGraph;
}
```

### Plan 示例

```yaml
plan:
  id: "plan-001"
  objective: "重构用户认证模块"
  
  tasks:
    - id: "analyze-auth"
      name: "分析现有认证代码"
      skill: { name: "code-analyzer" }
      input:
        params:
          target: "./src/auth"
      config:
        timeout: 300
        parallelizable: true

    - id: "design-refactor"
      name: "设计重构方案"
      skill: { name: "architecture-designer" }
      input:
        refs:
          analysis:
            taskId: "analyze-auth"
            path: "output"
      dependsOn: ["analyze-auth"]
      config:
        timeout: 600

    - id: "implement-refactor"
      name: "执行重构"
      skill: { name: "code-refactor" }
      input:
        refs:
          design:
            taskId: "design-refactor"
            path: "output.plan"
      dependsOn: ["design-refactor"]
      config:
        timeout: 1800
        parallelizable: false

    - id: "verify-refactor"
      name: "验证重构结果"
      skill: { name: "code-validator" }
      input:
        refs:
          changes:
            taskId: "implement-refactor"
            path: "output.changes"
      dependsOn: ["implement-refactor"]
      config:
        timeout: 300

  strategy:
    type: "pipeline"
    maxConcurrency: 2
    failFast: false

  globals:
    workingDirectory: "/tmp/refactor-plan"
    environment:
      PROJECT_ROOT: "${cwd}"
```

## 状态机

```
┌─────────┐     ┌───────────┐     ┌─────────┐
│ Pending │────▶│ Scheduled │────▶│ Running │
└─────────┘     └───────────┘     └────┬────┘
                                       │
       ┌───────────────────────────────┼───────────────┐
       │                               │               │
       ▼                               ▼               ▼
┌────────────┐                  ┌──────────┐    ┌──────────┐
│  Skipped   │                  │  Paused  │    │  Failed  │
│(条件不满足)│                  │(可恢复)  │    │(可重试)  │
└────────────┘                  └────┬─────┘    └────┬─────┘
                                     │               │
                                     ▼               ▼
                               ┌──────────┐    ┌──────────┐
                              │ Resuming │    │ Retrying │
                               └────┬─────┘    └────┬─────┘
                                    │               │
                                    └───────────────┘
                                                    │
                       ┌────────────────────────────┘
                       ▼
               ┌───────────────┐
               │   Completed   │
               │   (成功)      │
               └───────────────┘
                       │
                       ▼
               ┌───────────────┐
               │  Cancelled    │
               │  (手动取消)   │
               └───────────────┘
```

## 错误处理

### 错误分类

| 错误类型 | 说明 | 处理策略 |
|----------|------|----------|
| Transient | 临时错误 (网络超时) | 自动重试 |
| Permanent | 永久错误 (代码错误) | 立即失败 |
| Validation | 输入验证失败 | 跳过或失败 |
| Resource | 资源不足 | 等待或失败 |
| Dependency | 依赖任务失败 | 级联失败 |

### 重试策略

```yaml
config:
  retries: 3
  retryDelay: 5
  backoff: exponential  # fixed | linear | exponential
  retryOn:
    - "ETIMEDOUT"
    - "ECONNRESET"
    - "RATE_LIMIT"
  maxRetryTime: 600  # 最大重试时间
```

### 错误恢复

```yaml
# 失败回退
tasks:
  - id: "primary"
    skill: { name: "advanced-analyzer" }
    config:
      timeout: 300
    onError:
      fallback: "secondary"

  - id: "secondary"
    skill: { name: "basic-analyzer" }
    config:
      timeout: 60
    onError:
      fallback: "skip"
```

## 监控与可观测性

### 事件流

Task 执行过程中产生的事件：

```typescript
type TaskEvent =
  | { type: 'task.created'; taskId: string; planId: string }
  | { type: 'task.scheduled'; taskId: string; scheduledAt: Date }
  | { type: 'task.started'; taskId: string; startedAt: Date }
  | { type: 'task.progress'; taskId: string; percent: number; message: string }
  | { type: 'task.completed'; taskId: string; result: TaskResult }
  | { type: 'task.failed'; taskId: string; error: TaskError }
  | { type: 'task.cancelled'; taskId: string; reason: string }
  | { type: 'task.retried'; taskId: string; attempt: number }
  | { type: 'checkpoint.saved'; taskId: string; checkpointId: string }
  | { type: 'artifact.created'; taskId: string; artifact: Artifact };
```

### 指标收集

```typescript
interface TaskMetrics {
  // 执行指标
  totalTasks: number;
  completedTasks: number;
  failedTasks: number;
  skippedTasks: number;
  
  // 时间指标
  totalDuration: number;
  averageTaskDuration: number;
  queueWaitTime: number;
  
  // 资源指标
  totalMemoryUsed: number;
  totalCpuTime: number;
  
  // 效率指标
  parallelizationRatio: number;  // 实际并行度
  dependencyWaitTime: number;    // 等待依赖的时间
}
```

## 最佳实践

### Task 设计原则

1. **单一职责**: 一个 Task 只做一件事
2. **确定性的输入输出**: 相同的输入产生相同的输出
3. **最小依赖**: 只声明真正需要的依赖
4. **合理粒度**: 既不要太细(开销大)也不要太粗(难并行)

### 并行化建议

```yaml
# ✅ 好的设计 - 可以并行
tasks:
  - id: "analyze-file-1"
    input: { params: { file: "a.js" } }
  - id: "analyze-file-2"
    input: { params: { file: "b.js" } }
  - id: "analyze-file-3"
    input: { params: { file: "c.js" } }

# ❌ 坏的设计 - 无法并行
tasks:
  - id: "analyze-all"
    input: { params: { files: ["a.js", "b.js", "c.js"] } }
```

### 错误处理建议

```yaml
# ✅ 优雅降级
tasks:
  - id: "analyze-with-ai"
    skill: { name: "ai-analyzer" }
    config:
      timeout: 60
      retries: 2
    onError:
      fallback: "analyze-with-rules"

  - id: "analyze-with-rules"
    skill: { name: "rule-analyzer" }
    config:
      timeout: 30
    onError:
      action: "skip"  # 规则分析失败也没关系
```

## 实现参考

参见:
- [TypeScript 实现](../src/task/)
- [Python 实现](../src/task_python/)
- [示例 Plans](../examples/task-plans/)
