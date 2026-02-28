#!/usr/bin/env python3
"""改进 normalizeToolArguments 函数"""

# 读取文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/services/toolService.ts', 'r') as f:
    content = f.read()

# 替换 normalizeToolArguments 函数
old_func = '''  /**
   * 标准化工具参数格式
   */
  private normalizeToolArguments(args: Record<string, any>, toolId: string): Record<string, any> {
    if (!args || typeof args !== 'object') return args
    const n = { ...args }
    if (toolId === 'filesystem' && !n.operation) n.operation = 'list'
    if (toolId === 'bash') {
      if (!n.operation) n.operation = 'execute'
      if (!n.shell) n.shell = 'bash'
      if (!n.command) throw new Error('Bash requires command')
      if (!n.timeout_seconds) n.timeout_seconds = 30
      if (!n.environment) n.environment = []
    }
    if (toolId === 'ui_control' && !n.action) throw new Error('UI control requires action')
    return n
  }'''

new_func = '''  /**
   * 标准化工具参数格式
   * 确保参数符合 Rust 后端期望的格式
   */
  private normalizeToolArguments(args: Record<string, any>, toolId: string): Record<string, any> {
    if (!args || typeof args !== 'object') {
      return args
    }

    const n = { ...args }

    // FileSystem Tool 参数格式转换
    if (toolId === 'filesystem') {
      // 如果没有 operation 字段，根据其他字段推断
      if (!n.operation) {
        if (n.content) {
          n.operation = 'write'
        } else if (n.path) {
          n.operation = 'list'
        } else {
          n.operation = 'list'
        }
      }
      
      // 确保有 path 字段
      if (!n.path && n.operation !== 'write') {
        n.path = '.'
      }
      
      // 确保 create_dirs 字段存在（写操作需要）
      if (n.operation === 'write' && n.create_dirs === undefined) {
        n.create_dirs = true
      }
      
      // 确保 recursive 字段存在（list 操作需要）
      if (n.operation === 'list' && n.recursive === undefined) {
        n.recursive = false
      }
    }

    // Bash Tool 参数格式转换
    if (toolId === 'bash') {
      // 必须有 operation 字段
      n.operation = 'execute'
      
      // 必须有 shell 字段
      if (!n.shell) {
        n.shell = 'bash'
      }
      
      // 必须有 command 字段
      if (!n.command) {
        // 尝试从其他字段推断
        if (n.cmd) {
          n.command = n.cmd
        } else if (n.script) {
          n.command = n.script
        } else {
          throw new Error('Bash tool requires "command" argument')
        }
      }
      
      // 确保有 timeout_seconds 字段
      if (!n.timeout_seconds) {
        n.timeout_seconds = 30
      }
      
      // 确保 environment 是数组
      if (!n.environment || !Array.isArray(n.environment)) {
        n.environment = []
      }
      
      // working_dir 可选，默认为 null
      if (n.working_dir === undefined) {
        n.working_dir = null
      }
    }

    // UI Control Tool 参数格式转换
    if (toolId === 'ui_control') {
      if (!n.action) {
        throw new Error('UI control tool requires "action" argument')
      }
    }

    console.log(`[ToolService] normalizeToolArguments: ${toolId}`, { input: args, output: n })
    return n
  }'''

content = content.replace(old_func, new_func)

# 写入文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/services/toolService.ts', 'w') as f:
    f.write(content)

print("toolService.ts 已更新")

# 同样更新 useAsyncTaskPolling.ts
with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts', 'r') as f:
    content = f.read()

