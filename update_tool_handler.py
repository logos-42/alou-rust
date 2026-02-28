#!/usr/bin/env python3
"""更新 useToolCallHandler，添加对所有工具的参数转换支持"""

with open('/Users/apple/Downloads/alou/alou-desktop/src/hooks/useAgentChat.ts', 'r') as f:
    content = f.read()

# 查找并替换 normalizeToolCallArguments 函数
old_func_start = '''function normalizeToolCallArguments(
  args: Record<string, unknown>,
  toolId: string
): Record<string, unknown> {'''

# 查找函数结束位置
func_end_markers = ['\n}\n\nexport const', '\n}\n\n//', '\n}\n\nexport']
func_end_pos = -1
for marker in func_end_markers:
    pos = content.find(marker, content.find(old_func_start))
    if pos != -1:
        func_end_pos = pos + 1
        break

if func_end_pos == -1:
    print("未找到函数结束位置")
    exit(1)

# 新函数
new_func = '''function normalizeToolCallArguments(
  args: Record<string, unknown>,
  toolId: string
): Record<string, unknown> {
  if (!args || typeof args !== 'object') {
    return args;
  }

  const n = { ...args } as Record<string, unknown>;

  // ========== Bash Tool ==========
  if (toolId === 'bash') {
    n.operation = 'execute';
    
    // Shell 字段必须使用小写（serde 会自动转换为 Shell::Bash）
    if (!n.shell) {
      n.shell = 'bash';
    } else if (typeof n.shell === 'string') {
      const shellMap: Record<string, string> = {
        'bash': 'bash',
        'cmd': 'cmd',
        'powershell': 'powershell',
        'python': 'python',
        'node': 'node'
      };
      n.shell = shellMap[n.shell.toLowerCase()] || 'bash';
    }
    
    // 必须有 command 字段
    if (!n.command) {
      if (n.cmd) {
        n.command = n.cmd;
      } else if (n.script) {
        n.command = n.script;
      } else {
        throw new Error('Bash tool requires "command" argument');
      }
    }
    
    if (!n.timeout_seconds) {
      n.timeout_seconds = 30;
    }
    
    if (!n.environment || !Array.isArray(n.environment)) {
      n.environment = [];
    }
    
    if (n.working_dir === undefined) {
      n.working_dir = null;
    }
  }

  // ========== FileSystem Tool ==========
  else if (toolId === 'filesystem') {
    if (!n.operation) {
      n.operation = n.content ? 'write' : 'list';
    }
    if (!n.path && n.operation !== 'write') {
      n.path = '.';
    }
    if (n.operation === 'write' && n.create_dirs === undefined) {
      n.create_dirs = true;
    }
    if (n.operation === 'list' && n.recursive === undefined) {
      n.recursive = false;
    }
  }

  // ========== System Tool ==========
  else if (toolId === 'system') {
    if (!n.operation) {
      n.operation = 'info';
    }
  }

  // ========== Search Tool ==========
  else if (toolId === 'search') {
    if (!n.operation) {
      n.operation = n.query ? 'grep' : 'list';
    }
    if (!n.pattern && n.operation === 'grep') {
      n.pattern = n.query || '';
    }
    if (!n.directory) {
      n.directory = '.';
    }
    if (n.case_sensitive === undefined) {
      n.case_sensitive = false;
    }
  }

  // ========== UIControl Tool ==========
  else if (toolId === 'ui_control') {
    if (!n.action) {
      throw new Error('UI control tool requires "action" argument');
    }
  }

  // ========== TodoList Tool ==========
  else if (toolId === 'todolist') {
    if (!n.action) {
      n.action = 'list';
    }
  }

  // ========== Network Tool ==========
  else if (toolId === 'network') {
    if (!n.operation) {
      n.operation = 'get';
    }
    if (!n.url) {
      throw new Error('Network tool requires "url" argument');
    }
  }

  // ========== Browser Tool ==========
  else if (toolId === 'browser') {
    if (!n.action) {
      n.action = 'navigate';
    }
    if (!n.url) {
      throw new Error('Browser tool requires "url" argument');
    }
  }

  // ========== Iroh Tool ==========
  else if (toolId === 'iroh') {
    if (!n.action) {
      n.action = 'list';
    }
  }

  // ========== PubSub Tool ==========
  else if (toolId === 'pubsub') {
    if (!n.action) {
      n.action = 'subscribe';
    }
    if (!n.topic) {
      throw new Error('PubSub tool requires "topic" argument');
    }
  }

  // ========== MessagePassing Tool ==========
  else if (toolId === 'message_passing') {
    if (!n.action) {
      n.action = 'send';
    }
    if (!n.recipient) {
      throw new Error('MessagePassing tool requires "recipient" argument');
    }
    if (!n.content) {
      throw new Error('MessagePassing tool requires "content" argument');
    }
  }

  // ========== TaskQueue Tool ==========
  else if (toolId === 'task_queue') {
    if (!n.action) {
      n.action = 'list';
    }
  }

  // ========== Rollback Tool ==========
  else if (toolId === 'rollback') {
    if (!n.action) {
      n.action = 'create_snapshot';
    }
    if (!n.target_path) {
      throw new Error('Rollback tool requires "target_path" argument');
    }
  }

  // ========== GitHelper Tool ==========
  else if (toolId === 'git_helper') {
    if (!n.action) {
      n.action = 'status';
    }
    if (!n.repo_path) {
      n.repo_path = '.';
    }
  }

  // ========== AgentSkills/AgentCollaboration/AgentCreator/ToolCreation/Skills ==========
  else if (['agent_skills', 'agent_collaboration', 'agent_creator', 'tool_creation', 'skills', 'autonomous_executor'].includes(toolId)) {
    if (!n.action) {
      throw new Error(`${toolId} tool requires "action" argument`);
    }
  }

  // 其他工具保持原样
  return n;
}'''

# 替换
new_content = content[:content.find(old_func_start)] + new_func + content[func_end_pos:]

# 写入文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/hooks/useAgentChat.ts', 'w') as f:
    f.write(new_content)

print("useAgentChat.ts 已更新，支持所有 19 个工具的参数转换")
