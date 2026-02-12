/**
 * Alou 自主任务执行演示
 * 
 * 展示 Alou AI Agent 的自主能力：
 * 1. 接收任务
 * 2. 分析任务需求
 * 3. 自动选择合适的 Skill
 * 4. 执行任务
 * 5. 学习结果
 */

const { SkillLearner } = require('./skill-learner.cjs');

// 颜色
const GREEN = '\x1b[32m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const YELLOW = '\x1b[33m';
const MAGENTA = '\x1b[35m';
const RESET = '\x1b[0m';

function log(color, msg) {
  console.log(`${color}${msg}${RESET}`);
}

function logSection(msg) {
  console.log(`\n${MAGENTA}=== ${msg} ===${RESET}\n`);
}

function logTask(num, task) {
  console.log(`${CYAN}[任务 ${num}]${RESET} ${task}`);
}

/**
 * Alou 自主任务执行器
 */
class AlouAutonomousExecutor {
  constructor() {
    this.learner = new SkillLearner();
    this.taskHistory = [];
    this.completedTasks = 0;
    this.failedTasks = 0;
  }

  /**
   * 接收并执行任务
   */
  async executeTask(taskDescription) {
    logTask(this.taskHistory.length + 1, taskDescription);
    
    // Step 1: 发现可用 Skills
    log(BLUE, '  📚 发现 Skills...');
    await this.learner.discoverSkills();
    
    // Step 2: 分析任务，选择 Skill
    log(BLUE, '  🎯 分析任务...');
    const selected = await this.learner.selectSkill(taskDescription);
    
    if (!selected) {
      log(YELLOW, '  ⚠️  未找到匹配的 Skill');
      this.failedTasks++;
      return { success: false, reason: 'no_skill' };
    }
    
    // Step 3: 理解选中的 Skill
    const understanding = this.learner.understandSkill(selected.name);
    if (understanding) {
      log(BLUE, `  💡 理解: ${understanding.purpose}`);
      log(BLUE, `  🎯 最佳场景: ${understanding.bestFor?.join(', ')}`);
    }
    
    // Step 4: 执行 Skill
    log(BLUE, `  ⚙️ 执行 ${selected.name}...`);
    const result = await this.learner.executeSkill(selected.name, taskDescription);
    
    if (result.success) {
      this.completedTasks++;
      log(GREEN, `  ✅ 完成: ${result.output?.message}`);
    } else {
      this.failedTasks++;
      log(YELLOW, `  ❌ 失败: ${result.error}`);
    }
    
    // 记录历史
    this.taskHistory.push({
      task: taskDescription,
      skill: selected.name,
      result,
      timestamp: new Date().toISOString(),
    });
    
    return result;
  }

  /**
   * 报告执行统计
  */
  report() {
    logSection('📊 执行统计');
    
    console.log(`总任务: ${this.taskHistory.length}`);
    console.log(`完成: ${GREEN}${this.completedTasks}${RESET}`);
    console.log(`失败: ${YELLOW}${this.failedTasks}${RESET}`);
    
    // 从历史学习
    this.learner.learnFromHistory();
    
    console.log(`\n${GREEN}🎉 Alou 自主执行演示完成！${RESET}`);
  }
}

/**
 * 主演示
 */
async function main() {
  console.log('='.repeat(60));
  console.log('🤖 Alou 自主任务执行演示');
  console.log('='.repeat(60));
  console.log(`${CYAN}展示 AI Agent 如何自主完成工作:${RESET}`);
  console.log('  1. 接收任务');
  console.log('  2. 发现 Skills');
  console.log('  3. 选择 Skill');
  console.log('  4. 执行任务');
  console.log('  5. 学习进化');
  
  const executor = new AlouAutonomousExecutor();
  
  // 演示任务
  const demoTasks = [
    '检查我有多少封未读邮件',
    '帮我管理项目工作流程',
    '搜索最新的 AI 资讯',
    '分析项目代码结构',
    '查看 git 提交记录',
  ];
  
  // 逐个执行任务
  for (let i = 0; i < demoTasks.length; i++) {
    await executor.executeTask(demoTasks[i]);
    console.log(''); // 空行分隔
  }
  
  // 报告结果
  executor.report();
}

// 运行
main().catch(console.error);
