# AgentChat 组件结构

这个文件夹包含了从大型单体组件 `AgentChat.jsx` 拆分出来的功能模块。

## 文件结构

### 工具函数和常量

- **`agentUtils.js`** - 工具函数集合
  - `resolveAgentAvatar` - 解析智能体头像
  - `buildChannelFromAgent` - 从智能体数据构建频道对象
  - `extractAgentTarget` - 提取智能体目标标识
  - `extractErrorMessage` - 提取错误信息
  - `computeAgentProfile` - 计算智能体配置文件

- **`agentConstants.js`** - 常量定义

### 自定义 Hooks

#### `useAgentConnection.js` - 连接状态管理
管理智能体的连接状态和会话：
- 连接状态管理（connected/connecting/disconnected/error）
- Session 创建和管理
- 健康检查

```javascript
const {
  connectionStatus,
  sessionId,
  isSessionReady,
  checkConnection,
  createSession,
} = useAgentConnection({ activeChain, preferredChain, setPreferredChain })
```

#### `useAgentWallet.js` - 钱包管理
管理钱包状态和交易：
- 钱包快照和用户钱包信息
- 交易列表
- 刷新钱包余额
- 加载钱包概览
- 处理交易构建和广播
- 监听链切换事件

```javascript
const {
  walletSnapshot,
  userWalletInfo,
  transactions,
  sidebarWallet,
  refreshWallet,
  loadWalletOverview,
  handleTransactionBuild,
  handleTransactionBroadcast,
} = useAgentWallet({
  sessionId,
  activeChain,
  preferredChain,
  setPreferredChain,
  recordInteraction,
  appendMessage,
  scrollToBottom,
  setTransactions,
})
```

#### `useAgentMessages.js` - 消息和对话管理
管理消息列表和对话流程：
- 消息列表状态
- 当前输入消息
- 加载状态
- 发送消息
- 滚动到底部
- 上下文事件记录

```javascript
const {
  messages,
  currentMessage,
  setCurrentMessage,
  isLoading,
  appendMessage,
  scrollToBottom,
  sendMessage,
} = useAgentMessages({
  sessionId,
  isSessionReady,
  createSession,
  setSessionReady,
  activeChain,
  recordInteraction,
  handleToolCalls,
  conversationOverlayRef,
  consoleDockRef,
  isConversationVisible,
  setConversationVisible,
})
```

#### `useAgentDrag.js` - 拖拽功能管理
管理智能体节点的拖拽交互：
- 拖拽状态管理
- 位置计算和限制
- 拖拽事件处理
- 激活智能体面板

```javascript
const {
  agentPosition,
  clampPosition,
  startDrag,
  onDrag,
  stopDrag,
  handleGlobalPointerUp,
  handleAgentActivate,
} = useAgentDrag({ canvasRef, recordInteraction, openConversationPanel })
```

#### `useAgentUI.js` - UI 状态管理
管理 UI 相关状态和交互：
- 暗黑模式
- 侧边栏折叠状态
- 对话面板可见性
- 视口宽度
- 交互日志
- MCP UI 资源管理
- DIAP 面板控制

```javascript
const {
  isDarkMode,
  isSidebarCollapsed,
  isLeftSidebarCollapsed,
  isConversationVisible,
  interactionLogs,
  uiResource,
  isUiModalOpen,
  showDiapPanel,
  recordInteraction,
  openUiResource,
  fetchAndOpenUiResource,
  closeUiResource,
  handleUiAction,
  toggleSidebar,
  toggleDarkMode,
  toggleLeftSidebar,
  openConversationPanel,
  closeConversationPanel,
  consoleDockStyle,
} = useAgentUI({ sessionId, viewportWidth, isLeftSidebarCollapsed, isSidebarCollapsed })
```

#### `useChannelManager.js` - 频道管理（已存在）
管理频道列表和智能体解析：
- 加载频道列表
- 搜索频道
- 选择频道
- 解析智能体目标

```javascript
const {
  loadChannelList,
  refreshChannels,
  handleChannelKeywordChange,
  selectChannel,
  resolveExistingAgentTarget,
} = useChannelManager({
  sessionId,
  isSessionReady,
  channelKeyword,
  setChannelKeyword,
  channels,
  setChannels,
  activeChannelId,
  setActiveChannelId,
  selectedAgent,
  setSelectedAgent,
  isChannelLoading,
  setChannelLoading,
  channelError,
  setChannelError,
  recordInteraction,
  openConversationPanel,
})
```

### 其他文件

