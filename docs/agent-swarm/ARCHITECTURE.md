# Agent Swarm 架构设计

基于《人月神话》原则的 AI Agent 分布式协作系统

## 核心设计原则

### 1. 概念完整性 (Conceptual Integrity)
> "概念完整性是系统设计中最重要的因素" —— Fred Brooks

- **统一元模型**: 所有 Skill 和 Task 遵循相同的接口规范
- **最小惊讶原则**: 行为可预测，接口一致
- **正交设计**: Skill 之间相互独立，可自由组合

### 2. 减少沟通成本
> "向进度落后的项目增加人手，只会使进度更加落后" —— 人月神话

- **上下文隔离**: 每个 Task 拥有独立的执行上下文
- **接口契约**: 通过 Schema 定义明确的输入输出
- **无共享状态**: 状态通过消息传递，避免锁竞争

### 3. 外科手术式团队 (Surgical Team)
> "少数精英 + 大量支持 = 高效团队"

```
┌─────────────────────────────────────────┐
│           Chief Agent (主智能体)          │
│    - 任务分解与协调                       │
│    - 质量把控与整合                       │
│    - 对外统一接口                         │
└─────────────────┬───────────────────────┘
                  │ 派遣/协调
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌───────┐    ┌───────┐    ┌───────┐
│Task-1 │    │Task-2 │    │Task-N │  子智能体
│(领域A) │    │(领域B) │    │(领域X)│  并行执行
└───────┘    └───────┘    └───────┘
```

### 4. 模块化与信息隐藏
- **Skill Registry**: 全局可复用的能力目录
- **Task Sandbox**: 独立执行的沙箱环境
- **契约式编程**: 前置条件、后置条件、不变量

### 5. 渐进交付 (Incremental Delivery)
- **任务流水线**: Task 可以链式组合
- **中间结果**: 支持 Checkpoint 和恢复
- **热插拔**: 运行时动态加载 Skill

---

## 系统架构

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    Agent Swarm 系统                          │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Skill      │  │    Task      │  │   Swarm          │  │
│  │   Registry   │  │   Executor   │  │   Coordinator    │  │
│  │  (能力目录)   │  │  (执行引擎)   │  │  (协调器)         │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         │                 │                    │            │
│         ▼                 ▼                    ▼            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Agent Kernel (核心层)                    │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐ │  │
│  │  │ Context │  │  LLM    │  │  Tool   │  │ Memory  │ │  │
│  │  │ Manager │  │ Adapter │  │  Router │  │  Store  │ │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘ │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Skill 系统

Skill 是**全局可复用的能力单元**，存储在 `~/.config/agents/skills/`：

```yaml
# Skill 目录结构
skills/
├── SKILL.md                    # Skill 规范定义
├── SPEC.md                     # 详细规范
├── API_REFERENCE.md            # API 参考
├── scripts/                    # 可执行脚本
│   ├── lib/                    # 库文件
│   └── *.py, *.js, *.ts        # 脚本
├── assets/                     # 资源文件
└── tests/                      # 测试用例
```

#### Skill 规范 (SKILL.md)

```yaml
---
name: skill-name                 # Skill 标识
version: 1.0.0                   # 语义化版本
description: "简短的描述"         # 一句话描述
category: [web, data, ai, ...]   # 分类标签
author: "Author Name"            # 作者
license: MIT                     # 许可证

# 风险与权限
risk_level: [low|medium|high]    # 风险等级
allowed_tools:                   # 允许使用的工具
  - bash
  - python
  - node
  - file_write
  - web_search

# 依赖
requires:                        # 依赖的其他 skills
  - other-skill >= 1.0.0
  
# 资源限制
resources:                       # 执行资源限制
  max_memory: 512MB
  max_cpu: 2
  timeout: 300s
---

# Skill 详细文档

## 何时使用

明确说明该 Skill 的使用场景和触发条件。

## 能力范围

- 能力 1
- 能力 2
- 能力 3

## 使用示例

### 示例 1: 基本用法
```
输入: {...}
输出: {...}
```

### 示例 2: 高级用法
```
输入: {...}
输出: {...}
```

## 脚本引用

- `@scripts/main.py` - 主执行脚本
- `@scripts/utils.py` - 工具函数

## 最佳实践

1. 实践建议 1
2. 实践建议 2
```

