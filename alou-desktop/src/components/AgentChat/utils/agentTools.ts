// 工具配置和类别管理

/**
 * 根据模式和智能体信息获取工具类别
 * @param {string} mode - 当前模式 ('alou', 'agent', 'group_chat')
 * @param {object} agentInfo - 智能体信息
 * @returns {string[]} 工具类别数组
 */
export const getToolCategoriesByMode = (mode: string, agentInfo: any) => {
  console.log('[getToolCategoriesByMode] 参数:', { mode, hasAgentInfo: !!agentInfo });

  switch (mode) {
    case 'alou':
      // Alou模式：专注于Web3支付和区块链操作
      return ['WEB3', 'CORE'];

    case 'agent':
      // Agent模式：通用智能体，支持文件操作、终端、网络、搜索、计划管理等
      return ['CORE', 'NETWORK', 'CONTROL_FLOW', 'FILESYSTEM', 'SEARCH', 'PLANNING', 'AGENT', 'DEVELOPMENT'];

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
export const getToolsByCategories = (categories: string[]) => {
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

      // Git助手工具
      git_helper: {
        name: "git_helper",
        description: "Git版本控制操作助手。支持提交、分支、合并、远程操作等Git功能。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "Git操作类型", enum: ["status", "add", "commit", "push", "pull", "branch", "merge", "checkout", "log", "diff"] },
            files: { type: "array", description: "文件列表（add操作需要）", items: { type: "string" } },
            message: { type: "string", description: "提交消息（commit操作需要）" },
            branch_name: { type: "string", description: "分支名称（branch/checkout操作需要）" },
            remote: { type: "string", description: "远程仓库名称（push/pull操作可选）" },
            target_branch: { type: "string", description: "目标分支（merge操作需要）" }
          },
          required: ["operation"]
        }
      },

      // 网络工具
      network: {
        name: "network",
        description: "网络操作和请求。支持HTTP请求、DNS查询、网络连接测试等。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "网络操作类型", enum: ["http_request", "dns_query", "ping", "download"] },
            url: { type: "string", description: "URL地址（http_request/download操作需要）" },
            method: { type: "string", description: "HTTP方法（http_request操作可选）", enum: ["GET", "POST", "PUT", "DELETE"], default: "GET" },
            headers: { type: "object", description: "请求头（http_request操作可选）" },
            body: { type: "string", description: "请求体（http_request操作可选）" },
            domain: { type: "string", description: "域名（dns_query操作需要）" },
            host: { type: "string", description: "主机地址（ping操作需要）" },
            save_path: { type: "string", description: "保存路径（download操作需要）" }
          },
          required: ["operation"]
        }
      },

      // 系统工具
      system: {
        name: "system",
        description: "系统信息和操作。支持系统状态查询、进程管理、环境变量等。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "系统操作类型", enum: ["info", "processes", "environment", "memory", "disk", "network_status"] },
            process_id: { type: "number", description: "进程ID（进程操作时可选）" },
            variable_name: { type: "string", description: "环境变量名称（获取环境变量时可选）" }
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
      },

      // Agent Skills工具
      agent_skills: {
        name: "agent_skills",
        description: "管理和执行标准 Agent Skills 协议技能。支持 SKILL.md 格式、Progressive Disclosure 和脚本执行。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "技能操作类型", enum: ["list", "execute", "discover", "validate", "install"] },
            skill_name: { type: "string", description: "技能名称（execute/validate/install操作需要）" },
            parameters: { type: "object", description: "技能参数（execute操作需要）" },
            query: { type: "string", description: "搜索查询（discover操作可选）" },
            skill_url: { type: "string", description: "技能URL（install操作需要）" }
          },
          required: ["operation"]
        }
      },

      // Agent协作工具
      agent_collaboration: {
        name: "agent_collaboration",
        description: "Agent间协作和通信。支持IPFS分布式协作、消息传递、任务共享。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "协作操作类型", enum: ["create_session", "join_session", "send_message", "share_task", "get_status"] },
            session_id: { type: "string", description: "会话ID（join_session/send_message/share_task/get_status操作需要）" },
            message: { type: "string", description: "消息内容（send_message操作需要）" },
            task_data: { type: "object", description: "任务数据（share_task操作需要）" },
            agent_did: { type: "string", description: "Agent DID（create_session操作可选）" }
          },
          required: ["operation"]
        }
      },

      // Agent创建工具
      agent_creator: {
        name: "agent_creator",
        description: "创建和管理子Agent。支持动态创建、配置管理、生命周期控制。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "创建操作类型", enum: ["create", "configure", "start", "stop", "list", "delete"] },
            agent_config: { type: "object", description: "Agent配置（create操作需要）" },
            agent_id: { type: "string", description: "Agent ID（configure/start/stop/delete操作需要）" },
            config_updates: { type: "object", description: "配置更新（configure操作需要）" }
          },
          required: ["operation"]
        }
      },

      // 工具创建工具
      tool_creation: {
        name: "tool_creation",
        description: "动态创建和注册自定义工具。支持工具模板、代码生成、热更新。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "创建操作类型", enum: ["create", "register", "unregister", "list", "update"] },
            tool_definition: { type: "object", description: "工具定义（create/register操作需要）" },
            tool_id: { type: "string", description: "工具ID（unregister/update操作需要）" },
            template_type: { type: "string", description: "模板类型（create操作可选）", enum: ["basic", "advanced", "api", "filesystem"] }
          },
          required: ["operation"]
        }
      },

      // 回滚工具
      rollback: {
        name: "rollback",
        description: "操作回滚和状态恢复。支持事务回滚、文件恢复、状态重置。",
        parameters: {
          type: "object",
          properties: {
            operation: { type: "string", description: "回滚操作类型", enum: ["create_checkpoint", "rollback", "list_checkpoints", "delete_checkpoint"] },
            checkpoint_id: { type: "string", description: "检查点ID（rollback/delete_checkpoint操作需要）" },
            target_state: { type: "string", description: "目标状态（rollback操作可选）" },
            description: { type: "string", description: "检查点描述（create_checkpoint操作可选）" }
          },
          required: ["operation"]
        }
      }
    };

    // 工具类别映射
    const CATEGORY_TOOLS = {
      CORE: ["bash", "filesystem", "search", "git_helper", "network", "system"],
      NETWORK: ["web_search", "web_fetch"],
      CONTROL_FLOW: ["plan", "ask_user_question", "subagents"],
      WEB3: ["query_blockchain", "build_transaction", "broadcast_transaction", "wallet_manager", "agent_wallet"],
      FILESYSTEM: ["filesystem"],
      SEARCH: ["search"],
      PLANNING: ["plan", "todolist"],
      AGENT: ["agent_skills", "agent_collaboration", "agent_creator"],
      DEVELOPMENT: ["tool_creation", "rollback"]
    };

    const tools: any[] = [];
    const seen = new Set<string>();

    categories.forEach(category => {
      const categoryTools = CATEGORY_TOOLS[category.toUpperCase()] || [];
      categoryTools.forEach(toolName => {
        const tool = SIMPLE_TOOLS[toolName as keyof typeof SIMPLE_TOOLS];
        if (tool && !seen.has(tool.name)) {
          seen.add(tool.name);
          tools.push({ ...tool });
        }
      });
    });

    console.log('[getToolsByCategories] 获取到的工具数量:', tools.length);
    console.log('[getToolsByCategories] 工具列表:', tools.map((t: any) => t.name).join(', '));

    return tools;
  } catch (error) {
    console.error('[getToolsByCategories] 获取工具配置失败:', error);
    return [];
  }
};
