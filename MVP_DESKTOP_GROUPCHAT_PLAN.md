# Alou Desktop MVP 0.1.10 发布计划
## 聚焦：桌面端群聊功能完善

**制定时间**: 2026-02-28  
**发布范围**: 仅桌面端 (alou-desktop)  
**核心目标**: 群聊功能可用 (P0)  
**测试方式**: 手动验证  

---

## 📊 当前状态总结

### 智能体分析结果汇总

| 分析维度 | 完成度 | 状态 | 关键问题 |
|---------|--------|------|----------|
| 群聊按钮 | 70% | ⚠️ 可能有问题 | 事件处理函数引用不稳定 |
| 群聊核心功能 | 80% | ✅ 基本完成 | `activeActionId` 未定义错误 |
| Agent协作 | 60% | 🟡 部分实现 | 任务分配/进度汇报待完善 |
| 依赖整合 | 75% | ✅ 基本就绪 | 配置缺失，身份验证简化 |

### 代码统计

| 模块 | 相关文件数 | 代码行数 | 测试覆盖 |
|------|-----------|----------|----------|
| 群聊UI | 8 | ~3,500 | 0% |
| 群聊服务 | 6 | ~4,200 | 0% |
| Agent协作 | 5 | ~2,800 | 0% |
| **总计** | **19** | **~10,500** | **0%** |

---

## 🎯 MVP 发布目标

### P0 - 必须完成 (发布阻塞)

| # | 功能 | 验收标准 |
|---|------|----------|
| 1 | 群聊按钮可用 | 点击能打开/关闭群聊面板 |
| 2 | 创建群聊 | 能创建新群聊并显示在列表 |
| 3 | 发送消息 | 能在群聊中发送文本消息 |
| 4 | 接收消息 | 能实时接收其他用户消息 |
| 5 | 修复明显bug | `activeActionId` 未定义等 |

### P1 - 应该完成 (用户体验)

| # | 功能 | 验收标准 |
|---|------|----------|
| 6 | 群聊列表 | 显示所有群聊，支持切换 |
| 7 | 消息历史 | 加载最近100条消息 |
| 8 | Agent参与 | AI Agent能发送消息 |
| 9 | 错误提示 | 操作失败时有错误提示 |

### P2 - 可以延后 (后续版本)

| # | 功能 | 计划版本 |
|---|------|----------|
| 10 | 任务分配 | v0.1.11 |
| 11 | 进度汇报 | v0.1.11 |
| 12 | @提及响应 | v0.1.11 |
| 13 | 消息编辑/删除 | v0.1.12 |
| 14 | 文件分享 | v0.1.12 |

---

## 🔧 详细修复和开发计划

### Phase 1: 群聊按钮修复 (Day 1 - 4小时)

#### 1.1 问题诊断

**问题**: 群聊按钮可能无法点击

**原因分析**:
```jsx
// useGroupChatButton.jsx 第 23 行
onClick: showGroupChat ? closeGroupChat : openGroupChat,  // ⚠️ 函数引用不稳定

// 第 83-87 行
onClick={(e) => {
  e.stopPropagation();
  buttonConfig.onClick();  // 调用闭包中的函数引用，可能已过期
}}
```

#### 1.2 修复方案

**文件**: `alou-desktop/src/components/AgentChat/useGroupChatButton.jsx`

**修改内容**:

```jsx
// 修复前：useMemo 中存储函数引用
const buttonConfig = useMemo(
  () => ({
    onClick: showGroupChat ? closeGroupChat : openGroupChat,  // ❌ 不稳定
  }),
  [showGroupChat, closeGroupChat, openGroupChat],
)

// 修复后：使用稳定的回调函数
const handleToggleGroupChat = useCallback(() => {
  console.log('[GroupChatButton] 切换群聊, 当前状态:', showGroupChat)
  if (showGroupChat) {
    closeGroupChat?.()
  } else {
    openGroupChat?.()
  }
}, [showGroupChat, closeGroupChat, openGroupChat])

const buttonConfig = useMemo(
  () => ({
    className: `group-chat-toggle-btn-fixed ${showGroupChat ? 'active' : ''}`,
    title: showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open'),
    'aria-label': showGroupChat ? t('agent.groupChat.close') : t('agent.groupChat.open'),
    disabled: !canOpenGroupChat && !showGroupChat,
  }),
  [showGroupChat, canOpenGroupChat, t],
)

// 按钮渲染
<button 
  type="button" 
  {...buttonConfig}
  onClick={(e) => {
    e.preventDefault()
    e.stopPropagation()
    try {
      handleToggleGroupChat()
    } catch (error) {
      console.error('[GroupChatButton] 切换失败:', error)
    }
  }}
>
```

#### 1.3 添加调试日志

