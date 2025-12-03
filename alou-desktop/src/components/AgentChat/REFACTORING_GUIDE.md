# AgentChat.jsx 重构指南

本文档说明如何使用新拆分的 hooks 来重构 `AgentChat.jsx`，使其更简洁、更易维护。

## 重构前后对比

### 重构前
`AgentChat.jsx` 有 1700+ 行代码，包含：
- 连接状态管理
- 钱包状态管理
- 消息管理
- 拖拽逻辑
- UI 状态管理
- 频道管理
- 大量的 useState、useCallback、useEffect

### 重构后
`AgentChat.jsx` 将简化为 ~500 行，主要负责：
- 组合各个 hooks
- 渲染 UI 组件
- 处理组件间的数据流

## 逐步重构方案

### 第 1 步：引入 hooks（不破坏现有代码）

```javascript
import { useAgentConnection } from './AgentChat/useAgentConnection'
import { useAgentWallet } from './AgentChat/useAgentWallet'
import { useAgentMessages } from './AgentChat/useAgentMessages'
import { useAgentDrag } from './AgentChat/useAgentDrag'
import { useAgentUI } from './AgentChat/useAgentUI'
import { useChannelManager } from './AgentChat/useChannelManager'
import { computeAgentProfile } from './AgentChat/agentUtils'
```

### 第 2 步：替换连接状态管理

**原代码（需要删除）：**
```javascript
const [connectionStatus, setConnectionStatus] = useState('disconnected')
const [sessionId, setSessionId] = useState(`frontend_${Date.now()}...`)
const [isSessionReady, setSessionReady] = useState(false)

const checkConnection = useCallback(async () => {
  // ... 很多代码
}, [])

const createSession = useCallback(async () => {
  // ... 很多代码
}, [activeChain, preferredChain])
```

**替换为：**
```javascript
const connectionState = useAgentConnection({
  activeChain,
  preferredChain,
  setPreferredChain,
})

// 解构使用
const {
  connectionStatus,
  connectionStatusLabel,
  sessionId,
  isSessionReady,
  setSessionReady,
  checkConnection,
  createSession,
} = connectionState
```

### 第 3 步：替换 UI 状态管理

**原代码（需要删除）：**
```javascript
const [isDarkMode, setIsDarkMode] = useState(false)
const [isSidebarCollapsed, setSidebarCollapsed] = useState(true)
const [isLeftSidebarCollapsed, setLeftSidebarCollapsed] = useState(false)
const [isInteractionCollapsed, setInteractionCollapsed] = useState(true)
const [isConversationVisible, setConversationVisible] = useState(false)
const [viewportWidth, setViewportWidth] = useState(window.innerWidth)
const [interactionLogs, setInteractionLogs] = useState([])
const [uiResource, setUiResource] = useState(null)
const [isUiModalOpen, setUiModalOpen] = useState(false)
const [showDiapPanel, setShowDiapPanel] = useState(true)

const recordInteraction = useCallback((action, detail, label) => {
  // ... 很多代码
}, [])

const toggleDarkMode = useCallback(() => {
  // ... 代码
}, [recordInteraction])

// ... 更多 toggle 函数
```

**替换为：**
```javascript
const uiState = useAgentUI({
  sessionId,
  viewportWidth: typeof window !== 'undefined' ? window.innerWidth : 1440,
  isLeftSidebarCollapsed: undefined, // 让 hook 管理
  isSidebarCollapsed: undefined, // 让 hook 管理
})

// 解构使用
const {
  isDarkMode,
  isSidebarCollapsed,
  isLeftSidebarCollapsed,
  isConversationVisible,
  interactionLogs,
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
} = uiState
```

### 第 4 步：替换拖拽逻辑

**原代码（需要删除）：**
```javascript
const dragStateRef = useRef({ dragging: false, offsetX: 0, offsetY: 0, moved: false })
const agentPositionRef = useRef({ x: 0, y: 0 })
const [agentPosition, setAgentPosition] = useState({ x: 0, y: 0 })

const updateAgentPosition = useCallback((next) => {
  // ...
}, [])

const clampPosition = useCallback(() => {
  // ...
}, [updateAgentPosition])

const startDrag = useCallback((event) => {
  // ... 很多代码
}, [recordInteraction])

const onDrag = useCallback((event) => {
  // ... 很多代码
}, [updateAgentPosition])

const stopDrag = useCallback((event) => {
  // ...
}, [recordInteraction])
```

**替换为：**
```javascript
const dragState = useAgentDrag({
  canvasRef,
  recordInteraction,
  openConversationPanel,
})

const {
  agentPosition,
  clampPosition,
  startDrag,
  onDrag,
  stopDrag,
  handleGlobalPointerUp,
  handleAgentActivate,
} = dragState
```

### 第 5 步：替换钱包管理

