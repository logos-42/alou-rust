#!/usr/bin/env python3
"""安全地添加 normalizeToolCallArguments 函数到 useAgentChat.ts"""

with open('/Users/apple/Downloads/alou/alou-desktop/src/hooks/useAgentChat.ts', 'r') as f:
    content = f.read()

# 在文件末尾添加函数（在最后一个导出之前）
new_function = '''
/**
 * 标准化工具调用参数格式
 * 确保参数符合 Rust 期望的格式
 */
export function normalizeToolCallArguments(
  args: Record<string, unknown>,
  toolId: string
): Record<string, unknown> {
  if (!args || typeof args !== 'object') {
    return args;
  }

  const n = { ...args } as Record<string, unknown>;

  // Bash Tool
  if (toolId === 'bash') {
    n.operation = 'execute';
    if (!n.shell || typeof n.shell === 'string') {
      const shell = (n.shell as string) || 'bash';
      n.shell = shell.toLowerCase();
    }
    if (!n.command) {
      n.command = (n.cmd as string) || (n.script as string) || '';
    }
    if (!n.timeout_seconds) n.timeout_seconds = 30;
    n.environment = Array.isArray(n.environment) ? n.environment : [];
    if (n.working_dir === undefined) n.working_dir = null;
  }

  // FileSystem Tool
  else if (toolId === 'filesystem') {
    if (!n.operation) n.operation = n.content ? 'write' : 'list';
    if (!n.path) n.path = '.';
  }

  // System Tool
  else if (toolId === 'system') {
    if (!n.operation) n.operation = 'info';
  }

  // Search Tool
  else if (toolId === 'search') {
    if (!n.operation) n.operation = n.query ? 'grep' : 'list';
    if (!n.pattern) n.pattern = n.query || '';
    if (!n.directory) n.directory = '.';
  }

  // 其他工具保持原样
  return n;
}

'''

# 在文件末尾添加
content = content.rstrip() + '\n' + new_function

with open('/Users/apple/Downloads/alou/alou-desktop/src/hooks/useAgentChat.ts', 'w') as f:
    f.write(content)

print("已添加 normalizeToolCallArguments 函数")
