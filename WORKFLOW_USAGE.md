# Alou 工作流系统使用文档

## 概述

新的工作流系统直接在桌面版实现，不再依赖外部 Kilo Code CLI 工具。提供三种工作流模式：

1. **Interactive** - 交互式模式：逐步引导 AI 完成任务
2. **Auto** - 自动模式：自动分析并执行任务
3. **Parallel** - 并行模式：拆分任务并行处理

## 核心组件

### 1. WorkflowService (`src/services/workflowService.js`)

核心工作流服务类，提供三种工作流模式的实现。

**使用方法：**

```javascript
import { WorkflowService, WorkflowMode } from '@/services/workflowService'

const workflowService = new WorkflowService({
  sendMessage: async (message) => {
    // 发送消息到 AI 提供商
    return await api.post('/agent/chat', { prompt: message.content })
  },
  handleToolCalls: handleToolCalls, // 你的工具调用处理器
  appendMessage: appendMessage,     // 添加消息到界面
  options: {
    maxSteps: 10,
    timeout: 30000,
    verbose: false
  }
})

// 执行交互式工作流
const result = await workflowService.interactiveWorkflow('创建一个 React 组件')

// 执行自动工作流
const result = await workflowService.autoWorkflow('优化这段代码', { tools: [...] })

// 执行并行工作流
const result = await workflowService.parallelWorkflow([
  '创建用户组件',
  '创建按钮组件',
  '创建表单组件'
])

// 智能选择模式
const result = await workflowService.smartWorkflow('复杂任务描述')
```

### 2. useAlouWorkflow Hook (`src/components/AgentChat/hooks/useAlouWorkflow.js`)

React Hook，方便在组件中使用工作流。

**使用方法：**

```javascript
import { useAlouWorkflow, WorkflowMode } from '@/components/AgentChat/hooks/useAlouWorkflow'

function MyComponent() {
  const {
    isInitialized,
    isProcessing,
    currentMode,
    progress,
    WorkflowMode,
    executeInteractive,
    executeAuto,
    executeParallel,
    executeSmart,
    cancelTask
  } = useAlouWorkflow({
    sendMessage: sendMessage,
    handleToolCalls: handleToolCalls,
    appendMessage: appendMessage,
    scrollToBottom: scrollToBottom,
    options: {}
  })

  const handleTask = async () => {
    // 使用智能模式（自动选择最佳模式）
    const result = await executeSmart('创建一个完整的 React 应用')
    
    // 或指定特定模式
    // await executeInteractive('逐步完成一个复杂任务')
    // await executeAuto('自动执行一个简单任务')
    // await executeParallel(['任务1', '任务2', '任务3'])
  }

  return (
    <div>
      {isProcessing && (
        <div>
          处理中: {progress?.stage} {progress?.step && `${progress.step}/${progress.total}`}
        </div>
      )}
      <button onClick={handleTask} disabled={isProcessing || !isInitialized}>
        执行任务
      </button>
      <button onClick={cancelTask} disabled={!isProcessing}>
        取消
      </button>
    </div>
  )
}
```

### 3. workflowPrompts (`src/components/AgentChat/utils/workflowPrompts.js`)

工作流提示词生成器，为不同模式生成专门的系统提示词。

## 工作流模式详解

### 1. Interactive 模式（交互式）

**适用场景：**
- 复杂的多步骤任务
- 需要详细规划的任务
- 需要用户确认的任务

**执行流程：**
1. **任务规划**：AI 分析任务并制定执行计划
2. **逐步执行**：按照计划逐步执行每个步骤
3. **结果验证**：验证每个步骤的结果
4. **总结反馈**：汇总所有步骤的成果

**特点：**
- 清晰的执行步骤
- 易于控制和调试
- 适合需要精细控制的场景

### 2. Auto 模式（自动式）

**适用场景：**
- 简单明确的任务
- 工具调用明确的任务
- 无需人工干预的任务

**执行流程：**
1. **任务分析**：AI 快速理解任务需求
2. **工具选择**：自动选择最合适的工具
3. **执行操作**：调用工具完成任务
4. **验证结果**：检查任务完成情况

**特点：**
- 快速高效
- 无需人工干预
- 适合重复性任务

### 3. Parallel 模式（并行式）

**适用场景：**
- 可拆分的独立任务
- 批量处理任务
- 需要快速完成的多个任务

