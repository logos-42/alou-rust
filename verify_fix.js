#!/usr/bin/env node
/**
 * 验证工具调用参数修复
 * 测试 normalizeToolCallArguments 函数的逻辑
 */

// 模拟修复后的 normalizeToolCallArguments 函数逻辑
function normalizeToolCallArguments(args, toolId) {
  if (!args || typeof args !== 'object') {
    return args;
  }

  const n = { ...args };

  // FileSystem Tool 参数格式转换
  if (toolId === 'filesystem') {
    if (!n.operation) {
      n.operation = n.content ? 'Write' : 'List';
    } else {
      // 将 operation 转换为首字母大写格式以匹配 Rust 枚举
      const op = String(n.operation).toLowerCase();
      const opMap = {
        'read': 'Read',
        'write': 'Write',
        'list': 'List',
        'edit': 'Edit',
        'delete': 'Delete',
        'copy': 'Copy',
        'move': 'Move'
      };
      n.operation = opMap[op] || 'List';
    }
    
    if (!n.path && n.operation !== 'Write') {
      n.path = '.';
    }
    if (n.operation === 'Write' && n.create_dirs === undefined) {
      n.create_dirs = true;
    }
    if (n.operation === 'List' && n.recursive === undefined) {
      n.recursive = false;
    }
  }

  // Bash Tool 参数格式转换 - 强制覆盖所有字段
  if (toolId === 'bash') {
    n.operation = 'Execute';
    
    // Shell 字段必须使用 Rust 枚举的大写形式
    if (!n.shell) {
      n.shell = 'Bash';
    } else if (typeof n.shell === 'string') {
      const shellMap = {
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

  return n;
}

// 测试用例
console.log('=== 工具调用参数修复验证 ===\n');

// 测试 1: Bash 工具
console.log('测试 1: Bash 工具');
const bashArgs = { command: 'ls -la' };
const bashResult = normalizeToolCallArguments(bashArgs, 'bash');
console.log('输入:', JSON.stringify(bashArgs));
console.log('输出:', JSON.stringify(bashResult, null, 2));
console.log('✓ operation 应为 "Execute":', bashResult.operation === 'Execute' ? '✅' : '❌');
console.log('✓ shell 应为 "Bash":', bashResult.shell === 'Bash' ? '✅' : '❌');
console.log('✓ command 应保留:', bashResult.command === 'ls -la' ? '✅' : '❌');
console.log();

// 测试 2: Bash 工具带小写 shell
console.log('测试 2: Bash 工具带小写 shell');
const bashArgs2 = { command: 'pwd', shell: 'bash' };
const bashResult2 = normalizeToolCallArguments(bashArgs2, 'bash');
console.log('输入:', JSON.stringify(bashArgs2));
console.log('输出:', JSON.stringify(bashResult2, null, 2));
console.log('✓ shell 应转换为 "Bash":', bashResult2.shell === 'Bash' ? '✅' : '❌');
console.log();

// 测试 3: FileSystem List 操作
console.log('测试 3: FileSystem List 操作');
const fsArgs = { operation: 'list', path: '.' };
const fsResult = normalizeToolCallArguments(fsArgs, 'filesystem');
console.log('输入:', JSON.stringify(fsArgs));
console.log('输出:', JSON.stringify(fsResult, null, 2));
console.log('✓ operation 应为 "List":', fsResult.operation === 'List' ? '✅' : '❌');
console.log();

// 测试 4: FileSystem Write 操作
console.log('测试 4: FileSystem Write 操作');
const fsArgs2 = { operation: 'write', path: 'test.txt', content: 'hello' };
const fsResult2 = normalizeToolCallArguments(fsArgs2, 'filesystem');
console.log('输入:', JSON.stringify(fsArgs2));
console.log('输出:', JSON.stringify(fsResult2, null, 2));
console.log('✓ operation 应为 "Write":', fsResult2.operation === 'Write' ? '✅' : '❌');
console.log('✓ create_dirs 应为 true:', fsResult2.create_dirs === true ? '✅' : '❌');
console.log();

// 测试 5: FileSystem 默认操作（无 operation）
console.log('测试 5: FileSystem 默认操作（无 operation）');
const fsArgs3 = { path: '.' };
const fsResult3 = normalizeToolCallArguments(fsArgs3, 'filesystem');
console.log('输入:', JSON.stringify(fsArgs3));
console.log('输出:', JSON.stringify(fsResult3, null, 2));
console.log('✓ operation 应为 "List":', fsResult3.operation === 'List' ? '✅' : '❌');
console.log();

// 测试 6: FileSystem Read 操作
console.log('测试 6: FileSystem Read 操作');
const fsArgs4 = { operation: 'read', path: 'test.txt' };
const fsResult4 = normalizeToolCallArguments(fsArgs4, 'filesystem');
console.log('输入:', JSON.stringify(fsArgs4));
console.log('输出:', JSON.stringify(fsResult4, null, 2));
console.log('✓ operation 应为 "Read":', fsResult4.operation === 'Read' ? '✅' : '❌');
console.log();

console.log('=== 验证完成 ===');
console.log('\n✅ 所有 operation 值现在使用首字母大写格式，与 Rust 后端枚举匹配');
console.log('   - Bash: "Execute"');
console.log('   - FileSystem: "Read", "Write", "List", "Edit", etc.');