**原代码（需要删除）：**
```javascript
const [walletSnapshot, setWalletSnapshot] = useState(null)
const [userWalletInfo, setUserWalletInfo] = useState(null)
const [transactions, setTransactions] = useState(defaultTransactions)

const refreshWallet = useCallback(async () => {
  // ... 100+ 行代码
}, [preferredChain, recordInteraction])

const loadWalletOverview = useCallback(async (options = {}) => {
  // ... 100+ 行代码
}, [activeChain, preferredChain, sessionId])

const handleTransactionBuild = useCallback(async (result) => {
  // ... 100+ 行代码
}, [/* 很多依赖 */])
```

**替换为：**
```javascript
const walletState = useAgentWallet({
  sessionId,
  activeChain,
  preferredChain,
  setPreferredChain,
  recordInteraction,
  appendMessage: messageState.appendMessage, // 注意依赖顺序
  scrollToBottom: messageState.scrollToBottom,
  setTransactions: undefined, // 让 hook 管理
})

const {
  walletSnapshot,
  userWalletInfo,
  transactions,
  sidebarWallet,
  refreshWallet,
  loadWalletOverview,
  handleTransactionBuild,
  handleTransactionBroadcast,
  handleWalletChanged,
} = walletState
```

### 第 6 步：替换消息管理

**原代码（需要删除）：**
```javascript
const [messages, setMessages] = useState([])
const [currentMessage, setCurrentMessage] = useState('')
const [isLoading, setIsLoading] = useState(false)
const contextEventsRef = useRef([])

const appendMessage = useCallback((message) => {
  // ...
}, [isConversationVisible])

const scrollToBottom = useCallback(() => {
  // ...
}, [])

const sendMessage = useCallback(async () => {
  // ... 100+ 行代码
}, [/* 很多依赖 */])
```

**替换为：**
```javascript
const messageState = useAgentMessages({
  sessionId,
  isSessionReady,
  createSession,
  setSessionReady,
  activeChain,
  recordInteraction,
  handleToolCalls, // 需要定义
  conversationOverlayRef,
  consoleDockRef,
  isConversationVisible: uiState.isConversationVisible,
  setConversationVisible: uiState.setConversationVisible,
})

const {
  messages,
  currentMessage,
  setCurrentMessage,
  isLoading,
  appendMessage,
  scrollToBottom,
  sendMessage,
} = messageState
```

### 第 7 步：保持现有的频道管理（已拆分）

频道管理已经通过 `useChannelManager` 拆分，保持不变或微调：

```javascript
const channelManager = useChannelManager({
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
  openConversationPanel: uiState.openConversationPanel,
})
```

## 重构后的 AgentChat.jsx 结构示例

```javascript
const AgentChat = () => {
  const navigate = useNavigate()
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const logout = useAuthStore((state) => state.logout)
  const { t, initLanguage, setLanguage, currentLanguage } = useI18n()

  // Refs
  const canvasRef = useRef(null)
  const conversationOverlayRef = useRef(null)
  const consoleDockRef = useRef(null)

  // Local state that needs to be shared
  const [preferredChain, setPreferredChain] = useState(null)
  const [channelKeyword, setChannelKeyword] = useState('')
  const [channels, setChannels] = useState([])
  const [activeChannelId, setActiveChannelId] = useState(null)
  const [selectedAgent, setSelectedAgent] = useState(null)
  const [isChannelLoading, setChannelLoading] = useState(false)
  const [channelError, setChannelError] = useState(null)
  const [selectedModelType, setSelectedModelType] = useState(null)
  const [isCreateAgentModalOpen, setCreateAgentModalOpen] = useState(false)

  // 1. UI State Hook
  const uiState = useAgentUI({
    sessionId: null, // Will be set after connection
    viewportWidth: typeof window !== 'undefined' ? window.innerWidth : 1440,
  })

  // 2. Connection State Hook
  const connectionState = useAgentConnection({
    activeChain: null,
    preferredChain,
    setPreferredChain,
  })

  // 3. Drag State Hook
  const dragState = useAgentDrag({
    canvasRef,
    recordInteraction: uiState.recordInteraction,
    openConversationPanel: uiState.openConversationPanel,
  })

  // 4. Message State Hook (depends on UI and Connection)
  const messageState = useAgentMessages({
    sessionId: connectionState.sessionId,
    isSessionReady: connectionState.isSessionReady,
    createSession: connectionState.createSession,
    setSessionReady: connectionState.setSessionReady,
    activeChain: null,
    recordInteraction: uiState.recordInteraction,
    handleToolCalls: () => {}, // Define this
    conversationOverlayRef,
    consoleDockRef,
    isConversationVisible: uiState.isConversationVisible,
    setConversationVisible: uiState.setConversationVisible,
  })

  // 5. Wallet State Hook (depends on Message)
  const walletState = useAgentWallet({
    sessionId: connectionState.sessionId,
    activeChain: null,
    preferredChain,
    setPreferredChain,
    recordInteraction: uiState.recordInteraction,
    appendMessage: messageState.appendMessage,
    scrollToBottom: messageState.scrollToBottom,
    setTransactions: undefined,
  })

  // 6. Channel Manager Hook
  const channelManager = useChannelManager({
    sessionId: connectionState.sessionId,
    isSessionReady: connectionState.isSessionReady,
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
    recordInteraction: uiState.recordInteraction,
    openConversationPanel: uiState.openConversationPanel,
  })

  // Computed values
  const agentProfile = useMemo(
    () => computeAgentProfile(selectedAgent),
    [selectedAgent]
  )

  const agentStyle = useMemo(
    () => ({
      transform: `translate(calc(-50% + ${dragState.agentPosition.x}px), calc(-50% + ${dragState.agentPosition.y}px))`,
    }),
    [dragState.agentPosition],
  )

  // Event handlers
  const goToLogin = useCallback(() => {
    uiState.recordInteraction('navigate_login')
    navigate('/login')
  }, [navigate, uiState.recordInteraction])

  // ... other handlers

  // Effects
  useEffect(() => {
    // Bootstrap
    const bootstrap = async () => {
      try {
        initLanguage()
        await Promise.all([
          connectionState.checkConnection(),
          connectionState.createSession().then(() => connectionState.setSessionReady(true)),
          walletState.refreshWallet().catch(console.warn),
        ])
      } catch (error) {
        console.error('Error in AgentChat initialization:', error)
      }
    }
    bootstrap()

    // Event listeners
    if (typeof window !== 'undefined') {
      window.addEventListener('resize', uiState.handleResize)
      window.addEventListener('pointerup', dragState.handleGlobalPointerUp)
    }

    return () => {
      if (typeof window !== 'undefined') {
        window.removeEventListener('resize', uiState.handleResize)
        window.removeEventListener('pointerup', dragState.handleGlobalPointerUp)
      }
    }
  }, [])

  // Render
  return (
    <div className={`app-shell ${uiState.isDarkMode ? 'dark-mode' : ''}`}>
      <ChatHeader
        connectionStatus={connectionState.connectionStatus}
        isDarkMode={uiState.isDarkMode}
        isAuthenticated={isAuthenticated}
        onToggleTheme={uiState.toggleDarkMode}
        onGoToLogin={goToLogin}
        onLogout={logout}
      />
      
      {/* ... rest of the UI */}
    </div>
  )
}
```

