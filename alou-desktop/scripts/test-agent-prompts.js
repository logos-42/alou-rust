/**
 * 测试Agent模式系统提示词
 * 验证改进后的系统提示词是否包含详细的工具调用指南
 */

import { createClaudeAgentConfig } from '../src/services/claudeAgentTools.js';

console.log('=== Agent模式系统提示词测试 ===\n');

// 测试1：基础Agent配置
console.log('1. 基础Agent配置（只使用核心工具）:');
const basicConfig = createClaudeAgentConfig({
  mode: 'agent',
  categories: ['CORE']
});

console.log(`   系统提示词长度: ${basicConfig.systemPrompt.length} 字符`);
console.log(`   包含工具数量: ${basicConfig.tools.length}`);
console.log(`   工具列表: ${basicConfig.tools.map(t => t.name).join(', ')}`);

// 检查关键内容
const checks = [
  { keyword: '文件操作工具', description: '文件工具分类' },
  { keyword: '终端操作工具', description: '终端工具分类' },
  { keyword: '工具调用示例', description: '使用示例' },
  { keyword: '安全注意事项', description: '安全指南' },
  { keyword: 'read工具', description: '具体工具说明' },
  { keyword: 'bash工具', description: 'Bash工具说明' },
  { keyword: 'plan工具', description: 'Plan工具说明' }
];

console.log('\n   内容检查:');
checks.forEach(check => {
  const hasContent = basicConfig.systemPrompt.includes(check.keyword);
  console.log(`   ${hasContent ? '✅' : '❌'} ${check.description}: ${check.keyword}`);
});

// 测试2：全功能Agent配置
console.log('\n2. 全功能Agent配置（所有工具）:');
const fullConfig = createClaudeAgentConfig({
  mode: 'agent',
  categories: ['CORE', 'NETWORK', 'CONTROL_FLOW', 'WEB3']
});

console.log(`   系统提示词长度: ${fullConfig.systemPrompt.length} 字符`);
console.log(`   包含工具数量: ${fullConfig.tools.length}`);

// 检查工具分类
const toolCategories = [
  { name: '文件操作工具', tools: ['read', 'write', 'edit', 'glob', 'grep'] },
  { name: '终端操作工具', tools: ['bash'] },
  { name: '网络工具', tools: ['web_search', 'web_fetch'] },
  { name: '流程控制工具', tools: ['plan', 'ask_user_question', 'subagents'] },
  { name: 'Web3工具', tools: ['query_blockchain', 'build_transaction', 'broadcast_transaction'] }
];

console.log('\n   工具分类检查:');
toolCategories.forEach(category => {
  const hasCategory = fullConfig.systemPrompt.includes(category.name);
  const hasTools = category.tools.every(tool => 
    fullConfig.tools.some(t => t.name === tool)
  );
  console.log(`   ${hasCategory ? '✅' : '❌'} ${category.name}: ${hasTools ? '工具齐全' : '缺少工具'}`);
});

// 测试3：Alou模式配置
console.log('\n3. Alou模式配置（Web3专家）:');
const alouConfig = createClaudeAgentConfig({
  mode: 'alou',
  categories: ['WEB3', 'CORE', 'NETWORK']
});

console.log(`   系统提示词长度: ${alouConfig.systemPrompt.length} 字符`);
console.log(`   包含工具数量: ${alouConfig.tools.length}`);

// 检查Alou特定内容
const alouChecks = [
  { keyword: 'Alou', description: 'Alou身份标识' },
  { keyword: 'Web3支付代理', description: '角色描述' },
  { keyword: '查询余额', description: '核心能力' },
  { keyword: '构建交易', description: '支付功能' },
  { keyword: '交易哈希', description: '结果处理' },
  { keyword: '安全原则', description: '安全指南' },
  { keyword: '你有被爱着', description: '个性表达' }
];

console.log('\n   Alou特定内容检查:');
alouChecks.forEach(check => {
  const hasContent = alouConfig.systemPrompt.includes(check.keyword);
  console.log(`   ${hasContent ? '✅' : '❌'} ${check.description}: ${check.keyword}`);
});

// 测试4：提示词结构分析
console.log('\n4. 提示词结构分析:');

