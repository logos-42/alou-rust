# Alou Desktop 代码存档

**存档日期**: 2025-01-27  
**项目版本**: alou-desktop v0.1.2  
**存档原因**: 记录当前代码状态和待实现功能

---

## 📋 项目概览

### 项目名称
**Alou Desktop** - Web3 AI Agent 桌面应用

### 技术栈
- **前端框架**: React 18 + Vite
- **桌面框架**: Tauri 2.0
- **状态管理**: Zustand
- **路由**: React Router
- **Web3**: Ethers.js
- **钱包连接**: WalletConnect 2.0
- **IPFS**: Kubo (本地节点)

---

## ✅ 已完成功能

### 1. 核心智能体系统
- ✅ 多智能体频道管理 (`AgentChat.jsx`, `useChannelManager.js`)
- ✅ 智能体创建和解析 (`CreateAgentModal.jsx`, `agentService.js`)
- ✅ DIAP 身份集成 (`diapService.js`, `DiapIdentityPanel.jsx`)
- ✅ IPFS/IPNS 解析支持 (`agentService.js::loadAgentFromIpns`)
- ✅ 本地存储持久化 (`agentStore.js`)

### 2. 消息和对话系统
- ✅ 按频道分开存储消息 (`useAgentMessages.js`)
- ✅ 多智能体独立执行空间（部分实现）
  - `loadingByAgent` 状态管理
  - `sessionsByAgent` 会话隔离
- ✅ IPFS 消息持久化 (`saveMessagesToIpfs`, `loadMessagesFromIpfs`)
- ✅ 流式响应支持 (`useAgentStream`)

### 3. 多智能体协调器
- ✅ 智能体注册和注销 (`agentCoordinatorService.js`)
- ✅ 消息路由机制 (`routeMessage`, `sendAgentMessage`)
- ✅ PubSub 消息服务 (`pubsubService.js`)
- ✅ 多智能体聊天 Hook (`useMultiAgentChat.js`)

### 4. UI 组件
- ✅ 智能体画布 (`AgentCanvas.jsx`)
- ✅ 对话覆盖层 (`AgentConversationOverlay.jsx`)
- ✅ 控制台底座 (`AgentConsoleDock.jsx`)
- ✅ 侧边栏 (`AgentSidebarLeft.jsx`, `AgentSidebarRight.jsx`)
- ✅ 智能体资料面板 (`AgentProfilePanel.jsx`)

---

## 🔴 待实现功能（已知问题）

### 1. 群聊的实现，没有实现交流机制

**问题描述**:  
群聊功能的基础设施已存在，但缺少实际的交流机制实现。

**相关代码位置**:
- `alou-desktop/src/hooks/useMultiAgentChat.js` - 群聊 Hook
  - `createGroup()` - 已实现
  - `joinGroup()` - 已实现
  - `sendGroupMessage()` - 已实现
  - `handleGroupMessage()` - 已实现，但缺少实际的消息处理逻辑
- `alou-desktop/src/services/pubsubService.js` - PubSub 服务
- `alou-desktop/src/services/agentCoordinatorService.js` - 智能体协调器

**当前状态**:
```javascript
// useMultiAgentChat.js:325
onGroupMessage: useCallback((groupId, message) => {
  console.log('[AgentChat] 收到群聊消息:', groupId, message)
  // ⚠️ 缺少实际的消息处理逻辑
}, []),
```

**需要实现**:
- [ ] 群聊消息的 UI 展示
- [ ] 群聊消息的路由和分发
- [ ] 群聊成员管理
- [ ] 群聊消息的持久化

**建议实现方案**:
1. 在 `AgentChat.jsx` 中添加群聊消息处理逻辑
2. 创建群聊 UI 组件
3. 实现群聊消息的存储和检索
4. 集成到现有的消息系统

---

### 2. 删除本地智能体时，后端记录会在下次重新加载，没有删除完成

**问题描述**:  
删除本地智能体时，只删除了前端存储，后端 session 删除是异步的且可能失败，导致重新加载时智能体又出现。

**相关代码位置**:
- `alou-desktop/src/components/AgentChat/useChannelManager.js:394-444` - `deleteChannel()` 函数

