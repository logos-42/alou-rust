/**
 * Alou Skills 智能匹配引擎
 * 
 * 改进的任务匹配算法：
 * 1. 更好的关键词提取
 * 2. 语义理解
 * 3. 置信度评分
 * 4. 模糊匹配
 */

const fs = require('fs');
const path = require('path');

// 中文到英文关键词映射
const KEYWORD_MAP = {
  // 邮件相关
  '邮件': 'email',
  '检查邮件': 'email',
  '未读邮件': 'email',
  '收件箱': 'email',
  
  // 工作流相关
  '工作流': 'workflow',
  '流程': 'workflow',
  '任务': 'task',
  '管理': 'manage',
  
  // 搜索相关
  '搜索': 'search',
  '查找': 'search',
  '查询': 'search',
  
  // 代码相关
  '代码': 'code',
  '分析': 'analyze',
  '结构': 'structure',
  
  // Git 相关
  'git': 'git',
  '提交': 'commit',
  '分支': 'branch',
};

// Skills 定义（实际应该从文件系统读取）
const SKILLS = [
  {
    name: 'WorkflowSkill',
    description: '工作流管理 - 创建、执行和监控多步骤工作流',
    keywords: ['workflow', 'task', 'manage', 'execute', 'step', '工作流', '任务', '流程'],
    bestFor: ['管理工作流程', '执行自动化任务', '协调多步骤流程'],
  },
  {
    name: 'WebSearch',
    description: '网络搜索 - 搜索网络信息、查找最新资讯',
    keywords: ['search', 'web', 'find', 'query', 'search', '搜索', '查找', '资讯'],
    bestFor: ['搜索网络信息', '查找最新资讯', '获取新闻'],
  },
  {
    name: 'GitHelper',
    description: 'Git 操作 - 执行 Git 命令、管理代码版本',
    keywords: ['git', 'commit', 'branch', 'push', 'pull', 'git', '提交', '分支'],
    bestFor: ['提交代码', '查看提交历史', '管理分支'],
  },
  {
    name: 'CodeAnalyzer',
    description: '代码分析 - 分析项目结构、统计代码指标',
    keywords: ['code', 'analyze', 'structure', 'stat', '代码', '分析', '结构'],
    bestFor: ['分析项目代码', '查看代码结构', '统计代码指标'],
  },
  {
    name: 'EmailChecker',
    description: '邮件检查 - 检查邮箱、未读邮件统计',
    keywords: ['email', 'check', 'mail', 'unread', '邮件', '检查', '收件箱'],
    bestFor: ['检查邮件', '查看未读邮件', '统计邮件数量'],
  },
];

/**
 * 智能关键词提取器
 */
function extractKeywords(text) {
  const keywords = new Set();
  const textLower = text.toLowerCase();
  
  // 直接提取英文关键词
  const englishPatterns = [
    /\b(email|mail|check|search|workflow|task|manage|execute|analyze|code|git|commit|branch)\b/gi,
  ];
  
  for (const pattern of englishPatterns) {
    const matches = textLower.match(pattern);
    if (matches) {
      matches.forEach(m => keywords.add(m.toLowerCase()));
    }
  }
  
  // 中文到英文映射
  for (const [cn, en] of Object.entries(KEYWORD_MAP)) {
    if (text.includes(cn)) {
      keywords.add(en);
    }
  }
  
  // 语义关键词
  const semanticKeywords = {
    '检查': 'check',
    '查看': 'view',
    '统计': 'stat',
    '汇报': 'report',
    '搜索': 'search',
    '查找': 'find',
    '创建': 'create',
    '执行': 'execute',
    '管理': 'manage',
    '分析': 'analyze',
  };
  
  for (const [cn, en] of Object.entries(semanticKeywords)) {
    if (text.includes(cn)) {
      keywords.add(en);
    }
  }
  
  return Array.from(keywords);
}

/**
 * 语义相似度计算
 */