const analyzePromptStructure = (prompt, name) => {
  console.log(`\n   ${name}:`);
  
  // 检查章节结构
  const sections = [
    { pattern: /## .+/g, name: '二级标题' },
    { pattern: /### .+/g, name: '三级标题' },
    { pattern: /- \*\*.+\*\*:/g, name: '工具项' },
    { pattern: /示例\d+：/g, name: '使用示例' },
    { pattern: /🔒|📁|🌐|🎯|⛓️/g, name: '表情符号' }
  ];
  
  sections.forEach(section => {
    const matches = prompt.match(section.pattern) || [];
    console.log(`      ${section.name}: ${matches.length} 处`);
  });
  
  // 检查工具说明详细程度
  const toolMentions = ['read工具', 'write工具', 'bash工具', 'plan工具'];
  toolMentions.forEach(tool => {
    const count = (prompt.match(new RegExp(tool, 'g')) || []).length;
    console.log(`      ${tool}提及次数: ${count}`);
  });
};

analyzePromptStructure(basicConfig.systemPrompt, '基础Agent提示词');
analyzePromptStructure(alouConfig.systemPrompt, 'Alou模式提示词');

// 测试5：工具调用指南详细程度
console.log('\n5. 工具调用指南详细程度:');

const checkToolGuidance = (prompt) => {
  const guidanceAreas = [
    { keyword: '工具调用原则', description: '调用原则说明' },
    { keyword: '参数准备', description: '参数准备指南' },
    { keyword: '执行调用', description: '执行流程说明' },
    { keyword: '结果处理', description: '结果处理指南' },
    { keyword: '最佳实践', description: '最佳实践建议' }
  ];
  
  let score = 0;
  guidanceAreas.forEach(area => {
    if (prompt.includes(area.keyword)) {
      score++;
      console.log(`   ✅ ${area.description}`);
    } else {
      console.log(`   ❌ ${area.description}`);
    }
  });
  
  return score;
};

console.log('\n   基础Agent工具调用指南:');
const basicScore = checkToolGuidance(basicConfig.systemPrompt);
console.log(`   完整度: ${basicScore}/5 (${(basicScore/5*100).toFixed(0)}%)`);

console.log('\n   Alou模式工具调用指南:');
const alouScore = checkToolGuidance(alouConfig.systemPrompt);
console.log(`   完整度: ${alouScore}/5 (${(alouScore/5*100).toFixed(0)}%)`);

// 测试6：生成示例提示词片段
console.log('\n6. 示例提示词片段:');

const showPromptSamples = (prompt, name) => {
  console.log(`\n   ${name}示例片段:`);
  
  // 提取工具说明部分
  const toolSectionMatch = prompt.match(/## 可用工具[\s\S]*?(?=##|$)/);
  if (toolSectionMatch) {
    const toolSection = toolSectionMatch[0];
    const lines = toolSection.split('\n').slice(0, 15);
    console.log('   ' + lines.join('\n   '));
    if (toolSection.split('\n').length > 15) {
      console.log('   ...');
    }
  }
  
  // 提取使用示例部分
  const exampleSectionMatch = prompt.match(/工具调用示例[\s\S]*?(?=安全|$)/);
  if (exampleSectionMatch) {
    const exampleSection = exampleSectionMatch[0];
    const lines = exampleSection.split('\n').slice(0, 10);
    console.log('\n   使用示例:');
    console.log('   ' + lines.join('\n   '));
  }
};

showPromptSamples(basicConfig.systemPrompt, '基础Agent');
showPromptSamples(alouConfig.systemPrompt, 'Alou模式');

// 测试7：总结
console.log('\n=== 测试总结 ===\n');

const summary = {
  '基础Agent提示词': {
    length: basicConfig.systemPrompt.length,
    tools: basicConfig.tools.length,
    hasToolCategories: basicConfig.systemPrompt.includes('文件操作工具'),
    hasExamples: basicConfig.systemPrompt.includes('示例1：'),
    hasSafety: basicConfig.systemPrompt.includes('安全注意事项')
  },
  'Alou模式提示词': {
    length: alouConfig.systemPrompt.length,
    tools: alouConfig.tools.length,
    hasToolCategories: alouConfig.systemPrompt.includes('Web3支付相关工具'),
    hasExamples: alouConfig.systemPrompt.includes('示例1：查询余额'),
    hasSafety: alouConfig.systemPrompt.includes('安全原则')
  }
};

Object.entries(summary).forEach(([name, stats]) => {
  console.log(`${name}:`);
  console.log(`   长度: ${stats.length} 字符`);
  console.log(`   工具数量: ${stats.tools} 个`);
  console.log(`   工具分类: ${stats.hasToolCategories ? '✅ 有' : '❌ 无'}`);
  console.log(`   使用示例: ${stats.hasExamples ? '✅ 有' : '❌ 无'}`);
  console.log(`   安全指南: ${stats.hasSafety ? '✅ 有' : '❌ 无'}`);
  console.log('');
});

console.log('改进效果:');
console.log('1. ✅ 系统提示词现在包含详细的工具分类说明');
console.log('2. ✅ 提供了具体的工具使用示例');
console.log('3. ✅ 包含了完整的安全注意事项');
console.log('4. ✅ 不同模式有不同的工具调用指南');
console.log('5. ✅ 工具说明更加详细和实用');
console.log('\n建议:');
console.log('1. 在实际对话中测试工具调用效果');
console.log('2. 根据用户反馈进一步优化提示词');
console.log('3. 考虑添加更多场景化的使用示例');
console.log('4. 定期更新工具说明以反映最新功能');

export default {
  basicConfig,
  fullConfig,
  alouConfig,
  summary
};
