#!/usr/bin/env python3
"""修复 shell 字段的大小写问题"""

# 读取 toolService.ts
with open('/Users/apple/Downloads/alou/alou-desktop/src/services/toolService.ts', 'r') as f:
    content = f.read()

# 替换 Bash Tool 部分
old_bash = '''    // Bash Tool 参数格式转换 - 强制覆盖所有字段
    if (toolId === 'bash') {
      // 必须有 operation 字段
      n.operation = 'execute'
      
      // 必须有 shell 字段
      if (!n.shell) {
        n.shell = 'bash'
      }'''

new_bash = '''    // Bash Tool 参数格式转换 - 强制覆盖所有字段
    if (toolId === 'bash') {
      // 必须有 operation 字段
      n.operation = 'execute'
      
      // 必须有 shell 字段（注意：Rust 枚举使用大写形式）
      if (!n.shell) {
        n.shell = 'Bash'
      } else if (typeof n.shell === 'string') {
        // 将小写转换为大写形式
        const shellMap: Record<string, string> = {
          'bash': 'Bash',
          'cmd': 'Cmd',
          'powershell': 'PowerShell',
          'python': 'Python',
          'node': 'Node'
        }
        n.shell = shellMap[n.shell.toLowerCase()] || 'Bash'
      }'''

content = content.replace(old_bash, new_bash)

# 写入文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/services/toolService.ts', 'w') as f:
    f.write(content)

print("toolService.ts 已更新 shell 大小写转换")

# 同样更新 useAsyncTaskPolling.ts
with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts', 'r') as f:
    content = f.read()

# 替换 Bash Tool 部分
old_bash_hook = '''  // Bash Tool 参数格式转换 - 强制覆盖所有字段
  if (toolId === 'bash') {
    // 必须有 operation 字段
    n.operation = 'execute'
    
    // 必须有 shell 字段
    if (!n.shell) {
      n.shell = 'bash'
    }'''

new_bash_hook = '''  // Bash Tool 参数格式转换 - 强制覆盖所有字段
  if (toolId === 'bash') {
    // 必须有 operation 字段
    n.operation = 'execute'
    
    // 必须有 shell 字段（注意：Rust 枚举使用大写形式）
    if (!n.shell) {
      n.shell = 'Bash'
    } else if (typeof n.shell === 'string') {
      // 将小写转换为大写形式
      const shellMap: Record<string, string> = {
        'bash': 'Bash',
        'cmd': 'Cmd',
        'powershell': 'PowerShell',
        'python': 'Python',
        'node': 'Node'
      }
      n.shell = shellMap[n.shell.toLowerCase()] || 'Bash'
    }'''

if old_bash_hook in content:
    content = content.replace(old_bash_hook, new_bash_hook)
    with open('/Users/apple/Downloads/alou/alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts', 'w') as f:
        f.write(content)
    print("useAsyncTaskPolling.ts 已更新 shell 大小写转换")
else:
    print("未找到 useAsyncTaskPolling.ts 中的函数")
