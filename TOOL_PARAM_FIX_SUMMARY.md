# 工具参数格式修复总结

## 问题根源

所有 19 个工具都使用 serde 的 `rename_all = "lowercase"` 配置，这意味着：
- 枚举标签（如 operation）必须使用**小写**
- 字段名（如 working_dir）必须使用**snake_case**
- 枚举值（如 Shell::Bash）会根据配置自动转换

## 已修复的工具

### ✅ Bash Tool
**修复内容**:
- `operation`: `"execute"` (小写)
- `shell`: `"bash"` (小写，serde 会自动转换为 `Shell::Bash`)
- `command`: 命令字符串
- `timeout_seconds`: 默认 30
- `environment`: 空数组 `[]`
- `working_dir`: `null`

**日志验证**:
```
[BashTool] validate_args 收到的参数：{
  "command": "pwd && ls -la",
  "operation": "execute",
  "shell": "bash"
}
[BashTool] validate_args 解析成功：Execute { shell: Bash, command: "pwd && ls -la", ... }
```

## 需要同样修复的工具

### 1. FileSystem Tool
**期望格式**:
```json
{
  "operation": "list",  // 小写
  "path": ".",
  "recursive": false
}
```
**状态**: ✅ 已修复（从日志看已经成功）

### 2. System Tool
**期望格式**:
```json
{
  "operation": "info"  // 小写
}
```
**状态**: ❌ 待修复

### 3. Search Tool
**期望格式**:
```json
{
  "operation": "grep",  // 小写
  "pattern": "hello",
  "directory": ".",
  "case_sensitive": false
}
```
**状态**: ❌ 待修复

### 4. UIControl Tool
**期望格式**:
```json
{
  "action": "show",  // 小写
  "element_id": "..."
}
```
**状态**: ❌ 待修复

### 5. TodoList Tool
**期望格式**:
```json
{
  "action": "add",  // 小写
  "title": "任务",
  "priority": "medium"
}
```
**状态**: ❌ 待修复

### 6. Network Tool
**期望格式**:
```json
{
  "operation": "get",  // 小写
  "url": "https://..."
}
```
**状态**: ❌ 待修复

### 7. Browser Tool
**期望格式**:
```json
{
  "action": "navigate",  // 小写
  "url": "https://..."
}
```
**状态**: ❌ 待修复

### 8. Iroh Tool
**期望格式**:
```json
{
  "action": "list",  // 小写
  ...
}
```
**状态**: ❌ 待修复

### 9. PubSub Tool
**期望格式**:
```json
{
  "action": "subscribe",  // 小写
  "topic": "..."
}
```
**状态**: ❌ 待修复

### 10. MessagePassing Tool
**期望格式**:
```json
{
  "action": "send",  // 小写
  "recipient": "...",
  "content": "..."
}
```
**状态**: ❌ 待修复

### 11. TaskQueue Tool
**期望格式**:
```json
{
  "action": "add",  // 小写
  "task": {...}
}
```
**状态**: ❌ 待修复

### 12. Rollback Tool
**期望格式**:
```json
{
  "action": "create_snapshot",  // snake_case
  "target_path": "..."
}
```
**状态**: ❌ 待修复

### 13. GitHelper Tool
**期望格式**:
```json
{
  "action": "status",  // snake_case
  "repo_path": "..."
}
```
**状态**: ❌ 待修复

### 14-19. 其他工具
- AgentSkills: 使用 `action` 字段
- AgentCollaboration: 使用 `action` 字段
- AgentCreator: 使用 `action` 字段
- ToolCreation: 使用 `action` 字段
- Skills: 使用 `action` 字段
- AutonomousExecutorTool: 使用 `action` 字段

## 修复方案

### 方案 1: 前端统一转换（推荐）
在 `useToolCallHandler` 中添加通用的 `normalizeToolCallArguments` 函数，支持所有工具。

**优点**:
- 集中管理，易于维护
- 可以在前端验证参数
- 不影响其他模块

**缺点**:
- 需要为每个工具添加转换逻辑

### 方案 2: Rust 后端兼容层
在 Rust 后端添加参数兼容性处理，自动转换字段名。

**优点**:
- 前端无需修改
- 对所有客户端生效

**缺点**:
- 增加后端复杂度
- 可能掩盖 AI 配置问题

## 建议

采用**方案 1**，在前端添加统一的参数转换函数。这样可以：
1. 确保所有工具调用参数格式正确
2. 在前端验证参数，提前发现问题
3. 集中管理，便于维护和扩展

## 下一步

1. 更新 `normalizeToolCallArguments` 函数，支持所有 19 个工具
2. 添加参数验证，提前发现格式问题
3. 添加详细日志，便于调试
4. 测试所有工具