### Task 系统

Task 是**一次性的执行单元**，支持并行和分布式执行：

```typescript
// Task 定义接口
interface Task {
  // 唯一标识
  id: string;
  
  // 任务描述
  description: string;
  
  // 使用的 Skill
  skill?: string;
  
  // 输入参数 (符合 Skill 的输入 Schema)
  input: Record<string, any>;
  
  // 执行配置
  config: {
    // 超时时间 (秒)
    timeout: number;
    
    // 重试次数
    retries: number;
    
    // 是否可并行
    parallelizable: boolean;
    
    // 依赖的其他 Task
    dependsOn: string[];
    
    // 资源限制
    resources: {
      memory?: string;
      cpu?: number;
    };
  };
  
  // 状态
  status: 'pending' | 'running' | 'completed' | 'failed';
  
  // 结果
  result?: TaskResult;
  
  // 错误信息
  error?: TaskError;
}

// Task 结果
interface TaskResult {
  output: any;
  metrics: {
    startTime: Date;
    endTime: Date;
    duration: number;
    tokenUsed: number;
  };
  artifacts: string[];  // 生成的文件路径
}
```

#### Task 执行模式

1. **串行模式**: Task 按依赖顺序依次执行
2. **并行模式**: 无依赖的 Task 同时执行
3. **管道模式**: Task 输出作为下一个 Task 输入
4. **Map-Reduce 模式**: 分片处理，聚合结果

```typescript
// 并行执行示例
const tasks: Task[] = [
  {
    id: 'analyze-code',
    description: '分析代码质量',
    skill: 'code-analyzer',
    input: { files: ['src/**/*.ts'] },
    config: { timeout: 60, parallelizable: true }
  },
  {
    id: 'analyze-deps',
    description: '分析依赖安全',
    skill: 'security-scanner',
    input: { packageJson: true },
    config: { timeout: 60, parallelizable: true }
  },
  {
    id: 'generate-report',
    description: '生成综合报告',
    skill: 'report-generator',
    input: { 
      codeReport: '${analyze-code.result}',
      securityReport: '${analyze-deps.result}'
    },
    config: { 
      timeout: 30, 
      dependsOn: ['analyze-code', 'analyze-deps'] 
    }
  }
];
```

### Swarm 协调器

协调器负责任务调度、资源分配和结果聚合：

```typescript
interface SwarmCoordinator {
  // 注册 Skill
  registerSkill(skill: Skill): void;
  
  // 创建任务计划
  createPlan(objective: string): TaskPlan;
  
  // 执行任务
  execute(plan: TaskPlan): Promise<SwarmResult>;
  
  // 监控任务
  monitor(taskId: string): TaskStatus;
  
  // 取消任务
  cancel(taskId: string): void;
}

interface TaskPlan {
  objective: string;
  tasks: Task[];
  strategy: 'sequential' | 'parallel' | 'pipeline' | 'mapreduce';
  checkpoints: string[];
}
```

---

## 执行流程

```
1. 用户输入
   ↓
2. Chief Agent 分析意图
   ↓
3. 查询 Skill Registry
   ↓
4. 生成 Task Plan
   ├─ 识别可并行任务
   ├─ 识别依赖关系
   └─ 分配资源
   ↓
5. 派遣 Sub-Agents (Tasks)
   ├─ Task-1 ──→ Skill-A ──→ 结果
   ├─ Task-2 ──→ Skill-B ──→ 结果
   └─ Task-3 ──→ Skill-C ──→ 结果
   ↓
6. 收集结果
   ↓
7. Chief Agent 整合输出
   ↓
8. 返回用户
```

---

## 上下文管理

### 隔离原则

每个 Task 运行在独立的上下文中：

