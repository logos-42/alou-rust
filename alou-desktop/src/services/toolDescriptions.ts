/**
 * 工具描述配置 - 为AI提供详细的工具使用说明
 */

export const TOOL_DESCRIPTIONS = {
  // 网络通信类工具
  iroh: {
    name: "Iroh P2P网络工具",
    description: "基于Iroh协议的P2P网络通信工具，用于安全的点对点数据传输和群聊",
    categories: ["network", "p2p", "data-sync", "group_chat"],
    usage: [
      "创建安全的P2P连接",
      "在节点间同步数据",
      "创建P2P群聊",
      "加入/离开P2P群聊",
      "发送P2P群聊消息"
    ],
    parameters: {
      action: {
        type: "string",
        description: "操作类型：create_doc, open_doc, set, get, list_entries, get_node_id, connect_to_node, share_doc_ticket, create_group, join_group, leave_group, send_group_message, get_group_info, list_groups"
      }
    }
  },
  
  message_passing: {
    name: "消息传递工具",
    description: "用于在Iroh和PubSub网络中发送和接收消息的工具",
    categories: ["communication", "messaging", "pubsub"],
    usage: [
      "发送消息到指定主题",
      "订阅主题并接收消息",
      "管理消息历史记录"
    ],
    parameters: {
      action: {
        type: "string",
        description: "操作类型：send_message, subscribe_topic, list_topics, get_message_history, create_topic"
      }
    }
  },
  
  pubsub: {
    name: "发布-订阅工具",
    description: "发布-订阅消息系统，支持主题管理和群聊功能",
    categories: ["communication", "pubsub", "messaging", "group_chat"],
    usage: [
      "发布消息到主题",
      "订阅主题接收消息",
      "创建群聊",
      "加入/离开群聊",
      "发送群聊消息"
    ],
    parameters: {
      action: {
        type: "string",
        description: "操作类型：publish, subscribe, subscriber_count, list_topics, get_history, create_persistent_topic, create_group, join_group, leave_group, send_group_message, get_group_info, list_groups"
      }
    }
  },

  // 界面控制类工具
  ui_control: {
    name: "UI控制工具",
    description: "用于控制桌面应用程序界面元素的工具",
    categories: ["ui", "automation", "desktop"],
    usage: [
      "点击按钮",
      "输入文本到输入框",
      "选择下拉菜单选项",
      "切换复选框状态",
      "显示通知和对话框"
    ],
    parameters: {
      action: {
        type: "string",
        description: "操作类型：click_button, set_input_text, get_input_text, select_dropdown_option, toggle_checkbox, select_radio_button, show_notification, show_modal, get_window_state, set_window_state"
      }
    }
  },

  // 浏览器类工具
  browser: {
    name: "浏览器工具",
    description: "用于控制Web浏览器进行自动化操作的工具",
    categories: ["browser", "automation", "web"],
    usage: [
      "打开和导航网页",
      "与页面元素交互",
      "提取页面内容",
      "截取页面截图"
    ],
    parameters: {
      action: {
        type: "string",
        description: "操作类型：open_page, close_page, navigate, refresh, click_element, input_text, get_element_text, get_page_title, get_page_url, take_screenshot, wait_for_element, scroll_to_element, execute_script, get_page_source, set_window_size, open_new_tab, switch_tab"
      }
    }
  },

  // 系统类工具
  filesystem: {
    name: "文件系统工具",
    description: "用于文件和目录操作的工具",
    categories: ["filesystem", "file", "directory"],
    usage: [
      "读取、写入、删除文件",
      "创建、删除、遍历目录",
      "文件搜索和复制"
    ],
    parameters: {
      operation: {
        type: "string",
        description: "操作类型：read, write, delete, list, copy, move, search"
      }
    }
  },

  bash: {
    name: "Bash终端工具",
    description: "用于执行终端命令的工具",
    categories: ["terminal", "bash", "command"],
    usage: [
      "执行系统命令",
      "运行脚本",
      "管理系统进程"
    ],
    parameters: {
      command: {
        type: "string",
        description: "要执行的命令"
      }
    }
  },

  search: {
    name: "搜索工具",
    description: "用于在文件和文本中搜索内容的工具",
    categories: ["search", "find", "query"],
    usage: [
      "在文件中搜索文本",
      "查找文件",
      "搜索代码片段"
    ],
    parameters: {
      query: {
        type: "string",
        description: "搜索查询"
      }
    }
  },

  // 钱包类工具
  wallet_manager: {
    name: "钱包管理工具",
    description: "用于管理加密货币钱包的工具",
    categories: ["wallet", "crypto", "blockchain"],
    usage: [
      "查询钱包余额",
      "管理钱包地址",
      "切换网络"
    ],
    parameters: {
      action: {
        type: "string",
        description: "操作类型：list_networks, switch_network, get_current_network, get_wallet_info, check_balance"
      }
    }
  },

  // AI类工具
  skills: {
    name: "技能工具",
    description: "用于扩展AI能力的技能系统",
    categories: ["ai", "skills", "extension"],
    usage: [
      "执行特定技能",
      "管理技能集合",
      "扩展AI功能"
    ],
    parameters: {
      skill_name: {
        type: "string",
        description: "技能名称"
      }
    }
  }
};

/**
 * 获取工具描述
 */
export function getToolDescription(toolName: string) {
  return TOOL_DESCRIPTIONS[toolName as keyof typeof TOOL_DESCRIPTIONS] || {
    name: toolName,
    description: `未知工具 ${toolName}，请参考相关文档`,
    categories: ["unknown"],
    usage: ["未知用法"]
  };
}

/**
 * 获取所有工具名称
 */
export function getAllToolNames() {
  return Object.keys(TOOL_DESCRIPTIONS);
}

/**
 * 根据类别获取工具
 */
export function getToolsByCategory(category: string) {
  return Object.entries(TOOL_DESCRIPTIONS)
    .filter(([_, desc]) => desc.categories.includes(category))
    .map(([name, _]) => name);
}