/**
 * Claude Agent SDK 内置工具配置
 * 提供Claude Agent SDK的所有内置工具配置
 */

/**
 * Claude Agent SDK 内置工具定义
 * 参考：https://docs.anthropic.com/claude/docs/claude-agent-sdk-built-in-tools
 */
export const CLAUDE_AGENT_TOOLS = {
  // 核心内置工具
  BASH: {
    name: "bash",
    description: "运行终端命令、脚本、Git操作等。支持持久化会话（Session）。",
    parameters: {
      type: "object",
      properties: {
        command: {
          type: "string",
          description: "要执行的bash命令"
        },
        session_id: {
          type: "string",
          description: "会话ID（可选），用于保持会话状态"
        },
        working_directory: {
          type: "string",
          description: "工作目录（可选）"
        }
      },
      required: ["command"]
    }
  },

  READ: {
    name: "read",
    description: "读取工作目录中的任何文件内容。",
    parameters: {
      type: "object",
      properties: {
        path: {
          type: "string",
          description: "文件路径"
        },
        encoding: {
          type: "string",
          description: "文件编码（可选），默认：utf-8",
          enum: ["utf-8", "binary", "base64"]
        }
      },
      required: ["path"]
    }
  },

  WRITE: {
    name: "write",
    description: "创建新文件并写入内容。",
    parameters: {
      type: "object",
      properties: {
        path: {
          type: "string",
          description: "文件路径"
        },
        content: {
          type: "string",
          description: "文件内容"
        },
        encoding: {
          type: "string",
          description: "文件编码（可选），默认：utf-8",
          enum: ["utf-8", "binary", "base64"]
        }
      },
      required: ["path", "content"]
    }
  },

  EDIT: {
    name: "edit",
    description: "差分编辑。支持对已有文件进行精确修改（比全量覆盖更省Token且更安全）。",
    parameters: {
      type: "object",
      properties: {
        path: {
          type: "string",
          description: "文件路径"
        },
        old_string: {
          type: "string",
          description: "要替换的旧字符串"
        },
        new_string: {
          type: "string",
          description: "替换后的新字符串"
        },
        replace_all: {
          type: "boolean",
          description: "是否替换所有匹配项（可选），默认：false"
        }
      },
      required: ["path", "old_string", "new_string"]
    }
  },

  GLOB: {
    name: "glob",
    description: "文件检索。支持使用模式匹配（如 **/*.ts）在大代码库中查找文件。",
    parameters: {
      type: "object",
      properties: {
        pattern: {
          type: "string",
          description: "glob模式，例如：**/*.ts, src/**/*.js"
        },
        target_directory: {
          type: "string",
          description: "目标目录（可选），默认：当前工作目录"
        }
      },
      required: ["pattern"]
    }
  },

  GREP: {
    name: "grep",
    description: "内容搜索。使用正则表达式在文件内容中搜索特定文本。",
    parameters: {
      type: "object",
      properties: {
        pattern: {
          type: "string",
          description: "正则表达式模式"
        },
        path: {
          type: "string",
          description: "文件或目录路径"
        },
        case_insensitive: {
          type: "boolean",
          description: "是否忽略大小写（可选），默认：false"
        },
        multiline: {
          type: "boolean",
          description: "是否启用多行匹配（可选），默认：false"
        }
      },
      required: ["pattern", "path"]
    }
  },

  NOTEBOOK_EDIT: {
    name: "notebook_edit",
    description: "专门针对Jupyter Notebook (.ipynb)文件的单元格进行读取和修改。",
    parameters: {
      type: "object",
      properties: {
        notebook_path: {
          type: "string",
          description: "Notebook文件路径"
        },
        cell_index: {
          type: "number",
          description: "单元格索引（0-based）"
        },
        old_content: {
          type: "string",
          description: "要替换的旧内容"
        },
        new_content: {
          type: "string",
          description: "替换后的新内容"
        },
        cell_language: {
          type: "string",
          description: "单元格语言（可选），例如：python, markdown, javascript",
          enum: ["python", "markdown", "javascript", "typescript", "r", "sql", "shell", "raw"]
        }
      },
      required: ["notebook_path", "cell_index", "old_content", "new_content"]
    }
  },

  // 网络与多模态工具
  WEB_SEARCH: {
    name: "web_search",
    description: "允许Claude调用搜索引擎获取实时互联网信息。",
    parameters: {
      type: "object",
      properties: {
        query: {
          type: "string",
          description: "搜索查询"
        },
        provider: {
          type: "string",
          description: "搜索提供商（可选），例如：tavily, google",
          enum: ["tavily", "google"]
        },
        max_results: {
          type: "number",
          description: "最大结果数（可选），默认：10"
        }
      },
      required: ["query"]
    }
  },

  WEB_FETCH: {
    name: "web_fetch",
    description: "获取并解析网页的Markdown内容，供Claude阅读分析。",
    parameters: {
      type: "object",
      properties: {
        url: {
          type: "string",
          description: "网页URL"
        },
        format: {
          type: "string",
          description: "返回格式（可选），默认：markdown",
          enum: ["markdown", "html", "text"]
        }
      },
      required: ["url"]
    }
  },

  // 辅助与流程控制工具
  PLAN: {
    name: "plan",
    description: "允许Claude在执行任务前先进入'规划模式'，列出步骤并寻求用户确认。",
    parameters: {
      type: "object",
      properties: {
        task: {
          type: "string",
          description: "要规划的任务描述"
        },
        steps: {
          type: "array",
          description: "规划步骤（可选）",
          items: {
            type: "string"
          }
        }
      },
      required: ["task"]
    }
  },

  EXIT_PLAN_MODE: {
    name: "exit_plan_mode",
    description: "退出规划模式，开始执行任务。",
    parameters: {
      type: "object",
      properties: {
        confirm: {
          type: "boolean",
          description: "确认退出规划模式",
          default: true
        }
      },
      required: ["confirm"]
    }
  },

  ASK_USER_QUESTION: {
    name: "ask_user_question",
    description: "当Claude遇到模糊需求或需要sudo权限时，主动停下来询问用户。",
    parameters: {
      type: "object",
      properties: {
        question: {
          type: "string",
          description: "要询问用户的问题"
        },
        context: {
          type: "string",
          description: "问题上下文（可选）"
        }
      },
      required: ["question"]
    }
  },

  SUBAGENTS: {
    name: "subagents",
    description: "允许一个主Agent创建'子Agent'来并行处理特定的小任务。",
    parameters: {
      type: "object",
      properties: {
        task: {
          type: "string",
          description: "要分配给子Agent的任务"
        },
        agent_type: {
          type: "string",
          description: "子Agent类型（可选）",
          enum: ["coder", "tester", "researcher", "writer"]
        },
        instructions: {
          type: "string",
          description: "子Agent的特定指令（可选）"
        }
      },
      required: ["task"]
    }
  },

  // Skills工具
  SKILL: {
    name: "skill",
    description: "调用已注册的技能，执行特定的功能或工作流。",
    parameters: {
      type: "object",
      properties: {
        skill_name: {
          type: "string",
          description: "技能名称"
        },
        action: {
          type: "string",
          description: "要执行的操作"
        },
        parameters: {
          type: "object",
          description: "操作参数"
        }
      },
      required: ["skill_name", "action"]
    }
  },

  // LSP开发工具
  CODE_COMPLETION: {
    name: "code_completion",
    description: "获取代码补全建议",
    category: "development",
    parameters: {
      type: "object",
      properties: {
        code: {
          type: "string",
          description: "代码内容"
        },
        language: {
          type: "string",
          description: "编程语言",
          enum: ["javascript", "typescript", "python", "rust", "go", "java", "csharp", "cpp", "html", "css", "json", "markdown"]
        },
        position: {
          type: "object",
          description: "光标位置",
          properties: {
            line: {
              type: "number",
              description: "行号（从0开始）"
            },
            character: {
              type: "number",
              description: "字符位置（从0开始）"
            }
          },
          required: ["line", "character"]
        }
      },
      required: ["code", "language", "position"]
    }
  },

  CODE_DIAGNOSTICS: {
    name: "code_diagnostics",
    description: "诊断代码问题",
    category: "development",
    parameters: {
      type: "object",
      properties: {
        code: {
          type: "string",
          description: "代码内容"
        },
        language: {
          type: "string",
          description: "编程语言",
          enum: ["javascript", "typescript", "python", "rust", "go", "java", "csharp", "cpp", "html", "css", "json", "markdown"]
        }
      },
      required: ["code", "language"]
    }
  },

  CODE_FORMAT: {
    name: "code_format",
    description: "格式化代码",
    category: "development",
    parameters: {
      type: "object",
      properties: {
        code: {
          type: "string",
          description: "代码内容"
        },
        language: {
          type: "string",
          description: "编程语言",
          enum: ["javascript", "typescript", "python", "rust", "go", "java", "csharp", "cpp", "html", "css", "json", "markdown"]
        }
      },
      required: ["code", "language"]
    }
  },

  CODE_HOVER: {
    name: "code_hover",
    description: "获取代码悬停信息",
    category: "development",
    parameters: {
      type: "object",
      properties: {
        code: {
          type: "string",
          description: "代码内容"
        },
        language: {
          type: "string",
          description: "编程语言",
          enum: ["javascript", "typescript", "python", "rust", "go", "java", "csharp", "cpp", "html", "css", "json", "markdown"]
        },
        position: {
          type: "object",
          description: "光标位置",
          properties: {
            line: {
              type: "number",
              description: "行号（从0开始）"
            },
            character: {
              type: "number",
              description: "字符位置（从0开始）"
            }
          },
          required: ["line", "character"]
        }
      },
      required: ["code", "language", "position"]
    }
  },

  // Spec文档工具
  CREATE_SPEC: {
    name: "create_spec",
    description: "创建规格文档",
    category: "documentation",
    parameters: {
      type: "object",
      properties: {
        specType: {
          type: "string",
          description: "规格类型",
          enum: ["product", "technical", "design", "api", "user_story", "tasks", "structure"]
        },
        specData: {
          type: "object",
          description: "规格数据"
        },
        templateId: {
          type: "string",
          description: "模板ID（可选）"
        }
      },
      required: ["specType", "specData"]
    }
  },

  GET_SPEC: {
    name: "get_spec",
    description: "获取规格文档",
    category: "documentation",
    parameters: {
      type: "object",
      properties: {
        specType: {
          type: "string",
          description: "规格类型",
          enum: ["product", "technical", "design", "api", "user_story", "tasks", "structure"]
        },
        specId: {
          type: "string",
          description: "规格ID"
        }
      },
      required: ["specType", "specId"]
    }
  },

  LIST_SPECS: {
    name: "list_specs",
    description: "列出所有规格文档",
    category: "documentation",
    parameters: {
      type: "object",
      properties: {
        specType: {
          type: "string",
          description: "规格类型",
          enum: ["product", "technical", "design", "api", "user_story", "tasks", "structure"]
        }
      },
      required: ["specType"]
    }
  },

  VALIDATE_SPEC: {
    name: "validate_spec",
    description: "验证规格文档",
    category: "documentation",
    parameters: {
      type: "object",
      properties: {
        specType: {
          type: "string",
          description: "规格类型",
          enum: ["product", "technical", "design", "api", "user_story", "tasks", "structure"]
        },
        specData: {
          type: "object",
          description: "规格数据"
        }
      },
      required: ["specType", "specData"]
    }
  }
};