**当前实现**:
```javascript
// useChannelManager.js:425-434
// 4. 调用后端 API 删除 session（异步，不阻塞 UI）
if (agentSessionId) {
  agentService.deleteSession(agentSessionId)
    .then(() => {
      console.log('[useChannelManager] 已删除后端 session:', agentSessionId)
    })
    .catch((error) => {
      console.error('[useChannelManager] 删除后端 session 失败:', error)
      // ⚠️ 错误处理不完整，没有重试机制
    })
}
```

**问题分析**:
1. 后端删除是异步的，没有等待完成
2. 删除失败时没有重试机制
3. 重新加载时没有检查删除状态
4. 可能缺少后端 API 实现

**需要实现**:
- [ ] 添加删除确认机制（等待后端删除完成）
- [ ] 实现删除重试机制
- [ ] 在重新加载时过滤已删除的智能体
- [ ] 检查后端 `deleteSession` API 是否完整实现
- [ ] 添加删除状态标记（避免重复删除）

**建议实现方案**:
1. 在 `agentStore.js` 中添加删除状态标记
2. 修改 `deleteChannel` 函数，等待后端删除完成
3. 实现删除重试机制（最多 3 次）
4. 在 `loadChannelList` 中过滤已标记删除的智能体

---

### 3. 解析过程加载图片。使用 IPNS 解析时，加载的智能体图片名称，远程智能体无法加载出来

**问题描述**:  
使用 IPNS 解析智能体时，头像图片无法正确加载，特别是远程智能体。

**相关代码位置**:
- `alou-desktop/src/components/AgentChat/agentUtils.js:9-61` - `resolveAgentAvatar()` 函数
- `alou-desktop/src/services/agentService.js:404-478` - `loadAgentFromIpns()` 函数

**当前实现**:
```javascript
// agentUtils.js:9-61
export const resolveAgentAvatar = (agent) => {
  // ... 多种来源的解析逻辑
  // ⚠️ IPNS 解析后的智能体，avatar_cid 可能无法正确解析
  const avatarCid = agent.avatarCid || agent.avatar_cid
  if (avatarCid) {
    if (avatarCid.startsWith('http')) return avatarCid
    if (avatarCid.startsWith('Qm') || avatarCid.startsWith('bafy') || avatarCid.startsWith('bafk')) {
      return agentAssetsService.resolveIpfsUri(avatarCid)
    }
  }
  // ...
}
```

**问题分析**:
1. IPNS 解析后，`avatar_cid` 字段可能不存在或格式不正确
2. 远程智能体的 DID 文档中的 `avatar_cid` 可能没有正确提取
3. IPFS Gateway URL 解析可能失败
4. 缺少对 IPNS 解析结果的完整处理

**需要实现**:
- [ ] 检查 IPNS 解析结果中的头像字段
- [ ] 从 DID 文档的 `serviceEndpoint` 中正确提取 `avatar_cid`
- [ ] 实现 IPFS Gateway 的降级处理
- [ ] 添加头像加载失败的重试机制
- [ ] 优化 `resolveAgentAvatar` 函数，支持更多数据源

**建议实现方案**:
1. 在 `loadAgentFromIpns` 中确保提取完整的头像信息
2. 检查 DID 文档解析逻辑
3. 添加头像加载的调试日志
4. 实现头像加载的降级策略（IPFS Gateway → 公共 Gateway → 默认头像）

---

### 4. 多智能体实现分开交流，一个智能体在运行的时候，其他智能体都无法进行运行和输出，不能共用通道，在一个智能体执行和回复的时候，其他智能体也能有自己的执行空间

**问题描述**:  
多个智能体共享同一个执行通道，导致一个智能体运行时阻塞其他智能体的执行。

**相关代码位置**:
- `alou-desktop/src/components/AgentChat/useAgentMessages.js` - 消息管理 Hook
- `alou-desktop/src/components/AgentChat.jsx:642-737` - `sendMessage()` 函数

**当前实现**:
```javascript
// useAgentMessages.js:29-35
const [isLoading, setIsLoading] = useState(false) // ⚠️ 全局 loading 状态
const [loadingByAgent, setLoadingByAgent] = useState({}) // ✅ 按智能体的 loading
const [sessionsByAgent, setSessionsByAgent] = useState({}) // ✅ 按智能体的 session

// AgentChat.jsx:642-737
const sendMessage = useCallback(async () => {
  // ⚠️ 使用全局 isLoading，会阻塞其他智能体
  if (!text || isLoading) return
  setIsLoading(true)
  // ...
}, [isLoading, /* ... */])
```

