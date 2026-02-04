/**
 * workflowUtils - 工作流工具函数
 */

export interface TaskComplexity {
  score: number;
  level: 'simple' | 'medium' | 'complex';
  recommendedMode: 'interactive' | 'auto' | 'parallel';
  estimatedSteps: number;
  estimatedTime: number;
}

/**
 * 分析任务复杂度
 */
export async function analyzeTaskComplexity(message: string): Promise<TaskComplexity> {
  // 基于消息长度和关键词计算复杂度
  const length = message.length;
  const keywords = {
    complex: ['创建', '部署', '实现', '设计', '分析', '调试', '优化', '迁移', '集成', '测试'],
    simple: ['查询', '获取', '检查', '验证', '确认', '搜索', '显示', '列出'],
  };

  let complexityScore = 0;
  const lowerMessage = message.toLowerCase();

  // 基于长度评分
  if (length < 50) complexityScore += 1;
  else if (length < 200) complexityScore += 2;
  else complexityScore += 3;

  // 基于关键词评分
  keywords.complex.forEach(keyword => {
    if (lowerMessage.includes(keyword.toLowerCase())) {
      complexityScore += 2;
    }
  });

  keywords.simple.forEach(keyword => {
    if (lowerMessage.includes(keyword.toLowerCase())) {
      complexityScore -= 1;
    }
  });

  // 确保分数在合理范围内
  complexityScore = Math.max(1, Math.min(10, complexityScore));

  // 确定复杂度级别
  let level: TaskComplexity['level'];
  let recommendedMode: TaskComplexity['recommendedMode'];
  let estimatedSteps: number;
  let estimatedTime: number;

  if (complexityScore <= 3) {
    level = 'simple';
    recommendedMode = 'auto';
    estimatedSteps = 1;
    estimatedTime = 5;
  } else if (complexityScore <= 6) {
    level = 'medium';
    recommendedMode = 'interactive';
    estimatedSteps = 3;
    estimatedTime = 15;
  } else {
    level = 'complex';
    recommendedMode = 'parallel';
    estimatedSteps = 5;
    estimatedTime = 30;
  }

  return {
    score: complexityScore,
    level,
    recommendedMode,
    estimatedSteps,
    estimatedTime,
  };
}