# 查找并替换 normalizeToolArguments 函数
old_func_hook = '''/**
 * 标准化工具参数格式，确保符合 Rust 后端期望
 *
 * Rust 后端期望的格式：
 * - FileSystem: { operation: "list"|"read"|"write"|..., path: string, ... }
 * - Bash: { operation: "execute", shell: "bash"|"cmd"|"powershell", command: string, ... }
 */
function normalizeToolArguments(
  toolId: string,
  args: Record<string, unknown>
): Record<string, unknown> {
  // 如果已经有 operation 字段，说明格式已经正确
  if ((args as any).operation !== undefined) {
    return args
  }

  // 根据 toolId 推断并添加 operation 字段
  if (toolId === 'filesystem') {
    const normalizedArgs: Record<string, unknown> = { ...args }

    // 确保有 operation 字段
    if (normalizedArgs.operation === undefined) {
      normalizedArgs.operation = 'list'
    }

    // 确保有 path 字段
    if (!normalizedArgs.path) {
      normalizedArgs.path = '.'
    }

    // 确保有 recursive 字段
    if (normalizedArgs.recursive === undefined) {
      normalizedArgs.recursive = false
    }

    // 确保有 create_dirs 字段
    if (normalizedArgs.create_dirs === undefined) {
      normalizedArgs.create_dirs = false
    }

    return normalizedArgs
  }

  // Bash Tool 参数转换
  if (toolId === 'bash') {
    const normalizedArgs: Record<string, unknown> = { ...args }

    // 确保有 operation 字段
    normalizedArgs.operation = 'execute'

    // 确保有 shell 字段
    if (!normalizedArgs.shell) {
      normalizedArgs.shell = 'bash'
    }

    // 确保有 command 字段
    if (!normalizedArgs.command) {
      throw new Error('Bash tool requires "command" argument')
    }

    // 确保有 timeout_seconds 字段
    if (!normalizedArgs.timeout_seconds) {
      normalizedArgs.timeout_seconds = 30
    }

    // 确保 environment 是数组
    if (!normalizedArgs.environment) {
      normalizedArgs.environment = []
    }

    return normalizedArgs
  }

  // 其他工具保持原样
  return args
}'''

new_func_hook = '''/**
 * 标准化工具参数格式，确保符合 Rust 后端期望
 *
 * Rust 后端期望的格式：
 * - FileSystem: { operation: "list"|"read"|"write"|..., path: string, ... }
 * - Bash: { operation: "execute", shell: "bash"|"cmd"|"powershell", command: string, ... }
 */
function normalizeToolArguments(
  toolId: string,
  args: Record<string, unknown>
): Record<string, unknown> {
  if (!args || typeof args !== 'object') {
    return args
  }

  const n = { ...args } as Record<string, unknown>

  // FileSystem Tool 参数格式转换
  if (toolId === 'filesystem') {
    // 如果没有 operation 字段，根据其他字段推断
    if (!n.operation) {
      if (n.content) {
        n.operation = 'write'
      } else if (n.path) {
        n.operation = 'list'
      } else {
        n.operation = 'list'
      }
    }
    
    // 确保有 path 字段
    if (!n.path && n.operation !== 'write') {
      n.path = '.'
    }
    
    // 确保 create_dirs 字段存在（写操作需要）
    if (n.operation === 'write' && n.create_dirs === undefined) {
      n.create_dirs = true
    }
    
    // 确保 recursive 字段存在（list 操作需要）
    if (n.operation === 'list' && n.recursive === undefined) {
      n.recursive = false
    }
  }

  // Bash Tool 参数格式转换 - 强制覆盖所有字段
  if (toolId === 'bash') {
    // 必须有 operation 字段
    n.operation = 'execute'
    
    // 必须有 shell 字段
    if (!n.shell) {
      n.shell = 'bash'
    }
    
    // 必须有 command 字段
    if (!n.command) {
      // 尝试从其他字段推断
      if (n.cmd) {
        n.command = n.cmd
      } else if (n.script) {
        n.command = n.script
      } else {
        throw new Error('Bash tool requires "command" argument')
      }
    }
    
    // 确保有 timeout_seconds 字段
    if (!n.timeout_seconds) {
      n.timeout_seconds = 30
    }
    
    // 确保 environment 是数组
    if (!n.environment || !Array.isArray(n.environment)) {
      n.environment = []
    }
    
    // working_dir 可选，默认为 null
    if (n.working_dir === undefined) {
      n.working_dir = null
    }
  }

  // UI Control Tool 参数格式转换
  if (toolId === 'ui_control') {
    if (!n.action) {
      throw new Error('UI control tool requires "action" argument')
    }
  }

  console.log(`[normalizeToolArguments] ${toolId}`, { input: args, output: n })
  return n
}'''

if old_func_hook in content:
    content = content.replace(old_func_hook, new_func_hook)
    with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts', 'w') as f:
        f.write(content)
    print("useAsyncTaskPolling.ts 已更新")
else:
    print("未找到 useAsyncTaskPolling.ts 中的函数，可能已修改")
