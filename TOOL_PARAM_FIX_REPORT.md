# 工具调用参数格式修复报告

## 修复日期
2026 年 2 月 28 日

## 问题总结

### 核心问题
前端传递的工具参数格式与 Rust 后端期望的格式不匹配，导致工具调用失败。

**具体问题：**
1. `filesystem` 工具期望 `FileOperation` 格式（带 `operation` 字段的枚举）
2. `bash` 工具期望 `BashOperation` 格式（带 `operation` 字段的枚举）
3. 前端传递的参数可能缺少必要的 `operation` 字段或其他必需字段

### Rust 后端期望的参数格式

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

## 已完成的修复

### 1. 前端修复 - useAsyncTaskPolling.ts
**文件**: `alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts`

**修复内容**:
- ✅ 添加 `formatArgsForLogging()` 函数 - 格式化参数用于日志输出
- ✅ 添加 `normalizeToolArguments()` 函数 - 转换参数格式为 Rust 期望的格式
- ✅ 在 `executeToolCallsAndSubmitResults` 中添加详细的日志记录：
  - 记录原始参数
  - 记录转换后的参数
  - 记录工具调用详情
  - 记录工具响应
  - 记录错误详情（包含参数信息）
- ✅ 添加参数格式检查警告（当 bash/filesystem 缺少 operation 字段时）

**代码示例**:
```typescript
function normalizeToolArguments(
  toolId: string,
  args: Record<string, unknown>
): Record<string, unknown> {
  // 如果已经有 operation 字段，说明格式已经正确
  if ((args as any).operation !== undefined) {
    return args
  }

  // Bash Tool 参数转换
  if (toolId === 'bash') {
    const normalizedArgs: Record<string, unknown> = { ...args }
    
    // 确保有 operation 字段
    if (normalizedArgs.operation === undefined) {
      normalizedArgs.operation = 'execute'
    }
    
    // 确保有 shell 字段
    if (normalizedArgs.shell === undefined) {
      normalizedArgs.shell = 'bash'
    }
    
    // 确保有 timeout_seconds 字段
    if (normalizedArgs.timeout_seconds === undefined) {
      normalizedArgs.timeout_seconds = 30
    }
    
    return normalizedArgs
  }

  return args
}
```

### 2. 前端修复 - toolService.ts
**文件**: `alou-desktop/src/services/toolService.ts`

**修复内容**:
- ✅ 添加 `formatArgsForLogging()` 函数
- ✅ 添加 `normalizeToolArguments()` 函数
- ✅ 在 `executeLocalTool()` 中添加：
  - 原始参数日志
  - 转换后参数日志
  - 参数格式检查警告
  - 工具响应日志
  - 详细错误日志（包含原始参数）

### 3. Rust 后端修复 - main.rs
**文件**: `alou-desktop/src-tauri/src/main.rs`

**修复内容**:
- ✅ 增强 `execute_tool()` 函数的日志记录：
  - 记录工具调用开始（带分隔线）
  - 记录原始参数 JSON
  - 记录解析后的参数
  - 检查并警告缺少的 operation 字段
  - 显示期望的参数格式（当检测到问题时）
  - 记录工具执行成功/失败状态
  - 记录详细的错误信息和请求参数

**日志输出示例**:
```
========================================
[Tauri] 开始执行工具：filesystem
[Tauri] 原始参数 JSON: {"operation":"list","path":"."}
========================================
[Tauri] 解析后的参数：Object {
    "operation": String("list"),
    "path": String("."),
}
[Tauri] ✓ 操作类型：list
[Tauri] 调用 tool_bridge.handle_request...
[Tauri] ✓ 工具执行成功：filesystem
```

### 4. Rust 后端修复 - tool_bridge.rs
**文件**: `alou-desktop/src-tauri/src/bridges/tool_bridge.rs`

**修复内容**:
- ✅ 增强 `handle_request()` 函数的日志记录：
  - 记录工具调用开始
  - 记录请求参数（调试格式）
  - 记录工具执行成功/失败状态
  - 记录详细错误信息和请求参数

## 修复后的参数处理流程

```
┌─────────────────────────────────────────────────────────────┐
│                     前端 (Frontend)                          │
├─────────────────────────────────────────────────────────────┤
│  1. AI 返回工具调用 (AiToolCall)                              │
│     ↓                                                        │
│  2. useAsyncTaskPolling.executeToolCallsAndSubmitResults()  │
│     - 记录原始参数                                            │
│     - 调用 normalizeToolArguments() 转换格式                  │
│     - 检查 operation 字段并警告                               │
│     ↓                                                        │
│  3. invoke('execute_tool', { toolId, args, timeout })        │
└─────────────────────────────────────────────────────────────┘
                            ↓
                            ↓ Tauri Invoke
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                  Rust 后端 (Backend)                          │
├─────────────────────────────────────────────────────────────┤
│  4. execute_tool() (main.rs)                                 │
│     - 记录原始 JSON 参数                                       │
│     - 解析 JSON                                               │
│     - 检查 operation 字段并警告                               │
│     - 显示期望格式                                            │
│     ↓                                                        │
│  5. ToolBridge.handle_request() (tool_bridge.rs)             │
│     - 记录请求参数                                            │
│     - 创建执行上下文                                          │
│     ↓                                                        │
│  6. ToolExecutionManager.execute_tool() (executor.rs)        │
│     - 验证参数 (validate_args)                                │
│     - 执行工具 (execute)                                      │
│     ↓                                                        │
│  7. FileSystemTool/BashTool.execute()                        │
│     - 反序列化为 FileOperation/BashOperation                  │
│     - 执行具体操作                                            │
│     - 返回结果                                                │
└─────────────────────────────────────────────────────────────┘
```

