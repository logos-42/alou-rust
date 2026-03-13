# Agent Runtime v1 架构文档

## 🎯 概述

Alou Desktop 现已实现 **Agent Runtime v1**,从一个简单的聊天应用升级为真正的 **AI Agent 运行平台**.

## 🏗️ 完整架构

```
SessionActor (Actor 模型，无锁)
  ↓
SessionRuntime
  ├── AgentState (认知循环)
  ├── MemoryState (记忆管理)
  ├── WorkflowClient (外部调用)
  ├── HookManager (生命周期) ← 新增
  ├── CommandQueue (主动控制) ← 新增
  └── EventLog (日志/指标) ← 新增
  ↓
外部系统
  ├── LLM Providers (DeepSeek/Claude)
  ├── Tool Sandbox (安全执行)
  └── Scheduler (多任务调度)
```

## 📦 核心组件

### 1. Hook 系统
- 事件总线式设计
- 支持 Continue/Skip/Abort
- 内置：LoggingHook, MetricsHook, SafetyHook, PromptInjectionHook

### 2. Command 系统
- 20+ 命令类型
- 支持优先级队列
- 流程控制、消息注入、工具控制、模型控制

### 3. EventLog 系统 (新增)
- Debug/Replay: 逐步回放 Agent 执行
- Metrics: LLM 延迟、工具延迟、循环次数
- AI Training: 导出会话为训练集

### 4. Scheduler (骨架)
- 多任务调度
- 优先级管理
- 资源配额控制

### 5. Tool Sandbox (骨架)
- 工具隔离
- 资源限制
- 安全政策

## 📊 代码统计

| 模块 | 文件 | 行数 | 状态 |
|------|------|------|------|
| Hook | runtime/hook.rs | 454 | ✅ |
| Command | runtime/command.rs | 297 | ✅ |
| EventLog | runtime/event_log.rs | 550 | ✅ |
| Scheduler | scheduler/mod.rs | 200 | 🚧 |
| Sandbox | sandbox/mod.rs | 250 | 🚧 |
| **总计** | | **1751** | |

## 🚀 对比

| 特性 | 普通聊天 | Alou v1 | AutoGPT |
|------|---------|---------|---------|
| Hook 系统 | ❌ | ✅ | ✅ |
| Command | ❌ | ✅ | ✅ |
| EventLog | ❌ | ✅ | ❌ |
| Scheduler | ❌ | 🚧 | ✅ |
| Sandbox | ❌ | 🚧 | ✅ |

## 📚 文档

- `docs/AGENT_RUNTIME.md` - 使用文档
- `HOOK_COMMAND_IMPLEMENTATION.md` - 实现总结
- `AGENT_RUNTIME_V1.md` - 架构文档 (本文件)