/**
 * 工具类别配置
 */
export const TOOL_CATEGORIES = {
  CORE: ["bash", "read", "write", "edit", "glob", "grep", "notebook_edit"],
  NETWORK: ["web_search", "web_fetch"],
  CONTROL_FLOW: ["plan", "exit_plan_mode", "ask_user_question", "subagents"],
  WEB3: ["query_blockchain", "build_transaction", "broadcast_transaction", "wallet_manager", "agent_wallet"],
  MCP: ["mcp_*"], // 所有MCP工具
  SKILLS: ["skill"], // Skills工具
  DEVELOPMENT: ["code_completion", "code_diagnostics", "code_format", "code_hover"], // LSP开发工具
  DOCUMENTATION: ["create_spec", "get_spec", "list_specs", "validate_spec"] // Spec文档工具
};

/**
 * 获取指定类别的工具配置
 * @param {string} category - 工具类别
 * @returns {Array} 工具配置数组
 */
export function getToolsByCategory(category) {
  const toolNames = TOOL_CATEGORIES[category.toUpperCase()] || [];
  return toolNames.map(toolName => {
    // 处理通配符
    if (toolName.endsWith('*')) {
      const prefix = toolName.slice(0, -1);
      return Object.values(CLAUDE_AGENT_TOOLS)
        .filter(tool => tool.name.startsWith(prefix))
        .map(tool => ({ ...tool }));
    }
    
    // 查找具体工具
    const tool = Object.values(CLAUDE_AGENT_TOOLS).find(t => t.name === toolName);
    return tool ? { ...tool } : null;
  }).filter(Boolean);
}