```jsx
// useEffect 中添加状态监控
useEffect(() => {
  console.log('[useGroupChatButton] 状态更新:', {
    showGroupChat,
    canOpenGroupChat,
    hasOpenFn: !!openGroupChat,
    hasCloseFn: !!closeGroupChat,
  })
}, [showGroupChat, canOpenGroupChat, openGroupChat, closeGroupChat])
```

#### 1.4 验证清单

- [ ] 按钮点击控制台有日志输出
- [ ] 点击按钮能打开群聊面板
- [ ] 再次点击能关闭群聊面板
- [ ] 按钮状态（active类）正确切换

---

### Phase 2: 修复明显Bug (Day 1 - 2小时)

#### 2.1 修复 `activeActionId` 未定义错误

**文件**: `alou-desktop/src/components/agent/GroupChatPanel.jsx:338`

**问题代码**:
```jsx
<GroupChatArchiveList
  groupChatList={groupChatList}
  activeActionId={activeActionId}  // ❌ 未定义
  activeChannelId={activeChannelId}
  onSwitchGroupChat={onSwitchGroupChat}
/>
```

**修复方案**:
```jsx
// 方案1: 使用 activeGroup?.groupId
<GroupChatArchiveList
  groupChatList={groupChatList}
  activeActionId={activeGroup?.groupId || activeGroupId}  // ✅ 使用正确的变量
  activeChannelId={activeChannelId}
  onSwitchGroupChat={onSwitchGroupChat}
/>

// 方案2: 检查 GroupChatArchiveList 的 props 定义
// 如果只需要 activeGroupId，统一命名
```

#### 2.2 检查其他潜在错误

**检查清单**:
- [ ] 检查所有未定义变量
- [ ] 检查 props 类型不匹配
- [ ] 检查 import 语句

```bash
# 运行 TypeScript 类型检查
cd alou-desktop
npx tsc --noEmit 2>&1 | grep -E "(error|Cannot find|is not defined)"
```

---

### Phase 3: 群聊核心功能验证 (Day 2 - 6小时)

#### 3.1 创建群聊流程验证

**测试步骤**:
1. 打开群聊面板
2. 点击"创建群聊"按钮
3. 输入群聊名称
4. 添加成员（可选）
5. 确认创建

**检查点**:
- [ ] 表单能正常输入
- [ ] 点击创建后群聊出现在列表
- [ ] KV 存储中有新群聊数据
- [ ] 控制台无错误

**相关代码**:
```typescript
// localIpfsGroupChatService.ts
async createGroup(name: string, members: string[]): Promise<LocalGroup> {
  const groupId = generateGroupId()
  const group: LocalGroup = {
    groupId,
    name,
    members,
    createdAt: Date.now(),
  }
  await this.saveGroupToStorage(group)
  return group
}
```

#### 3.2 消息发送/接收验证

**测试步骤**:
1. 在群聊中输入消息
2. 点击发送或按 Enter
3. 观察消息是否显示
4. 从另一个客户端查看是否能收到

**检查点**:
- [ ] 消息输入框正常工作
- [ ] 发送后消息显示在列表
- [ ] 消息通过 IPFS PubSub 发布
- [ ] 其他客户端能收到消息

**调试命令**:
```javascript
// 在浏览器控制台测试 PubSub
await window.__TAURI__.core.invoke('ipfs_pubsub_peers', { topic: 'alou/group/test' })
```

#### 3.3 错误处理完善

**添加错误提示 UI**:

```jsx
// GroupChatPanel.jsx
const [error, setError] = useState(null)

// 错误显示
{error && (
  <div className="group-chat-error">
    <span>❌ {error}</span>
    <button onClick={() => setError(null)}>关闭</button>
  </div>
)}

// 发送消息时捕获错误
const handleSendMessage = async (content) => {
  try {
    await sendMessage(activeGroupId, content)
  } catch (err) {
    setError(`发送失败: ${err.message}`)
  }
}
```

---

### Phase 4: Agent参与验证 (Day 3 - 4小时)

#### 4.1 Agent消息发送验证

**测试步骤**:
1. 创建包含 Agent 的群聊
2. 观察 Agent 是否能自动发送消息
3. 检查 Agent 消息显示样式

**检查点**:
- [ ] Agent能加入群聊
- [ ] Agent能发送消息
- [ ] Agent消息正确显示头像和名称
- [ ] 区分人类消息和Agent消息

#### 4.2 简化任务分配（基础版）

**实现最小可行版本**:
```typescript
// useMultiAgentChat.ts
const handleAssignTask = async (agentId: string, task: string) => {
  const message: LocalGroupMessage = {
    messageId: generateId(),
    groupId: activeGroupId,
    type: MessageType.TASK_ASSIGN,
    content: `@${agentId} ${task}`,
    sender: currentUserId,
    timestamp: Date.now(),
  }
  await sendMessage(message)
}
```

---

### Phase 5: 手动测试和验收 (Day 4 - 4小时)

#### 5.1 功能测试清单

