#!/usr/bin/env python3
"""修复工具参数格式"""

import re

# 读取文件
with open('/Users/apple/Downloads/alou/alou-desktop/src/services/toolService.ts', 'r') as f:
    content = f.read()

# 查找 executeLocalTool 方法结束位置
pattern = r'(  private async executeLocalTool\(\s*toolId: string,\s*args: Record<string, any>,\s*timeout: number\s*\): Promise<Omit<ToolExecutionResult, \'executionMode\' \| \'toolId\' \| \'timestamp\'>> \{\s*const \{ invoke \} = await import\(\'@tauri-apps/api/core\'\))(\s*try \{\s*const result = await invoke<LocalToolResult>\(\'execute_tool\', \{\s*toolId,\s*args: JSON\.stringify\(args\),\s*timeout\s*\}\))'

match = re.search(pattern, content, re.MULTILINE)

if match:
    print("找到匹配位置")
    
    # 在 const { invoke } 之后添加新代码
    insert_pos = match.end(1)
    
    new_code = '''

    // 使用 normalizeToolArguments 转换参数格式
    const normalizedArgs = this.normalizeToolArguments(args, toolId)

    console.log(`[ToolService] 执行本地工具：${toolId}`)
    console.log(`[ToolService] 原始参数:`, JSON.stringify(args, null, 2))
    console.log(`[ToolService] 转换后参数:`, JSON.stringify(normalizedArgs, null, 2))
'''
    
    # 替换 try { 为新的 try {
    old_try = match.group(2)
    new_try = old_try.replace('args: JSON.stringify(args)', 'args: JSON.stringify(normalizedArgs)')
    
    # 构建新内容
    new_content = content[:insert_pos] + new_code + content[insert_pos:].replace(old_try, new_try, 1)
    
    # 在 executeLocalTool 方法后添加 normalizeToolArguments 方法
    # 找到方法结束位置
    end_pattern = r'    } catch \(error\) \{\s*console\.error\(\`\[ToolService\] 本地工具执行失败：\$\{toolId\}\`, error\)\s*throw new Error\(\`Local execution failed: \$\{\(error as Error\)\.message\}\`\)\s*}\s*}'
    end_match = re.search(end_pattern, new_content, re.MULTILINE)
    
    if end_match:
        method_end = end_match.end()
        
        normalize_method = '''

  /**
   * 标准化工具参数格式
   * @param args - 原始参数
   * @param toolId - 工具 ID
   * @returns 标准化后的参数
   */
  private normalizeToolArguments(args: Record<string, any>, toolId: string): Record<string, any> {
    if (!args || typeof args !== 'object') {
      return args
    }

    const normalizedArgs = { ...args }

    // FileSystem Tool 参数格式转换
    if (toolId === 'filesystem') {
      if (!normalizedArgs.operation) {
        // 如果没有 operation 字段，根据其他字段推断
        if (normalizedArgs.path && !normalizedArgs.content) {
          normalizedArgs.operation = 'list'
        } else if (normalizedArgs.path && normalizedArgs.content) {
          normalizedArgs.operation = 'write'
        } else {
          normalizedArgs.operation = 'list'
        }
      }
      
      // 确保有 path 字段
      if (!normalizedArgs.path && normalizedArgs.operation !== 'list') {
        normalizedArgs.path = '.'
      }
      
      // 确保 create_dirs 字段存在
      if (normalizedArgs.operation === 'write' && normalizedArgs.create_dirs === undefined) {
        normalizedArgs.create_dirs = true
      }
    }

    // Bash Tool 参数格式转换
    if (toolId === 'bash') {
      if (!normalizedArgs.operation) {
        normalizedArgs.operation = 'execute'
      }
      
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
        normalizedArgs.timeout_seconds = Math.floor(timeout / 1000) || 30
      }
      
      // 确保 environment 是数组
      if (!normalizedArgs.environment) {
        normalizedArgs.environment = []
      }
    }

    // UIControl Tool 参数格式转换
    if (toolId === 'ui_control') {
      if (!normalizedArgs.action) {
        throw new Error('UI control tool requires "action" argument')
      }
    }

    return normalizedArgs
  }
'''
        
        new_content = new_content[:method_end] + normalize_method + new_content[method_end:]
        
        # 写入文件
        with open('/Users/apple/Downloads/alou/alou-desktop/src/services/toolService.ts', 'w') as f:
            f.write(new_content)
        
        print("文件已成功修改")
    else:
        print("未找到方法结束位置")
else:
    print("未找到 executeLocalTool 方法")