```typescript
interface TaskContext {
  // 任务专属的工作目录
  workspace: string;
  
  // 输入参数 (只读)
  input: Readonly<Record<string, any>>;
  
  // 可用的工具
  tools: ToolRegistry;
  
  // 可用的 Skills
  skills: SkillRegistry;
  
  // 临时存储
  temp: Map<string, any>;
  
  // 持久化存储 (跨 Task)
  store: KeyValueStore;
}
```

### 通信机制

```typescript
// 1. 输入/输出传递
Task A → output → Task B input

// 2. 共享存储
Task A → store.set('key', value)
Task B → store.get('key')

// 3. 事件广播
eventBus.emit('analysis.complete', result)
```

---

## 安全与权限

### Skill 权限模型

```yaml
permissions:
  # 文件系统
  filesystem:
    read: ['/home/user/project/**']
    write: ['/home/user/project/temp/**']
    deny: ['**/.env', '**/secrets/**']
  
  # 网络
  network:
    allow: ['api.github.com', '*.openai.com']
    deny: ['localhost', '127.0.0.1']
  
  # 命令执行
  commands:
    allowed: ['git', 'npm', 'python', 'node']
    denied: ['rm -rf /', 'sudo', 'su']
  
  # 资源限制
  resources:
    max_memory: 1GB
    max_cpu: 2
    max_disk: 10GB
```

### Task 沙箱

```typescript
interface Sandbox {
  // 创建隔离环境
  create(options: SandboxOptions): Promise<Environment>;
  
  // 在沙箱中执行命令
  exec(command: string): Promise<ExecResult>;
  
  // 清理环境
  cleanup(): Promise<void>;
}
```

---

## 扩展机制

### 自定义 Skill

```bash
# 1. 创建 Skill 目录
mkdir -p ~/.config/agents/skills/my-skill

# 2. 编写 SKILL.md
cat > ~/.config/agents/skills/my-skill/SKILL.md << 'EOF'
---
name: my-skill
version: 1.0.0
description: "我的自定义 Skill"
---

## 使用说明

这是我的自定义 Skill...
EOF

# 3. 添加脚本
mkdir -p ~/.config/agents/skills/my-skill/scripts
```

### 自定义 Task 类型

```typescript
// 继承基础 Task 类
class AnalysisTask extends BaseTask {
  async execute(context: TaskContext): Promise<TaskResult> {
    // 自定义执行逻辑
    const result = await this.runAnalysis(context.input);
    return { output: result };
  }
}

// 注册到 Swarm
swarm.registerTaskType('analysis', AnalysisTask);
```

---

## 最佳实践

### 1. Skill 设计原则

- **单一职责**: 一个 Skill 只做一件事
- **可组合**: Skill 可以调用其他 Skill
- **文档完整**: 清晰的输入输出定义
- **测试覆盖**: 提供测试用例

### 2. Task 分解原则

- **粒度适中**: 既不要太细也不要太粗
- **依赖明确**: 显式声明依赖关系
- **幂等性**: 重复执行结果一致
- **可回滚**: 失败时可以恢复

### 3. Swarm 协调原则

- **早期失败**: 发现问题立即停止
- **结果校验**: 验证每个 Task 的输出
- **超时控制**: 避免无限等待
- **资源回收**: 及时清理无用资源

---

## 实现路线图

### Phase 1: 基础架构
- [x] Skill 规范定义
- [x] Task 规范定义
- [ ] Skill Registry 实现
- [ ] Task Executor 实现

### Phase 2: 并行执行
- [ ] 依赖图解析
- [ ] 并行调度器
- [ ] 结果聚合器

### Phase 3: 智能协调
- [ ] 自动任务分解
- [ ] 动态资源分配
- [ ] 自适应重试

### Phase 4: 高级特性
- [ ] 分布式执行
- [ ] 持久化状态
- [ ] 可视化监控

---

## 相关文档

- [SKILL_SPEC.md](./SKILL_SPEC.md) - Skill 详细规范
- [TASK_SPEC.md](./TASK_SPEC.md) - Task 详细规范
- [IMPLEMENTATION.md](./IMPLEMENTATION.md) - 实现指南