| 功能 | 测试步骤 | 期望结果 | 状态 |
|------|----------|----------|------|
| 打开群聊 | 点击群聊按钮 | 面板展开，显示群聊列表 | ⬜ |
| 关闭群聊 | 再次点击按钮 | 面板收起 | ⬜ |
| 创建群聊 | 填写名称，点击创建 | 新群聊出现在列表 | ⬜ |
| 切换群聊 | 点击不同群聊 | 显示对应群聊消息 | ⬜ |
| 发送消息 | 输入文字，按Enter | 消息显示在列表 | ⬜ |
| 接收消息 | 从另一客户端发送 | 消息实时显示 | ⬜ |
| Agent参与 | 在含Agent的群聊发消息 | Agent回复消息 | ⬜ |
| 错误提示 | 断开网络，发送消息 | 显示错误提示 | ⬜ |

#### 5.2 性能测试

- [ ] 加载100条消息耗时 < 2秒
- [ ] 发送消息延迟 < 500ms
- [ ] 界面无明显卡顿

#### 5.3 边界情况测试

- [ ] 空消息不能发送
- [ ] 超长消息正确处理
- [ ] 特殊字符正确处理
- [ ] 网络断开后重连

---

## 📁 关键文件清单

### 需要修改的文件

| 文件路径 | 修改内容 | 优先级 |
|---------|----------|--------|
| `src/components/AgentChat/useGroupChatButton.jsx` | 修复按钮事件 | P0 |
| `src/components/agent/GroupChatPanel.jsx` | 修复未定义变量 | P0 |
| `src/components/agent/GroupChatPanel.jsx` | 添加错误提示 | P1 |
| `src/services/localIpfsGroupChatService.ts` | 优化错误处理 | P1 |
| `src/hooks/useLocalIpfsGroupChat.ts` | 添加调试日志 | P2 |

### 需要验证的文件

| 文件路径 | 验证内容 |
|---------|----------|
| `src/services/pubsubService.ts` | PubSub功能正常 |
| `src/services/agentCoordinatorService.ts` | Agent协调正常 |
| `src-tauri/src/ipfs_commands.rs` | IPFS命令可用 |

---

## 🔍 调试指南

### 浏览器控制台调试

```javascript
// 1. 检查群聊状态
const state = window.useGroupChatManager?.getState()
console.log('群聊状态:', state)

// 2. 测试 IPFS PubSub
await window.__TAURI__.core.invoke('ipfs_pubsub_publish', {
  topic: 'test',
  message: 'hello'
})

// 3. 检查存储
const groups = await window.__TAURI__.core.invoke('kv_keys', { prefix: 'group:' })
console.log('群聊列表:', groups)
```

### Rust 后端调试

```rust
// 在 ipfs_commands.rs 中添加日志
#[tauri::command]
pub async fn ipfs_pubsub_publish(...) {
    println!("[PubSub] 发布消息到 topic: {}", topic);
    // ... 原有逻辑
}
```

---

## 📋 发布前检查清单

### 代码检查
- [ ] 所有P0问题已修复
- [ ] TypeScript类型检查通过
- [ ] ESLint无错误
- [ ] 控制台无报错

### 功能检查
- [ ] 群聊按钮可用
- [ ] 能创建群聊
- [ ] 能发送/接收消息
- [ ] Agent能参与群聊
- [ ] 错误提示正常

### 构建检查
- [ ] 开发模式构建成功
- [ ] 生产模式构建成功
- [ ] 安装包能正常安装

### 文档检查
- [ ] CHANGELOG.md 已更新
- [ ] README.md 群聊功能说明已添加

---

## 📅 修订后时间表

```
Day 1 (周一)
├── 09:00-11:00  Phase 1: 群聊按钮修复
├── 11:00-13:00  Phase 2: Bug修复
└── 14:00-18:00  Phase 3: 核心功能验证

Day 2 (周二)
├── 09:00-12:00  Phase 3: 核心功能验证 (续)
└── 14:00-18:00  Phase 4: Agent参与验证

Day 3 (周三)
├── 09:00-12:00  Phase 5: 手动测试
├── 14:00-16:00  问题修复
└── 16:00-18:00  最终验证

Day 4 (周四)
├── 09:00-10:00  发布前检查
├── 10:00-11:00  构建发布包
├── 11:00-12:00  Git标签和Release
└── 14:00-18:00  问题响应准备
```

---

## 🎯 成功标准

MVP 0.1.10 发布成功的标准：

1. **群聊按钮100%可用** - 用户能顺利打开/关闭群聊面板
2. **基础群聊功能可用** - 创建、发送、接收消息
3. **Agent能参与群聊** - AI Agent能发送和接收消息
4. **无阻断性Bug** - 无明显影响使用的错误
5. **文档完整** - 用户知道如何使用群聊功能

---

**计划制定完成** - 准备好开始修复了吗？ 🚀
