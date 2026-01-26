// 工具配置和类别管理

/**
 * 根据模式和智能体信息获取工具类别
 * @param {string} mode - 当前模式 ('alou', 'agent', 'group_chat')
 * @param {object} agentInfo - 智能体信息
 * @returns {string[]} 工具类别数组
 */
export const getToolCategoriesByMode = (mode, agentInfo) => {
  console.log('[getToolCategoriesByMode] 参数:', { mode, hasAgentInfo: !!agentInfo });

  switch (mode) {
    case 'alou':
      // Alou模式：专注于Web3支付和区块链操作
      return ['WEB3', 'CORE'];

    case 'agent':
      // Agent模式：通用智能体，支持文件操作、终端、网络、搜索、计划管理等
      return ['CORE', 'NETWORK', 'CONTROL_FLOW', 'FILESYSTEM', 'SEARCH', 'PLANNING'];

    case 'group_chat':
      // 群聊模式：简化工具集，避免复杂操作
      return ['CORE'];

    default:
      // 默认使用核心工具
      return ['CORE'];
  }
};

/**
 * 根据工具类别获取工具列表
 * @param {string[]} categories - 工具类别数组
 * @returns {object[]} 工具对象数组
 */
export const getToolsByCategories = (categories) => {
  console.log('[getToolsByCategories] 请求的类别:', categories);

  try {
    // 简化的工具定义（与后端保持一致）
    const SIMPLE_TOOLS = {
      // 核心工具
      bash: {
        name: "bash",
        description: "当任务需要执行命令或获得真实终端结果时优先使用。用于运行命令、脚本、Git操作（Bash/CMD/PowerShell/Python/Node）。返回的输出必须作为后续判断依据。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "操作类型", enum: ["execute"], default: "execute" },
            shell: { type: "string", description: "Shell类型", enum: ["bash", "cmd", "powershell", "python", "node"], default: "bash" },
            command: { type: "string", description: "要执行的命令" },
            working_dir: { type: "string", description: "工作目录（可选）" },
            environment: { type: "array", description: "环境变量（可选）", items: { type: "array", items: { type: "string" } } },
            timeout_seconds: { type: "number", description: "超时时间（秒，可选）" }
          },
          required: ["command"]
        }
      },

      // 文件系统工具
      filesystem: {
        name: "filesystem",
        description: "当需要读取/写入/编辑文件或验证文件内容时优先使用。支持读取、写入、编辑、列出、复制、移动、删除文件和目录。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "文件操作类型", enum: ["read", "write", "edit", "delete_lines", "delete_block", "list", "copy", "move", "delete", "dir"] },
            path: { type: "string", description: "文件或目录路径" },
            content: { type: "string", description: "写入的内容（write操作需要）" },
            create_dirs: { type: "boolean", description: "是否创建父目录（write操作可选）" },
            old_text: { type: "string", description: "要替换的旧文本（edit操作需要）" },
            new_text: { type: "string", description: "替换后的新文本（edit操作需要）" },
            start_line: { type: "number", description: "起始行号（delete_lines操作需要）" },
            end_line: { type: "number", description: "结束行号（delete_lines操作可选）" },
            block_text: { type: "string", description: "要删除的文本块（delete_block操作需要）" },
            all_occurrences: { type: "boolean", description: "是否删除所有匹配的块（delete_block操作可选）" },
            recursive: { type: "boolean", description: "是否递归操作（list/copy/delete操作可选）" },
            depth: { type: "number", description: "递归深度限制（list操作可选）" },
            src: { type: "string", description: "源路径（copy/move操作需要）" },
            dest: { type: "string", description: "目标路径（copy/move操作需要）" }
          },
          required: ["operation"]
        }
      },

      // 搜索工具
      search: {
        name: "search",
        description: "当需要确认项目结构、定位代码或查找关键文本时优先使用。支持文本搜索、文件模式匹配和高级文件查找。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "搜索操作类型", enum: ["grep", "glob", "find"] },
            pattern: { type: "string", description: "搜索模式（grep/glob操作需要）" },
            directory: { type: "string", description: "搜索目录" },
            file_pattern: { type: "string", description: "文件模式过滤（grep操作可选）" },
            case_sensitive: { type: "boolean", description: "是否区分大小写（grep操作可选）" },
            max_results: { type: "number", description: "最大结果数（grep/find操作可选）" },
            recursive: { type: "boolean", description: "是否递归搜索（glob操作可选）" },
            options: { type: "object", description: "高级查找选项（find操作需要）", properties: {
              file_type: { type: "string", enum: ["any", "file", "directory", "symlink"] },
              name_pattern: { type: "string" },
              content_pattern: { type: "string" },
              min_size: { type: "number" },
              max_size: { type: "number" },
              max_depth: { type: "number" },
              max_results: { type: "number" }
            }}
          },
          required: ["operation"]
        }
      },

      // 计划和任务管理工具
      plan: {
        name: "plan",
        description: "当任务较复杂且需要分步推进时使用。用于创建/调整计划与待办事项，帮助保证执行过程可追踪。",
        parameters: {
          type: "object",
          properties: {
            action: { type: "string", description: "操作类型", enum: ["create_plan", "update_step", "create_todo", "update_todo", "list_plans", "list_todos"] },
            name: { type: "string", description: "计划名称（create_plan需要）" },
            description: { type: "string", description: "计划描述（create_plan可选）" },
            goal: { type: "string", description: "计划目标（create_plan需要）" },
            steps: { type: "array", description: "计划步骤（create_plan需要）", items: { type: "object" } },
            plan_id: { type: "string", description: "计划ID（update_step需要）" },
            step_id: { type: "string", description: "步骤ID（update_step需要）" },
            status: { type: "string", description: "状态", enum: ["pending", "in_progress", "completed", "paused", "cancelled", "failed"] },
            progress: { type: "number", description: "进度（0.0-1.0，update_step可选）" },
            title: { type: "string", description: "待办事项标题（create_todo需要）" },
            priority: { type: "string", description: "优先级（create_todo可选）", enum: ["low", "medium", "high", "urgent"] },
            todo_id: { type: "string", description: "待办事项ID（update_todo需要）" },
            status_filter: { type: "string", description: "状态过滤器（list_todos可选）", enum: ["pending", "in_progress", "completed", "paused", "cancelled", "failed"] }
          },
          required: ["action"]
        }
      },

      // 待办事项工具
      todolist: {
        name: "todolist",
        description: "专门的待办事项管理工具。",
        parameters: {
          type: "object",
          properties: {
            action: { type: "string", description: "操作类型", enum: ["create", "update", "delete", "list", "complete"] },
            title: { type: "string", description: "待办事项标题（create需要）" },
            description: { type: "string", description: "描述（create可选）" },
            priority: { type: "string", description: "优先级（create可选）", enum: ["low", "medium", "high", "urgent"] },
            due_date: { type: "number", description: "截止时间（时间戳，create可选）" },
            id: { type: "string", description: "待办事项ID（update/delete/complete需要）" },
            status: { type: "string", description: "状态（update需要）", enum: ["pending", "in_progress", "completed", "cancelled"] },
            filter: { type: "object", description: "列表过滤器（list可选）", properties: {
              status: { type: "string", enum: ["pending", "in_progress", "completed", "cancelled"] },
              priority: { type: "string", enum: ["low", "medium", "high", "urgent"] },
              tags: { type: "array", items: { type: "string" } }
            }}
          },
          required: ["action"]
        }
      },
      read: {
        name: "read",
        description: "读取工作目录中的任何文件内容。",
        parameters: {
          type: "object",
          properties: {
            path: { type: "string", description: "文件路径" },
            encoding: { type: "string", description: "文件编码（可选）", enum: ["utf-8", "binary", "base64"] }
          },
          required: ["path"]
        }
      },
      write: {
        name: "write",
        description: "创建新文件并写入内容。",
        parameters: {
          type: "object",
          properties: {
            path: { type: "string", description: "文件路径" },
            content: { type: "string", description: "文件内容" },
            encoding: { type: "string", description: "文件编码（可选）", enum: ["utf-8", "binary", "base64"] }
          },
          required: ["path", "content"]
        }
      },
      edit: {
        name: "edit",
        description: "差分编辑。支持对已有文件进行精确修改。",
        parameters: {
          type: "object",
          properties: {
            path: { type: "string", description: "文件路径" },
            old_string: { type: "string", description: "要替换的旧字符串" },
            new_string: { type: "string", description: "替换后的新字符串" },
            replace_all: { type: "boolean", description: "是否替换所有匹配项（可选）" }
          },
          required: ["path", "old_string", "new_string"]
        }
      },
      glob: {
        name: "glob",
        description: "文件检索。支持使用模式匹配查找文件。",
        parameters: {
          type: "object",
          properties: {
            pattern: { type: "string", description: "glob模式" },
            target_directory: { type: "string", description: "目标目录（可选）" }
          },
          required: ["pattern"]
        }
      },
      grep: {
        name: "grep",
        description: "内容搜索。使用正则表达式在文件内容中搜索文本。",
        parameters: {
          type: "object",
          properties: {
            pattern: { type: "string", description: "正则表达式模式" },
            path: { type: "string", description: "文件或目录路径" },
            case_insensitive: { type: "boolean", description: "是否忽略大小写（可选）" }
          },
          required: ["pattern", "path"]
        }
      },

      // 网络工具
      web_search: {
        name: "web_search",
        description: "调用搜索引擎获取实时互联网信息。",
        parameters: {
          type: "object",
          properties: {
            query: { type: "string", description: "搜索查询" },
            provider: { type: "string", description: "搜索提供商（可选）", enum: ["tavily", "google"] },
            max_results: { type: "number", description: "最大结果数（可选）" }
          },
          required: ["query"]
        }
      },
      web_fetch: {
        name: "web_fetch",
        description: "获取并解析网页的Markdown内容。",
        parameters: {
          type: "object",
          properties: {
            url: { type: "string", description: "网页URL" },
            format: { type: "string", description: "返回格式（可选）", enum: ["markdown", "html", "text"] }
          },
          required: ["url"]
        }
      },

      ask_user_question: {
        name: "ask_user_question",
        description: "当遇到模糊需求时，主动询问用户。",
        parameters: {
          type: "object",
          properties: {
            question: { type: "string", description: "要询问用户的问题" },
            context: { type: "string", description: "问题上下文（可选）" }
          },
          required: ["question"]
        }
      },
      subagents: {
        name: "subagents",
        description: "创建'子Agent'来并行处理特定任务。",
        parameters: {
          type: "object",
          properties: {
            task: { type: "string", description: "要分配给子Agent的任务" },
            agent_type: { type: "string", description: "子Agent类型（可选）", enum: ["coder", "tester", "researcher", "writer"] },
            instructions: { type: "string", description: "子Agent的特定指令（可选）" }
          },
          required: ["task"]
        }
      },

      // Web3工具（简化版）
      query_blockchain: {
        name: "query_blockchain",
        description: "查询区块链数据（余额、交易状态等）。",
        parameters: {
          type: "object",
          properties: {
            address: { type: "string", description: "钱包地址" },
            network: { type: "string", description: "网络名称" },
            token: { type: "string", description: "代币符号（可选）" }
          },
          required: ["address", "network"]
        }
      },
      build_transaction: {
        name: "build_transaction",
        description: "构建区块链交易。",
        parameters: {
          type: "object",
          properties: {
            from: { type: "string", description: "发送方地址" },
            to: { type: "string", description: "接收方地址" },
            amount: { type: "string", description: "金额" },
            network: { type: "string", description: "网络名称" }
          },
          required: ["from", "to", "amount", "network"]
        }
      },
      broadcast_transaction: {
        name: "broadcast_transaction",
        description: "广播交易到区块链网络。",
        parameters: {
          type: "object",
          properties: {
            transaction: { type: "string", description: "交易数据" },
            network: { type: "string", description: "网络名称" }
          },
          required: ["transaction", "network"]
        }
      },
      wallet_manager: {
        name: "wallet_manager",
        description: "管理多个钱包地址。",
        parameters: {
          type: "object",
          properties: {
            action: { type: "string", description: "操作类型", enum: ["list", "add", "remove"] },
            address: { type: "string", description: "钱包地址（添加/删除时需要）" },
            label: { type: "string", description: "钱包标签（可选）" }
          },
          required: ["action"]
        }
      },
      agent_wallet: {
        name: "agent_wallet",
        description: "智能体专用钱包操作。",
        parameters: {
          type: "object",
          properties: {
            action: { type: "string", description: "操作类型", enum: ["balance", "transfer", "sign"] },
            amount: { type: "string", description: "金额（转账时需要）" },
            to: { type: "string", description: "接收地址（转账时需要）" },
            message: { type: "string", description: "签名消息（签名时需要）" }
          },
          required: ["action"]
        }
      }
    };

    // 工具类别映射
    const CATEGORY_TOOLS = {
      CORE: ["bash", "read", "write", "edit", "glob", "grep", "filesystem"],
      NETWORK: ["web_search", "web_fetch"],
      CONTROL_FLOW: ["plan", "ask_user_question", "subagents"],
      WEB3: ["query_blockchain", "build_transaction", "broadcast_transaction", "wallet_manager", "agent_wallet"],
      FILESYSTEM: ["filesystem"],
      SEARCH: ["search"],
      PLANNING: ["plan", "todolist"]
    };

    const tools = [];
    const seen = new Set();

    categories.forEach(category => {
      const categoryTools = CATEGORY_TOOLS[category.toUpperCase()] || [];
      categoryTools.forEach(toolName => {
        const tool = SIMPLE_TOOLS[toolName];
        if (tool && !seen.has(tool.name)) {
          seen.add(tool.name);
          tools.push({ ...tool });
        }
      });
    });

    console.log('[getToolsByCategories] 获取到的工具数量:', tools.length);
    console.log('[getToolsByCategories] 工具列表:', tools.map(t => t.name).join(', '));

    return tools;
  } catch (error) {
    console.error('[getToolsByCategories] 获取工具配置失败:', error);
    return [];
  }
};