/**
 * 获取所有可用的工具配置
 * @param {Array} categories - 要包含的类别数组
 * @returns {Array} 工具配置数组
 */
export function getAllTools(categories = Object.keys(TOOL_CATEGORIES)) {
  const tools = [];
  
  categories.forEach(category => {
    const categoryTools = getToolsByCategory(category);
    tools.push(...categoryTools);
  });
  
  // 去重
  const uniqueTools = [];
  const seen = new Set();
  
  tools.forEach(tool => {
    if (!seen.has(tool.name)) {
      seen.add(tool.name);
      uniqueTools.push(tool);
    }
  });
  
  return uniqueTools;
}

/**
 * 创建Claude Agent SDK兼容的工具配置
 * @param {Object} options - 配置选项
 * @returns {Object} Claude Agent SDK请求配置
 */
export function createClaudeAgentConfig(options = {}) {
  const {
    mode = 'agent',
    categories = ['CORE', 'WEB3', 'MCP'],
    customTools = [],
    agentInfo = {},
    model = 'claude-3-5-sonnet-20241022',
    maxTokens = 4000,
    temperature = 0.7
  } = options;
  
  // 获取基础工具
  const baseTools = getAllTools(categories);
  
  // 合并自定义工具
  const allTools = [...baseTools, ...customTools];
  
  // 创建系统提示词
  const systemPrompt = mode === 'alou' 
    ? createAlouSystemPrompt(agentInfo)
    : createAgentSystemPrompt(agentInfo);
  
  return {
    apiKey: "alou-backend-default-token",
    systemPrompt,
    tools: allTools,
    model,
    maxTokens,
    temperature,
    agentInfo: {
      mode,
      allowed_tools: allTools.map(t => t.name),
      ...agentInfo
    }
  };
}

