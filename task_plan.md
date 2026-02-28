# Alou Desktop 群聊功能 P1 问题修复计划

## 目标
修复 Alou Desktop 群聊功能的 4 个 P1 重要问题，提升用户体验和系统稳定性。

## 问题清单

### 问题 4: 输入框未检查 IPFS 可用性
- **文件**: `alou-desktop/src/components/agent/GroupChatPanel.jsx`
- **位置**: 消息输入框组件 (约 490 行)
- **状态**: 待修复

### 问题 5: IPFS 连接状态未实时监控
- **文件**: `alou-desktop/src/hooks/useLocalIpfsGroupChat.ts`
- **位置**: 初始化部分 (约 60-113 行)
- **状态**: 待修复

### 问题 6: 空身份创建群聊无提示
- **文件**: `alou-desktop/src/hooks/useLocalIpfsGroupChat.ts`
- **位置**: `createGroup` 方法 (约 78-95 行)
- **状态**: 待修复

### 问题 7: broadcastToAgents 参数错误
- **文件**: `alou-desktop/src/services/agentCoordinatorService.ts`
- **位置**: `broadcastToAgents` 方法 (约 791 行)
- **状态**: 待修复

## 修复计划

### Phase 1: 修复问题 4 - 输入框 IPFS 检查 ✅
- [x] 在 GroupChatPanel.jsx 中修改输入框的 disabled 属性
- [x] 添加 IPFS 不可用时的 placeholder 提示
- [x] 验证修复效果

### Phase 2: 修复问题 5 - IPFS 心跳检测 ✅
- [x] 在 useLocalIpfsGroupChat.ts 中添加定期 IPFS 健康检查（已存在）
- [x] 设置 30 秒心跳间隔
- [x] 添加连接断开和恢复的错误提示

### Phase 3: 修复问题 6 - 身份检查提示 ✅
- [x] 在 createGroup 方法中添加身份验证
- [x] 添加友好的错误提示
- [x] 在 UI 中显示错误状态

### Phase 4: 修复问题 7 - broadcastToAgents 参数 ✅
- [x] 修复 useMultiAgentChat.ts 中 broadcastToAgents 函数签名
- [x] 修复 useGroupChatRemoteControl.ts 中的调用
- [x] 确保 excludeAgentId 参数正确传递

## 验证标准
- [x] 所有 4 个问题已修复
- [x] UI 提示友好
- [x] 错误边界处理完善
- [x] 代码风格一致
- [ ] 无 TypeScript 类型错误（需要运行 tsc 验证）

## 错误记录
| 错误 | 尝试 | 解决方案 |
|------|------|----------|
| - | - | - |