- **`AgentChatContext.jsx`** - Context 定义（可选，用于未来扩展）
- **`AgentChatProvider.jsx`** - Context Provider 组件（可选）
- **`useAgentPersistence.js`** - 持久化逻辑
- **`index.css`** - 样式文件（从 `AgentChat.css` 移动）
- **`index.js`** - 导出文件

## 使用示例

在主组件 `AgentChat.jsx` 中使用这些 hooks：

```javascript
import React, { useRef, useMemo } from 'react'
import { useAgentConnection } from './AgentChat/useAgentConnection'
import { useAgentWallet } from './AgentChat/useAgentWallet'
import { useAgentMessages } from './AgentChat/useAgentMessages'
import { useAgentDrag } from './AgentChat/useAgentDrag'
import { useAgentUI } from './AgentChat/useAgentUI'
import { useChannelManager } from './AgentChat/useChannelManager'
import { computeAgentProfile } from './AgentChat/agentUtils'

const AgentChat = () => {
  // Refs
  const canvasRef = useRef(null)
  const conversationOverlayRef = useRef(null)
  const consoleDockRef = useRef(null)

  // UI State
  const uiState = useAgentUI({ sessionId: null, viewportWidth: window.innerWidth })
  
  // Connection State
  const connectionState = useAgentConnection({
    activeChain: null,
    preferredChain: null,
    setPreferredChain: () => {},
  })
  
  // Wallet State
  const walletState = useAgentWallet({
    sessionId: connectionState.sessionId,
    activeChain: null,
    preferredChain: null,
    setPreferredChain: () => {},
    recordInteraction: uiState.recordInteraction,
    appendMessage: () => {},
    scrollToBottom: () => {},
    setTransactions: () => {},
  })
  
  // Message State
  const messageState = useAgentMessages({
    sessionId: connectionState.sessionId,
    isSessionReady: connectionState.isSessionReady,
    createSession: connectionState.createSession,
    setSessionReady: connectionState.setSessionReady,
    activeChain: null,
    recordInteraction: uiState.recordInteraction,
    handleToolCalls: () => {},
    conversationOverlayRef,
    consoleDockRef,
    isConversationVisible: uiState.isConversationVisible,
    setConversationVisible: uiState.setConversationVisible,
  })
  
  // Drag State
  const dragState = useAgentDrag({
    canvasRef,
    recordInteraction: uiState.recordInteraction,
    openConversationPanel: uiState.openConversationPanel,
  })
  
  // Compute agent profile
  const agentProfile = useMemo(() => computeAgentProfile(null), [])
  
  return (
    <div className="app-shell">
      {/* Your UI components here */}
    </div>
  )
}

export default AgentChat
```

## 架构优势

### 1. 关注点分离
每个 hook 专注于单一职责：
- **Connection** - 连接和会话
- **Wallet** - 钱包和交易
- **Messages** - 消息和对话
- **Drag** - 拖拽交互
- **UI** - 界面状态

### 2. 可测试性
每个 hook 可以独立测试，不需要渲染整个组件。

### 3. 可复用性
Hooks 可以在其他组件中复用，实现逻辑共享。

### 4. 可维护性
- 代码结构清晰，易于定位问题
- 修改某个功能不会影响其他功能
- 新增功能只需添加新的 hook

### 5. 性能优化
- 使用 `useCallback` 和 `useMemo` 避免不必要的重渲染
- 状态更新粒度更细，减少连锁反应

## 开发指南

### 添加新功能
1. 评估功能属于哪个 hook
2. 如果是新的关注点，创建新的 hook
3. 在 `index.js` 中导出
4. 更新 README.md 文档

### 修改现有功能
1. 找到对应的 hook 文件
2. 修改逻辑
3. 确保返回值接口不变（或更新所有使用处）
4. 更新文档

### 性能优化建议
- 使用 `useCallback` 包装函数，避免子组件重渲染
- 使用 `useMemo` 缓存计算结果
- 合理使用 `useRef` 存储不需要触发渲染的值
- 避免在 hook 中创建过多的状态

## 未来扩展

可以根据需要进一步拆分或添加：
- `useAgentTools.js` - 工具调用管理
- `useAgentStream.js` - 流式响应处理（已存在于 `@/hooks`）
- `useAgentMCP.js` - MCP 协议管理
- `useAgentDIAP.js` - DIAP 身份管理

## 相关文档

- [React Hooks 官方文档](https://react.dev/reference/react)
- [自定义 Hooks 最佳实践](https://react.dev/learn/reusing-logic-with-custom-hooks)