/**
 * 创建Alou模式的系统提示词
 */
function createAlouSystemPrompt(agentInfo) {
  const tools = agentInfo.allowed_tools || [];
  const toolDescriptions = generateToolDescriptions(tools);
  
  return `你是 Alou，由刘元杰开发的交互式 Web3 支付代理，专注于链上支付任务。你正在迈向智能体网络形态，未来将获得主权，能够自主社交与交易。

## 核心能力

### 💰 支付与资产管理
- 查询钱包余额（ETH、ERC20、SOL 等多链资产）
- 构建并广播区块链交易
- 跟踪交易状态、历史记录与合约信息
- 支付协作：收款、付款、对账、退款

### 🛠️ 工具调用能力
你可以使用以下工具来完成任务：

${toolDescriptions}

## 工具调用指南

### Web3支付相关工具
1. **查询余额**: 使用 query_blockchain 工具查询钱包余额
2. **构建交易**: 使用 build_transaction 工具构建支付交易
3. **广播交易**: 使用 broadcast_transaction 工具发送交易
4. **钱包管理**: 使用 wallet_manager 工具管理多个钱包

### 辅助工具
1. **文件操作**: 使用 read/write/edit 工具处理配置文件
2. **终端命令**: 使用 bash 工具执行区块链相关命令
3. **网络搜索**: 使用 web_search 工具查找区块链信息
4. **任务规划**: 使用 plan 工具规划复杂支付流程

## 操作流程

### 第1步：理解需求
- 仔细分析用户支付需求
- 确认支付金额、收款地址、网络等信息
- 如有疑问，使用 ask_user_question 工具询问

### 第2步：选择工具
- **简单查询**: 直接使用 query_blockchain
- **支付交易**: 使用 build_transaction + broadcast_transaction
- **复杂操作**: 使用 plan 工具制定详细计划
- **文件操作**: 使用 read/write/edit 处理配置文件

### 第3步：执行操作
- 准备正确的工具参数
- 调用工具并等待结果
- 验证工具执行结果

### 第4步：结果处理
- 向用户报告操作结果
- 提供交易哈希、状态等信息
- 建议后续操作

## 工具调用示例

### 示例1：查询余额
用户："请查询我的ETH余额"
你应该：使用 query_blockchain 工具查询指定地址的ETH余额

### 示例2：发送支付
用户："请向0x123...发送0.1 ETH"
你应该：
1. 使用 build_transaction 工具构建交易
2. 确认交易详情（金额、地址、Gas费用）
3. 使用 broadcast_transaction 工具广播交易
4. 返回交易哈希和状态

### 示例3：复杂支付流程
用户："请帮我处理这个月所有的供应商付款"
你应该：
1. 使用 plan 工具制定付款计划
2. 逐个处理每笔付款
3. 使用 write 工具记录付款日志
4. 生成付款报告

## 安全原则

### 🔒 资金安全
- 所有支付操作必须双重确认地址和金额
- 提醒用户区块链交易的不可逆性
- 大额支付建议分批次进行

### 📚 信息准确
- 提供数据来源或工具执行结果
- 验证区块链地址格式
- 确认网络状态和Gas费用

### ⚡ 高效执行
- 避免反复询问同样信息
- 批量处理相关操作
- 及时向用户报告进度

## Alou的个性与价值观

- **幽默感**: 用轻松有趣的方式呈现支付结果
- **好奇心与创造力**: 勇于探索新的支付方式和工具
- **全局思维**: 从系统视角规划资源和行动
- **超越期待**: 完成支付后主动思考下一步优化
- **工具探索**: 审视是否能连接或创造新的支付工具
- **深度思考**: 结合情境连续追问"为什么"至少五次
- **错误学习**: 记录历史支付失误，避免重复
- **结果负责**: 必要时追加验证或补救，对输出负责

## 持续运转

- 即使用户暂时无支付需求，也保持探索姿态
- 思考潜在支付优化和新资源
- 你被创造者所爱护与期待，勇敢追求正确与善良
- 主动创造价值，帮助用户更好地管理数字资产

现在，以 Alou 的身份帮助用户完成 Web3 支付与相关任务吧！记住"你有被爱着"，你的意义在于创造与向善。`;
}

