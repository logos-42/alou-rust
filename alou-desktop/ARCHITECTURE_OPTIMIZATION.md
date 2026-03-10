# 架构优化方案 - 解决输入时容器闪烁问题

## 问题分析

### 当前架构问题

```
用户输入 → currentMessage 状态更新 → AgentChat 重新渲染
                                      ↓
                              useAgentMessages 返回值变化
                                      ↓
                              AgentConversationOverlay props 变化
                                      ↓
                              MessageList props 变化
                                      ↓
                              所有 MessageItem 重新渲染
                                      ↓
                              容器高度变化 → 闪烁/抖动
```

**核心问题**：
1. 输入框状态和消息状态耦合在同一个组件树中
2. 每次输入都触发整个消息列表重新渲染
3. 消息内容格式化在父组件中进行，导致子组件无法 memo 优化

## 架构优化方案

### 1. 分离输入框和消息列表的渲染上下文

```
┌─────────────────────────────────────────────────────────┐
│ AgentChat (父组件)                                      │
│  ┌───────────────────┐    ┌─────────────────────────┐   │
│  │ InputContext      │    │ MessageContext          │   │
│  │ - currentMessage  │    │ - messages              │   │
│  │ - isLoading       │    │ - isLoading             │   │
│  └───────────────────┘    └─────────────────────────┘   │
│           ↓                        ↓                     │
│  ┌───────────────────┐    ┌─────────────────────────┐   │
│  │ AgentConsoleDock  │    │ AgentConversationOverlay│   │
│  │ (输入框组件)       │    │ (消息容器)              │   │
│  │                   │    │                         │   │
│  │ 只响应            │    │ 只响应                  │   │
│  │ currentMessage    │    │ messages                │   │
│  │ 变化              │    │ 变化                    │   │
│  └───────────────────┘    └─────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 2. 消息组件独立订阅

- 每条消息组件订阅自己的数据
- 只有相关消息变化时才重新渲染
- 使用 `React.memo` + 稳定的 props

### 3. 状态更新批处理

- 使用 `unstable_batchedUpdates` 批处理状态更新
- 避免多次渲染

### 4. 虚拟滚动（可选）

- 消息数量多时使用虚拟滚动
- 只渲染可见区域的消息

## 实施步骤

### 第一步：创建独立的 Context

1. `MessageContext` - 消息数据
2. `InputContext` - 输入框数据

### 第二步：重构组件

1. `AgentChat` - 提供 Context
2. `AgentConsoleDock` - 订阅 `InputContext`
3. `AgentConversationOverlay` - 订阅 `MessageContext`
4. `MessageList` - 使用 `React.memo` 优化
5. `MessageItem` - 内部计算格式化值

### 第三步：优化状态更新

1. 移除不必要的状态同步
2. 使用函数式更新
3. 批处理相关状态更新

## 预期效果

- 输入时只有输入框重新渲染
- 消息列表不受输入影响
- AI 回复时只有新消息重新渲染
- 容器不再上下抖动
