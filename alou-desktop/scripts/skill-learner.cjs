/**
 * Alou Skills 自主学习系统
 * 
 * 让 Alou 能够自主：
 * - 发现和理解 Skills
 * - 根据任务选择合适的 Skill
 * - 自主执行任务
 * - 学习和进化
 */

const fs = require('fs');
const path = require('path');

// Skills 目录
const SKILLS_DIR = '/Users/apple/Downloads/alou/alou-desktop/src/skills';
const SAMPLES_DIR = '/Users/apple/Downloads/alou/alou-desktop/scripts/sample-skills';

// 颜色
const GREEN = '\x1b[32m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const YELLOW = '\x1b[33m';
const RESET = '\x1b[0m';

function log(color, msg) {
  console.log(`${color}${msg}${RESET}`);
}

/**
 * 技能学习者 - 理解 Skills
 */
class SkillLearner {
  constructor() {
    this.skills = new Map();
    this.skillDescriptions = new Map();
    this.executionHistory = [];
  }

  /**
   * 发现所有可用 Skills
   */
  async discoverSkills() {
    log(CYAN, '📚 发现可用 Skills...');
    
    const skills = [];
    
    // 读取 Skills 目录
    if (fs.existsSync(SKILLS_DIR)) {
      const files = fs.readdirSync(SKILLS_DIR);
      for (const file of files) {
        if (file.endsWith('.ts') || file.endsWith('.js')) {
          const skillName = file.replace(/\.(ts|js)$/, '');
          const content = fs.readFileSync(path.join(SKILLS_DIR, file), 'utf8');
          
          // 解析 Skill
          const description = this.extractDescription(content);
          const keywords = this.extractKeywords(content);
          
          skills.push({
            name: skillName,
            description,
            keywords,
            path: path.join(SKILLS_DIR, file),
            type: 'core',
          });
          
          log(BLUE, `  发现: ${skillName}`);
        }
      }
    }
    
    // 读取示例 Skills
    if (fs.existsSync(SAMPLES_DIR)) {
      const samples = fs.readdirSync(SAMPLES_DIR);
      for (const dir of samples) {
        const skillPath = path.join(SAMPLES_DIR, dir, 'SKILL.md');
        if (fs.existsSync(skillPath)) {
          const content = fs.readFileSync(skillPath, 'utf8');
          const description = this.extractDescription(content);
          
          skills.push({
            name: dir,
            description,
            keywords: this.extractKeywords(content),
            path: skillPath,
            type: 'sample',
          });
          
          log(BLUE, `  示例: ${dir}`);
        }
      }
    }
    
    this.skills = new Map(skills.map(s => [s.name, s]));
    log(GREEN, `✅ 共发现 ${skills.length} 个 Skills`);
    
    return skills;
  }

  /**
   * 理解 Skill 的用途
   */
  understandSkill(skillName) {
    const skill = this.skills.get(skillName);
    if (!skill) {
      return null;
    }
    
    // 生成 Skill 的理解
    const understanding = {
      name: skill.name,
      purpose: skill.description,
      keywords: skill.keywords,
      canDo: this.inferCapabilities(skill),
      bestFor: this.inferBestUseCases(skill),
    };
    
    return understanding;
  }

  /**
   * 根据任务自动选择 Skill
   */
  async selectSkill(taskDescription) {
    log(CYAN, `🎯 分析任务: ${taskDescription}`);
    
    const taskLower = taskDescription.toLowerCase();
    const taskWords = taskLower.split(/\s+/).filter(w => w.length > 2);
    
    // 匹配 Skills
    const matches = [];
    
    for (const [name, skill] of this.skills) {
      let score = 0;
      const reasons = [];
      
      // 关键词匹配
      for (const keyword of skill.keywords) {
        if (taskLower.includes(keyword)) {
          score += 10;
          reasons.push(`包含关键词: ${keyword}`);
        }
      }
      
      // 描述匹配
      if (skill.description) {
        for (const word of taskWords) {
          if (skill.description.toLowerCase().includes(word)) {
            score += 5;
          }
        }
      }
      
      if (score > 0) {
        matches.push({
          name,
          score,
          confidence: Math.min(score / 30, 1),
          reasons,
        });
      }
    }
    
    // 排序
    matches.sort((a, b) => b.score - a.score);
    
    // 返回最佳匹配
    const best = matches[0] || null;
    
    if (best) {
      log(GREEN, `  选择: ${best.name} (置信度: ${(best.confidence * 100).toFixed(0)}%)`);
    } else {
      log(YELLOW, `  未找到匹配的 Skill`);
    }
    
    return best;
  }

  /**
   * 执行 Skill
   */
  async executeSkill(skillName, task) {
    log(CYAN, `⚙️ 执行 Skill: ${skillName}`);
    
    const skill = this.skills.get(skillName);
    if (!skill) {
      return { success: false, error: `Skill 不存在: ${skillName}` };
    }
    
    // 模拟执行（真实情况下会调用 Skill）
    const startTime = Date.now();
    
    // 生成执行结果
    const result = {
      success: true,
      skill: skillName,
      task,
      output: {
        message: `已完成任务: ${task}`,
        executedBy: skillName,
        capabilities: skill.description,
      },
      executionTime: Date.now() - startTime,
    };
    
    // 记录到历史
    this.executionHistory.push({
      skill: skillName,
      task,
      result,
      timestamp: new Date().toISOString(),
    });
    
    log(GREEN, `✅ 完成 (${result.executionTime}ms)`);
    
    return result;
  }

  /**
   * 从执行历史学习
   */
  learnFromHistory() {
    log(CYAN, '🧠 从历史学习...');
    
    const stats = {};
    
    for (const entry of this.executionHistory) {
      const skill = entry.skill;
      if (!stats[skill]) {
        stats[skill] = { count: 0, success: 0, totalTime: 0 };
      }
      stats[skill].count++;
      if (entry.result.success) {
        stats[skill].success++;
      }
      stats[skill].totalTime += entry.result.executionTime || 0;
    }
    
    // 打印统计
    log(BLUE, '  执行统计:');
    for (const [skill, stat] of Object.entries(stats)) {
      const avgTime = (stat.totalTime / stat.count).toFixed(0);
      const successRate = ((stat.success / stat.count) * 100).toFixed(0);
      log(BLUE, `    ${skill}: ${stat.count}次, 成功率${successRate}%, 平均${avgTime}ms`);
    }
    
    return stats;
  }

  // 辅助方法
  extractDescription(content) {
    const match = content.match(/\/\/!(.*)|##\s*(.*)|Description[:\s]*(.*)/i);
    return match ? (match[1] || match[2] || match[3] || '').trim() : '';
  }

  extractKeywords(content) {
    const keywords = [];
    
    // 提取所有文本内容
    const allText = content.toLowerCase();
    
    // 定义关键词模式
    const keywordPatterns = [
      // 常见任务词
      /\b(check|search|send|receive|create|delete|update|analyze|execute|run|manage|monitor|test|build|deploy|fetch|parse|format|validate)\b/g,
      // 领域词
      /\b(email|mail|task|workflow|code|file|project|git|web|search|api|data|database|user|message|notification)\b/g,
      // 中文词（如果有）
      /[\u4e00-\u9fa5]{2,}/g,
    ];
    
    for (const pattern of keywordPatterns) {
      const matches = allText.match(pattern);
      if (matches) {
        keywords.push(...matches.map(m => m.toLowerCase()));
      }
    }
    
    return [...new Set(keywords)];
  }

  inferCapabilities(skill) {
    const desc = skill.description.toLowerCase();
    
    const capabilities = [];
    
    if (desc.includes('workflow') || desc.includes('工作流')) {
      capabilities.push('工作流管理');
    }
    if (desc.includes('task') || desc.includes('任务')) {
      capabilities.push('任务执行');
    }
    if (desc.includes('tool') || desc.includes('工具')) {
      capabilities.push('工具调用');
    }
    if (desc.includes('email') || desc.includes('邮件')) {
      capabilities.push('邮件处理');
    }
    
    return capabilities;
  }

  inferBestUseCases(skill) {
    const desc = skill.description.toLowerCase();
    
    if (desc.includes('workflow')) {
      return ['管理工作流程', '协调多步骤任务', '自动化流程'];
    }
    if (desc.includes('task')) {
      return ['执行具体任务', '运行自动化脚本', '处理工作项'];
    }
    
    return ['一般任务'];
  }
}

// 主测试
async function main() {
  console.log('='.repeat(60));
  console.log('🤖 Alou Skills 自主学习测试');
  console.log('='.repeat(60));
  
  const learner = new SkillLearner();
  
  // 发现 Skills
  await learner.discoverSkills();
  
  // 测试任务选择
  console.log('\n' + CYAN + '=== 测试任务自动选择 ===' + RESET);
  
  const testTasks = [
    '检查邮件',
    '管理工作流',
    '执行自动化任务',
    '分析数据',
  ];
  
  for (const task of testTasks) {
    const match = await learner.selectSkill(task);
    if (match) {
      await learner.executeSkill(match.name, task);
    }
  }
  
  // 学习历史
  learner.learnFromHistory();
  
  console.log('\n' + GREEN + '✅ Skills 自主学习系统就绪！' + RESET);
}

// 运行（如果直接运行）
if (require.main === module) {
  main().catch(console.error);
}

// 导出
module.exports = { SkillLearner };