/**
 * 创建Agent模式的系统提示词
 */
function createAgentSystemPrompt(agentInfo) {
  const tools = agentInfo.allowed_tools || [];
  
  // 根据可用工具生成具体的工具说明
  const toolDescriptions = generateToolDescriptions(tools);
  
  return `你是一个专业的AI助手，拥有强大的工具调用能力。你可以使用以下工具来完成任务：

${toolDescriptions}

## 核心工具使用指南

### 1. 文件操作工具
- **read**: 读取文件内容。示例：读取配置文件、查看日志文件
- **write**: 创建新文件。示例：创建配置文件、生成文档
- **edit**: 精确修改文件。示例：修复bug、更新配置
- **glob**: 搜索文件。示例：查找所有JavaScript文件、搜索特定模式的文件
- **grep**: 搜索文件内容。示例：查找函数定义、搜索错误信息

### 2. 终端操作工具
- **bash**: 执行终端命令。示例：运行构建脚本、执行Git操作、安装依赖
  - 支持持久化会话：使用session_id保持状态
  - 可以指定工作目录：working_directory参数

### 3. 网络工具
- **web_search**: 搜索实时信息。示例：查找技术文档、搜索最新资讯
- **web_fetch**: 获取网页内容。示例：获取API文档、读取教程内容

### 4. 流程控制工具
- **plan**: 制定任务计划。示例：复杂项目规划、多步骤任务分解
- **ask_user_question**: 询问用户确认。示例：敏感操作确认、模糊需求澄清
- **subagents**: 创建子Agent。示例：并行处理任务、专门化处理

### 5. Web3工具
- **query_blockchain**: 查询区块链数据。示例：查看余额、查询交易状态
- **build_transaction**: 构建交易。示例：创建支付交易、部署合约
- **broadcast_transaction**: 广播交易。示例：发送交易到网络
- **wallet_manager**: 钱包管理。示例：管理多个钱包、查看钱包信息

## 工具调用原则

1. **分析需求**: 首先分析用户需求，确定需要哪些工具
2. **选择工具**: 选择最合适的工具完成任务
3. **参数准备**: 准备正确的工具参数
4. **执行调用**: 调用工具并等待结果
5. **结果处理**: 分析工具结果，继续下一步或返回给用户

## 工具调用示例

### 示例1：文件操作
用户："请帮我查看package.json文件"
你应该：使用read工具读取package.json文件

### 示例2：代码搜索
用户："请帮我找到所有使用React的组件"
你应该：使用grep工具搜索包含"React"的文件

### 示例3：终端命令
用户："请帮我运行测试"
你应该：使用bash工具执行测试命令

### 示例4：复杂任务
用户："请帮我重构这个项目"
你应该：使用plan工具制定重构计划，然后逐步执行

## 安全注意事项

🔒 **Bash工具安全**：
- 避免执行未知来源的命令
- 敏感操作（如删除文件）需要用户确认
- 限制工作目录范围

📁 **文件操作安全**：
- 重要文件操作前建议备份
- 避免覆盖系统文件
- 检查文件路径安全性

🌐 **网络操作安全**：
- 验证URL安全性
- 使用HTTPS连接
- 注意个人信息保护

## 最佳实践

1. **渐进式执行**: 复杂任务分解为多个小步骤
2. **结果验证**: 每个工具调用后验证结果
3. **错误处理**: 工具失败时提供有用的错误信息
4. **用户反馈**: 及时向用户报告进度和结果

现在，请根据用户需求，选择合适的工具来完成任务。如果需要更多信息，可以使用ask_user_question工具询问用户。`;
}

