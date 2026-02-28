# 工具调用参数格式修复计划

## 问题总结

### 核心问题
前端传递的工具参数格式与 Rust 后端期望的格式不匹配：

1. **FileSystem Tool** 期望 `FileOperation` 格式（带 `operation` 字段的枚举）
2. **Bash Tool** 期望 `BashOperation` 格式（带 `operation` 字段的枚举）
3. 前端可能直接传递扁平化的参数对象，没有正确的 `operation` 标签

### Rust 期望的参数格式

#### FileSystem Tool
```json
{
  "operation": "list",
  "path": ".",
  "recursive": false,
  "depth": null
}
```

#### Bash Tool
```json
{
  "operation": "execute",
  "shell": "bash",
  "command": "echo 'Hello'",
  "working_dir": null,
  "environment": [],
  "timeout_seconds": 30
}
```

## 修复任务

### 任务 1: 检查前端参数格式
- 文件：`alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts`
- 目标：确认 `toolCall.arguments` 的结构

### 任务 2: 添加参数格式转换层
- 文件：`alou-desktop/src/services/toolService.ts`
- 目标：在调用 `execute_tool` 之前确保参数格式正确

### 任务 3: 增强错误日志
- 文件：`useAsyncTaskPolling.ts` 和 `toolService.ts`
- 目标：记录详细的参数内容和错误信息

### 任务 4: Rust 端参数验证增强
- 文件：`alou-desktop/src-tauri/src/main.rs`
- 目标：在参数解析失败时提供更详细的错误信息

## 实施步骤

1. ✅ 阅读并分析前端调用代码
2. ✅ 阅读并分析 Rust 后端代码
3. ⏳ 添加参数格式转换逻辑
4. ⏳ 添加详细的错误日志
5. ⏳ 测试验证

## 测试建议

1. 测试 FileSystem Tool 的各种操作
2. 测试 Bash Tool 的命令执行
3. 验证错误日志输出
4. 确保参数格式转换正确