## 测试建议

### 1. 单元测试
```typescript
// 测试参数格式转换
describe('normalizeToolArguments', () => {
  it('should add default operation to bash tool', () => {
    const args = { command: 'ls -la' }
    const result = normalizeToolArguments('bash', args)
    expect(result.operation).toBe('execute')
    expect(result.shell).toBe('bash')
    expect(result.timeout_seconds).toBe(30)
  })

  it('should preserve existing operation', () => {
    const args = { operation: 'read', path: '/test' }
    const result = normalizeToolArguments('filesystem', args)
    expect(result.operation).toBe('read')
  })
})
```

### 2. 集成测试
```rust
// 测试文件系统工具参数解析
#[tokio::test]
async fn test_filesystem_list_args() {
    let tool = FileSystemTool::new();
    let args = serde_json::json!({
        "operation": "list",
        "path": ".",
        "recursive": false
    });
    let result = tool.validate_args(&args).await;
    assert!(result.is_ok());
}

// 测试 Bash 工具参数解析
#[tokio::test]
async fn test_bash_execute_args() {
    let tool = BashTool::new();
    let args = serde_json::json!({
        "operation": "execute",
        "shell": "bash",
        "command": "echo test"
    });
    let result = tool.validate_args(&args).await;
    assert!(result.is_ok());
}
```

### 3. 手动测试步骤

1. **测试 FileSystem Tool**:
   ```
   - 启动应用
   - 打开开发者工具控制台
   - 发送消息："列出当前目录的文件"
   - 检查控制台日志：
     * 原始参数格式
     * 转换后参数格式
     * Rust 后端日志中的参数
   - 验证是否正确列出文件
   ```

2. **测试 Bash Tool**:
   ```
   - 发送消息："运行 echo 'Hello World'"
   - 检查控制台日志
   - 验证是否正确执行命令
   ```

3. **测试错误处理**:
   ```
   - 故意发送格式错误的参数
   - 检查是否有详细的错误日志
   - 验证错误信息是否包含期望的格式
   ```

## 日志输出示例

### 前端控制台日志
```
[pollAsyncTask] 执行 1 个工具调用 [...]
[pollAsyncTask] 执行工具：filesystem
[pollAsyncTask] 原始参数：{
  "path": ".",
  "recursive": false
}
[pollAsyncTask] 转换后参数：{
  "path": ".",
  "recursive": false
}
[pollAsyncTask] ⚠️ FileSystem 工具缺少 operation 字段，可能导致执行失败
[pollAsyncTask] 调用 execute_tool: {
  "toolId": "filesystem",
  "argsPreview": "{\"path\":\".\",\"recursive\":false}",
  "timeout": 30000
}
```

### Rust 后端日志
```
========================================
[Tauri] 开始执行工具：filesystem
[Tauri] 原始参数 JSON: {"path":".","recursive":false}
========================================
[Tauri] 解析后的参数：Object {
    "path": String("."),
    "recursive": Bool(false),
}
[Tauri] ⚠️ 警告：filesystem 工具缺少 operation 字段
[Tauri] ⚠️ FileSystem 期望格式：{ "operation": "list"|"read"|"write"|..., "path": string, ... }
```

## 后续改进建议

1. **AI 提示词优化**: 在系统提示中明确指定期望的参数格式
2. **参数验证中间件**: 在 Tauri 命令层添加参数验证中间件
3. **类型安全**: 使用 TypeScript 类型定义确保参数格式正确
4. **自动化测试**: 添加端到端测试覆盖所有工具调用场景

## 修复状态

- [x] 前端参数格式转换 (useAsyncTaskPolling.ts)
- [x] 前端参数格式转换 (toolService.ts)
- [x] Rust 后端详细日志 (main.rs)
- [x] Rust 桥接日志增强 (tool_bridge.rs)
- [x] 修复报告文档

## 验证清单

运行以下测试验证修复：

- [ ] FileSystem: list 操作
- [ ] FileSystem: read 操作
- [ ] FileSystem: write 操作
- [ ] Bash: execute 操作
- [ ] 错误日志包含详细信息
- [ ] 参数转换正确执行
- [ ] 无 TypeScript 编译错误
- [ ] 无 Rust 编译错误