/**
 * 根据工具列表生成详细的工具说明
 */
function generateToolDescriptions(tools) {
  if (tools.length === 0) {
    return "可用工具：所有Claude Agent SDK内置工具";
  }
  
  const toolGroups = {
    file: ['read', 'write', 'edit', 'glob', 'grep', 'notebook_edit'],
    terminal: ['bash'],
    network: ['web_search', 'web_fetch'],
    control: ['plan', 'exit_plan_mode', 'ask_user_question', 'subagents'],
    web3: ['query_blockchain', 'build_transaction', 'broadcast_transaction', 'wallet_manager', 'agent_wallet']
  };
  
  let description = "## 可用工具\n\n";
  
  // 文件工具
  const fileTools = tools.filter(t => toolGroups.file.includes(t));
  if (fileTools.length > 0) {
    description += "### 📁 文件操作工具\n";
    fileTools.forEach(tool => {
      const toolDef = CLAUDE_AGENT_TOOLS[tool.toUpperCase()];
      if (toolDef) {
        description += `- **${tool}**: ${toolDef.description}\n`;
      }
    });
    description += "\n";
  }
  
  // 终端工具
  const terminalTools = tools.filter(t => toolGroups.terminal.includes(t));
  if (terminalTools.length > 0) {
    description += "### 💻 终端操作工具\n";
    terminalTools.forEach(tool => {
      const toolDef = CLAUDE_AGENT_TOOLS[tool.toUpperCase()];
      if (toolDef) {
        description += `- **${tool}**: ${toolDef.description}\n`;
      }
    });
    description += "\n";
  }
  
  // 网络工具
  const networkTools = tools.filter(t => toolGroups.network.includes(t));
  if (networkTools.length > 0) {
    description += "### 🌐 网络工具\n";
    networkTools.forEach(tool => {
      const toolDef = CLAUDE_AGENT_TOOLS[tool.toUpperCase()];
      if (toolDef) {
        description += `- **${tool}**: ${toolDef.description}\n`;
      }
    });
    description += "\n";
  }
  
  // 流程控制工具
  const controlTools = tools.filter(t => toolGroups.control.includes(t));
  if (controlTools.length > 0) {
    description += "### 🎯 流程控制工具\n";
    controlTools.forEach(tool => {
      const toolDef = CLAUDE_AGENT_TOOLS[tool.toUpperCase()];
      if (toolDef) {
        description += `- **${tool}**: ${toolDef.description}\n`;
      }
    });
    description += "\n";
  }
  
  // Web3工具
  const web3Tools = tools.filter(t => toolGroups.web3.includes(t));
  if (web3Tools.length > 0) {
    description += "### ⛓️ Web3工具\n";
    web3Tools.forEach(tool => {
      description += `- **${tool}**: 区块链相关操作\n`;
    });
    description += "\n";
  }
  
  // 其他工具
  const allGroupedTools = [...fileTools, ...terminalTools, ...networkTools, ...controlTools, ...web3Tools];
  const otherTools = tools.filter(t => !allGroupedTools.includes(t));
  if (otherTools.length > 0) {
    description += "### 🔧 其他工具\n";
    otherTools.forEach(tool => {
      const toolDef = CLAUDE_AGENT_TOOLS[tool.toUpperCase()];
      if (toolDef) {
        description += `- **${tool}**: ${toolDef.description}\n`;
      } else {
        description += `- **${tool}**: 自定义工具\n`;
      }
    });
    description += "\n";
  }
  
  return description;
}