## 重构步骤总结

1. ✅ **创建所有 hooks** - 已完成
2. ⚠️ **逐步替换** - 从最独立的模块开始（UI、Connection、Drag）
3. ⚠️ **处理依赖** - 注意 hooks 之间的依赖顺序
4. ⚠️ **测试功能** - 每替换一个模块就测试一次
5. ⚠️ **清理代码** - 删除已替换的旧代码
6. ⚠️ **更新文档** - 更新相关文档和注释

## 注意事项

### Hook 调用顺序很重要
```javascript
// ❌ 错误：messageState 需要 recordInteraction，但 uiState 还没定义
const messageState = useAgentMessages({ recordInteraction })
const uiState = useAgentUI({})

// ✅ 正确：先定义 uiState
const uiState = useAgentUI({})
const messageState = useAgentMessages({ 
  recordInteraction: uiState.recordInteraction 
})
```

### 避免循环依赖
```javascript
// ❌ 错误：walletState 依赖 messageState，messageState 又依赖 walletState
const walletState = useAgentWallet({ 
  appendMessage: messageState.appendMessage 
})
const messageState = useAgentMessages({ 
  handleToolCalls: walletState.handleTransactionBuild 
})

// ✅ 正确：通过回调函数解耦
const messageState = useAgentMessages({ 
  handleToolCalls: (toolCalls) => walletState.handleTransactionBuild(toolCalls)
})
```

### 保持状态同步
某些状态可能需要在多个 hooks 之间共享，可以：
1. 在父组件中管理，传递给多个 hooks
2. 使用 Context API
3. 使用状态管理库（Zustand, Redux）

## 测试清单

重构后需要测试的功能：
- [ ] 页面加载和初始化
- [ ] 连接状态显示
- [ ] 钱包连接和余额显示
- [ ] 发送消息
- [ ] 拖拽智能体节点
- [ ] 切换暗黑模式
- [ ] 侧边栏折叠/展开
- [ ] 频道列表加载和搜索
- [ ] 选择频道
- [ ] 创建新智能体
- [ ] 交易构建和广播
- [ ] MCP UI 资源显示
- [ ] DIAP 身份面板

## 下一步

1. 先不修改原 `AgentChat.jsx`，保持它能正常工作
2. 创建一个新文件 `AgentChat.refactored.jsx` 进行实验
3. 逐步迁移和测试
4. 确认无误后，替换原文件
5. 提交到 git，保留历史记录

## 参考资料

- [React Hooks 官方文档](https://react.dev/reference/react)
- [自定义 Hooks 最佳实践](https://react.dev/learn/reusing-logic-with-custom-hooks)
- [Hooks 常见问题](https://react.dev/reference/react/hooks#troubleshooting)

