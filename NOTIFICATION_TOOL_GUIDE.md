# Notification Tool 使用指南

## 概述

Notification Tool 是一个用于在技能/工具执行过程中发送进度和状态通知的工具。它允许你在长时间运行的任务中提供实时反馈，增强用户体验和任务可观测性。

## 工具信息

- **工具 ID**: `notification`
- **工具名称**: `Notification Tool`
- **分类**: `Communication`
- **优先级**: `High`
- **版本**: 1.0.0

## 通知级别 (NotificationLevel)

| 级别 | 说明 | 使用场景 |
|------|------|----------|
| `info` | 信息通知 | 一般性状态更新 |
| `warning` | 警告通知 | 需要注意但不影响执行的情况 |
| `error` | 错误通知 | 执行失败或出错 |
| `success` | 成功通知 | 任务成功完成 |
| `debug` | 调试通知 | 开发和调试信息 |

## 通知类型 (NotificationType)

| 类型 | 说明 | 使用场景 |
|------|------|----------|
| `execution_start` | 执行开始 | 工具/技能开始执行时 |
| `progress_update` | 进度更新 | 任务执行过程中更新进度 |
| `execution_complete` | 执行完成 | 任务成功完成时 |
| `execution_failed` | 执行失败 | 任务执行失败时 |
| `custom` | 自定义通知 | 其他自定义通知 |
| `system` | 系统通知 | 系统级别的通知 |
| `user_interaction` | 用户交互 | 需要用户交互时 |

## 支持的操作

| 操作 | 说明 | 必需参数 | 可选参数 |
|------|------|----------|----------|
| `send` | 发送自定义通知 | level, notification_type, title, message | tool_id, progress, metadata |
| `progress` | 发送进度更新 | tool_id, progress, status | metadata |
| `start` | 发送执行开始通知 | tool_id | description |
| `complete` | 发送执行完成通知 | tool_id | result_summary |
| `error` | 发送执行失败通知 | tool_id, error_message | error_code |
| `get_history` | 获取通知历史 | - | tool_id, level, limit |
| `clear_history` | 清除通知历史 | - | tool_id |
| `get_stats` | 获取统计信息 | - | - |

## 使用示例

### 发送进度通知

```json
{
  "action": "progress",
  "tool_id": "file_downloader",
  "progress": 75.0,
  "status": "正在下载第 750 个文件..."
}
```

### 发送执行开始通知

```json
{
  "action": "start",
  "tool_id": "backup_tool",
  "description": "开始备份用户数据到云端"
}
```

### 发送错误通知

```json
{
  "action": "error",
  "tool_id": "payment_processor",
  "error_message": "支付网关连接超时",
  "error_code": "TIMEOUT_001"
}
```

## 在 TypeScript 中使用 (Tauri)

```typescript
import { invoke } from '@tauri-apps/api/core'

// 发送进度更新
await invoke('execute_tool', {
  toolId: 'notification',
  args: JSON.stringify({
    action: 'progress',
    tool_id: 'data_importer',
    progress: 25.0,
    status: 'Importing batch 1 of 4'
  })
})
```

## 相关文件

- Rust 实现：`alou-desktop/src-tauri/src/tools/notification.rs`
- 工具注册：`alou-desktop/src-tauri/src/tools/mod.rs`
