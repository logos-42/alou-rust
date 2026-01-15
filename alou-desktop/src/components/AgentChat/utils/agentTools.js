import { createClaudeAgentConfig, CLAUDE_AGENT_TOOLS, TOOL_CATEGORIES } from '@/services/claudeAgentTools'

/**
 * 根据模式和智能体信息获取工具类别
 */
export const getToolCategoriesByMode = (mode, agentInfo) => {
  // 基础类别配置
  const baseCategories = ['CORE', 'NETWORK', 'CONTROL_FLOW'];

  // 根据模式添加特定类别
  if (mode === 'alou') {
    return ['WEB3', ...baseCategories];
  }

  // Agent模式：检查智能体是否有特定配置
  if (agentInfo?.tool_categories) {
    return agentInfo.tool_categories;
  }

  // 默认返回所有基础类别
  return baseCategories;
};

/**
 * 根据类别获取工具配置
 */
export const getToolsByCategories = (categories) => {
  try {
    // 直接使用已导入的工具配置
    const tools = [];
    const seen = new Set();

    categories.forEach(category => {
      const categoryTools = TOOL_CATEGORIES[category.toUpperCase()] || [];
      categoryTools.forEach(toolName => {
        // 处理通配符
        if (toolName.endsWith('*')) {
          const prefix = toolName.slice(0, -1);
          Object.values(CLAUDE_AGENT_TOOLS).forEach(tool => {
            if (tool.name.startsWith(prefix) && !seen.has(tool.name)) {
              seen.add(tool.name);
              tools.push({ ...tool });
            }
          });
        } else {
          // 查找具体工具
          const tool = Object.values(CLAUDE_AGENT_TOOLS).find(t => t.name === toolName);
          if (tool && !seen.has(tool.name)) {
            seen.add(tool.name);
            tools.push({ ...tool });
          }
        }
      });
    });

    return tools;
  } catch (error) {
    console.error('[getToolsByCategories] 获取工具配置失败:', error);
    return [];
  }
};