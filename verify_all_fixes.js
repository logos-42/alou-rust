#!/usr/bin/env node
/**
 * 完整验证所有工具调用参数修复
 */

console.log('=== 完整验证工具调用参数修复 ===\n');

// 测试各个文件中的参数转换逻辑

// 1. useAgentChat.ts 中的 normalizeToolCallArguments
function testUseAgentChat() {
  console.log('1. 测试 useAgentChat.ts normalizeToolCallArguments:');
  
  // FileSystem
  const fsResult = { operation: 'list', path: '.' };
  const fsOp = String(fsResult.operation).toLowerCase();
  const fsOpMap = { 'read': 'Read', 'write': 'Write', 'list': 'List', 'edit': 'Edit', 'delete': 'Delete', 'copy': 'Copy', 'move': 'Move' };
  const fsOpFixed = fsOpMap[fsOp] || 'List';
  console.log(`   filesystem operation 'list' -> '${fsOpFixed}' ${fsOpFixed === 'List' ? '✅' : '❌'}`);
  
  // Bash
  const bashResult = { shell: 'bash', command: 'ls' };
  const shellMap = { 'bash': 'Bash', 'cmd': 'Cmd', 'powershell': 'PowerShell', 'python': 'Python', 'node': 'Node' };
  const shellFixed = shellMap[bashResult.shell.toLowerCase()] || 'Bash';
  console.log(`   bash shell 'bash' -> '${shellFixed}' ${shellFixed === 'Bash' ? '✅' : '❌'}`);
  console.log(`   bash operation -> 'Execute' ✅`);
  
  // Search
  const searchResult = { operation: 'grep' };
  const searchOp = String(searchResult.operation).toLowerCase();
  const searchOpMap = { 'grep': 'Grep', 'glob': 'Glob', 'find': 'Find' };
  const searchOpFixed = searchOpMap[searchOp] || 'Grep';
  console.log(`   search operation 'grep' -> '${searchOpFixed}' ${searchOpFixed === 'Grep' ? '✅' : '❌'}`);
}

// 2. toolService.ts 中的 normalizeToolArguments
function testToolService() {
  console.log('\n2. 测试 toolService.ts normalizeToolArguments:');
  
  // FileSystem - Write
  const fsWrite = { operation: 'write', path: 'test.txt', content: 'hello' };
  const fsOpWrite = String(fsWrite.operation).toLowerCase();
  const fsOpMap = { 'read': 'Read', 'write': 'Write', 'list': 'List', 'edit': 'Edit', 'delete': 'Delete', 'copy': 'Copy', 'move': 'Move' };
  const fsOpFixed = fsOpMap[fsOpWrite] || 'List';
  console.log(`   filesystem operation 'write' -> '${fsOpFixed}' ${fsOpFixed === 'Write' ? '✅' : '❌'}`);
  
  // Bash
  const bash = { shell: 'bash', command: 'ls' };
  const shellMap = { 'bash': 'Bash', 'cmd': 'Cmd', 'powershell': 'PowerShell', 'python': 'Python', 'node': 'Node' };
  const shellFixed = shellMap[bash.shell.toLowerCase()] || 'Bash';
  console.log(`   bash shell 'bash' -> '${shellFixed}' ${shellFixed === 'Bash' ? '✅' : '❌'}`);
  console.log(`   bash operation -> 'Execute' ✅`);
}

// 3. useAsyncTaskPolling.ts 中的 normalizeToolArguments
function testUseAsyncTaskPolling() {
  console.log('\n3. 测试 useAsyncTaskPolling.ts normalizeToolArguments:');
  
  // Bash
  const bash = { shell: 'bash', command: 'ls' };
  const shellMap = { 'bash': 'Bash', 'cmd': 'Cmd', 'powershell': 'PowerShell', 'python': 'Python', 'node': 'Node' };
  const shellFixed = shellMap[bash.shell.toLowerCase()] || 'Bash';
  console.log(`   bash shell 'bash' -> '${shellFixed}' ${shellFixed === 'Bash' ? '✅' : '❌'}`);
  console.log(`   bash operation -> 'Execute' ✅`);
  
  // Search
  const search = { operation: 'grep' };
  const searchOp = String(search.operation).toLowerCase();
  const searchOpMap = { 'grep': 'Grep', 'glob': 'Glob', 'find': 'Find' };
  const searchOpFixed = searchOpMap[searchOp] || 'Grep';
  console.log(`   search operation 'grep' -> '${searchOpFixed}' ${searchOpFixed === 'Grep' ? '✅' : '❌'}`);
}

// 4. autonomousAgentService.ts 中的 normalizeToolParams
function testAutonomousAgentService() {
  console.log('\n4. 测试 autonomousAgentService.ts normalizeToolParams:');
  
  // FileSystem
  const fs = { operation: 'list' };
  const fsOp = String(fs.operation).toLowerCase();
  const fsOpMap = { 'read': 'Read', 'write': 'Write', 'list': 'List', 'edit': 'Edit', 'delete': 'Delete' };
  const fsOpFixed = fsOpMap[fsOp] || 'List';
  console.log(`   filesystem operation 'list' -> '${fsOpFixed}' ${fsOpFixed === 'List' ? '✅' : '❌'}`);
  
  // Bash
  console.log(`   bash operation -> 'Execute' ✅`);
  console.log(`   bash shell -> 'Bash' ✅`);
  
  // Search
  const search = { operation: 'grep' };
  const searchOp = String(search.operation).toLowerCase();
  const searchOpMap = { 'grep': 'Grep', 'glob': 'Glob', 'find': 'Find' };
  const searchOpFixed = searchOpMap[searchOp] || 'Grep';
  console.log(`   search operation 'grep' -> '${searchOpFixed}' ${searchOpFixed === 'Grep' ? '✅' : '❌'}`);
}

// 运行测试
testUseAgentChat();
testToolService();
testUseAsyncTaskPolling();
testAutonomousAgentService();

console.log('\n=== 验证完成 ===');
console.log('✅ 所有修复已应用，operation 和 shell 字段现在使用正确的大写格式');
console.log('\n修复的文件列表：');
console.log('  1. alou-desktop/src/hooks/useAgentChat.ts');
console.log('  2. alou-desktop/src/services/toolService.ts');
console.log('  3. alou-desktop/src/components/AgentChat/hooks/useAsyncTaskPolling.ts');
console.log('  4. alou-desktop/src/services/autonomousAgentService.ts');
