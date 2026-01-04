/**
 * Claude Agent SDK 工具配置示例
 * 展示如何在Alou项目中配置和使用Claude Agent SDK的内置工具
 */

// 示例1：基础Agent配置（使用所有核心工具）
export const BASIC_AGENT_CONFIG = {
  mode: 'agent',
  categories: ['CORE'], // 只使用核心工具
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 4000,
  temperature: 0.7,
  agentInfo: {
    name: '基础编程助手',
    role_description: '帮助用户进行文件操作、代码搜索和终端命令执行',
    custom_instructions: '优先使用read/write/edit工具进行文件操作，使用glob/grep进行代码搜索'
  }
};

// 示例2：Web3专家Agent配置
export const WEB3_AGENT_CONFIG = {
  mode: 'alou',
  categories: ['WEB3', 'CORE', 'NETWORK'],
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 4000,
  temperature: 0.7,
  agentInfo: {
    name: 'Web3支付专家',
    role_description: '专注于区块链支付、智能合约和钱包管理',
    custom_instructions: '优先使用Web3相关工具，对于复杂操作可以使用plan工具进行规划'
  }
};

// 示例3：全功能Agent配置（使用所有工具）
export const FULL_FEATURED_AGENT_CONFIG = {
  mode: 'agent',
  categories: ['CORE', 'NETWORK', 'CONTROL_FLOW', 'WEB3', 'MCP'],
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 4000,
  temperature: 0.7,
  agentInfo: {
    name: '全功能助手',
    role_description: '拥有文件操作、网络搜索、任务规划和Web3能力的全能助手',
    custom_instructions: '根据任务类型选择最合适的工具，复杂任务使用plan工具'
  }
};

// 示例4：开发者工具专用配置
export const DEVELOPER_AGENT_CONFIG = {
  mode: 'agent',
  categories: ['CORE'], // 只使用核心开发工具
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 4000,
  temperature: 0.3, // 较低温度以获得更确定的输出
  agentInfo: {
    name: '代码开发助手',
    role_description: '专注于代码编写、文件操作和终端命令',
    custom_instructions: `
1. 对于代码编辑，优先使用edit工具进行精确修改
2. 对于文件操作，使用read/write工具
3. 对于代码搜索，使用glob和grep工具
4. 对于构建/测试命令，使用bash工具
5. 保持代码风格一致，遵循项目规范
    `
  }
};

// 示例5：研究助手配置
export const RESEARCH_AGENT_CONFIG = {
  mode: 'agent',
  categories: ['NETWORK', 'CORE'],
  model: 'claude-3-5-sonnet-20241022',
  maxTokens: 4000,
  temperature: 0.8, // 较高温度以获得更多创意
  agentInfo: {
    name: '研究助手',
    role_description: '帮助用户进行网络研究和信息收集',
    custom_instructions: `
1. 使用web_search工具获取最新信息
2. 使用web_fetch工具获取网页内容
3. 使用write工具整理研究笔记
4. 对于复杂研究任务，使用plan工具制定研究计划
5. 确保信息来源可靠，提供引用
    `
  }
};