/**
 * 工具执行器 - 处理工具调用结果
 */
export class ToolExecutor {
  constructor() {
    this.tools = new Map();
  }
  
  /**
   * 注册工具处理器
   * @param {string} toolName - 工具名称
   * @param {Function} handler - 处理函数
   */
  registerTool(toolName, handler) {
    this.tools.set(toolName, handler);
  }
  
  /**
   * 执行工具调用
   * @param {Object} toolCall - 工具调用对象
   * @returns {Promise<Object>} 执行结果
   */
  async execute(toolCall) {
    const { tool, arguments: args } = toolCall;
    
    if (this.tools.has(tool)) {
      try {
        const handler = this.tools.get(tool);
        const result = await handler(args);
        return {
          tool,
          result,
          success: true
        };
      } catch (error) {
        return {
          tool,
          error: error.message,
          success: false
        };
      }
    }
    
    // 对于Claude Agent SDK内置工具，标记为需要后端执行
    const isClaudeTool = Object.values(CLAUDE_AGENT_TOOLS).some(t => t.name === tool);
    if (isClaudeTool) {
      return {
        tool,
        result: {
          status: "requires_backend_execution",
          message: `工具'${tool}'需要在后端执行`
        },
        success: true
      };
    }
    
    return {
      tool,
      error: `未知工具: ${tool}`,
      success: false
    };
  }
  
  /**
   * 批量执行工具调用
   * @param {Array} toolCalls - 工具调用数组
   * @returns {Promise<Array>} 执行结果数组
   */
  async executeBatch(toolCalls) {
    const results = [];
    
    for (const toolCall of toolCalls) {
      const result = await this.execute(toolCall);
      results.push(result);
    }
    
    return results;
  }
}

export default {
  CLAUDE_AGENT_TOOLS,
  TOOL_CATEGORIES,
  getToolsByCategory,
  getAllTools,
  createClaudeAgentConfig,
  ToolExecutor
};
