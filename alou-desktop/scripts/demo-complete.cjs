/**
 * Alou 自主系统完整演示
 * 
 * 展示：
 * 1. 任务接收
 * 2. 技能选择
 * 3. 执行
 * 4. 用户反馈
 * 5. 学习改进
 */

const skillsExecutor = require('./skillsExecutorService');
const feedbackService = require('./feedbackService');

// 颜色
const GREEN = '\x1b[32m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const MAGENTA = '\x1b[35m';
const YELLOW = '\x1b[33m';
const RESET = '\x1b[0m';

function log(color, msg) {
  console.log(`${color}${msg}${RESET}`);
}

function logSection(msg) {
  console.log(`\n${MAGENTA}=== ${msg} ===${RESET}\n`);
}

// 演示任务
const demoTasks = [
  {
    task: '检查邮件',
    expected: 'EmailChecker',
    isGood: true,
  },
  {
    task: '管理工作流',
    expected: 'WorkflowSkill', 
    isGood: true,
  },
  {
    task: '搜索最新 AI 资讯',
    expected: 'WebSearch',
    isGood: true,
  },
];

async function main() {
  console.log('='.repeat(60));
  console.log('🤖 Alou 自主系统完整演示');
  console.log('='.repeat(60));
  console.log(`\n${CYAN}任务 → 选择 → 执行 → 反馈 → 学习${RESET}`);

  // 收集反馈用于统计
  const results = [];

  for (let i = 0; i < demoTasks.length; i++) {
    const { task, expected, isGood } = demoTasks[i];
    
    logSection(`演示 ${i + 1}: ${task}`);
    
    // Step 1: 发现技能
    log(BLUE, 'Step 1: 发现可用技能...');
    const skills = await skillsExecutor.discoverSkills();
    log(GREEN, `  发现 ${skills.length} 个技能`);
    
    // Step 2: 匹配技能
    log(BLUE, 'Step 2: 分析任务并选择技能...');
    const match = await skillsExecutor.matchTaskToSkill(task);
    const isCorrect = match.matched_skill === expected;
    log(match.matched_skill === expected ? GREEN : YELLOW, 
      `  选择: ${match.matched_skill} (${match.confidence.toFixed(0)}%) ${isCorrect ? '✅' : '⚠️'}`);
    
    // Step 3: 执行技能
    log(BLUE, 'Step 3: 执行技能...');
    const execResult = await skillsExecutor.executeSkill(match.matched_skill, { task });
    log(execResult.success ? GREEN : YELLOW, 
      `  结果: ${execResult.success ? '成功' : '失败'} (${execResult.execution_time_ms}ms)`);
    
    // Step 4: 用户反馈
    log(BLUE, 'Step 4: 收集用户反馈...');
    await feedbackService.quickFeedback(
      task,
      match.matched_skill,
      isGood,
      isGood ? '完成得很好' : '需要改进'
    );
    
    // 保存结果
    results.push({
      task,
      matched: match.matched_skill,
      expected,
      isCorrect,
      success: execResult.success,
    });
    
    console.log('');
  }

  // Step 5: 学习统计
  logSection('📊 学习结果统计');
  
  const stats = await feedbackService.getStats();
  log(BLUE, `  总反馈: ${stats.total_feedbacks}`);
  log(BLUE, `  平均评分: ${stats.average_score}/5`);
  
  // 计算匹配准确率
  const correctCount = results.filter(r => r.isCorrect).length;
  const accuracy = (correctCount / results.length * 100).toFixed(1);
  
  log(GREEN, `\n🎯 匹配准确率: ${accuracy}% (${correctCount}/${results.length})`);
  log(GREEN, `✅ 执行成功率: ${results.filter(r => r.success).length}/${results.length}`);

  // 改进建议
  if (stats.improvement_suggestions?.length > 0) {
    log(YELLOW, `\n💡 改进建议:`);
    for (const suggestion of stats.improvement_suggestions.slice(0, 3)) {
      log(YELLOW, `  • ${suggestion}`);
    }
  }

  // 总结
  logSection('📋 演示总结');
  
  console.log(`完成的流程:`);
  for (const r of results) {
    const emoji = r.isCorrect && r.success ? '✅' : '⚠️';
    console.log(`  ${emoji} ${r.task} → ${r.matched}`);
  }
  
  console.log(`\n${GREEN}🎉 演示完成！`);
  console.log(`${CYAN}系统已收集反馈，可用于持续改进。`);
}

// 运行
main().catch(console.error);