// 工具使用示例
export const TOOL_USAGE_EXAMPLES = {
  // Bash工具示例
  bash: {
    description: '运行终端命令',
    example: {
      tool: 'bash',
      arguments: {
        command: 'ls -la',
        working_directory: '/home/user/projects'
      }
    }
  },
  
  // Read工具示例
  read: {
    description: '读取文件内容',
    example: {
      tool: 'read',
      arguments: {
        path: 'package.json',
        encoding: 'utf-8'
      }
    }
  },
  
  // Write工具示例
  write: {
    description: '创建新文件',
    example: {
      tool: 'write',
      arguments: {
        path: 'README.md',
        content: '# 项目说明\n\n这是一个示例项目。',
        encoding: 'utf-8'
      }
    }
  },
  
  // Edit工具示例
  edit: {
    description: '修改文件内容',
    example: {
      tool: 'edit',
      arguments: {
        path: 'src/index.js',
        old_string: 'console.log("Hello");',
        new_string: 'console.log("Hello World!");'
      }
    }
  },
  
  // Glob工具示例
  glob: {
    description: '搜索文件',
    example: {
      tool: 'glob',
      arguments: {
        pattern: '**/*.ts',
        target_directory: 'src'
      }
    }
  },
  
  // Grep工具示例
  grep: {
    description: '搜索文件内容',
    example: {
      tool: 'grep',
      arguments: {
        pattern: 'function\\s+\\w+\\(',
        path: 'src',
        case_insensitive: false
      }
    }
  },
  
  // Web Search工具示例
  web_search: {
    description: '搜索网络信息',
    example: {
      tool: 'web_search',
      arguments: {
        query: 'Claude Agent SDK最新版本',
        provider: 'tavily',
        max_results: 5
      }
    }
  },
  
  // Plan工具示例
  plan: {
    description: '制定任务计划',
    example: {
      tool: 'plan',
      arguments: {
        task: '重构用户认证模块',
        steps: [
          '1. 分析当前认证代码',
          '2. 设计新的认证架构',
          '3. 实现核心认证逻辑',
          '4. 添加测试用例',
          '5. 部署和验证'
        ]
      }
    }
  },
  
  // Subagents工具示例
  subagents: {
    description: '创建子Agent',
    example: {
      tool: 'subagents',
      arguments: {
        task: '为项目编写单元测试',
        agent_type: 'tester',
        instructions: '使用Jest框架，覆盖率达到80%以上'
      }
    }
  }
};

// 如何在项目中使用这些配置
export const USAGE_EXAMPLES = {
  // 在AgentService中使用
  agentServiceUsage: `
import { AgentService } from './services/agentService';
import { DEVELOPER_AGENT_CONFIG } from './config/claude-agent-tools.example';

const agentService = new AgentService();

// 发送消息时使用特定配置
const response = await agentService.sendMessage(
  sessionId,
  message,
  walletAddress,
  {
    mode: 'agent',
    model: DEVELOPER_AGENT_CONFIG.model,
    maxTokens: DEVELOPER_AGENT_CONFIG.maxTokens,
    temperature: DEVELOPER_AGENT_CONFIG.temperature,
    agentName: DEVELOPER_AGENT_CONFIG.agentInfo.name,
    roleDescription: DEVELOPER_AGENT_CONFIG.agentInfo.role_description,
    customInstructions: DEVELOPER_AGENT_CONFIG.agentInfo.custom_instructions
  }
);
  `,
  
  // 在UI组件中使用
  uiComponentUsage: `
import { useState } from 'react';
import { AgentService } from '../services/agentService';
import { WEB3_AGENT_CONFIG } from '../config/claude-agent-tools.example';

function AgentChat() {
  const [message, setMessage] = useState('');
  const agentService = new AgentService();
  
  const handleSend = async () => {
    const response = await agentService.sendMessage(
      sessionId,
      message,
      walletAddress,
      {
        mode: 'alou',
        ...WEB3_AGENT_CONFIG
      }
    );
    
    // 处理响应，包括工具调用
    if (response.tool_calls && response.tool_calls.length > 0) {
      console.log('Agent调用了工具:', response.tool_calls);
      
      // 显示工具调用结果
      if (response.tool_results) {
        response.tool_results.forEach(result => {
          console.log(\`工具 \${result.tool} 执行结果:\`, result);
        });
      }
    }
  };
  
  return (
    // UI组件代码
  );
}
  `
};

// 安全注意事项
export const SECURITY_NOTES = {
  bash_tool: `
Bash工具安全注意事项：
1. 避免执行未知来源的命令
2. 对于敏感操作（如rm -rf），要求用户确认
3. 限制工作目录范围
4. 记录所有执行的命令
  `,
  
  file_operations: `
文件操作安全注意事项：
1. 重要文件操作前建议备份
2. 避免覆盖重要系统文件
3. 检查文件路径安全性
4. 限制文件操作范围
  `,
  
  web_operations: `
网络操作安全注意事项：
1. 验证URL安全性
2. 避免访问恶意网站
3. 注意个人信息保护
4. 使用HTTPS连接
  `
};

export default {
  BASIC_AGENT_CONFIG,
  WEB3_AGENT_CONFIG,
  FULL_FEATURED_AGENT_CONFIG,
  DEVELOPER_AGENT_CONFIG,
  RESEARCH_AGENT_CONFIG,
  TOOL_USAGE_EXAMPLES,
  USAGE_EXAMPLES,
  SECURITY_NOTES
};