**问题分析**:
1. `sendMessage` 函数使用全局 `isLoading` 状态
2. 消息发送逻辑没有按智能体隔离
3. 缺少独立的执行通道机制
4. 工具调用处理可能共享状态

**需要实现**:
- [ ] 为每个智能体创建独立的执行通道
- [ ] 修改 `sendMessage` 函数，支持按智能体发送
- [ ] 实现智能体级别的消息队列
- [ ] 隔离工具调用的执行空间
- [ ] 实现智能体级别的流式响应处理

**建议实现方案**:
1. 创建 `useAgentExecution` Hook，管理每个智能体的执行状态
2. 修改 `sendMessage` 为 `sendMessageToAgent(agentId, message)`
3. 实现智能体级别的消息队列（`messageQueueByAgent`）
4. 隔离工具调用的上下文（每个智能体独立的 `contextEventsRef`）
5. 实现独立的流式响应处理（每个智能体独立的 stream）

**相关文件**:
- `alou-desktop/src/components/AgentChat/useAgentMessages.js` - 需要重构
- `alou-desktop/src/components/AgentChat.jsx` - 需要修改消息发送逻辑
- `alou-desktop/src/hooks/useAgentStream.js` - 可能需要支持多智能体

---

### 5. 多智能体之间进行交流，一个智能体和其他智能体可以使用协议自主交流，当用户输入指令的时候，智能体可以自动找合适的智能体进行交流

**问题描述**:  
智能体间通信的基础设施已存在，但缺少自动路由和协议化的交流机制。

**相关代码位置**:
- `alou-desktop/src/services/agentCoordinatorService.js` - 智能体协调器
- `alou-desktop/src/hooks/useMultiAgentChat.js` - 多智能体聊天 Hook
- `alou-desktop/src/components/AgentChat.jsx:308-335` - 多智能体协调器集成

**当前实现**:
```javascript
// agentCoordinatorService.js:203-215
async routeMessage(message, fromAgentId = null) {
  // ⚠️ 路由逻辑比较简单，缺少智能匹配
  const targetAgent = await this.findBestMatch(message)
  return targetAgent
}

// AgentChat.jsx:642-737
const sendMessage = useCallback(async () => {
  // ⚠️ 用户消息直接发送给当前智能体，没有智能路由
  const response = await fetch(`${API_BASE_URL}/api/agent/chat`, {
    // ...
  })
}, [/* ... */])
```

**问题分析**:
1. `routeMessage` 函数的智能匹配逻辑不完整
2. 用户消息没有经过智能路由
3. 缺少智能体能力描述和匹配机制
4. 缺少协议化的消息格式
5. 缺少智能体间的协商机制

**需要实现**:
- [ ] 实现智能体能力描述系统（基于 `role_description`）
- [ ] 改进消息路由算法（基于意图分析）
- [ ] 实现用户消息的智能路由（自动选择合适的智能体）
- [ ] 定义智能体间通信协议（消息格式、响应格式）
- [ ] 实现智能体间的协商机制（多智能体协作）
- [ ] 添加智能体间的上下文共享机制

**建议实现方案**:
1. **能力描述系统**:
   - 扩展智能体元数据，添加 `capabilities` 字段
   - 基于 `role_description` 自动提取能力关键词
   - 实现能力匹配算法

2. **智能路由**:
   - 在 `sendMessage` 中添加意图分析步骤
   - 使用 `analyzeIntent` 分析用户消息
   - 调用 `routeMessageToAgent` 找到合适的智能体
   - 如果找不到，使用当前智能体作为默认

3. **通信协议**:
   - 定义标准的消息格式（`AgentMessage`）
   - 定义请求-响应协议（`AgentRequest`, `AgentResponse`）
   - 实现消息签名和验证

4. **协商机制**:
   - 实现智能体间的请求转发
   - 实现多智能体协作流程
   - 添加智能体间的上下文传递

**相关文件**:
- `alou-desktop/src/services/agentCoordinatorService.js` - 需要扩展路由逻辑
- `alou-desktop/src/hooks/useMultiAgentChat.js` - 需要实现智能路由
- `alou-desktop/src/components/AgentChat.jsx` - 需要集成智能路由
- `alou-desktop/src/services/pubsubService.js` - 可能需要扩展消息格式

---

## 📁 关键文件清单

