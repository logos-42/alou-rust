#!/usr/bin/env python3
"""修复 useToolCallHandler - 添加通用工具处理"""

with open('/Users/apple/Downloads/alou/alou-desktop/src/hooks/useAgentChat.ts', 'r') as f:
    content = f.read()

# 在文件开头添加 normalizeToolCallArguments 函数
import_pos = content.find('export const useToolCallHandler')
if import_pos > 0:
    helper_function = '''
/**
 * 标准化工具调用参数格式
 * 确保参数符合 Rust 后端期望的格式
 */
function normalizeToolCallArguments(
  args: Record<string, unknown>,
  toolId: string
): Record<string, unknown> {
  if (!args || typeof args !== 'object') {
    return args;
  }

  const n = { ...args } as Record<string, unknown>;

  // Bash Tool 参数格式转换 - 强制覆盖所有字段
  if (toolId === 'bash') {
    n.operation = 'execute';
    
    // Shell 字段必须使用 Rust 枚举的大写形式
    if (!n.shell) {
      n.shell = 'Bash';
    } else if (typeof n.shell === 'string') {
      const shellMap: Record<string, string> = {
        'bash': 'Bash',
        'cmd': 'Cmd',
        'powershell': 'PowerShell',
        'python': 'Python',
        'node': 'Node'
      };
      n.shell = shellMap[n.shell.toLowerCase()] || 'Bash';
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

  // FileSystem Tool 参数格式转换
  if (toolId === 'filesystem') {
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

  // System Tool 参数格式转换
  if (toolId === 'system') {
    if (!n.operation) {
      n.operation = 'info';
    }
  }

  // Search Tool 参数格式转换
  if (toolId === 'search') {
    if (!n.operation) {
      n.operation = n.query ? 'search' : 'list';
    }
    if (!n.query && n.operation === 'search') {
      n.query = '';
    }
  }

  return n;
}

'''
    content = content[:import_pos] + helper_function + content[import_pos:]

# 在工具处理逻辑中添加通用工具处理
old_tool_logic = '''          else if (toolCall.name === 'ui_resource' || toolCall.name === 'mcp_ui') {
            const { resource, resources } = result || {};
            if (Array.isArray(resources) && resources.length > 0) {
              openUiResource(resources[0], { source: toolCall.name });
            } else if (resource) {
              openUiResource(resource, { source: toolCall.name });
            }
            toolResult.content = 'UI resource opened successfully';
          }
          else {
            toolResult.content = JSON.stringify(result) || 'Tool executed successfully';
          }'''

new_tool_logic = '''          else if (toolCall.name === 'ui_resource' || toolCall.name === 'mcp_ui') {
            const { resource, resources } = result || {};
            if (Array.isArray(resources) && resources.length > 0) {
              openUiResource(resources[0], { source: toolCall.name });
            } else if (resource) {
              openUiResource(resource, { source: toolCall.name });
            }
            toolResult.content = 'UI resource opened successfully';
          }
          // 通用工具处理：bash, filesystem, system, search 等 - 通过 Tauri 调用 Rust 后端
          else if (['bash', 'filesystem', 'system', 'search', 'network', 'plan', 'todolist', 'git_helper', 'agent_skills'].includes(toolCall.name)) {
            // 使用 normalizeToolCallArguments 转换参数格式
            const normalizedArgs = normalizeToolCallArguments(toolCall.arguments, toolCall.name);
            
            try {
              const { invoke } = await import('@tauri-apps/api/core');
              const toolResponse = await invoke('execute_tool', {
                toolId: toolCall.name,
                args: JSON.stringify(normalizedArgs),
                timeout: 30000
              });
              
              if (toolResponse && typeof toolResponse === 'object' && 'success' in toolResponse) {
                const resp = toolResponse as any;
                if (resp.success) {
                  toolResult.content = resp.data?.content || resp.output || resp.data || '工具执行成功';
                  toolResult.success = true;
                } else {
                  toolResult.content = resp.error || '工具执行失败';
                  toolResult.success = false;
                }
              } else {
                toolResult.content = JSON.stringify(toolResponse);
                toolResult.success = true;
              }
            } catch (error) {
              console.error(`[ToolCall] 工具执行失败 ${toolCall.name}:`, error);
              toolResult.content = `Error: ${(error instanceof Error ? error.message : 'Unknown error')}`;
              toolResult.success = false;
            }
          }
          else {
            toolResult.content = JSON.stringify(result) || 'Tool executed successfully';
          }'''

if old_tool_logic in content:
    content = content.replace(old_tool_logic, new_tool_logic)
    print("工具处理逻辑已更新")
else:
    print("未找到工具处理逻辑")

# 写入文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/hooks/useAgentChat.ts', 'w') as f:
    f.write(content)

print("useAgentChat.ts 修复完成")