function calculateSimilarity(text1, text2) {
  const words1 = new Set(extractKeywords(text1));
  const words2 = new Set(extractKeywords(text2));
  
  // Jaccard 相似度
  const intersection = new Set([...words1].filter(x => words2.has(x)));
  const union = new Set([...words1, ...words2]);
  
  if (union.size === 0) return 0;
  return intersection.size / union.size;
}

/**
 * 智能任务匹配
 */
function matchTask(taskDescription) {
  const keywords = extractKeywords(taskDescription);
  
  console.log(`\n${'='.repeat(50)}`);
  console.log(`🎯 任务: "${taskDescription}"`);
  console.log(`📝 提取关键词: [${keywords.join(', ')}]`);
  
  // 计算每个 Skill 的匹配度
  const scores = [];
  
  for (const skill of SKILLS) {
    let score = 0;
    const reasons = [];
    
    // 1. 关键词匹配
    for (const kw of keywords) {
      if (skill.keywords.includes(kw)) {
        score += 20;
        reasons.push(`关键词"${kw}"`);
      }
    }
    
    // 2. 描述匹配
    const descSimilarity = calculateSimilarity(taskDescription, skill.description);
    score += descSimilarity * 30;
    if (descSimilarity > 0) {
      reasons.push(`描述相似度${(descSimilarity * 100).toFixed(0)}%`);
    }
    
    // 3. 使用场景匹配
    for (const useCase of skill.bestFor) {
      const caseSimilarity = calculateSimilarity(taskDescription, useCase);
      if (caseSimilarity > 0.3) {
        score += caseSimilarity * 25;
        reasons.push(`场景"${useCase}"`);
      }
    }
    
    // 归一化分数 (0-100)
    const normalizedScore = Math.min(score, 100);
    
    scores.push({
      name: skill.name,
      description: skill.description,
      score: normalizedScore,
      reasons,
    });
  }
  
  // 排序
  scores.sort((a, b) => b.score - a.score);
  
  // 显示分数
  console.log(`\n${'='.repeat(50)}`);
  console.log(`📊 匹配分数:`);
  
  for (const [i, s] of scores.entries()) {
    const bar = '█'.repeat(Math.ceil(s.score / 5)) + '░'.repeat(20 - Math.ceil(s.score / 5));
    const emoji = i === 0 ? '🥇' : (i === 1 ? '🥈' : '  ');
    console.log(`${emoji} ${bar} ${s.score.toFixed(1)}% - ${s.name}`);
    if (s.reasons.length > 0) {
      console.log(`      原因: ${s.reasons.join(', ')}`);
    }
  }
  
  // 返回最佳匹配
  const best = scores[0];
  
  console.log(`\n${'='.repeat(50)}`);
  if (best.score > 30) {
    console.log(`✅ 最佳选择: ${best.name} (${best.score.toFixed(1)}%)`);
  } else {
    console.log(`⚠️  未找到高置信度匹配，最高: ${best.name} (${best.score.toFixed(1)}%)`);
  }
  
  return {
    best,
    allScores: scores,
    keywords,
  };
}

/**
 * 测试匹配
 */
function testMatching() {
  console.log('='.repeat(60));
  console.log('🧪 Alou 智能匹配测试');
  console.log('='.repeat(60));
  
  const testTasks = [
    '检查邮件',
    '管理工作流',
    '搜索最新 AI 资讯',
    '查看代码结构',
    '提交代码到仓库',
    '分析项目统计',
    '有什么新消息',
    '执行自动化任务',
  ];
  
  let correct = 0;
  
  for (const task of testTasks) {
    const result = matchTask(task);
    
    // 判断是否合理
    const isReasonable = result.best.score > 20;
    if (isReasonable) correct++;
  }
  
  console.log(`\n${'='.repeat(60)}`);
  console.log(`📊 测试结果: ${correct}/${testTasks.length} 任务有合理匹配`);
  console.log(`📈 合理率: ${(correct / testTasks.length * 100).toFixed(1)}%`);
  
  return correct / testTasks.length >= 0.7;
}

/**
 * 导出
 */
module.exports = {
  extractKeywords,
  calculateSimilarity,
  matchTask,
  testMatching,
  SKILLS,
};

// CLI 运行
if (require.main === module) {
  testMatching();
}