**执行流程：**
1. **任务拆分**：将任务拆分为多个子任务
2. **并行执行**：同时执行多个子任务
3. **结果合并**：整合所有子任务的结果
4. **总结反馈**：提供完整的合并结果

**特点：**
- 高效率
- 适合独立任务
- 批量处理能力

## 在 AgentChat 中的使用

工作流系统已集成到 `useAgentMessages` 中，自动处理任务：

```javascript
// useAgentMessages.js

const {
  isWorkflowInitialized,
  executeSmart
} = useAlouWorkflow({...})

// 发送消息时自动使用工作流
const sendMessageToAgent = async (agentId, text) => {
  // ...
  
  // 如果工作流已初始化，使用智能模式处理
  if (isWorkflowInitialized) {
    await executeSmart(text)
    return
  }
  
  // 否则使用标准流程
  // ...
}
```

## 配置选项

### WorkflowService 配置

```javascript
{
  maxSteps: 10,        // 最大执行步骤数（Interactive模式）
  timeout: 30000,      // 超时时间（毫秒）
  verbose: false,      // 是否输出详细日志
  maxConcurrent: 3     // 最大并行任务数（Parallel模式）
}
```

### useAlouWorkflow 配置

```javascript
{
  sendMessage: async (message) => { /* ... */ },
  handleToolCalls: async (toolCalls) => { /* ... */ },
  appendMessage: (message, channelId) => { /* ... */ },
  scrollToBottom: () => { /* ... */ },
  options: {
    maxSteps: 10,
    timeout: 30000,
    verbose: false
  }
}
```

## 优势

1. **无外部依赖**：不再依赖 Kilo Code CLI，减少安装和配置复杂度
2. **完全集成**：直接使用现有的 AgentChat 工具调用系统
3. **灵活可控**：三种模式可选，智能模式自动选择最佳策略
4. **易于扩展**：基于现有的架构，容易添加新功能
5. **更好的错误处理**：集成到现有的错误处理机制中

## 迁移指南

从旧版 Kilo Code CLI 迁移到新工作流系统：

### 旧版（Kilo Code CLI）

```javascript
import { useCliWorkflow } from './hooks/useCliWorkflow'

const { processWithCliWorkflow } = useCliWorkflow({...})

await processWithCliWorkflow(text, agentId, agentInfo)
```

### 新版（Alou 工作流）

```javascript
import { useAlouWorkflow } from './hooks/useAlouWorkflow'

const { executeSmart } = useAlouWorkflow({
  sendMessage: sendMessage,
  handleToolCalls: handleToolCalls,
  appendMessage: appendMessage,
  scrollToBottom: scrollToBottom
})

await executeSmart(text)
```

主要变化：
- 不再需要 `agentId` 和 `agentInfo` 参数
- 需要显式提供 `sendMessage`、`handleToolCalls` 等函数
- 工作流服务自动处理消息和工具调用

## 故障排除

### 问题1: 工作流未初始化

**症状**: `isInitialized` 始终为 `false`

**解决**: 确保正确传递了所有必需的回调函数（sendMessage、handleToolCalls、appendMessage）

### 问题2: 任务执行失败

**症状**: 工作流执行报错

**解决**: 
- 检查 `sendMessage` 函数是否正确返回响应
- 检查 `handleToolCalls` 是否能正确处理工具调用
- 查看控制台日志获取详细错误信息

### 问题3: 进度不更新

**症状**: `progress` 状态不变化

**解决**: 确保在 `onProgress` 回调中调用了 `scrollToBottom?.()`

## 未来扩展

工作流系统可以轻松扩展：

1. **添加新模式**：继承 `WorkflowService` 类，添加新的工作流模式
2. **自定义提示词**：修改 `workflowPrompts.js` 中的提示词模板
3. **集成更多工具**：在 `handleToolCalls` 中添加新的工具处理器
4. **添加监控**：在工作流执行过程中添加性能监控和日志记录

## 相关文件

- `src/services/workflowService.js` - 核心工作流服务
- `src/components/AgentChat/hooks/useAlouWorkflow.js` - React Hook
- `src/components/AgentChat/utils/workflowPrompts.js` - 提示词生成器
- `src/components/AgentChat/utils/workflowUtils.js` - 工作流工具函数
- `src/components/AgentChat/useAgentMessages.js` - 集成到消息系统