### 核心组件
- `alou-desktop/src/components/AgentChat.jsx` (925行) - 主智能体聊天组件
- `alou-desktop/src/components/AgentChat/useChannelManager.js` (459行) - 频道管理 Hook
- `alou-desktop/src/components/AgentChat/useAgentMessages.js` - 消息管理 Hook
- `alou-desktop/src/components/CreateAgentModal.jsx` - 创建智能体模态框

### 服务文件
- `alou-desktop/src/services/agentService.js` (706行) - 智能体服务
- `alou-desktop/src/services/diapService.js` (96行) - DIAP 服务
- `alou-desktop/src/services/agentCoordinatorService.js` - 智能体协调器
- `alou-desktop/src/services/pubsubService.js` - PubSub 消息服务

### 状态管理
- `alou-desktop/src/stores/agentStore.js` (214行) - 智能体状态存储

### Hooks
- `alou-desktop/src/hooks/useMultiAgentChat.js` (261行) - 多智能体聊天 Hook
- `alou-desktop/src/hooks/useAgentStream.js` - 流式响应 Hook

### 工具函数
- `alou-desktop/src/components/AgentChat/agentUtils.js` (112行) - 智能体工具函数

---

## 🔧 技术债务

### 代码质量
- [ ] 添加 TypeScript 类型定义（部分文件使用 `.ts`，但大部分是 `.js`）
- [ ] 统一错误处理机制
- [ ] 添加单元测试
- [ ] 优化代码注释和文档

### 性能优化
- [ ] 消息列表虚拟滚动（大量消息时）
- [ ] 智能体列表懒加载
- [ ] IPFS 解析结果缓存
- [ ] 减少不必要的重新渲染

### 用户体验
- [ ] 加载状态优化（骨架屏）
- [ ] 错误提示优化
- [ ] 操作确认对话框
- [ ] 快捷键支持

---

## 📊 代码统计

### 组件数量
- **前端组件**: ~69 个文件 (30 JSX, 27 CSS, 10 JS, 2 TS)
- **服务文件**: 14 个
- **Hooks**: 5 个
- **视图**: 10 个

### 核心文件行数
- `AgentChat.jsx`: 925 行
- `useChannelManager.js`: 459 行
- `agentService.js`: 706 行
- `agentStore.js`: 214 行
- `useMultiAgentChat.js`: 261 行

---

## 🚀 实现优先级

### 高优先级（影响核心功能）
1. **多智能体独立执行空间** (#4)
   - 影响：用户体验，多智能体并发使用
   - 预计工作量：3-5 天

2. **删除智能体的后端同步** (#2)
   - 影响：数据一致性
   - 预计工作量：1-2 天

3. **IPNS 解析图片加载** (#3)
   - 影响：远程智能体显示
   - 预计工作量：2-3 天

### 中优先级（功能完善）
4. **智能体间自动交流** (#5)
   - 影响：多智能体协作能力
   - 预计工作量：5-7 天

5. **群聊交流机制** (#1)
   - 影响：群聊功能可用性
   - 预计工作量：3-4 天

---

## 📝 开发建议

### 实现顺序
1. 先修复数据一致性问题（#2）
2. 然后实现独立执行空间（#4）
3. 修复图片加载问题（#3）
4. 最后实现高级功能（#5, #1）

### 测试策略
- 每个功能实现后，添加手动测试用例
- 记录测试步骤和预期结果
- 添加错误场景测试

### 代码审查要点
- 检查异步操作的错误处理
- 确保状态更新的原子性
- 验证数据持久化的完整性
- 检查内存泄漏（事件监听器、订阅等）

---

## 🔗 相关文档

- [PROGRESS_ARCHIVE.md](PROGRESS_ARCHIVE.md) - 项目进度存档
- [ARCHITECTURE.md](ARCHITECTURE.md) - 系统架构说明
- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - 项目结构说明
- [DESKTOP_WALLET_CONNECTION.md](DESKTOP_WALLET_CONNECTION.md) - 桌面钱包连接

---

## ✅ 存档检查清单

- [x] 项目结构记录
- [x] 已完成功能清单
- [x] 待实现功能详细描述
- [x] 问题分析和解决方案
- [x] 关键文件清单
- [x] 技术债务记录
- [x] 实现优先级
- [x] 开发建议

---

**备注**: 此存档文档记录了 alou-desktop 的当前代码状态和待实现功能。建议在实现每个功能后更新此文档，标记完成状态并记录实现细节。

