# 异步任务支持 - 桌面版前端

## 概述

桌面版前端现已支持AI API的异步任务处理。后端会自动处理长时间运行的AI任务，前端无需关心同步/异步的区别。

## 主要修改

### 1. AgentService 增强

`src/services/agentService.js` 中的 `sendMessage` 方法已增强：

```javascript
// 现有的调用方式（保持不变）
const result = await agentService.sendMessage(sessionId, message, walletAddress)

// 新的调用方式（支持选项）
const result = await agentService.sendMessage(sessionId, message, walletAddress, {
  chain: 'ethereum',           // 链名称
  contextEvents: [],           // 上下文事件
  eventSummary: '',            // 事件摘要
  useAsync: true,              // 是否使用异步处理（默认true）
  timeout: 30000               // 超时时间（毫秒）
})
```

### 2. 新增功能

#### 2.1 自动异步处理
- 当 `useAsync: true`（默认）时，系统会尝试使用后端的异步任务接口
- 如果异步接口不可用，会自动回退到同步接口
- 前端调用方式完全一致，无需修改现有代码

#### 2.2 任务状态查询（可选）
```javascript
// 获取任务状态
const status = await agentService.getTaskStatus(taskId)

// 等待任务完成
const result = await agentService.waitForTaskCompletion(taskId, 2000, 60000)
```

### 3. 响应格式

无论使用同步还是异步处理，返回格式都保持一致：

```javascript
{
  content: "AI回复内容",
  session_id: "会话ID",
  // 如果是异步任务，还会包含以下字段
  task_id: "任务ID",
  is_async: true,
  status: "running", // pending, running, completed, failed, cancelled
  progress: 50,      // 进度百分比
  current_step: "正在处理..." // 当前步骤描述
}
```

## 向后兼容性

### 完全兼容现有代码
- 所有现有的 `agentService.sendMessage()` 调用都继续工作
- 不需要修改任何现有组件
- 返回格式保持一致

### 渐进式增强
- 默认启用异步处理，但会优雅降级
- 新增功能都是可选的，不影响核心功能

## 使用示例

### 示例1：现有代码（无需修改）
```javascript
// 现有的代码继续工作
const response = await agentService.sendMessage(
  'session-123',
  '你好，请帮我分析这个交易',
  '0x1234...'
)
console.log(response.content)
```

### 示例2：启用异步处理
```javascript
// 明确启用异步处理
const response = await agentService.sendMessage(
  'session-456',
  '这是一个复杂的分析请求，需要较长时间',
  '0x5678...',
  {
    useAsync: true,
    timeout: 60000 // 60秒超时
  }
)

if (response.is_async && response.task_id) {
  console.log(`任务已创建: ${response.task_id}`)
  console.log(`当前状态: ${response.status}, 进度: ${response.progress}%`)
}
```

### 示例3：监控任务进度（可选）
```javascript
// 如果需要监控任务进度
const response = await agentService.sendMessage(...)

if (response.is_async && response.task_id) {
  // 定期检查状态
  const checkInterval = setInterval(async () => {
    const status = await agentService.getTaskStatus(response.task_id)
    console.log(`进度: ${status.progress}%, 步骤: ${status.current_step}`)
    
    if (status.status === 'completed') {
      clearInterval(checkInterval)
      console.log('任务完成:', status.result)
    } else if (status.status === 'failed') {
      clearInterval(checkInterval)
      console.error('任务失败:', status.error)
    }
  }, 2000)
}
```

## 错误处理

### 自动错误恢复
- 异步接口失败时自动回退到同步接口
- 网络错误时提供友好的错误信息
- 超时处理机制

### 错误示例
```javascript
try {
  const response = await agentService.sendMessage(...)
} catch (error) {
  if (error.message.includes('timeout')) {
    console.error('请求超时，请稍后重试')
  } else if (error.message.includes('不可用')) {
    console.error('AI服务暂时不可用')
  } else {
    console.error('未知错误:', error.message)
  }
}
```

## 部署说明

### 1. 前端部署
- 无需特殊配置
- 现有构建流程不变
- 兼容所有现有环境

### 2. 后端要求
- 需要支持 `/ai-task/init-and-start` 接口
- 需要支持 `/ai-task/{taskId}/status` 接口
- 如果后端不支持异步接口，前端会自动回退

### 3. 环境变量
无需新增环境变量，现有配置继续有效。

## 测试

已创建测试文件 `src/test-async-support.js`，可用于验证功能：

```javascript
import testAsyncSupport from './test-async-support'
testAsyncSupport()
```

## 总结

桌面版前端现已完全支持AI API的异步任务处理，主要特点：

1. **无缝集成** - 现有代码无需修改
2. **自动降级** - 异步失败时自动回退到同步
3. **渐进增强** - 新增功能都是可选的
4. **向后兼容** - 完全兼容现有系统
5. **易于使用** - API保持简洁一致

通过这种方式，前端可以充分利用后端的异步处理能力，同时保持代码的简洁和可维护性。
